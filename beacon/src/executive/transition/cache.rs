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

use ssz::Digestible;
use crate::primitives::H256;
use crate::{Config, ExecutiveMut};

impl<'state, 'config, C: Config> ExecutiveMut<'state, 'config, C> {
	/// State caching.
	///
	/// Run at every slot > GENESIS_SLOT.
	pub fn cache_state(&mut self) {
		let latest_state_root = H256::from_slice(
			Digestible::<C::Digest>::hash(self.state).as_slice()
		);

		self.state.latest_state_roots[
			(self.state.slot % self.config.slots_per_historical_root()) as usize
		] = latest_state_root;

		if self.state.latest_block_header.state_root == H256::default() {
			self.state.latest_block_header.state_root = latest_state_root;
		}

		let latest_block_root = H256::from_slice(
			Digestible::<C::Digest>::truncated_hash(
				&self.state.latest_block_header
			).as_slice()
		);
		self.state.latest_block_roots[
			(self.state.slot % self.config.slots_per_historical_root()) as usize
		] = latest_block_root;
	}
}
