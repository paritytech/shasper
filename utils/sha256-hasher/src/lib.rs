// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

//! Hasher implementation for the Keccak-256 hash

extern crate parity_codec as codec;
extern crate parity_codec_derive as codec_derive;

use hash_db::Hasher;
use primitive_types::H256;
use sha2::{Sha256, Digest};
use plain_hasher::PlainHasher;
use codec_derive::{Encode, Decode};

/// Concrete `Hasher` impl for the Keccak-256 hash
#[derive(Default, Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Sha256Hasher;
impl Hasher for Sha256Hasher {
	type Out = H256;
	type StdHasher = PlainHasher;
	const LENGTH: usize = 32;

	fn hash(x: &[u8]) -> Self::Out {
		let result = Sha256::digest(x);
		let mut out = [0; 32];
		(&mut out[..]).copy_from_slice(&result[..]);
		out.into()
	}

}
