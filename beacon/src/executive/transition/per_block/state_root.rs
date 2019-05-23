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
use crate::types::Block;
use crate::{Config, ExecutiveRef, Error};

impl<'state, 'config, C: Config> ExecutiveRef<'state, 'config, C> {
	/// Verify block state root.
	pub fn verify_block_state_root<B: Block>(&self, block: &B) -> Result<(), Error> {
		if !(block.state_root() == &H256::from_slice(
			Digestible::<C::Digest>::hash(self.state).as_slice()
		)) {
			return Err(Error::BlockStateRootInvalid)
		}

		Ok(())
	}
}
