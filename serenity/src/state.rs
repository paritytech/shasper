// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use primitives::{H256, ValidatorId, BitField};
use ssz::Hashable;
use ssz_derive::Ssz;
use crate::{Gwei, Slot, Epoch, Timestamp, ValidatorIndex, Shard};
use crate::eth1::{Eth1Data, Eth1DataVote, Deposit};
use crate::slashing::{SlashableAttestation, ProposerSlashing, AttesterSlashing};
use crate::attestation::{
	PendingAttestation, Crosslink, AttestationDataAndCustodyBit,
	AttestationData,
};
use crate::validator::Validator;
use crate::block::{BeaconBlock, BeaconBlockHeader};
use crate::consts::*;
use crate::error::Error;
use crate::util::{
	Hasher, bls_domain, slot_to_epoch, hash3, to_bytes, bls_aggregate_pubkeys,
	bls_verify_multiple, shuffling, is_power_of_two, epoch_committee_count,
	epoch_start_slot, compare_hash, integer_squareroot, bls_verify, hash
};

#[derive(Ssz)]
pub struct BeaconState {
	// Misc
	pub slot: Slot,
	pub genesis_time: Timestamp,
	pub fork: Fork, // For versioning hard forks

	// Validator registry
	pub validator_registry: Vec<Validator>,
	pub validator_balances: Vec<u64>,
	pub validator_registry_update_epoch: Epoch,

	// Randomness and committees
	pub latest_randao_mixes: [H256; LATEST_RANDAO_MIXES_LENGTH],
	pub previous_shuffling_start_shard: Shard,
	pub current_shuffling_start_shard: Shard,
	pub previous_shuffling_epoch: Epoch,
	pub current_shuffling_epoch: Epoch,
	pub previous_shuffling_seed: H256,
	pub current_shuffling_seed: H256,

	// Finality
	pub previous_epoch_attestations: Vec<PendingAttestation>,
	pub current_epoch_attestations: Vec<PendingAttestation>,
	pub previous_justified_epoch: Epoch,
	pub justified_epoch: Epoch,
	pub justification_bitfield: u64,
	pub finalized_epoch: Epoch,

	// Recent state
	pub latest_crosslinks: [Crosslink; SHARD_COUNT],
	pub latest_block_roots: [H256; SLOTS_PER_HISTORICAL_ROOT],
	pub latest_state_roots: [H256; SLOTS_PER_HISTORICAL_ROOT],
	pub latest_active_index_roots: [H256; LATEST_ACTIVE_INDEX_ROOTS_LENGTH],
	pub latest_slashed_balances: [u64; LATEST_SLASHED_EXIT_LENGTH], // Balances slashed at every withdrawal period
	pub latest_block_header: BeaconBlockHeader,
	pub historical_roots: Vec<H256>,

	// Ethereum 1.0 chain data
	pub latest_eth1_data: Eth1Data,
	pub eth1_data_votes: Vec<Eth1DataVote>,
	pub deposit_index: u64,
}

#[derive(Ssz)]
pub struct HistoricalBatch {
	/// Block roots
	pub block_roots: [H256; SLOTS_PER_HISTORICAL_ROOT],
	/// State roots
	pub state_roots: [H256; SLOTS_PER_HISTORICAL_ROOT],
}

#[derive(Ssz)]
pub struct Fork {
	/// Previous fork version
	pub previous_version: u64,
	/// Current fork version
	pub current_version: u64,
	/// Fork epoch number
	pub epoch: u64,
}

impl Default for Fork {
	fn default() -> Self {
		Self {
			previous_version: GENESIS_FORK_VERSION,
			current_version: GENESIS_FORK_VERSION,
			epoch: GENESIS_EPOCH,
		}
	}
}

impl BeaconState {
	pub fn current_epoch(&self) -> Epoch {
		slot_to_epoch(self.slot)
	}

	pub fn previous_epoch(&self) -> Epoch {
		self.current_epoch().saturating_sub(1)
	}

	pub fn delayed_activation_exit_epoch(&self) -> u64 {
		self.current_epoch() + 1 + ACTIVATION_EXIT_DELAY
	}

	pub fn randao_mix(&self, epoch: Epoch) -> Result<H256, Error> {
		if self.current_epoch().saturating_sub(LATEST_RANDAO_MIXES_LENGTH as u64) >= epoch ||
			epoch > self.current_epoch()
		{
			return Err(Error::EpochOutOfRange)
		}

		Ok(self.latest_randao_mixes[(epoch % LATEST_RANDAO_MIXES_LENGTH as u64) as usize])
	}

	pub fn block_root(&self, slot: Slot) -> Result<H256, Error> {
		if slot >= self.slot || self.slot > slot + SLOTS_PER_HISTORICAL_ROOT as u64 {
			return Err(Error::SlotOutOfRange)
		}
		Ok(self.latest_block_roots[(slot % SLOTS_PER_HISTORICAL_ROOT as u64) as usize])
	}

	pub fn state_root(&self, slot: Slot) -> Result<H256, Error> {
		if slot >= self.slot || self.slot > slot + SLOTS_PER_HISTORICAL_ROOT as u64 {
			return Err(Error::SlotOutOfRange)
		}
		Ok(self.latest_state_roots[(slot % SLOTS_PER_HISTORICAL_ROOT as u64) as usize])
	}

	pub fn active_index_root(&self, epoch: Epoch) -> Result<H256, Error> {
		if self.current_epoch().saturating_sub(
			LATEST_ACTIVE_INDEX_ROOTS_LENGTH as u64 - ACTIVATION_EXIT_DELAY
		) >= epoch || epoch > self.current_epoch() + ACTIVATION_EXIT_DELAY {
			return Err(Error::EpochOutOfRange)
		}

		Ok(self.latest_active_index_roots[(epoch % LATEST_ACTIVE_INDEX_ROOTS_LENGTH as u64) as usize])
	}

	pub fn seed(&self, epoch: Epoch) -> Result<H256, Error> {
		Ok(hash3(
			self.randao_mix(epoch.saturating_sub(MIN_SEED_LOOKAHEAD))?.as_ref(),
			self.active_index_root(epoch)?.as_ref(),
			to_bytes(epoch).as_ref()
		))
	}

	pub fn beacon_proposer_index(&self, slot: Slot, registry_change: bool) -> Result<ValidatorIndex, Error> {
		let epoch = slot_to_epoch(slot);
		let current_epoch = self.current_epoch();
		let previous_epoch = self.previous_epoch();
		let next_epoch = current_epoch + 1;

		if previous_epoch > epoch || epoch > next_epoch {
			return Err(Error::EpochOutOfRange)
		}

		let (first_committee, _) = self.crosslink_committees_at_slot(slot, registry_change)?[0].clone();
		Ok(first_committee[(slot % first_committee.len() as u64) as usize])
	}

	pub fn validator_by_id(&self, validator_id: &ValidatorId) -> Option<&Validator> {
		for validator in &self.validator_registry {
			if &validator.pubkey == validator_id {
				return Some(validator)
			}
		}

		None
	}

	fn effective_balance(&self, index: ValidatorIndex) -> Gwei {
		core::cmp::min(self.validator_balances[index as usize], MIN_DEPOSIT_AMOUNT)
	}

	fn activate_validator(&mut self, index: ValidatorIndex, is_genesis: bool) {
		let delayed_activation_exit_epoch = self.delayed_activation_exit_epoch();
		self.validator_registry[index as usize].activate(delayed_activation_exit_epoch, is_genesis);
	}

	pub fn initiate_validator_exit(&mut self, index: ValidatorIndex) {
		self.validator_registry[index as usize].initiate_exit();
	}

	pub fn slash_validator(&mut self, index: ValidatorIndex) -> Result<(), Error> {
		if self.slot >= epoch_start_slot(self.validator_registry[index as usize].withdrawable_epoch) {
			return Err(Error::ValidatorNotWithdrawable);
		}
		self.exit_validator(index);

		self.latest_slashed_balances[(self.current_epoch() % LATEST_SLASHED_EXIT_LENGTH as u64) as usize] += self.effective_balance(index);

		let whistleblower_index = self.beacon_proposer_index(self.slot, false)?;
		let whistleblower_reward = self.effective_balance(index) / WHISTLEBLOWER_REWARD_QUOTIENT;
		self.validator_balances[whistleblower_index as usize] += whistleblower_reward;
		self.validator_balances[index as usize] -= whistleblower_reward;
		self.validator_registry[index as usize].slashed = true;
		self.validator_registry[index as usize].withdrawable_epoch = self.current_epoch() + LATEST_SLASHED_EXIT_LENGTH as u64;

		Ok(())
	}

	pub fn prepare_validator_for_withdrawal(&mut self, index: ValidatorIndex) {
		self.validator_registry[index as usize].withdrawable_epoch = self.current_epoch() + MIN_VALIDATOR_WITHDRAWABILITY_DELAY;
	}

	pub fn exit_validator(&mut self, index: ValidatorIndex) {
		let delayed_activation_exit_epoch = self.delayed_activation_exit_epoch();
		self.validator_registry[index as usize].exit(delayed_activation_exit_epoch);
	}

	pub fn push_deposit(&mut self, deposit: Deposit) -> Result<(), Error> {
		if deposit.index != self.deposit_index {
			return Err(Error::DepositIndexMismatch)
		}

		if !deposit.is_merkle_valid(&self.latest_eth1_data.deposit_root) {
			return Err(Error::DepositMerkleInvalid)
		}

		self.deposit_index += 1;

		if !deposit.is_proof_valid(
			bls_domain(&self.fork, self.current_epoch(), DOMAIN_DEPOSIT)
		) {
			return Err(Error::DepositProofInvalid)
		}

		match self.validator_by_id(&deposit.deposit_data.deposit_input.pubkey) {
			Some(validator) => {
				if validator.withdrawal_credentials != deposit.deposit_data.deposit_input.withdrawal_credentials {
					return Err(Error::DepositWithdrawalCredentialsMismatch)
				}
			},
			None => {

			},
		}

		Ok(())
	}

	pub fn push_proposer_slashing(&mut self, proposer_slashing: ProposerSlashing) -> Result<(), Error> {
		if proposer_slashing.header_a.slot != proposer_slashing.header_b.slot {
			return Err(Error::ProposerSlashingInvalidSlot)
		}

		if proposer_slashing.header_a == proposer_slashing.header_b {
			return Err(Error::ProposerSlashingSameHeader)
		}

		{
			let proposer = &self.validator_registry[proposer_slashing.proposer_index as usize];

			if proposer.slashed {
				return Err(Error::ProposerSlashingAlreadySlashed)
			}

			for header in [&proposer_slashing.header_a, &proposer_slashing.header_b].into_iter() {
				if !bls_verify(&proposer.pubkey, &header.truncated_hash::<Hasher>(), &header.signature, bls_domain(&self.fork, slot_to_epoch(header.slot), DOMAIN_BEACON_BLOCK)) {
					return Err(Error::ProposerSlashingInvalidSignature)
				}
			}
		}

		self.slash_validator(proposer_slashing.proposer_index)
	}

	pub fn push_attester_slashing(&mut self, attester_slashing: AttesterSlashing) -> Result<(), Error> {
		let attestation1 = attester_slashing.slashable_attestation_a;
		let attestation2 = attester_slashing.slashable_attestation_b;

		if attestation1.data == attestation2.data {
			return Err(Error::AttesterSlashingSameAttestation)
		}

		if !(attestation1.data.is_double_vote(&attestation2.data) || attestation1.data.is_surround_vote(&attestation2.data)) {
			return Err(Error::AttesterSlashingNotSlashable)
		}

		if !self.verify_slashable_attestation(&attestation1) {
			return Err(Error::AttesterSlashingInvalid)
		}

		if !self.verify_slashable_attestation(&attestation2) {
			return Err(Error::AttesterSlashingInvalid)
		}

		let mut slashable_indices = Vec::new();
		for index in &attestation1.validator_indices {
			if attestation2.validator_indices.contains(index) && !self.validator_registry[*index as usize].slashed {
				slashable_indices.push(*index);
			}
		}

		if slashable_indices.len() == 0 {
			return Err(Error::AttesterSlashingEmptyIndices)
		}

		for index in slashable_indices {
			self.slash_validator(index)?;
		}

		Ok(())
	}

	pub fn active_validator_indices(&self, epoch: Epoch) -> Vec<ValidatorIndex> {
		self.validator_registry.iter()
			.enumerate()
			.filter(|(_, v)| v.is_active(epoch))
			.map(|(i, _)| i as u64)
			.collect::<Vec<_>>()
	}

	pub fn genesis(deposits: Vec<Deposit>, genesis_time: Timestamp, latest_eth1_data: Eth1Data) -> Result<Self, Error> {
		let mut state = Self {
			slot: GENESIS_SLOT,
			genesis_time,
			fork: Fork::default(),

			validator_registry: Vec::new(),
			validator_balances: Vec::new(),
			validator_registry_update_epoch: GENESIS_EPOCH,

			latest_randao_mixes: [H256::default(); LATEST_RANDAO_MIXES_LENGTH],
			previous_shuffling_start_shard: GENESIS_START_SHARD,
			current_shuffling_start_shard: GENESIS_START_SHARD,
			previous_shuffling_epoch: GENESIS_EPOCH,
			current_shuffling_epoch: GENESIS_EPOCH,
			previous_shuffling_seed: H256::default(),
			current_shuffling_seed: H256::default(),

			previous_epoch_attestations: Vec::new(),
			current_epoch_attestations: Vec::new(),
			previous_justified_epoch: GENESIS_EPOCH,
			justified_epoch: GENESIS_EPOCH,
			justification_bitfield: 0,
			finalized_epoch: GENESIS_EPOCH,

			latest_crosslinks: unsafe {
				let mut ret: [Crosslink; SHARD_COUNT] = core::mem::uninitialized();
				for item in &mut ret[..] {
					core::ptr::write(item, Crosslink::default());
				}
				ret
			},
			latest_block_roots: [H256::default(); SLOTS_PER_HISTORICAL_ROOT],
			latest_state_roots: [H256::default(); SLOTS_PER_HISTORICAL_ROOT],
			latest_active_index_roots: [H256::default(); LATEST_ACTIVE_INDEX_ROOTS_LENGTH],
			latest_slashed_balances: [0; LATEST_SLASHED_EXIT_LENGTH],
			latest_block_header: BeaconBlockHeader::with_state_root(&BeaconBlock::empty(), H256::default()),
			historical_roots: Vec::new(),

			latest_eth1_data,
			eth1_data_votes: Vec::new(),
			deposit_index: 0,
		};

		for deposit in deposits {
			state.push_deposit(deposit)?;
		}

		for validator_index in 0..(state.validator_registry.len() as u64) {
			if state.effective_balance(validator_index) >= MAX_DEPOSIT_AMOUNT {
				state.activate_validator(validator_index, true);
			}
		}

		let genesis_active_index_root = state.active_validator_indices(GENESIS_EPOCH).hash::<Hasher>();
		for index in 0..LATEST_ACTIVE_INDEX_ROOTS_LENGTH {
			state.latest_active_index_roots[index] = genesis_active_index_root;
		}
		state.current_shuffling_seed = state.seed(GENESIS_EPOCH)?;

		Ok(state)
	}

	pub fn update_cache(&mut self) {
		let previous_slot_state_root = self.hash::<Hasher>();

		self.latest_state_roots[(self.slot % SLOTS_PER_HISTORICAL_ROOT as u64) as usize] = previous_slot_state_root;

		if self.latest_block_header.state_root == H256::default() {
			self.latest_block_header.state_root = previous_slot_state_root;
		}

		self.latest_block_roots[(self.slot % SLOTS_PER_HISTORICAL_ROOT as u64) as usize] = self.latest_block_header.hash::<Hasher>();
	}

	pub fn update_justification_and_finalization(&mut self) -> Result<(), Error> {
		let mut new_justified_epoch = self.justified_epoch;
		self.justification_bitfield <<= 1;

		let previous_boundary_attesting_balance = self.attesting_balance(&self.previous_epoch_boundary_attestations()?)?;
		if previous_boundary_attesting_balance * 3 >= self.previous_total_balance() * 2 {
			new_justified_epoch = self.current_epoch() - 1;
			self.justification_bitfield |= 2;
		}

		let current_boundary_attesting_balance = self.attesting_balance(&self.current_epoch_boundary_attestations()?)?;
		if current_boundary_attesting_balance * 3 >= self.current_total_balance() * 2 {
			new_justified_epoch = self.current_epoch();
			self.justification_bitfield |= 1;
		}

		let bitfield = self.justification_bitfield;
		let current_epoch = self.current_epoch();
		if (bitfield >> 1) % 8 == 0b111 && self.previous_justified_epoch == current_epoch - 3 {
			self.finalized_epoch = self.previous_justified_epoch;
		}
		if (bitfield >> 1) % 4 == 0b011 && self.previous_justified_epoch == current_epoch - 2 {
			self.finalized_epoch = self.previous_justified_epoch;
		}
		if (bitfield >> 0) % 8 == 0b111 && self.justified_epoch == current_epoch - 2 {
			self.finalized_epoch = self.justified_epoch;
		}
		if (bitfield >> 0) % 4 == 0b011 && self.justified_epoch == current_epoch - 1 {
			self.finalized_epoch = self.justified_epoch;
		}

		self.previous_justified_epoch = self.justified_epoch;
		self.justified_epoch = new_justified_epoch;

		Ok(())
	}

	pub fn update_crosslinks(&mut self) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		let previous_epoch = current_epoch.saturating_sub(1);
		let next_epoch = current_epoch + 1;

		for slot in epoch_start_slot(previous_epoch)..epoch_start_slot(next_epoch) {
			for (crosslink_committee, shard) in self.crosslink_committees_at_slot(slot, false)? {
				let (winning_root, participants) = self.winning_root_and_participants(shard)?;
				let participating_balance = self.total_balance(&participants);
				let total_balance = self.total_balance(&crosslink_committee);
				if 3 * participating_balance >= 2 * total_balance {
					self.latest_crosslinks[shard as usize] = Crosslink {
						epoch: slot_to_epoch(slot),
						crosslink_data_root: winning_root,
					};
				}
			}
		}

		Ok(())
	}

	pub fn update_eth1_period(&mut self) {
		if (self.current_epoch() + 1) % EPOCHS_PER_ETH1_VOTING_PERIOD == 0 {
			for eth1_data_vote in &self.eth1_data_votes {
				if eth1_data_vote.vote_count * 2 > EPOCHS_PER_ETH1_VOTING_PERIOD * SLOTS_PER_EPOCH {
					self.latest_eth1_data = eth1_data_vote.eth1_data.clone();
				}
			}
			self.eth1_data_votes = Vec::new();
		}
	}

	pub fn update_rewards(&mut self) -> Result<(), Error> {
		let delta1 = self.justification_and_finalization_deltas()?;
		let delta2 = self.crosslink_deltas()?;
		for i in 0..self.validator_registry.len() {
			self.validator_balances[i] = (self.validator_balances[i] + delta1.0[i] + delta2.0[i]).saturating_sub(delta1.1[i] + delta2.1[i]);
		}

		Ok(())
	}

	pub fn update_ejections(&mut self) {
		for index in self.active_validator_indices(self.current_epoch()) {
			if self.validator_balances[index as usize] < EJECTION_BALANCE {
				self.exit_validator(index);
			}
		}
	}

	pub fn should_update_validator_registry(&self) -> bool {
		if self.finalized_epoch <= self.validator_registry_update_epoch {
			return false
		}

		for i in 0..self.current_epoch_committee_count() {
			let s = (self.current_shuffling_start_shard as usize + i) % SHARD_COUNT;
			if self.latest_crosslinks[s].epoch <= self.validator_registry_update_epoch {
				return false
			}
		}

		true
	}

	pub fn update_validator_registry(&mut self) {
		let current_epoch = self.current_epoch();
		let active_validator_indices = self.active_validator_indices(current_epoch);
		let total_balance = self.total_balance(&active_validator_indices);

		let max_balance_churn = core::cmp::max(
			MAX_DEPOSIT_AMOUNT,
			total_balance / (2 * MAX_BALANCE_CHURN_QUOTIENT)
		);

		let mut balance_churn = 0;
		for (i, validator) in self.validator_registry.clone().into_iter().enumerate() {
			let index = i as u64;
			if validator.activation_epoch == FAR_FUTURE_EPOCH && self.validator_balances[i] >= MAX_DEPOSIT_AMOUNT {
				balance_churn += self.effective_balance(index);
				if balance_churn > max_balance_churn {
					break
				}

				self.activate_validator(index, false);
			}
		}

		let mut balance_churn = 0;
		for (i, validator) in self.validator_registry.clone().into_iter().enumerate() {
			let index = i as u64;
			if validator.exit_epoch == FAR_FUTURE_EPOCH && validator.initiated_exit {
				balance_churn += self.effective_balance(index);
				if balance_churn > max_balance_churn {
					break
				}

				self.exit_validator(index);
			}
		}

		self.validator_registry_update_epoch = current_epoch;
	}

	pub fn update_registry_and_shuffling_data(&mut self) -> Result<(), Error> {
		self.previous_shuffling_epoch = self.current_shuffling_epoch;
		self.previous_shuffling_start_shard = self.current_shuffling_start_shard;
		self.previous_shuffling_seed = self.current_shuffling_seed;

		let current_epoch = self.current_epoch();
		let next_epoch = current_epoch + 1;

		if self.should_update_validator_registry() {
			self.update_validator_registry();

			self.current_shuffling_epoch = next_epoch;
			self.current_shuffling_start_shard = self.current_shuffling_start_shard + (self.current_epoch_committee_count() % SHARD_COUNT) as u64;
			self.current_shuffling_seed = self.seed(self.current_shuffling_epoch)?;
		} else {
			let epochs_since_last_registry_update = current_epoch - self.validator_registry_update_epoch;
			if epochs_since_last_registry_update > 1 && is_power_of_two(epochs_since_last_registry_update) {
				self.current_shuffling_epoch = next_epoch;
				self.current_shuffling_seed = self.seed(self.current_shuffling_epoch)?;
			}
		}

		Ok(())
	}

	pub fn update_slashings(&mut self) {
		let current_epoch = self.current_epoch();
		let active_validator_indices = self.active_validator_indices(current_epoch);
		let total_balance = self.total_balance(&active_validator_indices);

		let total_at_start = self.latest_slashed_balances[((current_epoch + 1) % LATEST_SLASHED_EXIT_LENGTH as u64) as usize];
		let total_at_end = self.latest_slashed_balances[(current_epoch % LATEST_SLASHED_EXIT_LENGTH as u64) as usize];
		let total_penalties = total_at_end - total_at_start;

		for (i, validator) in self.validator_registry.clone().into_iter().enumerate() {
			let index = i as u64;
			if validator.slashed && current_epoch == validator.withdrawable_epoch - LATEST_SLASHED_EXIT_LENGTH as u64 / 2 {
				let penalty = core::cmp::max(
					self.effective_balance(index) * core::cmp::min(total_penalties * 3, total_balance) / total_balance,
					self.effective_balance(index) / MIN_PENALTY_QUOTIENT
				);
				self.validator_balances[i] -= penalty;
			}
		}
	}


	pub fn update_exit_queue(&mut self) {
		let mut eligible_indices = (0..(self.validator_registry.len() as u64)).filter(|index| {
			if self.validator_registry[*index as usize].withdrawable_epoch != FAR_FUTURE_EPOCH {
				false
			} else {
				self.current_epoch() >= self.validator_registry[*index as usize].exit_epoch + MIN_VALIDATOR_WITHDRAWABILITY_DELAY
			}
		}).collect::<Vec<_>>();
		eligible_indices.sort_by_key(|index| {
			self.validator_registry[*index as usize].exit_epoch
		});

		for (dequeues, index) in eligible_indices.into_iter().enumerate() {
			if dequeues >= MAX_EXIT_DEQUEUES_PER_EPOCH {
				break
			}
			self.prepare_validator_for_withdrawal(index);
		}
	}

	pub fn update_finalize(&mut self) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		let next_epoch = current_epoch + 1;

		let index_root_position = (next_epoch + ACTIVATION_EXIT_DELAY) % LATEST_ACTIVE_INDEX_ROOTS_LENGTH as u64;
		self.latest_active_index_roots[index_root_position as usize] = self.active_validator_indices(next_epoch + ACTIVATION_EXIT_DELAY).hash::<Hasher>();
		self.latest_slashed_balances[(next_epoch % LATEST_SLASHED_EXIT_LENGTH as u64) as usize] = self.latest_slashed_balances[(current_epoch % LATEST_SLASHED_EXIT_LENGTH as u64) as usize];
		self.latest_randao_mixes[(next_epoch % LATEST_RANDAO_MIXES_LENGTH as u64) as usize] = self.randao_mix(current_epoch)?;

		if next_epoch % (SLOTS_PER_HISTORICAL_ROOT as u64 / SLOTS_PER_EPOCH) == 0 {
			self.historical_roots.push(HistoricalBatch {
				block_roots: self.latest_block_roots.clone(),
				state_roots: self.latest_state_roots.clone(),
			}.hash::<Hasher>());
		}
		self.previous_epoch_attestations = self.current_epoch_attestations.clone();
		self.current_epoch_attestations = Vec::new();

		Ok(())
	}

	pub fn advance_slot(&mut self) {
		self.slot += 1;
	}

	pub fn process_block_header(&mut self, block: &BeaconBlock) -> Result<(), Error> {
		if block.slot != self.slot {
			return Err(Error::BlockSlotInvalid)
		}

		if block.previous_block_root != self.latest_block_header.hash::<Hasher>() {
			return Err(Error::BlockPreviousRootInvalid)
		}

		self.latest_block_header = BeaconBlockHeader::with_state_root(block, H256::default());

		let proposer = &self.validator_registry[self.beacon_proposer_index(self.slot, false)? as usize];

		if !bls_verify(&proposer.pubkey, &block.truncated_hash::<Hasher>(), &block.signature, bls_domain(&self.fork, self.current_epoch(), DOMAIN_BEACON_BLOCK)) {
			return Err(Error::BlockSignatureInvalid)
		}

		Ok(())
	}

	pub fn process_randao(&mut self, block: &BeaconBlock) -> Result<(), Error> {
		let proposer = &self.validator_registry[self.beacon_proposer_index(self.slot, false)? as usize];

		if !bls_verify(&proposer.pubkey, &self.current_epoch().hash::<Hasher>(), &block.body.randao_reveal, bls_domain(&self.fork, self.current_epoch(), DOMAIN_RANDAO)) {
			return Err(Error::RandaoSignatureInvalid)
		}

		self.latest_randao_mixes[(self.current_epoch() % LATEST_RANDAO_MIXES_LENGTH as u64) as usize] = self.randao_mix(self.current_epoch())? ^ hash(&block.body.randao_reveal[..]);

		Ok(())
	}

	pub fn process_eth1_data(&mut self, block: &BeaconBlock) {
		for eth1_data_vote in &mut self.eth1_data_votes {
			if eth1_data_vote.eth1_data == block.body.eth1_data {
				eth1_data_vote.vote_count += 1;
				return
			}
		}

		self.eth1_data_votes.push(Eth1DataVote {
			eth1_data: block.body.eth1_data.clone(),
			vote_count: 1
		});
	}

	pub fn previous_epoch_committee_count(&self) -> usize {
		let previous_active_validators = self.active_validator_indices(self.previous_shuffling_epoch);
		epoch_committee_count(previous_active_validators.len())
	}

	pub fn current_epoch_committee_count(&self) -> usize {
		let current_active_validators = self.active_validator_indices(self.current_shuffling_epoch);
		epoch_committee_count(current_active_validators.len())
	}

	pub fn next_epoch_committee_count(&self) -> usize {
		let next_active_validators = self.active_validator_indices(self.current_epoch() + 1);
		epoch_committee_count(next_active_validators.len())
	}

	pub fn crosslink_committees_at_slot(&self, slot: Slot, registry_change: bool) -> Result<Vec<(Vec<ValidatorIndex>, Shard)>, Error> {
		let epoch = slot_to_epoch(slot);
		let current_epoch = self.current_epoch();
		let previous_epoch = self.previous_epoch();
		let next_epoch = current_epoch + 1;

		if previous_epoch > epoch || epoch > next_epoch {
			return Err(Error::EpochOutOfRange);
		}

		let (committees_per_epoch, seed, shuffling_epoch, shuffling_start_shard) = if epoch == current_epoch {
			let committees_per_epoch = self.current_epoch_committee_count();
			let seed = self.current_shuffling_seed;
			let shuffling_epoch = self.current_shuffling_epoch;
			let shuffling_start_shard = self.current_shuffling_start_shard;

			(committees_per_epoch, seed, shuffling_epoch, shuffling_start_shard)
		} else if epoch == previous_epoch {
			let committees_per_epoch = self.previous_epoch_committee_count();
			let seed = self.previous_shuffling_seed;
			let shuffling_epoch = self.previous_shuffling_epoch;
			let shuffling_start_shard = self.previous_shuffling_start_shard;

			(committees_per_epoch, seed, shuffling_epoch, shuffling_start_shard)
		} else {
			let epochs_since_last_registry_update = current_epoch - self.validator_registry_update_epoch;

			if registry_change {
				let committees_per_epoch = self.next_epoch_committee_count();
				let seed = self.seed(next_epoch)?;
				let shuffling_epoch = next_epoch;
				let current_committees_per_epoch = self.current_epoch_committee_count();
				let shuffling_start_shard = (self.current_shuffling_start_shard + current_committees_per_epoch as u64) % SHARD_COUNT as u64;

				(committees_per_epoch, seed, shuffling_epoch, shuffling_start_shard)
			} else if epochs_since_last_registry_update > 1 && is_power_of_two(epochs_since_last_registry_update) {
				let committees_per_epoch = self.next_epoch_committee_count();
				let seed = self.seed(next_epoch)?;
				let shuffling_epoch = next_epoch;
				let shuffling_start_shard = self.current_shuffling_start_shard;

				(committees_per_epoch, seed, shuffling_epoch, shuffling_start_shard)
			} else {
				let committees_per_epoch = self.current_epoch_committee_count();
				let seed = self.current_shuffling_seed;
				let shuffling_epoch = self.current_shuffling_epoch;
				let shuffling_start_shard = self.current_shuffling_start_shard;

				(committees_per_epoch, seed, shuffling_epoch, shuffling_start_shard)
			}
		};

		let active_validators = self.active_validator_indices(shuffling_epoch);
		let shuffling = shuffling(&seed, active_validators);
		let offset = slot % SLOTS_PER_EPOCH;
		let committees_per_slot = committees_per_epoch as u64 / SLOTS_PER_EPOCH;
		let slot_start_shard = (shuffling_start_shard + committees_per_slot * offset) % SHARD_COUNT as u64;

		let mut ret = Vec::new();
		for i in 0..committees_per_slot {
			ret.push((shuffling[(committees_per_slot * offset + i as u64) as usize].clone(),
					  (slot_start_shard + i as u64) % SHARD_COUNT as u64));
		}
		Ok(ret)
	}

	fn total_balance(&self, indices: &[ValidatorIndex]) -> Gwei {
		indices.iter().fold(0, |sum, index| {
			sum + self.effective_balance(*index)
		})
	}

	pub fn current_total_balance(&self) -> Gwei {
		self.total_balance(&self.active_validator_indices(self.current_epoch())[..])
	}

	pub fn previous_total_balance(&self) -> Gwei {
		self.total_balance(&self.active_validator_indices(self.previous_epoch())[..])
	}

	pub fn attestation_participants(&self, attestation: &AttestationData, bitfield: &BitField) -> Result<Vec<ValidatorIndex>, Error> {
		let crosslink_committees = self.crosslink_committees_at_slot(attestation.slot, false)?;

		let matched_committees = crosslink_committees.iter().filter(|(_, s)| s == &attestation.shard).collect::<Vec<_>>();
		if matched_committees.len() == 0 {
			return Err(Error::AttestationShardInvalid);
		}

		let crosslink_committee = matched_committees[0];
		if bitfield.count() != crosslink_committee.0.len() {
			return Err(Error::AttestationBitFieldInvalid);
		}

		let mut participants = Vec::new();
		for (i, validator_index) in crosslink_committee.0.iter().enumerate() {
			if bitfield.has_voted(i) {
				participants.push(*validator_index);
			}
		}
		Ok(participants)
	}

	pub fn attesting_indices(&self, attestations: &[PendingAttestation]) -> Result<Vec<ValidatorIndex>, Error> {
		let mut ret = Vec::new();
		for attestation in attestations {
			for index in self.attestation_participants(&attestation.data, &attestation.aggregation_bitfield)? {
				if !ret.contains(&index) {
					ret.push(index);
				}
			}
		}
		Ok(ret)
	}

	pub fn attesting_balance(&self, attestations: &[PendingAttestation]) -> Result<Gwei, Error> {
		Ok(self.total_balance(&self.attesting_indices(attestations)?))
	}

	pub fn current_epoch_boundary_attestations(&self) -> Result<Vec<PendingAttestation>, Error> {
		let block_root = self.block_root(epoch_start_slot(self.current_epoch()))?;
		Ok(self.current_epoch_attestations.clone().into_iter()
		   .filter(|a| a.data.epoch_boundary_root == block_root)
		   .collect())
	}

	pub fn previous_epoch_boundary_attestations(&self) -> Result<Vec<PendingAttestation>, Error> {
		let block_root = self.block_root(epoch_start_slot(self.previous_epoch()))?;
		Ok(self.previous_epoch_attestations.clone().into_iter()
		   .filter(|a| a.data.epoch_boundary_root == block_root)
		   .collect())
	}

	pub fn previous_epoch_matching_head_attestations(&self) -> Result<Vec<PendingAttestation>, Error> {
		let mut ret = Vec::new();
		for attestation in self.previous_epoch_attestations.clone() {
			if attestation.data.beacon_block_root == self.block_root(attestation.data.slot)? {
				ret.push(attestation);
			}
		}
		Ok(ret)
	}

	pub fn winning_root_and_participants(&self, shard: Shard) -> Result<(H256, Vec<ValidatorIndex>), Error> {
		let all_attestations = self.current_epoch_attestations.clone().into_iter()
			.chain(self.previous_epoch_attestations.clone().into_iter());
		let valid_attestations = all_attestations.filter(|a| {
			a.data.latest_crosslink == self.latest_crosslinks[shard as usize]
		}).collect::<Vec<_>>();
		let all_roots = valid_attestations.iter()
			.map(|a| a.data.crosslink_data_root)
			.collect::<Vec<_>>();

		let attestations_for = |root| {
			valid_attestations.clone().into_iter()
				.filter(|a| a.data.crosslink_data_root == root)
				.collect::<Vec<_>>()
		};

		let all_roots_with_balances = {
			let mut ret = Vec::new();
			for root in all_roots {
				let balance = self.attesting_balance(&attestations_for(root))?;
				ret.push((root, balance));
			}
			ret
		};

		let winning_root = match all_roots_with_balances.into_iter()
			.max_by(|(a, a_balance), (b, b_balance)| {
				if a_balance == b_balance {
					compare_hash(a, b)
				} else {
					a_balance.cmp(b_balance)
				}
			})
		{
			Some(winning_root) => winning_root.0,
			None => return Ok((H256::default(), Vec::new()))
		};

		Ok((winning_root, self.attesting_indices(&attestations_for(winning_root))?))
	}

	pub fn earliest_attestation(&self, index: ValidatorIndex) -> Result<PendingAttestation, Error> {
		let attestations = {
			let mut ret = Vec::new();
			for attestation in self.previous_epoch_attestations.clone() {
				if self.attestation_participants(&attestation.data, &attestation.aggregation_bitfield)?.contains(&index) {
					ret.push(attestation);
				}
			}
			ret
		};

		attestations.into_iter().min_by_key(|a| a.inclusion_slot).ok_or(Error::ValidatorAttestationNotFound)
	}

	pub fn inclusion_slot(&self, index: ValidatorIndex) -> Result<Slot, Error> {
		Ok(self.earliest_attestation(index)?.inclusion_slot)
	}

	pub fn inclusion_distance(&self, index: ValidatorIndex) -> Result<Slot, Error> {
		let attestation = self.earliest_attestation(index)?;
		Ok(attestation.inclusion_slot - attestation.data.slot)
	}

	pub fn base_reward(&self, index: ValidatorIndex) -> Gwei {
		if self.previous_total_balance() == 0 {
			return 0
		}

		let adjusted_quotient = integer_squareroot(self.previous_total_balance()) / BASE_REWARD_QUOTIENT;
		self.effective_balance(index) / adjusted_quotient / 5
	}

	pub fn inactivity_penalty(&self, index: ValidatorIndex, epochs_since_finality: Epoch) -> Gwei {
		self.base_reward(index) + self.effective_balance(index) * epochs_since_finality / INACTIVITY_PENALTY_QUOTIENT / 2
	}

	pub fn justification_and_finalization_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let epochs_since_finality = self.current_epoch() + 1 - self.finalized_epoch;
		if epochs_since_finality <= 4 {
			self.normal_justification_and_finalization_deltas()
		} else {
			self.inactivity_leak_deltas()
		}
	}

	pub fn normal_justification_and_finalization_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let mut rewards = Vec::new();
		rewards.resize(self.validator_registry.len(), 0);
		let mut penalties = Vec::new();
		penalties.resize(self.validator_registry.len(), 0);

		let boundary_attestations = self.previous_epoch_boundary_attestations()?;
		let boundary_attesting_balance = self.attesting_balance(&boundary_attestations)?;
		let total_balance = self.previous_total_balance();
		let total_attesting_balance = self.attesting_balance(&self.previous_epoch_attestations)?;
		let matching_head_attestations = self.previous_epoch_matching_head_attestations()?;
		let matching_head_balance = self.attesting_balance(&matching_head_attestations)?;

		for index in self.active_validator_indices(self.previous_epoch()) {
			if self.attesting_indices(&self.previous_epoch_attestations)?.contains(&index) {
				rewards[index as usize] += self.base_reward(index) * total_attesting_balance / total_balance;
				rewards[index as usize] += self.base_reward(index) * MIN_ATTESTATION_INCLUSION_DELAY / self.inclusion_distance(index)?;
			} else {
				penalties[index as usize] += self.base_reward(index);
			}

			if self.attesting_indices(&boundary_attestations)?.contains(&index) {
				rewards[index as usize] += self.base_reward(index) * boundary_attesting_balance / total_balance;
			} else {
				penalties[index as usize] += self.base_reward(index);
			}

			if self.attesting_indices(&matching_head_attestations)?.contains(&index) {
				rewards[index as usize] += self.base_reward(index) * matching_head_balance / total_balance;
			} else {
				penalties[index as usize] += self.base_reward(index);
			}

			if self.attesting_indices(&self.previous_epoch_attestations)?.contains(&index) {
				let proposer_index = self.beacon_proposer_index(self.inclusion_slot(index)?, false)?;
				rewards[proposer_index as usize] += self.base_reward(index) / ATTESTATION_INCLUSION_REWARD_QUOTIENT;
			}
		}

		Ok((rewards, penalties))
	}

	pub fn inactivity_leak_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let mut rewards = Vec::new();
		rewards.resize(self.validator_registry.len(), 0);
		let mut penalties = Vec::new();
		penalties.resize(self.validator_registry.len(), 0);

		let boundary_attestations = self.previous_epoch_boundary_attestations()?;
		let matching_head_attestations = self.previous_epoch_matching_head_attestations()?;
		let active_validator_indices = self.active_validator_indices(self.previous_epoch());
		let epochs_since_finality = self.current_epoch() + 1 - self.finalized_epoch;

		for index in &active_validator_indices {
			if !self.attesting_indices(&self.previous_epoch_attestations)?.contains(index) {
				penalties[*index as usize] += self.inactivity_penalty(*index, epochs_since_finality);
			} else {
				rewards[*index as usize] += self.base_reward(*index) * MIN_ATTESTATION_INCLUSION_DELAY / self.inclusion_distance(*index)?;
				penalties[*index as usize] += self.base_reward(*index);
			}

			if !self.attesting_indices(&boundary_attestations)?.contains(index) {
				penalties[*index as usize] += self.inactivity_penalty(*index, epochs_since_finality);
			}

			if !self.attesting_indices(&matching_head_attestations)?.contains(index) {
				penalties[*index as usize] += self.base_reward(*index);
			}
		}

		for index in 0..(self.validator_registry.len() as u64) {
			let eligible = !active_validator_indices.contains(&index) &&
				self.validator_registry[index as usize].slashed &&
				self.current_epoch() < self.validator_registry[index as usize].withdrawable_epoch;

			if eligible {
				penalties[index as usize] += 2 * self.inactivity_penalty(index, epochs_since_finality) + self.base_reward(index);
			}
		}

		Ok((rewards, penalties))
	}

	pub fn crosslink_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let mut rewards = Vec::new();
		rewards.resize(self.validator_registry.len(), 0);
		let mut penalties = Vec::new();
		penalties.resize(self.validator_registry.len(), 0);

		let previous_epoch_start_slot = epoch_start_slot(self.previous_epoch());
		let current_epoch_start_slot = epoch_start_slot(self.current_epoch());

		for slot in previous_epoch_start_slot..current_epoch_start_slot {
			for (crosslink_committee, shard) in self.crosslink_committees_at_slot(slot, false)? {
				let (_, participants) = self.winning_root_and_participants(shard)?;
				let participating_balance = self.total_balance(&participants);
				let total_balance = self.total_balance(&crosslink_committee);
				for index in crosslink_committee {
					if participants.contains(&index) {
						rewards[index as usize] += self.base_reward(index) * participating_balance / total_balance;
					} else {
						penalties[index as usize] += self.base_reward(index);
					}
				}
			}
		}

		Ok((rewards, penalties))
	}

	pub fn verify_slashable_attestation(&self, slashable: &SlashableAttestation) -> bool {
		if slashable.custody_bitfield.count() != 0 {
			return false;
		}

		if slashable.validator_indices.len() == 0 {
			return false;
		}

		for i in 0..(slashable.validator_indices.len() - 1) {
			if slashable.validator_indices[i] > slashable.validator_indices[i + 1] {
				return false;
			}
		}

		if slashable.custody_bitfield.count() != slashable.validator_indices.len() {
			return false;
		}

		if slashable.validator_indices.len() > MAX_INDICES_PER_SLASHABLE_VOTE {
			return false;
		}

		let mut custody_bit_0_indices = Vec::new();
		let mut custody_bit_1_indices = Vec::new();
		for (i, validator_index) in slashable.validator_indices.iter().enumerate() {
			if !slashable.custody_bitfield.has_voted(i) {
				custody_bit_0_indices.push(validator_index);
			} else {
				custody_bit_1_indices.push(validator_index);
			}
		}

		bls_verify_multiple(
			&[
				bls_aggregate_pubkeys(&custody_bit_0_indices.iter().map(|i| self.validator_registry[**i as usize].pubkey).collect::<Vec<_>>()[..]),
				bls_aggregate_pubkeys(&custody_bit_1_indices.iter().map(|i| self.validator_registry[**i as usize].pubkey).collect::<Vec<_>>()[..]),
			],
			&[
				AttestationDataAndCustodyBit {
					data: slashable.data.clone(),
					custody_bit: false,
				}.hash::<Hasher>(),
				AttestationDataAndCustodyBit {
					data: slashable.data.clone(),
					custody_bit: true,
				}.hash::<Hasher>(),
			],
			&slashable.aggregate_signature,
			bls_domain(&self.fork, slot_to_epoch(slashable.data.slot), DOMAIN_ATTESTATION)
		)
	}
}
