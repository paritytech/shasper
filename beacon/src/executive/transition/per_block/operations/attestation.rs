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

use core::cmp::min;
use ssz::Digestible;
use crate::primitives::H256;
use crate::types::{Attestation, PendingAttestation};
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Push a new `Attestation` to the state.
	pub fn process_attestation(&mut self, attestation: Attestation) -> Result<(), Error> {
		let data = attestation.data.clone();
		let attestation_slot = self.attestation_data_slot(&data)?;

		if !(attestation_slot + self.config.min_attestation_inclusion_delay() <=
			 self.state.slot &&
			 self.state.slot <=
			 attestation_slot + self.config.slots_per_epoch())
		{
			return Err(Error::AttestationSubmittedTooQuickly)
		}

		let pending_attestation = PendingAttestation {
			data: data.clone(),
			aggregation_bitfield: attestation.aggregation_bitfield.clone(),
			inclusion_delay: self.state.slot - attestation_slot,
			proposer_index: self.beacon_proposer_index()?,
		};

		if !(data.target_epoch == self.current_epoch() ||
			 data.target_epoch == self.previous_epoch())
		{
			return Err(Error::AttestationIncorrectJustifiedEpochOrBlockRoot)
		}

		let (ffg_data, parent_crosslink, push_current) = if data.target_epoch == self.current_epoch() {
			let ffg_data = (self.state.current_justified_epoch,
							self.state.current_justified_root,
							self.current_epoch());
			let parent_crosslink = self.state.current_crosslinks[
				data.crosslink.shard as usize
			].clone();
			(ffg_data, parent_crosslink, true)
		} else {
			let ffg_data = (self.state.previous_justified_epoch,
							self.state.previous_justified_root,
							self.previous_epoch());
			let parent_crosslink = self.state.previous_crosslinks[
				data.crosslink.shard as usize
			].clone();
			(ffg_data, parent_crosslink, false)
		};

		if ffg_data != (data.source_epoch, data.source_root, data.target_epoch) {
			return Err(Error::AttestationIncorrectJustifiedEpochOrBlockRoot)
		}

		if data.crosslink.start_epoch != parent_crosslink.end_epoch {
			return Err(Error::AttestationIncorrectCrosslinkData)
		}

		if data.crosslink.end_epoch != min(
			data.target_epoch,
			parent_crosslink.end_epoch + self.config.max_epochs_per_crosslink()
		) {
			return Err(Error::AttestationIncorrectCrosslinkData)
		}

		if data.crosslink.parent_root != H256::from_slice(Digestible::<C::Digest>::hash(
			&parent_crosslink
		).as_slice()) {
			return Err(Error::AttestationIncorrectCrosslinkData)
		}

		if data.crosslink.data_root != H256::default() {
			return Err(Error::AttestationIncorrectCrosslinkData)
		}

		self.validate_indexed_attestation(&self.convert_to_indexed(attestation.clone())?)?;

		if push_current {
			self.state.current_epoch_attestations.push(pending_attestation);
		} else {
			self.state.previous_epoch_attestations.push(pending_attestation);
		}

		Ok(())
	}
}
