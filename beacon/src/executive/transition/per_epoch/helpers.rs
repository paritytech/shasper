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

use core::cmp::{min, Ordering};
use ssz::Digestible;
use crate::primitives::{Epoch, ValidatorIndex, Gwei, Shard, H256};
use crate::types::{AttestationData, PendingAttestation, Crosslink};
use crate::utils::compare_hash;
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub(crate) fn total_active_balance(&self) -> Gwei {
		self.total_balance(&self.active_validator_indices(self.current_epoch()))
	}

	pub(crate) fn matching_source_attestations(&self, epoch: Epoch) -> Result<Vec<PendingAttestation>, Error> {
		if epoch == self.current_epoch() {
			Ok(self.state.current_epoch_attestations.clone())
		} else if epoch == self.previous_epoch() {
			Ok(self.state.previous_epoch_attestations.clone())
		} else {
			Err(Error::EpochOutOfRange)
		}
	}

	pub(crate) fn matching_target_attestations(&self, epoch: Epoch) -> Result<Vec<PendingAttestation>, Error> {
		let block_root = self.block_root(epoch)?;
		Ok(self.matching_source_attestations(epoch)?.into_iter()
		   .filter(|a| a.data.target_root == block_root)
		   .collect())
	}

	pub(crate) fn matching_head_attestations(&self, epoch: Epoch) -> Result<Vec<PendingAttestation>, Error> {
		self.matching_source_attestations(epoch)?.into_iter()
			.map(|a| {
				Ok((a.data.beacon_block_root == self.block_root_at_slot(
					self.attestation_data_slot(&a.data)?
				)?, a))
			})
			.collect::<Result<Vec<_>, _>>()
			.map(|r| {
				r.into_iter()
					.filter(|(c, _)| *c)
					.map(|(_, v)| v)
					.collect::<Vec<_>>()
			})
	}

	pub(crate) fn unslashed_attesting_indices(
		&self, attestations: &[PendingAttestation]
	) -> Result<Vec<ValidatorIndex>, Error> {
		let mut ret = Vec::new();
		for a in attestations {
			for index in self.attesting_indices(&a.data, &a.aggregation_bitfield)? {
				if !ret.contains(&index) {
					ret.push(index);
				}
			}
		}
		ret.retain(|index| {
			!self.state.validator_registry[*index as usize].slashed
		});
		ret.sort();
		Ok(ret)
	}

	pub(crate) fn attesting_balance(
		&self, attestations: &[PendingAttestation]
	) -> Result<Gwei, Error> {
		Ok(self.total_balance(&self.unslashed_attesting_indices(attestations)?))
	}

	pub(crate) fn winning_crosslink_and_attesting_indices(
		&self, epoch: Epoch, shard: Shard
	) -> Result<(Crosslink, Vec<ValidatorIndex>), Error> {
		let attestations = self.matching_source_attestations(epoch)?.into_iter()
			.filter(|a| a.data.shard == shard)
			.collect::<Vec<_>>();
		let crosslinks = attestations.clone().into_iter()
			.map(|a| a.data.crosslink)
			.filter(|c| {
				let current_root = H256::from_slice(
					Digestible::<C::Digest>::hash(
						&self.state.current_crosslinks[shard as usize]
					).as_slice()
				);
				let root = H256::from_slice(
					Digestible::<C::Digest>::hash(c).as_slice()
				);

				current_root == root || current_root == c.parent_root
			})
			.collect::<Vec<_>>();

		if candidate_crosslinks.len() == 0 {
			return Ok((Crosslink::default(), Vec::new()))
		}

		let attestations_for = |crosslink: &Crosslink| {
			attestations.clone().into_iter()
				.filter(|a| a.crosslink == crosslink)
				.collect::<Vec<_>>()
		};
		let winning_crosslink = if candidate_crosslinks.len() == 0 {
			Crosslink::default()
		} else {
			candidate_crosslinks
				.iter()
				.fold(Ok(candidate_crosslinks[0].clone()), |a, b| {
					let a = a?;
					let cmp1 = self.attesting_balance(&attestations_for(&a))?
						.cmp(&self.attesting_balance(&attestations_for(b))?);
					let cmp2 = compare_hash(&a.data_root, &b.data_root);

					Ok(match (cmp1, cmp2) {
						(Ordering::Greater, _) |
						(Ordering::Equal, Ordering::Greater) => a,
						_ => b.clone(),
					})
				})?;
		};
		let winning_attestations = attestations_for(winning_crosslink);

		Ok((winning_crosslink, self.unslashed_attesting_indices(winning_attestations)))
	}
}
