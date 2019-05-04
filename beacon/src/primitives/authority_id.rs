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

const SIZE: usize = 48;

construct_fixed_hash! {
	/// Fixed 384-bit hash.
	pub struct H384(SIZE);
}

/// BLS 384-bit public key.
pub type ValidatorId = H384;

#[cfg(feature = "serde")]
impl Serialize for H384 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		bytes::serialize(&self.0, serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for H384 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		bytes::deserialize_check_len(deserializer, bytes::ExpectedLen::Exact(SIZE))
			.map(|x| H384::from_slice(&x))
	}
}

#[cfg(feature = "parity-codec")]
impl codec::Encode for H384 {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.using_encoded(f)
	}
}

#[cfg(feature = "parity-codec")]
impl codec::Decode for H384 {
	fn decode<I: codec::Input>(input: &mut I) -> Option<Self> {
		<[u8; SIZE] as codec::Decode>::decode(input).map(H384)
	}
}

impl ssz::Encode for H384 {
	fn encode_to<W: ::ssz::Output>(&self, dest: &mut W) {
		dest.write(self.as_ref())
	}
}
impl ssz::Decode for H384 {
	fn decode_as<I: ::ssz::Input>(input: &mut I) -> Option<(Self, usize)> {
		let mut vec = [0u8; SIZE];
		if input.read(&mut vec[..SIZE]) != SIZE {
			None
		} else {
			Some((H384::from(&vec), SIZE))
		}
	}
}

impl ssz::Prefixable for H384 {
	fn prefixed() -> bool {
		false
	}
}

impl ssz::Composite for H384 { }

impl<H: hash_db::Hasher> ssz::Hashable<H> for H384 {
	fn hash(&self) -> H::Out {
		ssz::hash::hash_db_hasher::merkleize::<H>(ssz::hash::hash_db_hasher::chunkify(self.as_ref()))
	}
}

impl<D: digest::Digest> ssz::Digestible<D> for H384 {
	fn hash(&self) -> generic_array::GenericArray<u8, D::OutputSize> {
		ssz::hash::digest_hasher::merkleize::<D>(ssz::hash::digest_hasher::chunkify(self.as_ref()))
	}
}

impl Into<primitive_types::H256> for H384 {
	fn into(self) -> primitive_types::H256 {
		primitive_types::H256::from_slice(&self[0..32])
	}
}
