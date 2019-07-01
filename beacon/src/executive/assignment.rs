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

use crate::{Executive, Error, Config};

/// Committee assignment.
pub struct CommitteeAssignment {
	/// List of validators in the committee.
	pub validators: Vec<u64>,
	/// Shard to which the committee is assigned.
	pub shard: u64,
	/// Slot at which the committee is assigned.
	pub slot: u64,
}

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
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

		let committees_per_slot =
			self.committee_count(epoch) / self.config.slots_per_epoch();
		let epoch_start_slot = self.compute_start_slot_of_epoch(epoch);
		for slot in epoch_start_slot..(epoch_start_slot + self.config.slots_per_epoch()) {
			let offset = committees_per_slot *
				(slot % self.config.slots_per_epoch());
			let slot_start_shard =
				(self.start_shard(epoch)? + offset) % self.config.shard_count();
			for i in 0..committees_per_slot {
				let shard = (slot_start_shard + i) % self.config.shard_count();
				let committee = self.crosslink_committee(epoch, shard)?;
				if committee.contains(&validator_index) {
					return Ok(Some(CommitteeAssignment {
						validators: committee,
						shard, slot,
					}))
				}
			}
		}
		Ok(None)
	}
}
