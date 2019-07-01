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
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit, Validator};
use crate::utils::{self, to_bytes};
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub(crate) fn is_active_validator(&self, validator: &Validator, epoch: Uint) -> bool {
		validator.is_active(epoch)
	}

	pub(crate) fn is_slashable_validator(&self, validator: &Validator, epoch: Uint) -> bool {
		validator.is_slashable(epoch)
	}

	pub(crate) fn is_slashable_attestation_data(&self, data_1: &AttestationData, data_2: &AttestationData) -> bool {
		data_1.is_slashable(data_2)
	}

	pub(crate) fn is_valid_indexed_attestation(&self, indexed_attestation: &IndexedAttestation) -> bool {
		let bit_0_indices = &indexed_attestation.custody_bit_0_indices;
		let bit_1_indices = &indexed_attestation.custody_bit_1_indices;

		if bit_1_indices.len() > 0 {
			return false
		}

		let total_len =
			(bit_0_indices.len() + bit_1_indices.len()) as u64;
		if !(total_len <= self.config.max_indices_per_attestation()) {
			return false
		}

		// Ensure no duplicate indices across custody bits
		for index in bit_0_indices {
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

		if !self.config.bls_verify_multiple(
			&[
				self.config.bls_aggregate_pubkeys(
					&bit_0_indices
						.iter()
						.map(|i| self.state.validator_registry[*i as usize].pubkey)
						.collect::<Vec<_>>()[..]
				),
				self.config.bls_aggregate_pubkeys(
					&bit_1_indices
						.iter()
						.map(|i| self.state.validator_registry[*i as usize].pubkey)
						.collect::<Vec<_>>()[..]
				),
			],
			&[
				H256::from_slice(
					Digestible::<C::Digest>::hash(&AttestationDataAndCustodyBit {
						data: indexed_attestation.data.clone(),
						custody_bit: false,
					}).as_slice()
				),
				H256::from_slice(
					Digestible::<C::Digest>::hash(&AttestationDataAndCustodyBit {
						data: indexed_attestation.data.clone(),
						custody_bit: true,
					}).as_slice()
				),
			],
			&indexed_attestation.signature,
			self.domain(
				self.config.domain_attestation(),
				Some(indexed_attestation.data.target_epoch)
			)
		) {
			return false
		}

		true
	}

	pub(crate) fn is_valid_merkle_branch(&self, leaf: H256, proof: &[H256], depth: u64, index: u64, root: H256) -> bool {
		if proof.len() as u64 != depth {
			return false
		}

		let mut value = leaf;
		for i in 0..depth {
			if index / 2u64.pow(i as u32) % 2 != 0 {
				value = self.hash(&[&proof[i as usize][..], &value[..]]);
			} else {
				value = self.hash(&[&value[..], &proof[i as usize][..]]);
			}
		}
		value == root
	}
}
