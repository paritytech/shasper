use ssz::Hashable;

use super::Executive;
use crate::{
	Config, Error, BeaconBlockHeader, Transfer, VoluntaryExit, Validator, Deposit, PendingAttestation,
	AttestationDataAndCustodyBit, Crosslink, Attestation, AttesterSlashing, ProposerSlashing,
	Eth1DataVote, SlashableAttestation, ValidatorIndex, Block,
};
use crate::primitives::H256;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	fn slash_validator(&mut self, index: ValidatorIndex) -> Result<(), Error> {
		if self.state.slot >= self.config.epoch_start_slot(self.state.validator_registry[index as usize].withdrawable_epoch) {
			return Err(Error::ValidatorNotWithdrawable);
		}
		self.exit_validator(index);

		let current_epoch = self.current_epoch();
		self.state.latest_slashed_balances[(current_epoch % self.config.latest_slashed_exit_length() as u64) as usize] += self.effective_balance(index);

		let whistleblower_index = self.beacon_proposer_index(self.state.slot, false)?;
		let whistleblower_reward = self.effective_balance(index) / self.config.whistleblower_reward_quotient();
		self.state.validator_balances[whistleblower_index as usize] += whistleblower_reward;
		self.state.validator_balances[index as usize] -= whistleblower_reward;
		self.state.validator_registry[index as usize].slashed = true;
		self.state.validator_registry[index as usize].withdrawable_epoch = self.current_epoch() + self.config.latest_slashed_exit_length() as u64;

		Ok(())
	}

	/// Process a block header.
	pub fn process_block_header<B: Block + Hashable<C::Hasher>>(&mut self, block: &B) -> Result<(), Error> {
		if block.slot() != self.state.slot {
			return Err(Error::BlockSlotInvalid)
		}

		if block.previous_block_root() != &Hashable::<C::Hasher>::truncated_hash(&self.state.latest_block_header) {
			return Err(Error::BlockPreviousRootInvalid)
		}

		self.state.latest_block_header = BeaconBlockHeader::with_state_root_no_signature::<_, C::Hasher>(block, H256::default());

		if let Some(signature) = block.signature() {
			let proposer = &self.state.validator_registry[self.beacon_proposer_index(self.state.slot, false)? as usize];

			if !self.config.bls_verify(&proposer.pubkey, &Hashable::<C::Hasher>::truncated_hash(block), signature, self.config.domain_id(&self.state.fork, self.current_epoch(), self.config.domain_beacon_block())) {
				return Err(Error::BlockSignatureInvalid)
			}
		}

		Ok(())
	}

	/// Process randao information given in a block.
	pub fn process_randao<B: Block>(&mut self, block: &B) -> Result<(), Error> {
		let proposer = &self.state.validator_registry[self.beacon_proposer_index(self.state.slot, false)? as usize];

		if !self.config.bls_verify(&proposer.pubkey, &Hashable::<C::Hasher>::hash(&self.current_epoch()), &block.body().randao_reveal, self.config.domain_id(&self.state.fork, self.current_epoch(), self.config.domain_randao())) {
			return Err(Error::RandaoSignatureInvalid)
		}

		let current_epoch = self.current_epoch();
		self.state.latest_randao_mixes[(current_epoch % self.config.latest_randao_mixes_length() as u64) as usize] = self.randao_mix(current_epoch)? ^ self.config.hash(&block.body().randao_reveal[..]);

		Ok(())
	}

	/// Process eth1 data vote given in a block.
	pub fn process_eth1_data<B: Block>(&mut self, block: &B) {
		for eth1_data_vote in &mut self.state.eth1_data_votes {
			if eth1_data_vote.eth1_data == block.body().eth1_data {
				eth1_data_vote.vote_count += 1;
				return
			}
		}

		self.state.eth1_data_votes.push(Eth1DataVote {
			eth1_data: block.body().eth1_data.clone(),
			vote_count: 1
		});
	}

	/// Push a new `ProposerSlashing` to the state.
	pub fn push_proposer_slashing(&mut self, proposer_slashing: ProposerSlashing) -> Result<(), Error> {
		if self.config.slot_to_epoch(proposer_slashing.header_a.slot) != self.config.slot_to_epoch(proposer_slashing.header_b.slot) {
			return Err(Error::ProposerSlashingInvalidSlot)
		}

		if proposer_slashing.header_a == proposer_slashing.header_b {
			return Err(Error::ProposerSlashingSameHeader)
		}

		{
			let proposer = &self.state.validator_registry[proposer_slashing.proposer_index as usize];

			if proposer.slashed {
				return Err(Error::ProposerSlashingAlreadySlashed)
			}

			for header in [&proposer_slashing.header_a, &proposer_slashing.header_b].into_iter() {
				if !self.config.bls_verify(&proposer.pubkey, &Hashable::<C::Hasher>::truncated_hash(*header), &header.signature, self.config.domain_id(&self.state.fork, self.config.slot_to_epoch(header.slot), self.config.domain_beacon_block())) {
					return Err(Error::ProposerSlashingInvalidSignature)
				}
			}
		}

		self.slash_validator(proposer_slashing.proposer_index)
	}

	fn verify_slashable_attestation(&self, slashable: &SlashableAttestation) -> bool {
		for bit in &slashable.custody_bitfield.0 {
			if *bit != 0 {
				return false;
			}
		}

		if slashable.validator_indices.len() == 0 {
			return false;
		}

		for i in 0..(slashable.validator_indices.len() - 1) {
			if slashable.validator_indices[i] > slashable.validator_indices[i + 1] {
				return false;
			}
		}

		if !slashable.custody_bitfield.verify(slashable.validator_indices.len()) {
			return false;
		}

		if slashable.validator_indices.len() > self.config.max_indices_per_slashable_vote() {
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

		self.config.bls_verify_multiple(
			&[
				match self.config.bls_aggregate_pubkeys(&custody_bit_0_indices.iter().map(|i| self.state.validator_registry[**i as usize].pubkey).collect::<Vec<_>>()[..]) {
					Some(k) => k,
					None => return false,
				},
				match self.config.bls_aggregate_pubkeys(&custody_bit_1_indices.iter().map(|i| self.state.validator_registry[**i as usize].pubkey).collect::<Vec<_>>()[..]) {
					Some(k) => k,
					None => return false,
				},
			],
			&[
				Hashable::<C::Hasher>::hash(&AttestationDataAndCustodyBit {
					data: slashable.data.clone(),
					custody_bit: false,
				}),
				Hashable::<C::Hasher>::hash(&AttestationDataAndCustodyBit {
					data: slashable.data.clone(),
					custody_bit: true,
				}),
			],
			&slashable.aggregate_signature,
			self.config.domain_id(&self.state.fork, self.config.slot_to_epoch(slashable.data.slot), self.config.domain_attestation())
		)
	}

	/// Push a new `AttesterSlashing` to the state.
	pub fn push_attester_slashing(&mut self, attester_slashing: AttesterSlashing) -> Result<(), Error> {
		let attestation1 = attester_slashing.slashable_attestation_a;
		let attestation2 = attester_slashing.slashable_attestation_b;

		if attestation1.data == attestation2.data {
			return Err(Error::AttesterSlashingSameAttestation)
		}

		if !(attestation1.data.is_double_vote(&attestation2.data, self.config) || attestation1.data.is_surround_vote(&attestation2.data, self.config)) {
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
			if attestation2.validator_indices.contains(index) && !self.state.validator_registry[*index as usize].slashed {
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

	/// Push a new `Attestation` to the state.
	pub fn push_attestation(&mut self, attestation: Attestation) -> Result<(), Error> {
		if attestation.data.slot < self.config.genesis_slot() {
			return Err(Error::AttestationTooFarInHistory)
		}

		if self.state.slot > attestation.data.slot + self.config.slots_per_epoch() {
			return Err(Error::AttestationTooFarInHistory)
		}

		if attestation.data.slot > self.state.slot - self.config.min_attestation_inclusion_delay() {
			return Err(Error::AttestationSubmittedTooQuickly)
		}

		let target_epoch = self.config.slot_to_epoch(attestation.data.slot);
		let is_target_current_epoch = target_epoch == self.current_epoch() &&
			attestation.data.source_epoch == self.state.current_justified_epoch &&
			attestation.data.source_root == self.state.current_justified_root;
		let is_target_previous_epoch = target_epoch == self.previous_epoch() &&
			attestation.data.source_epoch == self.state.previous_justified_epoch &&
			attestation.data.source_root == self.state.previous_justified_root;

		if !is_target_current_epoch && !is_target_previous_epoch {
			return Err(Error::AttestationIncorrectJustifiedEpochOrBlockRoot)
		}

		if !(self.state.latest_crosslinks[attestation.data.shard as usize] == attestation.data.previous_crosslink || self.state.latest_crosslinks[attestation.data.shard as usize] == Crosslink { crosslink_data_root: attestation.data.crosslink_data_root, epoch: self.config.slot_to_epoch(attestation.data.slot) }) {
			return Err(Error::AttestationIncorrectCrosslinkData)
		}

		if attestation.aggregation_bitfield.0.len() == 0 {
			return Err(Error::AttestationEmptyAggregation)
		}

		if attestation.custody_bitfield.0.len() == 0 {
			return Err(Error::AttestationEmptyCustody)
		}

		let crosslink_committee = self.crosslink_committees_at_slot(attestation.data.slot, false)?
			.into_iter()
			.filter(|(_, s)| s == &attestation.data.shard)
			.map(|(c, _)| c)
			.next()
			.ok_or(Error::AttestationInvalidShard)?;

		for i in 0..crosslink_committee.len() {
			if !attestation.aggregation_bitfield.has_voted(i) {
				if attestation.custody_bitfield.has_voted(i) {
					return Err(Error::AttestationInvalidCustody)
				}
			}
		}

		let participants = self.attestation_participants(&attestation.data, &attestation.aggregation_bitfield)?;
		let custody_bit_1_participants = self.attestation_participants(&attestation.data, &attestation.custody_bitfield)?;
		let custody_bit_0_participants = participants.clone().into_iter().filter(|p| !custody_bit_1_participants.contains(p)).collect::<Vec<_>>();

		if !self.config.bls_verify_multiple(
			&[
				self.config.bls_aggregate_pubkeys(&custody_bit_0_participants.iter().map(|i| self.state.validator_registry[*i as usize].pubkey).collect::<Vec<_>>()[..]).ok_or(Error::AttestationInvalidSignature)?,
				self.config.bls_aggregate_pubkeys(&custody_bit_1_participants.iter().map(|i| self.state.validator_registry[*i as usize].pubkey).collect::<Vec<_>>()[..]).ok_or(Error::AttestationInvalidSignature)?,
			],
			&[
				Hashable::<C::Hasher>::hash(&AttestationDataAndCustodyBit {
					data: attestation.data.clone(),
					custody_bit: false,
				}),
				Hashable::<C::Hasher>::hash(&AttestationDataAndCustodyBit {
					data: attestation.data.clone(),
					custody_bit: true,
				}),
			],
			&attestation.aggregate_signature,
			self.config.domain_id(&self.state.fork, self.config.slot_to_epoch(attestation.data.slot), self.config.domain_attestation())
		) {
			return Err(Error::AttestationInvalidSignature)
		}

		if attestation.data.crosslink_data_root != H256::default() {
			return Err(Error::AttestationInvalidCrosslink)
		}

		let attestation_data_slot = attestation.data.slot;
		let pending_attestation = PendingAttestation {
			data: attestation.data,
			aggregation_bitfield: attestation.aggregation_bitfield,
			custody_bitfield: attestation.custody_bitfield,
			inclusion_slot: self.state.slot,
		};

		if self.config.slot_to_epoch(attestation_data_slot) == self.current_epoch() {
			self.state.current_epoch_attestations.push(pending_attestation);
		} else if self.config.slot_to_epoch(attestation_data_slot) == self.previous_epoch() {
			self.state.previous_epoch_attestations.push(pending_attestation);
		}

		Ok(())
	}

	/// Push a new `Deposit` to the state.
	pub fn push_deposit(&mut self, deposit: Deposit) -> Result<(), Error> {
		if deposit.index != self.state.deposit_index {
			return Err(Error::DepositIndexMismatch)
		}

		if !deposit.is_merkle_valid(&self.state.latest_eth1_data.deposit_root, self.config) {
			return Err(Error::DepositMerkleInvalid)
		}

		self.state.deposit_index += 1;

		match self.state.validator_index_by_id(&deposit.deposit_data.deposit_input.pubkey) {
			Some(index) => {
				self.state.validator_balances[index as usize] += deposit.deposit_data.amount;
			},
			None => {
				if !deposit.is_proof_valid(
					self.config.domain_id(&self.state.fork, self.current_epoch(), self.config.domain_deposit()),
					self.config,
				) {
					return Ok(())
				}

				let validator = Validator {
					pubkey: deposit.deposit_data.deposit_input.pubkey,
					withdrawal_credentials: deposit.deposit_data.deposit_input.withdrawal_credentials,
					activation_epoch: self.config.far_future_epoch(),
					exit_epoch: self.config.far_future_epoch(),
					withdrawable_epoch: self.config.far_future_epoch(),
					initiated_exit: false,
					slashed: false,
				};

				self.state.validator_registry.push(validator);
				self.state.validator_balances.push(deposit.deposit_data.amount);
			},
		}

		Ok(())
	}

	/// Push a new `VoluntaryExit` to the state.
	pub fn push_voluntary_exit(&mut self, exit: VoluntaryExit) -> Result<(), Error> {
		{
			let validator = &self.state.validator_registry[exit.validator_index as usize];

			if validator.exit_epoch != self.config.far_future_epoch() {
				return Err(Error::VoluntaryExitAlreadyExited)
			}

			if validator.initiated_exit {
				return Err(Error::VoluntaryExitAlreadyInitiated)
			}

			if self.current_epoch() < exit.epoch {
				return Err(Error::VoluntaryExitNotYetValid)
			}

			if self.current_epoch() - validator.activation_epoch < self.config.persistent_committee_period() {
				return Err(Error::VoluntaryExitNotLongEnough)
			}

			if !self.config.bls_verify(
				&validator.pubkey,
				&Hashable::<C::Hasher>::truncated_hash(&exit),
				&exit.signature,
				self.config.domain_id(&self.state.fork, exit.epoch, self.config.domain_voluntary_exit())
			) {
				return Err(Error::VoluntaryExitInvalidSignature)
			}
		}

		self.initiate_validator_exit(exit.validator_index);
		Ok(())
	}

	/// Push a new `Transfer` to the state.
	pub fn push_transfer(&mut self, transfer: Transfer) -> Result<(), Error> {
		if self.state.validator_balances[transfer.sender as usize] < core::cmp::max(transfer.amount, transfer.fee) {
			return Err(Error::TransferNoFund)
		}

		if !(self.state.validator_balances[transfer.sender as usize] == transfer.amount + transfer.fee || self.state.validator_balances[transfer.sender as usize] >= transfer.amount + transfer.fee + self.config.min_deposit_amount()) {
			return Err(Error::TransferNoFund)
		}

		if self.state.slot != transfer.slot {
			return Err(Error::TransferNotValidSlot)
		}

		if !(self.current_epoch() >= self.state.validator_registry[transfer.sender as usize].withdrawable_epoch || self.state.validator_registry[transfer.sender as usize].activation_epoch == self.config.far_future_epoch()) {
			return Err(Error::TransferNotWithdrawable)
		}

		if !(self.state.validator_registry[transfer.sender as usize].withdrawal_credentials[0] == self.config.bls_withdrawal_prefix_byte() && &self.state.validator_registry[transfer.sender as usize].withdrawal_credentials[1..] == &self.config.hash(&transfer.pubkey[..])[1..]) {
			return Err(Error::TransferInvalidPublicKey)
		}

		if !self.config.bls_verify(
			&transfer.pubkey,
			&Hashable::<C::Hasher>::truncated_hash(&transfer),
			&transfer.signature,
			self.config.domain_id(&self.state.fork, self.config.slot_to_epoch(transfer.slot), self.config.domain_transfer())
		) {
			return Err(Error::TransferInvalidSignature)
		}

		self.state.validator_balances[transfer.sender as usize] -= transfer.amount + transfer.fee;
		self.state.validator_balances[transfer.recipient as usize] += transfer.amount;
		let proposer_index = self.beacon_proposer_index(self.state.slot, false)?;
		self.state.validator_balances[proposer_index as usize] += transfer.fee;

		Ok(())
	}
}
