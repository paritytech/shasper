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

use crate::types::{UnsealedBeaconBlock, SigningBeaconBlockHeader, BeaconBlockHeader, Block};
use crate::{Config, BeaconExecutive, Error, BLSConfig};
use bm_le::tree_root;

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Process a block header.
	pub fn process_block_header<'b, B: Block, BLS: BLSConfig>(
		&mut self,
		block: &'b B
	) -> Result<(), Error> where
		UnsealedBeaconBlock<C>: From<&'b B>,
	{
		if block.slot() != self.slot {
			return Err(Error::BlockSlotInvalid)
		}

		if block.parent_root() != &tree_root::<C::Digest, _>(
			&SigningBeaconBlockHeader::from(self.latest_block_header.clone())
		) {
			return Err(Error::BlockPreviousRootInvalid)
		}

		self.state.latest_block_header = BeaconBlockHeader {
			slot: block.slot(),
			parent_root: *block.parent_root(),
			body_root: tree_root::<C::Digest, _>(block.body()),
			..Default::default()
		};

		let proposer = &self.validators[
			self.beacon_proposer_index()? as usize
		];
		if proposer.slashed {
			return Err(Error::BlockProposerSlashed)
		}

		if let Some(signature) = block.signature() {
			if !BLS::verify(
				&proposer.pubkey,
				&tree_root::<C::Digest, _>(&UnsealedBeaconBlock::from(block)),
				signature,
				self.domain(C::domain_beacon_proposer(), None)
			) {
				return Err(Error::BlockSignatureInvalid)
			}
		}

		Ok(())
	}
}
