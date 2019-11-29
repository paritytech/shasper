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

use crate::primitives::ValidatorId;
use crate::{Config, BeaconExecutive, Error, utils};

/// Committee assignment.
pub struct CommitteeAssignment {
	/// List of validators in the committee.
	pub validators: Vec<u64>,
	/// Shard to which the committee is assigned.
	pub index: u64,
	/// Slot at which the committee is assigned.
	pub slot: u64,
}

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Find committee assignment at slot.
	pub fn committee_assignment(
		&self,
		epoch: u64,
		validator_index: u64
	) -> Result<Option<CommitteeAssignment>, Error> {
		let next_epoch = self.current_epoch() + 1;
		if epoch > next_epoch {
			return Err(Error::EpochOutOfRange)
		}

		let epoch_start_slot = utils::start_slot_of_epoch::<C>(epoch);
		for slot in epoch_start_slot..(epoch_start_slot + C::slots_per_epoch()) {
			let committee_count = self.committee_count_at_slot(slot);
			for index in 0..committee_count {
				let index = index as u64;
				let committee = self.beacon_committee(slot, index)?;
				if committee.contains(&validator_index) {
					return Ok(Some(CommitteeAssignment {
						validators: committee,
						index, slot,
					}))
				}
			}
		}
		Ok(None)
	}

	/// Get validator public key.
	pub fn validator_pubkey(&self, index: u64) -> Option<ValidatorId> {
		if index as usize >= self.validators.len() {
			return None
		}

		let validator = &self.validators[index as usize];
		Some(validator.pubkey.clone())
	}

	/// Get validator index from public key.
	pub fn validator_index(&self, pubkey: &ValidatorId) -> Option<u64> {
		let validator_pubkeys = self.validators.iter()
			.map(|v| v.pubkey.clone()).collect::<Vec<_>>();
		validator_pubkeys.iter().position(|v| v == pubkey).map(|v| v as u64)
	}
}
