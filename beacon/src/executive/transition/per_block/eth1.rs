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

use crate::{Config, Executive};
use crate::types::BeaconBlockBody;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Process eth1 data vote given in a block.
	pub fn process_eth1_data(&mut self, body: &BeaconBlockBody) {
		self.state.eth1_data_votes.push(body.eth1_data.clone());
		if self.state.eth1_data_votes.iter()
			.filter(|d| d == &&body.eth1_data)
			.count() * 2 >
			self.config.slots_per_eth1_voting_period() as usize
		{
			self.state.latest_eth1_data = body.eth1_data.clone();
		}
	}
}
