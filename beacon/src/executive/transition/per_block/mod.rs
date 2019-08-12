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

mod header;
mod randao;
mod eth1;
mod operations;

use crate::types::*;
use crate::{Config, BLSConfig, BeaconState, Error};

impl<C: Config> BeaconState<C> {
	/// Process a block, assuming we are at given slot.
	pub fn process_block<'a, 'b, B: Block<Config=C>, BLS: BLSConfig>(
		&'a mut self,
		block: &'b B,
	) -> Result<(), Error> where
		UnsealedBeaconBlock<C>: From<&'b B>,
	{
		self.process_block_header::<_, BLS>(block)?;
		self.process_randao::<BLS>(block.body())?;
		self.process_eth1_data(block.body());
		self.process_operations::<BLS>(block.body())?;

		Ok(())
	}
}
