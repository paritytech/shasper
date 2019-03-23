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

use primitives::H256;
use ssz::Hashable;
use crate::state::BeaconState;
use crate::block::{BeaconBlock, BeaconBlockHeader};
use crate::eth1::{Eth1DataVote, Deposit};
use crate::validator::{VoluntaryExit, Transfer};
use crate::attestation::{
	Attestation, Crosslink, AttestationDataAndCustodyBit, PendingAttestation,
};
use crate::slashing::{ProposerSlashing, AttesterSlashing};
use crate::util::{
	Hasher, bls_verify, bls_domain, hash, slot_to_epoch, epoch_start_slot,
	bls_verify_multiple, bls_aggregate_pubkeys,
};
use crate::consts::{
	DOMAIN_BEACON_BLOCK, DOMAIN_RANDAO, LATEST_RANDAO_MIXES_LENGTH,
	GENESIS_SLOT, SLOTS_PER_EPOCH, MIN_ATTESTATION_INCLUSION_DELAY,
	DOMAIN_ATTESTATION, FAR_FUTURE_EPOCH, PERSISTENT_COMMITTEE_PERIOD,
	DOMAIN_DEPOSIT, DOMAIN_VOLUNTARY_EXIT, MIN_DEPOSIT_AMOUNT,
	BLS_WITHDRAWAL_PREFIX_BYTE, DOMAIN_TRANSFER
};
use crate::error::Error;

impl BeaconState {
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

		let current_epoch = self.current_epoch();
		self.latest_randao_mixes[(current_epoch % LATEST_RANDAO_MIXES_LENGTH as u64) as usize] = self.randao_mix(current_epoch)? ^ hash(&block.body.randao_reveal[..]);

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

	pub fn push_proposer_slashing(&mut self, proposer_slashing: ProposerSlashing) -> Result<(), Error> {
		if slot_to_epoch(proposer_slashing.header_a.slot) != slot_to_epoch(proposer_slashing.header_b.slot) {
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

	pub fn push_attestation(&mut self, attestation: Attestation) -> Result<(), Error> {
		if attestation.data.slot < GENESIS_SLOT {
			return Err(Error::AttestationTooFarInHistory)
		}

		if self.slot > attestation.data.slot + SLOTS_PER_EPOCH {
			return Err(Error::AttestationTooFarInHistory)
		}

		if attestation.data.slot + MIN_ATTESTATION_INCLUSION_DELAY > self.slot {
			return Err(Error::AttestationSubmittedTooQuickly)
		}

		if slot_to_epoch(attestation.data.slot) >= self.current_epoch() {
			if attestation.data.source_epoch != self.current_justified_epoch {
				return Err(Error::AttestationIncorrectJustifiedEpoch)
			}
		} else {
			if attestation.data.source_epoch != self.previous_justified_epoch {
				return Err(Error::AttestationIncorrectJustifiedEpoch)
			}
		}

		if attestation.data.source_root != self.block_root(epoch_start_slot(attestation.data.source_epoch))? {
			return Err(Error::AttestationIncorrectJustifiedBlockRoot)
		}

		if !(self.latest_crosslinks[attestation.data.shard as usize] == attestation.data.previous_crosslink || self.latest_crosslinks[attestation.data.shard as usize] == Crosslink { crosslink_data_root: attestation.data.crosslink_data_root, epoch: slot_to_epoch(attestation.data.slot) }) {
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

		if !bls_verify_multiple(
			&[
				bls_aggregate_pubkeys(&custody_bit_0_participants.iter().map(|i| self.validator_registry[*i as usize].pubkey).collect::<Vec<_>>()[..]).ok_or(Error::AttestationInvalidSignature)?,
				bls_aggregate_pubkeys(&custody_bit_1_participants.iter().map(|i| self.validator_registry[*i as usize].pubkey).collect::<Vec<_>>()[..]).ok_or(Error::AttestationInvalidSignature)?,
			],
			&[
				AttestationDataAndCustodyBit {
					data: attestation.data.clone(),
					custody_bit: false,
				}.hash::<Hasher>(),
				AttestationDataAndCustodyBit {
					data: attestation.data.clone(),
					custody_bit: true,
				}.hash::<Hasher>(),
			],
			&attestation.aggregate_signature,
			bls_domain(&self.fork, slot_to_epoch(attestation.data.slot), DOMAIN_ATTESTATION)
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
			inclusion_slot: self.slot,
		};

		if slot_to_epoch(attestation_data_slot) == self.current_epoch() {
			self.current_epoch_attestations.push(pending_attestation);
		} else if slot_to_epoch(attestation_data_slot) == self.previous_epoch() {
			self.previous_epoch_attestations.push(pending_attestation);
		}

		Ok(())
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

	pub fn push_voluntary_exit(&mut self, exit: VoluntaryExit) -> Result<(), Error> {
		{
			let validator = &self.validator_registry[exit.validator_index as usize];

			if validator.exit_epoch != FAR_FUTURE_EPOCH {
				return Err(Error::VoluntaryExitAlreadyExited)
			}

			if validator.initiated_exit {
				return Err(Error::VoluntaryExitAlreadyInitiated)
			}

			if self.current_epoch() < exit.epoch {
				return Err(Error::VoluntaryExitNotYetValid)
			}

			if self.current_epoch() - validator.activation_epoch < PERSISTENT_COMMITTEE_PERIOD {
				return Err(Error::VoluntaryExitNotLongEnough)
			}

			if !bls_verify(
				&validator.pubkey,
				&exit.truncated_hash::<Hasher>(),
				&exit.signature,
				bls_domain(&self.fork, exit.epoch, DOMAIN_VOLUNTARY_EXIT)
			) {
				return Err(Error::VoluntaryExitInvalidSignature)
			}
		}

		self.initiate_validator_exit(exit.validator_index);
		Ok(())
	}

	pub fn push_transfer(&mut self, transfer: Transfer) -> Result<(), Error> {
		if self.validator_balances[transfer.sender as usize] < core::cmp::max(transfer.amount, transfer.fee) {
			return Err(Error::TransferNoFund)
		}

		if !(self.validator_balances[transfer.sender as usize] == transfer.amount + transfer.fee || self.validator_balances[transfer.sender as usize] >= transfer.amount + transfer.fee + MIN_DEPOSIT_AMOUNT) {
			return Err(Error::TransferNoFund)
		}

		if self.slot != transfer.slot {
			return Err(Error::TransferNotValidSlot)
		}

		if !(self.current_epoch() >= self.validator_registry[transfer.sender as usize].withdrawable_epoch || self.validator_registry[transfer.sender as usize].activation_epoch == FAR_FUTURE_EPOCH) {
			return Err(Error::TransferNotWithdrawable)
		}

		if !(self.validator_registry[transfer.sender as usize].withdrawal_credentials[0] == BLS_WITHDRAWAL_PREFIX_BYTE && &self.validator_registry[transfer.sender as usize].withdrawal_credentials[1..] == &hash(&transfer.pubkey[..])[1..]) {
			return Err(Error::TransferInvalidPublicKey)
		}

		if !bls_verify(
			&transfer.pubkey,
			&transfer.truncated_hash::<Hasher>(),
			&transfer.signature,
			bls_domain(&self.fork, slot_to_epoch(transfer.slot), DOMAIN_TRANSFER)
		) {
			return Err(Error::TransferInvalidSignature)
		}

		self.validator_balances[transfer.sender as usize] -= transfer.amount + transfer.fee;
		self.validator_balances[transfer.recipient as usize] += transfer.amount;
		let proposer_index = self.beacon_proposer_index(self.slot, false)?;
		self.validator_balances[proposer_index as usize] += transfer.fee;

		Ok(())
	}
}
