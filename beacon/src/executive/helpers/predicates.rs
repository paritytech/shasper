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

use crate::types::{IndexedAttestation, AttestationDataAndCustodyBit};
use crate::{BeaconExecutive, Config, BLSConfig};
use bm_le::tree_root;

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Check if ``indexed_attestation`` has valid indices and signature.
	pub fn is_valid_indexed_attestation<BLS: BLSConfig>(
		&self,
		indexed_attestation: &IndexedAttestation<C>
	) -> bool {
		let bit_0_indices = &indexed_attestation.custody_bit_0_indices;
		let bit_1_indices = &indexed_attestation.custody_bit_1_indices;

		if bit_1_indices.len() > 0 {
			return false
		}

		let total_len =
			(bit_0_indices.len() + bit_1_indices.len()) as u64;
		if !(total_len <= C::max_validators_per_committee()) {
			return false
		}

		// Ensure no duplicate indices across custody bits
		for index in bit_0_indices.as_ref() {
			if bit_1_indices.contains(index) {
				return false
			}
		}

		if !bit_0_indices.windows(2).all(|w| w[0] <= w[1]) {
			return false
		}

		if !bit_1_indices.windows(2).all(|w| w[0] <= w[1]) {
			return false
		}

		BLS::verify_multiple(
			&[
				BLS::aggregate_pubkeys(
					&bit_0_indices
						.iter()
						.map(|i| self.validators[*i as usize].pubkey)
						.collect::<Vec<_>>()[..]
				),
				BLS::aggregate_pubkeys(
					&bit_1_indices
						.iter()
						.map(|i| self.validators[*i as usize].pubkey)
						.collect::<Vec<_>>()[..]
				),
			],
			&[
				tree_root::<C::Digest, _>(&AttestationDataAndCustodyBit {
					data: indexed_attestation.data.clone(),
					custody_bit: false,
				}),
				tree_root::<C::Digest, _>(&AttestationDataAndCustodyBit {
					data: indexed_attestation.data.clone(),
					custody_bit: true,
				}),
			],
			&indexed_attestation.signature,
			self.domain(
				C::domain_beacon_attester(),
				Some(indexed_attestation.data.target.epoch)
			)
		)
	}
}
