// Copyright 2017-2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

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

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use fixed_hash::construct_fixed_hash;
#[cfg(feature = "serde")]
use impl_serde::serialize as bytes;

const SIZE: usize = 4;

construct_fixed_hash! {
	/// Fixed 384-bit hash.
	pub struct H32(SIZE);
}

/// Beacon fork version.
pub type Version = H32;

#[cfg(feature = "serde")]
impl Serialize for H32 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		bytes::serialize(&self.0, serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for H32 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		bytes::deserialize_check_len(deserializer, bytes::ExpectedLen::Exact(SIZE))
			.map(|x| H32::from_slice(&x))
	}
}

#[cfg(feature = "parity-codec")]
impl codec::Encode for H32 {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.using_encoded(f)
	}
}

#[cfg(feature = "parity-codec")]
impl codec::Decode for H32 {
	fn decode<I: codec::Input>(input: &mut I) -> Option<Self> {
		<[u8; SIZE] as codec::Decode>::decode(input).map(H32)
	}
}

impl ssz::Encode for H32 {
	fn encode_to<W: ::ssz::Output>(&self, dest: &mut W) {
		dest.write(self.as_ref())
	}
}
impl ssz::Decode for H32 {
	fn decode_as<I: ::ssz::Input>(input: &mut I) -> Option<(Self, usize)> {
		let mut vec = [0u8; SIZE];
		if input.read(&mut vec[..SIZE]) != SIZE {
			None
		} else {
			Some((H32::from(&vec), SIZE))
		}
	}
}

impl ssz::Prefixable for H32 {
	fn prefixed() -> bool {
		false
	}
}

impl ssz::Composite for H32 { }

impl<H: hash_db::Hasher> ssz::Hashable<H> for H32 {
	fn hash(&self) -> H::Out {
		ssz::hash::merkleize::<H>(ssz::hash::chunkify(self.as_ref()))
	}
}
