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
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit};
use crate::utils::{self, to_bytes};
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub(crate) fn compute_shuffled_index(
		&self, mut index: Uint, index_count: Uint, seed: H256
	) -> Option<ValidatorIndex> {
		if !(index < index_count && index_count <= 2u64.pow(40)) {
			return None
		}

		// Swap or not
		// (https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf)
		// See the 'generalized domain' algorithm on page 3

		for round in 0..self.shuffle_round_count() {
			let pivot = to_uint(
				&self.hash(&[
					&seed[..],
					&round.to_le_bytes()[..1]
				])[..8]
			) % index_count;
			let flip = (pivot + index_count - index) % index_count;
			let position = max(index, flip);
			let source = self.hash(&[
				&seed[..],
				&round.to_le_bytes()[..1],
				&(position / 256).to_le_bytes()[..4]
			]);
			let byte = source[((position % 256) / 8) as usize];
			let bit = (byte >> (position % 8)) % 2;
			index = if bit != 0 { flip } else { index };
		}

		Some(index)
	}

	pub(crate) fn compute_committee(&self, indices: &[ValidatorIndex], seed: H256, index: Uint, count: Uint) -> Result<Vec<ValidatorIndex>, Error> {
		let start = (indices.len() as u64 * index) / count;
		let end = (indices.len() as u64 * (index + 1)) / count;

		(start..end).into_iter().map(move |i| {
			Ok(indices[
				self.compute_shuffled_index(i, indices.len() as u64, seed)
					.ok_or(Error::IndexOutOfRange)? as usize
			])
		}).collect::<Result<Vec<_>, Error>>()
	}

	pub(crate) fn compute_epoch_of_slot(&self, slot: Uint) -> Uint {
		slot / self.slots_per_epoch()
	}

	pub(crate) fn compute_start_slot_of_epoch(&self, epoch: Uint) -> Uint {
		epoch.saturating_mul(self.slots_per_epoch())
	}

	pub(crate) fn compute_activation_exit_epoch(&self, epoch: Uint) -> Uint {
		epoch + 1 + self.activation_exit_delay()
	}

	pub(crate) fn compute_domain(&self, domain_type: Uint, fork_version: Option<Version>) -> Uint {
		utils::bls_domain(domain_type, fork_version)
	}
}
