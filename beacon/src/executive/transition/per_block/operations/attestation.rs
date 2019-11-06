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
		if !(data.index < self.committee_count_at_slot(data.slot)) {
			return Err(Error::AttestationIncorrectCrosslinkData)
		}
		if !(data.target.epoch == self.current_epoch() ||
			 data.target.epoch == self.previous_epoch())
		{
			return Err(Error::AttestationIncorrectJustifiedEpochOrBlockRoot)
		}

		if !(data.slot + C::min_attestation_inclusion_delay() <=
			 self.slot &&
			 self.slot <=
			 data.slot + C::slots_per_epoch())
		{
			return Err(Error::AttestationSubmittedTooQuickly)
		}

		let pending_attestation = PendingAttestation {
			data: data.clone(),
			aggregation_bits: attestation.aggregation_bits.clone(),
			inclusion_delay: self.slot - data.slot,
			proposer_index: self.beacon_proposer_index()?,
		};

		let push_current =
			if data.target.epoch == self.current_epoch() {
				if data.source != self.current_justified_checkpoint {
					return Err(Error::AttestationInvalidData)
				}

				true
			} else {
				false
			};

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
