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

mod per_epoch;
mod per_block;

use ssz::Digestible;
use crate::{Error, Config, Executive};
use crate::types::Block;
use crate::primitives::{H256, Uint};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub fn state_transition<B: Block + Digestible<C::Digest>>(
		&mut self,
		block: &B,
		validate_state_root: bool
	) -> Result<(), Error> {
		self.process_slots(block.slot())?;
		self.process_block(block)?;

		if validate_state_root {
			if !(block.state_root() == &H256::from_slice(
				Digestible::<C::Digest>::hash(self.state).as_slice()
			)) {
				return Err(Error::BlockStateRootInvalid)
			}
		}

		Ok(())
	}

	pub fn process_slots(&mut self, slot: Uint) -> Result<(), Error> {
		if self.state.slot > slot {
			return Err(Error::SlotOutOfRange)
		}

		while self.state.slot < slot {
			self.process_slot();
			if (self.state.slot + 1) % self.config.slots_per_epoch() == 0 {
				self.process_epoch()?;
			}
			self.state.slot += 1;
		}

		Ok(())
	}

	/// Advance slot
	pub fn process_slot(&mut self) {
		let previous_state_root = H256::from_slice(
			Digestible::<C::Digest>::hash(self.state).as_slice()
		);
		self.state.latest_state_roots[
			(self.state.slot % self.config.slots_per_historical_root()) as usize
		] = previous_state_root;

		if self.state.latest_block_header.state_root == H256::default() {
			self.state.latest_block_header.state_root = previous_state_root;
		}

		let previous_block_root = H256::from_slice(
			Digestible::<C::Digest>::truncated_hash(
				&self.state.latest_block_header
			).as_slice()
		);
		self.state.latest_block_roots[
			(self.state.slot % self.config.slots_per_historical_root()) as usize
		] = previous_block_root;
	}
}
