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

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "serde")]
pub use self::serde::*;

use crate::{Config, Error};
use crate::primitives::*;
use core::cmp::max;
use primitive_types::H256;

/// Convert integer to bytes.
pub fn to_bytes(v: Uint) -> H256 {
	let bytes = v.to_le_bytes();
	let mut ret = H256::default();
	(&mut ret[0..bytes.len()]).copy_from_slice(&bytes);
	ret
}

/// Convert byte to integer.
pub fn to_uint(v: &[u8]) -> Uint {
	let mut ret = 0u64.to_le_bytes();
	(&mut ret[..]).copy_from_slice(&v[..v.len()]);
	u64::from_le_bytes(ret)
}

/// Get integer squareroot.
pub fn integer_squareroot(n: Uint) -> Uint {
	let mut x = n;
	let mut y = (x + 1) / 2;
	while y < x {
		x = y;
		y = (x + n / x) / 2
	}
	x
}

/// Compare hash.
pub fn compare_hash(a: &H256, b: &H256) -> core::cmp::Ordering {
	for i in 0..32 {
		if a[i] > b[i] {
			return core::cmp::Ordering::Greater
		} else if a[i] < b[i] {
			return core::cmp::Ordering::Less
		}
	}
	core::cmp::Ordering::Equal
}

/// Compute shuffled index.
pub fn shuffled_index<C: Config>(
	mut index: Uint,
	index_count: Uint,
	seed: H256
) -> Result<ValidatorIndex, Error> {
	if !(index < index_count && index_count <= 2u64.pow(40)) {
		return Err(Error::IndexOutOfRange)
	}

	// Swap or not
	// (https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf)
	// See the 'generalized domain' algorithm on page 3

	for round in 0..C::shuffle_round_count() {
		let pivot = to_uint(
			&C::hash(&[
				&seed[..],
				&round.to_le_bytes()[..1]
			])[..8]
		) % index_count;
		let flip = (pivot + index_count - index) % index_count;
		let position = max(index, flip);
		let source = C::hash(&[
			&seed[..],
			&round.to_le_bytes()[..1],
			&(position / 256).to_le_bytes()[..4]
		]);
		let byte = source[((position % 256) / 8) as usize];
		let bit = (byte >> (position % 8)) % 2;
		index = if bit != 0 { flip } else { index };
	}

	Ok(index)
}

/// Compute committee indices.
pub fn compute_committee<C: Config>(
	indices: &[ValidatorIndex],
	seed: H256,
	index: Uint,
	count: Uint,
) -> Result<Vec<ValidatorIndex>, Error> {
	let start = (indices.len() as u64 * index) / count;
	let end = (indices.len() as u64 * (index + 1)) / count;

	(start..end).into_iter().map(move |i| {
		Ok(indices[
			shuffled_index::<C>(i, indices.len() as u64, seed)? as usize
		])
	}).collect::<Result<Vec<_>, Error>>()
}

/// Get epoch of slot.
pub fn epoch_of_slot<C: Config>(slot: Uint) -> Uint {
	slot / C::slots_per_epoch()
}

/// Get start slot of epoch.
pub fn start_slot_of_epoch<C: Config>(epoch: Uint) -> Uint {
	epoch * C::slots_per_epoch()
}

/// Get activation exit epoch.
pub fn activation_exit_epoch<C: Config>(epoch: Uint) -> Uint {
	epoch + 1 + C::activation_exit_delay()
}

/// Check whether given proof is valid merkle branch.
pub fn is_valid_merkle_branch<C: Config>(
	leaf: H256, proof: &[H256], depth: u64, index: u64, root: H256
) -> bool {
	if proof.len() as u64 != depth {
		return false
	}

	let mut value = leaf;
	for i in 0..depth {
		if index / 2u64.pow(i as u32) % 2 != 0 {
			value = C::hash(&[&proof[i as usize][..], &value[..]]);
		} else {
			value = C::hash(&[&value[..], &proof[i as usize][..]]);
		}
	}

	value == root
}

/// BLS signing domain given a domain type and fork version.
pub fn bls_domain(domain_type: u64, fork_version: Version) -> u64 {
	let mut bytes = [0u8; 8];
	(&mut bytes[0..4]).copy_from_slice(&domain_type.to_le_bytes()[0..4]);
	(&mut bytes[4..8]).copy_from_slice(fork_version.as_ref());

	u64::from_le_bytes(bytes)
}
