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
use beacon::{Config, BLSConfig};
use beacon::primitives::H256;
use beacon::types::{Attestation, AttestationDataAndCustodyBit};
use std::collections::HashMap;
use core::marker::PhantomData;
use bm_le::tree_root;

pub struct AttestationPool<C: Config, BLS: BLSConfig> {
	pool: HashMap<H256, Vec<Attestation<C>>>,
	_marker: PhantomData<BLS>,
}

impl<C: Config, BLS: BLSConfig> AttestationPool<C, BLS> {
	pub fn new() -> Self {
		Self {
			pool: Default::default(),
			_marker: PhantomData,
		}
	}

	pub fn push(&mut self, attestation: Attestation<C>) {
		let hash = tree_root::<C::Digest, _>(&AttestationDataAndCustodyBit {
			data: attestation.data.clone(),
			custody_bit: false,
		});

		self.pool.entry(hash)
			.and_modify(|existings| {
				let attestation = attestation.clone();
				let mut aggregated = false;

				for existing in existings.iter_mut() {
					let has_duplicate = {
						let mut has_duplicate = false;
						for i in 0..existing.aggregation_bits.len() {
							if attestation.aggregation_bits[i] {
								has_duplicate = true;
							}
						}
						has_duplicate
					};

					if has_duplicate {
						continue
					}

					for (i, bit) in attestation.aggregation_bits.iter().cloned().enumerate() {
						existing.aggregation_bits[i] |= bit;
					}
					for (i, bit) in attestation.custody_bits.iter().cloned().enumerate() {
						existing.custody_bits[i] |= bit;
					}
					for i in 0..existing.custody_bits.len() {
						assert_eq!(attestation.custody_bits[i], false);
					}
					existing.signature = BLS::aggregate_signatures(&[
						existing.signature, attestation.signature.clone()
					]);

					aggregated = true;
					break;
				}

				if !aggregated {
					existings.push(attestation);
				}
			})
			.or_insert(vec![attestation]);
    }

	pub fn pop(&mut self, key: &H256) {
		self.pool.remove(key);
	}

	pub fn iter(&self) -> impl Iterator<Item=(&H256, &Attestation<C>)> {
		self.pool.iter().flat_map(|(h, ats)| ats.iter().map(move |at| (h, at)))
	}
}
