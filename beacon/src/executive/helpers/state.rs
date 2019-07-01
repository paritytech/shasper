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

use core::cmp::{min, max};
use ssz::{Digestible, Fixed};
use crate::primitives::{Uint, Epoch, Slot, ValidatorIndex, Gwei, Shard, H256, BitField};
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit, CompactCommittee, Checkpoint};
use crate::utils::{self, to_bytes};
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Get the current state epoch.
	pub fn current_epoch(&self) -> Epoch {
		self.config.slot_to_epoch(self.state.slot)
	}

	/// Get the previous state epoch.
	pub fn previous_epoch(&self) -> Epoch {
		let current_epoch = self.current_epoch();
		if current_epoch == self.config.genesis_epoch() {
			self.config.genesis_epoch()
		} else {
			current_epoch - 1
		}
	}

	/// Get the block root at slot.
	pub fn block_root_at_slot(&self, slot: Slot) -> Result<H256, Error> {
		if !(slot < self.state.slot &&
			 self.state.slot <= slot + self.config.slots_per_historical_root())
		{
			return Err(Error::SlotOutOfRange)
		}

		Ok(self.state.latest_block_roots[
			(slot % self.config.slots_per_historical_root()) as usize
		])
	}

	/// Get the block root at epoch start slot.
	pub fn block_root(&self, epoch: Epoch) -> Result<H256, Error> {
		self.block_root_at_slot(self.config.epoch_start_slot(epoch))
	}

	pub(crate) fn randao_mix(&self, epoch: Epoch) -> H256 {
		// `epoch` expected to be between
		// (current_epoch - LATEST_RANDAO_MIXES_LENGTH, current_epoch].
		self.state.latest_randao_mixes[
			(epoch % self.config.latest_randao_mixes_length()) as usize
		]
	}

	/// Get active validator indices at epoch.
	pub fn active_validator_indices(&self, epoch: Uint) -> Vec<ValidatorIndex> {
		self.state.validator_registry
			.iter()
			.enumerate()
			.filter(move |(_, v)| v.is_active(epoch))
			.map(|(i, _)| i as u64)
			.collect()
	}

	/// Return the churn limit based on the active validator count.
	pub(crate) fn validator_churn_limit(&self) -> Uint {
		max(
			self.config.min_per_epoch_churn_limit(),
			self.active_validator_indices(self.current_epoch()).len() as u64 /
				self.config.churn_limit_quotient()
		)
	}

	pub(crate) fn seed(&self, epoch: Epoch) -> H256 {
		self.config.hash(&[
			&self.randao_mix(epoch +
							 self.config.latest_randao_mixes_length() -
							 self.config.min_seed_lookahead())[..],
			&self.active_index_root(epoch)[..],
			&to_bytes(epoch)[..],
		])
	}

	pub(crate) fn committee_count(&self, epoch: Epoch) -> Uint {
		let active_validator_indices = self.active_validator_indices(epoch);
		max(
			1,
			min(
				self.config.shard_count() / self.config.slots_per_epoch(),
				active_validator_indices.len() as u64 /
					self.config.slots_per_epoch() /
					self.config.target_committee_size(),
			)
		) * self.config.slots_per_epoch()
	}

	pub(crate) fn crosslink_committee(
		&self, epoch: Epoch, shard: Shard
	) -> Result<Vec<ValidatorIndex>, Error> {
		let indices = self.active_validator_indices(epoch);
		let seed = self.generate_seed(epoch);
		let index = (shard +
					 self.config.shard_count() - self.epoch_start_shard(epoch)?) %
			self.config.shard_count();
		let count = self.epoch_committee_count(epoch);

		self.compute_committee(&indices, seed, index, count)
	}

	pub(crate) fn start_shard(&self, epoch: Epoch) -> Result<Shard, Error> {
		if !(epoch <= self.current_epoch() + 1) {
			return Err(Error::EpochOutOfRange)
		}

		let mut check_epoch = self.current_epoch() + 1;
		let mut shard = (self.state.latest_start_shard +
						 self.shard_delta(self.current_epoch())) %
			self.config.shard_count();

		while check_epoch > epoch {
			check_epoch -= 1;
			shard = (shard + self.config.shard_count() -
					 self.shard_delta(check_epoch)) %
				self.config.shard_count();
		}

		Ok(shard)
	}

	pub(crate) fn shard_delta(&self, epoch: Epoch) -> Uint {
		min(
			self.epoch_committee_count(epoch),
			self.config.shard_count() -
				self.config.shard_count() / self.config.slots_per_epoch()
		)
	}

	/// Find the current beacon block proposer index.
	pub fn beacon_proposer_index(&self) -> Result<ValidatorIndex, Error> {
		let epoch = self.current_epoch();
		let committees_per_slot =
			self.epoch_committee_count(epoch) / self.config.slots_per_epoch();
		let offset = committees_per_slot *
			(self.state.slot % self.config.slots_per_epoch());
		let shard = (self.epoch_start_shard(epoch)? + offset) %
			self.config.shard_count();
		let first_committee = self.crosslink_committee(epoch, shard)?;
		let seed = self.generate_seed(epoch);

		let mut i = 0;
		loop {
			let candidate_index = first_committee[
				((epoch + i) % first_committee.len() as u64) as usize
			];
			let random_byte = self.config.hash(&[
				&seed[..],
				&to_bytes(i / 32)[..],
			])[(i % 32) as usize];
			let effective_balance = self.state
				.validator_registry[candidate_index as usize].effective_balance;
			if effective_balance * u8::max_value() as u64 >=
				self.config.max_effective_balance() * random_byte as u64
			{
				return Ok(candidate_index)
			}

			i+= 1
		}
	}

	pub(crate) fn attestation_data_slot(&self, attestation: &AttestationData) -> Result<Slot, Error> {
		let committee_count = self.epoch_committee_count(
			attestation.target_epoch
		);
		let offset = (attestation.crosslink.shard + self.config.shard_count() -
					  self.epoch_start_shard(attestation.target_epoch)?) %
			self.config.shard_count();

		Ok(self.config.epoch_start_slot(attestation.target_epoch) +
		   offset / (committee_count / self.config.slots_per_epoch()))
	}

	pub(crate) fn compact_committees_root(&self, epoch: Uint) -> H256 {
		let mut committees = Vec::new();
		committees.resize(self.config.shard_count(), CompactCommittee::default());
		let start_shard = self.start_shard(epoch);

		for committee_number in 0..self.committee_count(epoch) {
			let shard = (start_shard + committee_number) % self.config.shard_Count();
			for index in self.crosslink_committee(epoch, shard) {
				let validator = self.state.validators[index];
				committees[shard].pubkeys.append(validator.pubkey);
				let compact_balance = validator.effective_balance / self.config.effective_balance_increment();
				let compact_validator = (index << 16) + (if validator.slashed { 1 } else { 0 } << 15) + compact_balance;
				committees[shard].compact_validators.append(compact_validator);
			}
		}

		H256::from_slice(
			Digestible::<C::Digest>::hash(Fixed(committees)).as_slice()
		)
	}

	pub(crate) fn total_balance(&self, indices: &[ValidatorIndex]) -> Gwei {
		max(
			indices.iter().fold(0, |sum, index| {
				sum + self.state.validator_registry[*index as usize].effective_balance
			}),
			1
		)
	}

	pub(crate) fn total_active_balance(&self) -> Gwei {
		self.total_balance(self.active_validator_indices(self.current_epoch()))
	}

	/// Find domain integer of type and epoch.
	pub fn domain(&self, domain_type: Uint, message_epoch: Option<Uint>) -> Uint {
		let epoch = message_epoch.unwrap_or(self.current_epoch());
		let fork_version = if epoch < self.state.fork.epoch {
			self.state.fork.previous_version
		} else {
			self.state.fork.current_version
		};

		utils::bls_domain(domain_type, fork_version)
	}

	pub(crate) fn indexed_attestation(&self, attestation: Attestation) -> Result<IndexedAttestation, Error> {
		let attesting_indices = self.attesting_indices(
			&attestation.data, &attestation.aggregation_bitfield
		)?;
		let custody_bit_1_indices = self.attesting_indices(
			&attestation.data, &attestation.custody_bitfield
		)?;
		let custody_bit_0_indices = attesting_indices.clone()
			.into_iter()
			.filter(|index| !custody_bit_1_indices.contains(index))
			.collect();

		Ok(IndexedAttestation {
			data: attestation.data,
			signature: attestation.signature,
			custody_bit_0_indices, custody_bit_1_indices,
		})
	}

	pub(crate) fn attesting_indices(
		&self, attestation_data: &AttestationData, bitfield: &BitField,
	) -> Result<Vec<ValidatorIndex>, Error> {
		let committee = self.crosslink_committee(
			attestation_data.target_epoch, attestation_data.crosslink.shard
		)?;
		if !bitfield.verify(committee.len()) {
			return Err(Error::AttestationBitFieldInvalid);
		}

		let mut ret = committee.into_iter()
			.enumerate()
			.filter(|(i, _)| bitfield.get_bit(*i) == true)
			.map(|(_, val)| val)
			.collect::<Vec<_>>();
		ret.sort();
		Ok(ret)
	}
}
