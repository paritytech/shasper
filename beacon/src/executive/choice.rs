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

use crate::{Executive, Error, Config};
use crate::primitives::H256;
use crate::types::BeaconBlock;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Get justified active validators from current state.
	pub fn justified_active_validators(&self) -> Vec<u64> {
		let current_justified_epoch = self.state.current_justified_checkpoint.epoch;
		self.active_validator_indices(current_justified_epoch)
	}

	/// Get block attestation vote targets.
	pub fn block_vote_targets(&self, block: &BeaconBlock) -> Result<Vec<(u64, H256)>, Error> {
		let mut ret = Vec::new();
		for attestation in block.body.attestations.clone() {
			let indexed = self.indexed_attestation(attestation)?;

			for v in indexed.custody_bit_0_indices.into_iter()
				.chain(indexed.custody_bit_1_indices.into_iter())
			{
				ret.push((v, indexed.data.target.root));
			}
		}

		Ok(ret)
	}
}
