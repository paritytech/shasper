// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.

use crate::types::{AttestationData, Attestation, IndexedAttestation};
use crate::primitives::{Epoch, H256, Uint, ValidatorIndex, Gwei, Slot};
use crate::{BeaconExecutive, Config, Error, utils};
use core::cmp::{max, min};

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Get current epoch.
	pub fn current_epoch(&self) -> Epoch {
		utils::epoch_of_slot::<C>(self.slot)
	}

	/// Get previous epoch.
	pub fn previous_epoch(&self) -> Epoch {
		let current_epoch = self.current_epoch();
		if current_epoch == C::genesis_epoch() {
			C::genesis_epoch()
		} else {
			current_epoch.saturating_sub(1)
		}
	}

	/// Get block root of the start slot of an epoch.
	pub fn block_root(&self, epoch: Epoch) -> Result<H256, Error> {
		self.block_root_at_slot(utils::start_slot_of_epoch::<C>(epoch))
	}

	/// Get block root at slot.
	pub fn block_root_at_slot(&self, slot: Slot) -> Result<H256, Error> {
		if !(slot < self.slot &&
			 self.slot <= slot + C::slots_per_historical_root())
		{
			return Err(Error::SlotOutOfRange)
		}

		Ok(self.block_roots[
			(slot % C::slots_per_historical_root()) as usize
		])
	}

	/// Get the randao mix at epoch.
	pub fn randao_mix(&self, epoch: Epoch) -> H256 {
		self.randao_mixes[
			(epoch % C::epochs_per_historical_vector()) as usize
		]
	}

	/// Get active validator indices at epoch.
	pub fn active_validator_indices(&self, epoch: Uint) -> Vec<ValidatorIndex> {
		self.validators
			.iter()
			.enumerate()
			.filter(move |(_, v)| v.is_active(epoch))
			.map(|(i, _)| i as u64)
			.collect()
	}

	/// Get active validator length.
	pub fn active_validator_len(&self, epoch: Uint) -> usize {
		self.active_validator_indices(epoch).len()
	}

	/// Get churn limit for validator exits.
	pub fn validator_churn_limit(&self) -> Uint {
		max(
			C::min_per_epoch_churn_limit(),
			self.active_validator_len(self.current_epoch()) as u64 /
				C::churn_limit_quotient()
		)
	}

	/// Get the random seed for epoch.
	pub fn seed(&self, epoch: Epoch, domain_type: u32) -> H256 {
		C::hash(&[
			&domain_type.to_le_bytes()[..],
			&epoch.to_le_bytes()[..8],
			&self.randao_mix(epoch +
							 C::epochs_per_historical_vector() -
							 C::min_seed_lookahead() - 1)[..],
		])
	}

	/// Get committee count for epoch.
	pub fn committee_count_at_slot(&self, slot: Uint) -> Uint {
		let epoch = utils::epoch_of_slot::<C>(slot);
		let active_validator_len = self.active_validator_len(epoch);
		max(
			1,
			min(
				C::max_committees_per_slot(),
				active_validator_len() as u64 /
					C::slots_per_epoch() /
					C::target_committee_size(),
			)
		)
	}

	/// Get the crosslink committee.
	pub fn beacon_committee(
		&self, slot: Uint, index: Uint,
	) -> Result<Vec<ValidatorIndex>, Error> {
		let epoch = utils::epoch_of_slot::<C>(slot);
		let committees_per_slot = self.committee_count_at_slot(slot);
		let indices = self.active_validator_indices(epoch);
		let seed = self.seed(epoch, C::domain_beacon_attester());
		let index = (slot % C::slots_per_epoch()) * committees_per_slot + index;
		let count = committees_per_slot * C::slots_per_epoch();

		utils::compute_committee::<C>(&indices, seed, index, count)
	}

	/// Get the current beacon proposer index.
	pub fn beacon_proposer_index(&self) -> Result<ValidatorIndex, Error> {
		let epoch = self.current_epoch();
		let seed = C::hash(&[
			&self.seed(epoch, C::domain_beacon_proposer())[..],
			&self.slot.to_le_bytes()[..8]
		]);
		let indices = self.active_validator_indices(epoch);

		let mut i = 0;
		loop {
			let candidate_index = indices[
				utils::shuffled_index::<C>(
					i % indices.len() as u64,
					indices.len() as u64,
					seed
				)? as usize
			];
			let random_byte = C::hash(&[
				&seed[..],
				&utils::to_bytes(i / 32)[..8],
			])[(i % 32) as usize];
			let effective_balance = self.validators[candidate_index as usize].effective_balance;
			if effective_balance * u8::max_value() as u64 >=
				C::max_effective_balance() * random_byte as u64
			{
				return Ok(candidate_index)
			}

			i += 1;
		}
	}

	/// Get total balance of validator indices.
	pub fn total_balance(&self, indices: &[ValidatorIndex]) -> Gwei {
		max(
			indices.iter().fold(0, |sum, index| {
				sum + self.validators[*index as usize].effective_balance
			}),
			1
		)
	}

	/// Get total balance of active validators.
	pub fn total_active_balance(&self) -> Gwei {
		self.total_balance(&self.active_validator_indices(self.current_epoch()))
	}

	/// Get signing domain, given domain type and message epoch.
	pub fn domain(&self, domain_type: u32, message_epoch: Option<Uint>) -> Uint {
		let epoch = message_epoch.unwrap_or(self.current_epoch());
		let fork_version = if epoch < self.fork.epoch {
			self.fork.previous_version
		} else {
			self.fork.current_version
		};

		utils::bls_domain(domain_type, fork_version)
	}

	/// Convert an attestation to indexed attestation.
	pub fn indexed_attestation(
		&self,
		attestation: Attestation<C>
	) -> Result<IndexedAttestation<C>, Error> {
		let attesting_indices = self.attesting_indices(
			&attestation.data, &attestation.aggregation_bits
		)?;
		let custody_bit_1_indices = self.attesting_indices(
			&attestation.data, &attestation.custody_bits
		)?;
		let custody_bit_0_indices = attesting_indices.clone()
			.into_iter()
			.filter(|index| !custody_bit_1_indices.contains(index))
			.collect::<Vec<_>>();

		Ok(IndexedAttestation {
			data: attestation.data,
			signature: attestation.signature,
			custody_bit_0_indices: custody_bit_0_indices.into(),
			custody_bit_1_indices: custody_bit_1_indices.into(),
		})
	}

	/// Get attesting indices of given attestation.
	pub fn attesting_indices(
		&self, attestation_data: &AttestationData, bitfield: &[bool],
	) -> Result<Vec<ValidatorIndex>, Error> {
		let committee = self.beacon_committee(
			attestation_data.slot, attestation_data.index,
		)?;

		if committee.len() != bitfield.len() {
			return Err(Error::AttestationBitFieldInvalid)
		}

		let mut ret = committee.into_iter()
			.enumerate()
			.filter(|(i, _)| bitfield[*i])
			.map(|(_, val)| val)
			.collect::<Vec<_>>();
		ret.sort();
		Ok(ret)
	}
}
