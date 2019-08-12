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

use crate::primitives::*;
use crate::types::*;
use crate::{Config, BeaconState, Error};

impl<C: Config> BeaconState<C> {
	/// Get justified active validators from current state.
	pub fn justified_active_validators(&self) -> Vec<u64> {
		let current_justified_epoch = self.current_justified_checkpoint.epoch;
		self.active_validator_indices(current_justified_epoch)
	}

	/// Get block attestation vote targets.
	pub fn block_vote_targets(&self, block: &BeaconBlock<C>) -> Result<Vec<(u64, H256)>, Error> {
		let mut ret = Vec::new();
		for attestation in block.body.attestations.iter() {
			let indexed = self.indexed_attestation(attestation.clone())?;

			for v in indexed.custody_bit_0_indices.iter().cloned()
				.chain(indexed.custody_bit_1_indices.iter().cloned())
			{
				ret.push((v, indexed.data.target.root));
			}
		}

		Ok(ret)
	}
}
