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

use crate::{Config, BeaconState, Error};

impl<C: Config> BeaconState<C> {
	/// Update crosslinks
	pub fn process_crosslinks(&mut self) -> Result<(), Error> {
		self.previous_crosslinks = self.current_crosslinks.clone();
		for epoch in &[self.previous_epoch(), self.current_epoch()] {
			for offset in 0..self.committee_count(*epoch) {
				let shard = (self.start_shard(*epoch)? + offset) % C::shard_count();
				let crosslink_committee = self.crosslink_committee(*epoch, shard)?;
				let (winning_crosslink, attesting_indices) =
					self.winning_crosslink_and_attesting_indices(*epoch, shard)?;
				if 3 * self.total_balance(&attesting_indices) >=
					2 * self.total_balance(&crosslink_committee)
				{
					self.current_crosslinks[shard as usize] = winning_crosslink;
				}
			}
		}

		Ok(())
	}
}
