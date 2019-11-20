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
use crate::{Config, BeaconExecutive, Error};

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Update casper justification and finalization.
	pub fn process_justification_and_finalization(&mut self) -> Result<(), Error> {
		if self.current_epoch() <= C::genesis_epoch() + 1 {
			return Ok(())
		}

		let previous_epoch = self.previous_epoch();
		let current_epoch = self.current_epoch();
		let old_previous_justified_checkpoint = self.previous_justified_checkpoint.clone();
		let old_current_justified_checkpoint = self.current_justified_checkpoint.clone();;

		// Process justifications
		self.state.previous_justified_checkpoint = self.current_justified_checkpoint.clone();
		let old_justification_bits = self.justification_bits.clone();
		let justification_bits_len = self.justification_bits.len();
		self.state.justification_bits[1..].copy_from_slice(
			&old_justification_bits[0..(justification_bits_len - 1)]
		);
		self.state.justification_bits[0] = false;
		let matching_target_attestations =
			self.matching_target_attestations(previous_epoch)?;
		if self.attesting_balance(&matching_target_attestations)? * 3 >=
			self.total_active_balance() * 2
		{
			self.state.current_justified_checkpoint = Checkpoint {
				epoch: previous_epoch,
				root: self.block_root(previous_epoch)?
			};
			self.state.justification_bits[1] = true;
		}
		let matching_target_attestations =
			self.matching_target_attestations(current_epoch)?;
		if self.attesting_balance(&matching_target_attestations)? * 3 >=
			self.total_active_balance() * 2
		{
			self.state.current_justified_checkpoint = Checkpoint {
				epoch: current_epoch,
				root: self.block_root(current_epoch)?
			};
			self.state.justification_bits[0] = true;
		}

		// Process finalizations
		let bits = self.justification_bits.clone();
		// The 2nd/3rd/4th most recent epochs are justified,
		// the 2nd using the 4th as source
		if bits[1..4].iter().all(|v| *v) &&
			old_previous_justified_checkpoint.epoch + 3 == current_epoch
		{
			self.state.finalized_checkpoint = old_previous_justified_checkpoint.clone();
		}
		// The 2nd/3rd most recent epochs are justified,
		// the 2nd using the 3rd as source
		if bits[1..3].iter().all(|v| *v) &&
			old_previous_justified_checkpoint.epoch + 2 == current_epoch
		{
			self.state.finalized_checkpoint = old_previous_justified_checkpoint.clone();
		}
		// The 1st/2nd/3rd most recent epochs are justified,
		// the 1st using the 3rd as source
		if bits[0..3].iter().all(|v| *v) &&
			old_current_justified_checkpoint.epoch + 2 == current_epoch
		{
			self.state.finalized_checkpoint = old_current_justified_checkpoint.clone();
		}
		// The 1st/2nd most recent epochs are justified,
		// the 1st using the 2nd as source
		if bits[0..2].iter().all(|v| *v) &&
			old_current_justified_checkpoint.epoch + 1 == current_epoch
		{
			self.state.finalized_checkpoint = old_current_justified_checkpoint.clone();
		}

		Ok(())
	}
}
