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
use crate::{Config, BeaconState, Error, BLSConfig, utils};
use bm_le::tree_root;

impl<C: Config> BeaconState<C> {
	/// Push a new `ProposerSlashing` to the state.
	pub fn process_proposer_slashing<BLS: BLSConfig>(
		&mut self,
		proposer_slashing: ProposerSlashing
	) -> Result<(), Error> {
		if utils::epoch_of_slot::<C>(proposer_slashing.header_1.slot) !=
			utils::epoch_of_slot::<C>(proposer_slashing.header_2.slot)
		{
			return Err(Error::ProposerSlashingInvalidSlot)
		}

		if proposer_slashing.header_1 == proposer_slashing.header_2 {
			return Err(Error::ProposerSlashingSameHeader)
		}

		{
			if proposer_slashing.proposer_index as usize >= self.validators.len() {
				return Err(Error::ProposerSlashingInvalidProposerIndex)
			}

			let proposer = &self.validators[
				proposer_slashing.proposer_index as usize
			];

			if !proposer.is_slashable(self.current_epoch()) {
				return Err(Error::ProposerSlashingAlreadySlashed)
			}

			for header in &[&proposer_slashing.header_1, &proposer_slashing.header_2] {
				let domain = self.domain(
					C::domain_beacon_proposer(),
					Some(utils::epoch_of_slot::<C>(header.slot))
				);

				if !BLS::verify(
					&proposer.pubkey,
					&tree_root::<C::Digest, _>(&SigningBeaconBlockHeader::from((*header).clone())),
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
