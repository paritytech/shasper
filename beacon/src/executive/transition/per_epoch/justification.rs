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

use core::cmp::{min, max, Ordering};
use ssz::Digestible;
use crate::primitives::{Uint, Epoch, Slot, ValidatorIndex, Gwei, Shard, H256, BitField};
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit, PendingAttestation, Crosslink};
use crate::utils::{to_bytes, compare_hash};
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Update casper justification and finalization.
	pub fn process_justification_and_finalization(&mut self) -> Result<(), Error> {
		if self.current_epoch() <= self.config.genesis_epoch() + 1 {
			return Ok(())
		}

		let previous_epoch = self.previous_epoch();
		let current_epoch = self.current_epoch();
		let old_previous_justified_epoch = self.state.previous_justified_epoch;
		let old_current_justified_epoch = self.state.current_justified_epoch;

		// Process justifications
		self.state.previous_justified_epoch = self.state.current_justified_epoch;
		self.state.previous_justified_root = self.state.current_justified_root;
		self.state.justification_bitfield <<= 1;
		let previous_epoch_matching_target_balance = self.attesting_balance(
			&self.matching_target_attestations(previous_epoch)?
		)?;
		if previous_epoch_matching_target_balance * 3 >=
			self.total_active_balance() * 2
		{
			self.state.current_justified_epoch = previous_epoch;
			self.state.current_justified_root =
				self.block_root(self.state.current_justified_epoch)?;
			self.state.justification_bitfield |= 1 << 1;
		}
		let current_epoch_matching_target_balance = self.attesting_balance(
			&self.matching_target_attestations(current_epoch)?
		)?;
		if current_epoch_matching_target_balance * 3 >=
			self.total_active_balance() * 2
		{
			self.state.current_justified_epoch = current_epoch;
			self.state.current_justified_root =
				self.block_root(self.state.current_justified_epoch)?;
			self.state.justification_bitfield |= 1 << 0;
		}

		// Process finalizations
		let bitfield = self.state.justification_bitfield;
		// The 2nd/3rd/4th most recent epochs are justified,
		// the 2nd using the 4th as source
		if (bitfield >> 1) % 8 == 0b111 &&
			old_previous_justified_epoch == current_epoch - 3
		{
			self.state.finalized_epoch = old_previous_justified_epoch;
			self.state.finalized_root = self.block_root(self.state.finalized_epoch)?;
		}
		// The 2nd/3rd most recent epochs are justified,
		// the 2nd using the 3rd as source
		if (bitfield >> 1) % 4 == 0b011 &&
			old_previous_justified_epoch == current_epoch - 2
		{
			self.state.finalized_epoch = old_previous_justified_epoch;
			self.state.finalized_root = self.block_root(self.state.finalized_epoch)?;
		}
		// The 1st/2nd/3rd most recent epochs are justified,
		// the 1st using the 3rd as source
		if (bitfield >> 0) % 8 == 0b111 &&
			old_current_justified_epoch == current_epoch - 2
		{
			self.state.finalized_epoch = old_current_justified_epoch;
			self.state.finalized_root = self.block_root(self.state.finalized_epoch)?;
		}
		// The 1st/2nd most recent epochs are justified,
		// the 1st using the 2nd as source
		if (bitfield >> 0) % 4 == 0b011 &&
			old_current_justified_epoch == current_epoch - 1
		{
			self.state.finalized_epoch = old_current_justified_epoch;
			self.state.finalized_root = self.block_root(self.state.finalized_epoch)?;
		}

		Ok(())
	}
}
