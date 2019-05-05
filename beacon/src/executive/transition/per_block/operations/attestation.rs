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
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit, Block, BeaconBlockHeader, ProposerSlashing, AttesterSlashing, PendingAttestation};
use crate::utils::to_bytes;
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Push a new `Attestation` to the state.
	pub fn process_attestation(&mut self, attestation: Attestation) -> Result<(), Error> {
		let attestation_slot = self.attestation_slot(&attestation.data)?;

		if !(attestation_slot + self.config.min_attestation_inclusion_delay() <=
			 self.state.slot &&
			 self.state.slot <=
			 attestation_slot + self.config.slots_per_epoch())
		{
			return Err(Error::AttestationSubmittedTooQuickly)
		}

		let data = attestation.data.clone();
		// Check target epoch, source epoch, source root, and source crosslink
		let attestation_pair =
			(data.target_epoch, data.source_epoch,
			 data.source_root, data.previous_crosslink_root);
		let current_pair =
			(self.current_epoch(), self.state.current_justified_epoch,
			 self.state.current_justified_root,
			 H256::from_slice(Digestible::<C::Digest>::hash(
				 &self.state.current_crosslinks[data.shard as usize]
			 ).as_slice()));
		let previous_pair =
			(self.previous_epoch(), self.state.previous_justified_epoch,
			 self.state.previous_justified_root,
			 H256::from_slice(Digestible::<C::Digest>::hash(
				 &self.state.previous_crosslinks[data.shard as usize]
			 ).as_slice()));

		if !(attestation_pair == current_pair || attestation_pair == previous_pair) {
			return Err(Error::AttestationIncorrectJustifiedEpochOrBlockRoot)
		}

		if !(data.crosslink_data_root == H256::default()) {
			return Err(Error::AttestationIncorrectCrosslinkData)
		}

		if !self.verify_indexed_attestation(
			&self.convert_to_indexed(attestation.clone())?
		)? {
			return Err(Error::AttestationInvalidSignature)
		}

		let pending_attestation = PendingAttestation {
			data: data.clone(),
			aggregation_bitfield: attestation.aggregation_bitfield,
			inclusion_delay: self.state.slot - attestation_slot,
			proposer_index: self.beacon_proposer_index()?,
		};

		if data.target_epoch == self.current_epoch() {
			self.state.current_epoch_attestations.push(pending_attestation);
		} else {
			self.state.previous_epoch_attestations.push(pending_attestation);
		}

		Ok(())
	}
}
