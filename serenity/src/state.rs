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

use primitives::{H256, ValidatorId};
use ssz::Hashable;
use ssz_derive::Ssz;
use crate::eth1::{Eth1Data, Eth1DataVote, Deposit};
use crate::attestation::{PendingAttestation, Crosslink};
use crate::validator::Validator;
use crate::block::{BeaconBlock, BeaconBlockHeader};
use crate::consts::{
	SLOTS_PER_HISTORICAL_ROOT, LATEST_SLASHED_EXIT_LENGTH,
	LATEST_ACTIVE_INDEX_ROOTS_LENGTH, SHARD_COUNT,
	LATEST_RANDAO_MIXES_LENGTH, DOMAIN_DEPOSIT,
	ACTIVATION_EXIT_DELAY, MIN_SEED_LOOKAHEAD,
	GENESIS_EPOCH, GENESIS_START_SHARD, GENESIS_SLOT,
	GENESIS_FORK_VERSION, MIN_DEPOSIT_AMOUNT,
	MAX_DEPOSIT_AMOUNT,
};
use crate::error::Error;
use crate::util::{Hasher, bls_domain, slot_to_epoch, hash3, to_bytes};

#[derive(Ssz)]
pub struct BeaconState {
	// Misc
	pub slot: u64,
	pub genesis_time: u64,
	pub fork: Fork, // For versioning hard forks

	// Validator registry
	pub validator_registry: Vec<Validator>,
	pub validator_balances: Vec<u64>,
	pub validator_registry_update_epoch: u64,

	// Randomness and committees
	pub latest_randao_mixes: [H256; LATEST_RANDAO_MIXES_LENGTH],
	pub previous_shuffling_start_shard: u64,
	pub current_shuffling_start_shard: u64,
	pub previous_shuffling_epoch: u64,
	pub current_shuffling_epoch: u64,
	pub previous_shuffling_seed: H256,
	pub current_shuffling_seed: H256,

	// Finality
	pub previous_epoch_attestations: Vec<PendingAttestation>,
	pub current_epoch_attestations: Vec<PendingAttestation>,
	pub previous_justified_epoch: u64,
	pub justified_epoch: u64,
	pub justification_bitfield: u64,
	pub finalized_epoch: u64,

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
	pub fn current_epoch(&self) -> u64 {
		slot_to_epoch(self.slot)
	}

	pub fn previous_epoch(&self) -> u64 {
		self.current_epoch().saturating_sub(1)
	}

	pub fn delayed_activation_exit_epoch(&self) -> u64 {
		self.current_epoch() + 1 + ACTIVATION_EXIT_DELAY
	}

	pub fn randao_mix(&self, epoch: u64) -> Result<H256, Error> {
		if self.current_epoch().saturating_sub(LATEST_RANDAO_MIXES_LENGTH as u64) >= epoch ||
			epoch > self.current_epoch()
		{
			return Err(Error::EpochOutOfRange)
		}

		Ok(self.latest_randao_mixes[(epoch % LATEST_RANDAO_MIXES_LENGTH as u64) as usize])
	}

	pub fn active_index_root(&self, epoch: u64) -> Result<H256, Error> {
		if self.current_epoch().saturating_sub(
			LATEST_ACTIVE_INDEX_ROOTS_LENGTH as u64 - ACTIVATION_EXIT_DELAY
		) >= epoch || epoch > self.current_epoch() + ACTIVATION_EXIT_DELAY {
			return Err(Error::EpochOutOfRange)
		}

		Ok(self.latest_active_index_roots[(epoch % LATEST_ACTIVE_INDEX_ROOTS_LENGTH as u64) as usize])
	}

	pub fn seed(&self, epoch: u64) -> Result<H256, Error> {
		Ok(hash3(
			self.randao_mix(epoch.saturating_sub(MIN_SEED_LOOKAHEAD))?.as_ref(),
			self.active_index_root(epoch)?.as_ref(),
			to_bytes(epoch).as_ref()
		))
	}

	pub fn validator_by_id(&self, validator_id: &ValidatorId) -> Option<&Validator> {
		for validator in &self.validator_registry {
			if &validator.pubkey == validator_id {
				return Some(validator)
			}
		}

		None
	}

	fn effective_balance(&self, index: usize) -> u64 {
		core::cmp::min(self.validator_balances[index], MIN_DEPOSIT_AMOUNT)
	}

	fn activate_validator(&mut self, index: usize, is_genesis: bool) {
		let delayed_activation_exit_epoch = self.delayed_activation_exit_epoch();
		self.validator_registry[index].activate(delayed_activation_exit_epoch, is_genesis);
	}

	pub fn initiate_validator_exit(&mut self, index: usize) {
		self.validator_registry[index].initiate_exit();
	}

	pub fn exit_validator(&mut self, index: usize) {
		let delayed_activation_exit_epoch = self.delayed_activation_exit_epoch();
		self.validator_registry[index].exit(delayed_activation_exit_epoch);
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

	pub fn active_validator_indices(&self, epoch: u64) -> Vec<u64> {
		self.validator_registry.iter()
			.enumerate()
			.filter(|(_, v)| v.is_active(epoch))
			.map(|(i, _)| i as u64)
			.collect::<Vec<_>>()
	}

	pub fn genesis(deposits: Vec<Deposit>, genesis_time: u64, latest_eth1_data: Eth1Data) -> Result<Self, Error> {
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

		for validator_index in 0..state.validator_registry.len() {
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
}
