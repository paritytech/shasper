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

use primitives::H256;
use ssz::Hashable;
use crate::state::BeaconState;
use crate::consts::SLOTS_PER_HISTORICAL_ROOT;
use crate::util::Hasher;

impl BeaconState {
	pub fn update_cache(&mut self) {
		let previous_slot_state_root = self.hash::<Hasher>();

		self.latest_state_roots[(self.slot % SLOTS_PER_HISTORICAL_ROOT as u64) as usize] = previous_slot_state_root;

		if self.latest_block_header.state_root == H256::default() {
			self.latest_block_header.state_root = previous_slot_state_root;
		}

		self.latest_block_roots[(self.slot % SLOTS_PER_HISTORICAL_ROOT as u64) as usize] = self.latest_block_header.hash::<Hasher>();
	}
}
