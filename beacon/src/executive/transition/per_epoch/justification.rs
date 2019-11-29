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

use crate::types::Checkpoint;
use crate::components::Justifier;
use crate::{Config, BeaconExecutive, Error};

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Update casper justification and finalization.
	pub fn process_justification_and_finalization(&mut self) -> Result<(), Error> {
		if self.current_epoch() <= C::genesis_epoch() + 1 {
			return Ok(())
		}

		let previous_epoch = self.previous_epoch();
		let previous_checkpoint = Checkpoint {
			epoch: previous_epoch,
			root: self.block_root(previous_epoch)?,
		};
		let current_epoch = self.current_epoch();
		let current_checkpoint = Checkpoint {
			epoch: current_epoch,
			root: self.block_root(current_epoch)?,
		};

		let mut processor = Justifier {
			justification_bits: self.justification_bits.clone(),
			current_justified_checkpoint: self.current_justified_checkpoint.clone(),
			previous_justified_checkpoint: self.previous_justified_checkpoint.clone(),
			finalized_checkpoint: self.finalized_checkpoint.clone(),
		};

		processor.process(previous_checkpoint, current_checkpoint, self)?;

		self.state.justification_bits = processor.justification_bits;
		self.state.current_justified_checkpoint = processor.current_justified_checkpoint;
		self.state.previous_justified_checkpoint = processor.previous_justified_checkpoint;
		self.state.finalized_checkpoint = processor.finalized_checkpoint;

		Ok(())
	}
}
