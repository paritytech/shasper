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
use ssz::Digestible;
use crate::primitives::{Uint, Epoch, Slot, ValidatorIndex, Gwei, Shard, H256, BitField};
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit};
use crate::utils::to_bytes;
use crate::{Config, Executive, Error};

mod validator;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	fn current_epoch(&self) -> Epoch {
		self.config.slot_to_epoch(self.state.slot)
	}

	fn previous_epoch(&self) -> Epoch {
		let current_epoch = self.current_epoch();
		if current_epoch > self.config.genesis_epoch() {
			current_epoch.saturating_sub(1)
		} else {
			current_epoch
		}
	}

	fn active_validator_indices(&self, epoch: Uint) -> Vec<ValidatorIndex> {
		self.state.validator_registry
			.iter()
			.enumerate()
			.filter(move |(_, v)| v.is_active(epoch))
			.map(|(i, _)| i as u64)
			.collect()
	}

	fn increase_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
		self.state.balances[index as usize] += delta;
	}

	fn decrease_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
		self.state.balances[index as usize] =
			self.state.balances[index as usize].saturating_sub(delta);
	}

	fn epoch_committee_count(&self, epoch: Epoch) -> Uint {
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

	fn shard_delta(&self, epoch: Epoch) -> Uint {
		min(
			self.epoch_committee_count(epoch),
			self.config.shard_count() -
				self.config.shard_count() / self.config.slots_per_epoch()
		)
	}

	fn epoch_start_shard(&self, epoch: Epoch) -> Result<Shard, Error> {
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

	fn attestation_slot(&self, attestation: &Attestation) -> Result<Slot, Error> {
		let epoch = attestation.data.target_epoch;
		let committee_count = self.epoch_committee_count(epoch);
		let offset = (attestation.data.shard + self.config.shard_count() -
					  self.epoch_start_shard(epoch)?) %
			self.config.shard_count();

		Ok(self.config.epoch_start_slot(epoch) +
		   offset / (committee_count / self.config.slots_per_epoch()))
	}

	fn block_root_at_slot(&self, slot: Slot) -> Result<H256, Error> {
		if !(slot < self.state.slot &&
			 self.state.slot <= slot + self.config.slots_per_historical_root())
		{
			return Err(Error::SlotOutOfRange)
		}

		Ok(self.state.latest_block_roots[
			(slot % self.config.slots_per_historical_root()) as usize
		])
	}

	fn block_root(&self, epoch: Epoch) -> Result<H256, Error> {
		self.block_root_at_slot(self.config.epoch_start_slot(epoch))
	}

	fn randao_mix(&self, epoch: Epoch) -> H256 {
		// `epoch` expected to be between
		// (current_epoch - LATEST_RANDAO_MIXES_LENGTH, current_epoch].
		self.state.latest_randao_mixes[
			(epoch % self.config.latest_randao_mixes_length()) as usize
		]
	}

	fn active_index_root(&self, epoch: Epoch) -> H256 {
		self.state.latest_active_index_roots[
			(epoch % self.config.latest_active_index_roots_length()) as usize
		]
	}

	fn generate_seed(&self, epoch: Epoch) -> H256 {
		self.config.hash(&[
			&self.randao_mix(epoch +
							 self.config.latest_randao_mixes_length() -
							 self.config.min_seed_lookahead())[..],
			&self.active_index_root(epoch)[..],
			&to_bytes(epoch)[..],
		])
	}

	fn beacon_proposer_index(&self) -> Result<ValidatorIndex, Error> {
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

	fn crosslink_committee(
		&self, epoch: Epoch, shard: Shard
	) -> Result<Vec<ValidatorIndex>, Error> {
		let indices = self.active_validator_indices(epoch);
		let seed = self.generate_seed(epoch);
		let index = (shard +
					 self.config.shard_count() - self.epoch_start_shard(epoch)?) %
			self.config.shard_count();
		let count = self.epoch_committee_count(epoch);

		let start = (indices.len() as u64 * index) / count;
		let end = (indices.len() as u64 * (index + 1)) / count;

		(start..end).into_iter().map(move |i| {
			Ok(indices[
				self.config.shuffled_index(i, indices.len() as u64, seed)
					.ok_or(Error::IndexOutOfRange)? as usize
			])
		}).collect::<Result<Vec<_>, Error>>()
	}

	fn attesting_indices(
		&self, attestation_data: &AttestationData, bitfield: &BitField,
	) -> Result<Vec<ValidatorIndex>, Error> {
		let committee = self.crosslink_committee(
			attestation_data.target_epoch, attestation_data.shard
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

	fn total_balance(&self, indices: &[ValidatorIndex]) -> Gwei {
		indices.iter().fold(0, |sum, index| {
			sum + self.state.validator_registry[*index as usize].effective_balance
		})
	}

	fn domain(&self, domain_type: Uint, message_epoch: Option<Uint>) -> Uint {
		let epoch = message_epoch.unwrap_or(self.current_epoch());
		let fork_version = if epoch < self.state.fork.epoch {
			self.state.fork.previous_version
		} else {
			self.state.fork.current_version
		};

		let mut bytes = [0u8; 8];
		(&mut bytes[0..4]).copy_from_slice(fork_version.as_ref());
		(&mut bytes[4..8]).copy_from_slice(&domain_type.to_le_bytes()[0..4]);

		u64::from_le_bytes(bytes)
	}

	fn covert_to_indexed(&self, attestation: Attestation) -> Result<IndexedAttestation, Error> {
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

	fn verify_indexed_attestation(&self, indexed_attestation: &IndexedAttestation) -> Result<bool, Error> {
		let custody_bit_0_indices = &indexed_attestation.custody_bit_0_indices;
		let custody_bit_1_indices = &indexed_attestation.custody_bit_1_indices;

		// Ensure no duplicate indices across custody bits
		for index in custody_bit_0_indices {
			if custody_bit_1_indices.contains(index) {
				return Err(Error::DuplicateIndexes)
			}
		}

		if custody_bit_1_indices.len() > 0 {
			return Ok(false)
		}

		let total_len =
			(custody_bit_0_indices.len() + custody_bit_1_indices.len()) as u64;
		if !(1 <= total_len && total_len <= self.config.max_indices_per_attestation()) {
			return Ok(false)
		}

		if !custody_bit_0_indices.windows(2).all(|w| w[0] <= w[1]) {
			return Ok(false)
		}

		if !custody_bit_1_indices.windows(2).all(|w| w[0] <= w[1]) {
			return Ok(false)
		}

		Ok(self.config.bls_verify_multiple(
			&[
				self.config.bls_aggregate_pubkeys(
					&custody_bit_0_indices
						.iter()
						.map(|i| self.state.validator_registry[*i as usize].pubkey)
						.collect::<Vec<_>>()[..]
				),
				self.config.bls_aggregate_pubkeys(
					&custody_bit_1_indices
						.iter()
						.map(|i| self.state.validator_registry[*i as usize].pubkey)
						.collect::<Vec<_>>()[..]
				),
			],
			&[
				H256::from_slice(
					Digestible::<C::Digest>::hash(&AttestationDataAndCustodyBit {
						data: indexed_attestation.data.clone(),
						custody_bit: false,
					}).as_slice()
				),
				H256::from_slice(
					Digestible::<C::Digest>::hash(&AttestationDataAndCustodyBit {
						data: indexed_attestation.data.clone(),
						custody_bit: true,
					}).as_slice()
				),
			],
			&indexed_attestation.signature,
			self.domain(
				self.config.domain_attestation(),
				Some(indexed_attestation.data.target_epoch)
			)
		))
	}

	fn churn_limit(&self) -> Uint {
		max(
			self.config.min_per_epoch_churn_limit(),
			self.active_validator_indices(self.current_epoch()).len() as u64 /
				self.config.churn_limit_quotient()
		)
	}
}
