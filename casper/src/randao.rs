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

//! RANDAO constructs.
//!
//! RANAO is for generating random numbers in a decentralized fashion.
//! In RANDAO, participants publish "onion" of hashed chains. Each time
//! when the participant is required to add entropy into the system, it
//! reveals one layer of the onion.

use hash_db::{Hasher, DebugIfStd};
use core::hash;
use core::ops::BitXor;

/// A RANDAO mix. Combine revealed values together.
pub struct RandaoMix<T>(T);

impl<T> RandaoMix<T> where
	T: BitXor<Output=T> + AsRef<[u8]> + AsMut<[u8]> + Default + DebugIfStd + PartialEq + Eq + hash::Hash + Send + Sync + Clone + Copy
{
	/// Create a new mix.
	pub fn new(val: T) -> Self {
		RandaoMix(val)
	}

	/// Mix the current value with a new reveal.
	pub fn mix<H: Hasher<Out=T>>(&mut self, reveal: &T) {
		let input = self.0 ^ *reveal;
		self.0 = H::hash(input.as_ref());
	}
}

impl<T> AsRef<T> for RandaoMix<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}

/// A RANDAO commitment.
pub struct RandaoCommitment<T>(T);

impl<T> RandaoCommitment<T> where
	T: AsRef<[u8]> + AsMut<[u8]> + Default + DebugIfStd + PartialEq + Eq + hash::Hash + Send + Sync + Clone + Copy
{
	/// Create a new commitment.
	pub fn new(val: T) -> Self {
		RandaoCommitment(val)
	}

	/// Reveal the commitment, with the given revealed value, and how many
	/// layers to be revealed. Returns whether the reveal is successful.
	pub fn reveal<H: Hasher<Out=T>>(&mut self, reveal: &T, layers: usize) -> bool {
		let mut revealed = *reveal;
		for _ in 0..layers {
			revealed = H::hash(revealed.as_ref());
		}

		if revealed != self.0 {
			false
		} else {
			self.0 = *reveal;
			true
		}
	}
}

impl<T> AsRef<T> for RandaoCommitment<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}

#[cfg(test)]
mod tests {
	use hash_db::Hasher;
	use plain_hasher::PlainHasher;
	use super::*;

	/// A dummy hasher for `[u8; 1]`. A hash for `n` is `n + 1`.
	struct DummyHasher;

	impl Hasher for DummyHasher {
		type Out = [u8; 1];
		type StdHasher = PlainHasher;
		const LENGTH: usize = 1;

		fn hash(x: &[u8]) -> Self::Out {
			assert!(x.len() == 1);
			[x[0] + 1]
		}
	}

	#[test]
	fn reveal_commitment_255_layers() {
		let mut commitment = RandaoCommitment::new([255]);
		assert!(!commitment.reveal::<DummyHasher>(&[0], 254));
		assert_eq!(commitment.as_ref(), &[255]);
		assert!(commitment.reveal::<DummyHasher>(&[0], 255));
		assert_eq!(commitment.as_ref(), &[0]);
	}
}
