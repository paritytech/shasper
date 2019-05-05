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
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit, Block, BeaconBlockHeader, ProposerSlashing};
use crate::utils::to_bytes;
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Push a new `ProposerSlashing` to the state.
	pub fn process_proposer_slashing(
		&mut self,
		proposer_slashing: ProposerSlashing
	) -> Result<(), Error> {
		if self.config.slot_to_epoch(proposer_slashing.header_1.slot) !=
			self.config.slot_to_epoch(proposer_slashing.header_2.slot)
		{
			return Err(Error::ProposerSlashingInvalidSlot)
		}

		if proposer_slashing.header_1 == proposer_slashing.header_2 {
			return Err(Error::ProposerSlashingSameHeader)
		}

		{
			let proposer = &self.state.validator_registry[
				proposer_slashing.proposer_index as usize
			];

			if !proposer.is_slashable(self.current_epoch()) {
				return Err(Error::ProposerSlashingAlreadySlashed)
			}

			for header in &[&proposer_slashing.header_1, &proposer_slashing.header_2] {
				let domain = self.domain(
					self.config.domain_beacon_proposer(),
					Some(self.config.slot_to_epoch(header.slot))
				);

				if !self.config.bls_verify(
					&proposer.pubkey,
					&H256::from_slice(
						Digestible::<C::Digest>::truncated_hash(*header).as_slice()
					),
					&header.signature,
					domain,
				) {
					return Err(Error::ProposerSlashingInvalidSignature)
				}
			}
		}

		self.slash_validator(proposer_slashing.proposer_index, None)
	}
}
