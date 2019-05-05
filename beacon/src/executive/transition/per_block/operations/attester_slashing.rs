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
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit, Block, BeaconBlockHeader, ProposerSlashing, AttesterSlashing};
use crate::utils::to_bytes;
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Push a new `AttesterSlashing` to the state.
	pub fn process_attester_slashing(&mut self, attester_slashing: AttesterSlashing) -> Result<(), Error> {
		let attestation_1 = attester_slashing.attestation_1;
		let attestation_2 = attester_slashing.attestation_2;

		if !attestation_1.data.is_slashable(&attestation_2.data) {
			return Err(Error::AttesterSlashingNotSlashable)
		}

		if !self.verify_indexed_attestation(&attestation_1)? {
			return Err(Error::AttesterSlashingInvalid)
		}

		if !self.verify_indexed_attestation(&attestation_2)? {
			return Err(Error::AttesterSlashingInvalid)
		}

		let mut slashed_any = false;
		let attesting_indices_1 = attestation_1.custody_bit_0_indices.clone().into_iter()
			.chain(attestation_1.custody_bit_1_indices.clone().into_iter());
		let attesting_indices_2 = attestation_2.custody_bit_0_indices.clone().into_iter()
			.chain(attestation_2.custody_bit_1_indices.clone().into_iter());

		let mut full = Vec::new();
		for index in attesting_indices_1.chain(attesting_indices_2) {
			if !full.contains(&index) {
				full.push(index);
			}
		}

		for index in full {
			if self.state.validator_registry[index as usize]
				.is_slashable(self.current_epoch())
			{
				self.slash_validator(index, None)?;
				slashed_any = true;
			}
		}

		if !slashed_any {
			return Err(Error::AttesterSlashingEmptyIndices)
		}

		Ok(())
	}
}
