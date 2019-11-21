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

use crate::primitives::{Epoch, Gwei, ValidatorIndex};
use crate::types::PendingAttestation;
use crate::{Config, BeaconExecutive, Error};

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Get attestations with matching source at given epoch.
	pub fn matching_source_attestations(
		&self,
		epoch: Epoch
	) -> Result<Vec<PendingAttestation<C>>, Error> {
		if epoch == self.current_epoch() {
			Ok(self.current_epoch_attestations.clone().into())
		} else if epoch == self.previous_epoch() {
			Ok(self.previous_epoch_attestations.clone().into())
		} else {
			Err(Error::EpochOutOfRange)
		}
	}

	/// Get attestations with matching target at given epoch.
	pub fn matching_target_attestations(
		&self,
		epoch: Epoch
	) -> Result<Vec<PendingAttestation<C>>, Error> {
		let block_root = self.block_root(epoch)?;
		Ok(self.matching_source_attestations(epoch)?.into_iter()
		   .filter(|a| a.data.target.root == block_root)
		   .collect())
	}

	/// Get attestations with matching head at given epoch.
	pub fn matching_head_attestations(
		&self,
		epoch: Epoch
	) -> Result<Vec<PendingAttestation<C>>, Error> {
		self.matching_source_attestations(epoch)?.into_iter()
			.map(|a| {
				Ok((a.data.beacon_block_root == self.block_root_at_slot(
					a.data.slot
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

	/// Get indices of all unslashed validators within attestations.
	pub fn unslashed_attesting_indices(
		&self, attestations: &[PendingAttestation<C>]
	) -> Result<Vec<ValidatorIndex>, Error> {
		let mut ret = Vec::new();
		for a in attestations {
			for index in self.attesting_indices(&a.data, &a.aggregation_bits)? {
				if !ret.contains(&index) {
					ret.push(index);
				}
			}
		}
		ret.retain(|index| {
			!self.validators[*index as usize].slashed
		});
		ret.sort();
		Ok(ret)
	}

	/// Get the total attesting balance given a list of attestations.
	pub fn attesting_balance(
		&self, attestations: &[PendingAttestation<C>]
	) -> Result<Gwei, Error> {
		Ok(self.total_balance(&self.unslashed_attesting_indices(attestations)?))
	}
}
