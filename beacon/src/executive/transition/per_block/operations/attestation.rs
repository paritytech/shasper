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
use crate::{Config, BeaconState, Error, BLSConfig};
use bm_le::tree_root;
use core::cmp::min;

impl<C: Config> BeaconState<C> {
	/// Push a new `Attestation` to the state.
	pub fn process_attestation<BLS: BLSConfig>(&mut self, attestation: Attestation<C>) -> Result<(), Error> {
		let data = attestation.data.clone();
		if !(data.crosslink.shard < C::shard_count()) {
			return Err(Error::AttestationIncorrectCrosslinkData)
		}
		if !(data.target.epoch == self.current_epoch() ||
			 data.target.epoch == self.previous_epoch())
		{
			return Err(Error::AttestationIncorrectJustifiedEpochOrBlockRoot)
		}

		let attestation_slot = self.attestation_data_slot(&data)?;

		if !(attestation_slot + C::min_attestation_inclusion_delay() <=
			 self.slot &&
			 self.slot <=
			 attestation_slot + C::slots_per_epoch())
		{
			return Err(Error::AttestationSubmittedTooQuickly)
		}

		let pending_attestation = PendingAttestation {
			data: data.clone(),
			aggregation_bits: attestation.aggregation_bits.clone(),
			inclusion_delay: self.slot - attestation_slot,
			proposer_index: self.beacon_proposer_index()?,
		};

		let (push_current, parent_crosslink) =
			if data.target.epoch == self.current_epoch() {
				if data.source != self.current_justified_checkpoint {
					return Err(Error::AttestationInvalidData)
				}

				(true, self.current_crosslinks[data.crosslink.shard as usize].clone())
			} else {
				(false, self.previous_crosslinks[data.crosslink.shard as usize].clone())
			};

		if !(data.crosslink.parent_root == tree_root::<C::Digest, _>(&parent_crosslink) &&
			 data.crosslink.start_epoch == parent_crosslink.end_epoch &&
			 data.crosslink.end_epoch == min(data.target.epoch,
											 parent_crosslink.end_epoch +
											 C::max_epochs_per_crosslink()) &&
			 data.crosslink.data_root == Default::default())
		{
			return Err(Error::AttestationInvalidCrosslink)
		}

		if !self.is_valid_indexed_attestation::<BLS>(&self.indexed_attestation(attestation)?) {
			return Err(Error::AttestationInvalidSignature)
		}

		if push_current {
			self.current_epoch_attestations.push(pending_attestation);
		} else {
			self.previous_epoch_attestations.push(pending_attestation);
		}

		Ok(())
	}
}
