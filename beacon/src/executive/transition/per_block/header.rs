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

use core::cmp::{min, max};
use ssz::Digestible;
use crate::primitives::{Uint, Epoch, Slot, ValidatorIndex, Gwei, Shard, H256, BitField};
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit, Block, BeaconBlockHeader};
use crate::utils::to_bytes;
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Process a block header.
	pub fn process_block_header<B: Block + Digestible<C::Digest>>(&mut self, block: &B) -> Result<(), Error> {
		if block.slot() != self.state.slot {
			return Err(Error::BlockSlotInvalid)
		}

		if block.previous_block_root() != &H256::from_slice(
			Digestible::<C::Digest>::truncated_hash(
				&self.state.latest_block_header
			).as_slice())
		{
			return Err(Error::BlockPreviousRootInvalid)
		}

		self.state.latest_block_header = BeaconBlockHeader {
			slot: block.slot(),
			previous_block_root: *block.previous_block_root(),
			block_body_root: H256::from_slice(
				Digestible::<C::Digest>::hash(block.body()).as_slice()
			),
			..Default::default()
		};

		let proposer = &self.state.validator_registry[
			self.beacon_proposer_index()? as usize
		];
		if proposer.slashed {
			return Err(Error::BlockProposerSlashed)
		}

		if let Some(signature) = block.signature() {
			if !self.config.bls_verify(
				&proposer.pubkey,
				&H256::from_slice(
					Digestible::<C::Digest>::truncated_hash(block).as_slice()
				),
				signature,
				self.domain(self.config.domain_beacon_proposer(), None)
			) {
				return Err(Error::BlockSignatureInvalid)
			}
		}

		Ok(())
	}
}
