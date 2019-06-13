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

mod header;
mod randao;
mod eth1;
mod operations;

use ssz::Digestible;
use crate::{Config, Error, Executive};
use crate::types::Block;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub fn process_block<B: Block + Digestible<C::Digest>>(
		&mut self,
		block: &B
	) -> Result<(), Error> {
		self.process_block_header(block)?;
		self.process_randao(block.body())?;
		self.process_eth1_data(block.body());
		self.process_operations(block.body())?;

		Ok(())
	}
}
