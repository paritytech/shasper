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










use crate::types::*;
use crate::{Config, BeaconState};

impl<C: Config> BeaconState<C> {
	/// Process eth1 data vote given in a block.
	pub fn process_eth1_data(&mut self, body: &BeaconBlockBody<C>) {
		self.eth1_data_votes.push(body.eth1_data.clone());
		if self.eth1_data_votes.iter()
			.filter(|d| d == &&body.eth1_data)
			.count() * 2 >
			C::slots_per_eth1_voting_period() as usize
		{
			self.eth1_data = body.eth1_data.clone();
		}
	}
}
