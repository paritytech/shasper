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

#[cfg(feature = "std")]
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use substrate_primitives::H256;
use crypto::bls;
use fixed_hash::construct_fixed_hash;

#[cfg(feature = "std")]
use substrate_primitives::bytes;

const SIZE: usize = 48;

construct_fixed_hash! {
	/// Fixed 384-bit hash.
	pub struct H384(SIZE);
}

pub type AuthorityId = H384;

#[cfg(feature = "std")]
impl Serialize for H384 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		bytes::serialize(&self.0, serializer)
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for H384 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		bytes::deserialize_check_len(deserializer, bytes::ExpectedLen::Exact(SIZE))
			.map(|x| H384::from_slice(&x))
	}
}

impl codec::Encode for H384 {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.using_encoded(f)
	}
}
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
	fn decode<I: ::ssz::Input>(input: &mut I) -> Option<Self> {
		let mut vec = [0u8; SIZE];
		if input.read(&mut vec[..SIZE]) != SIZE {
			None
		} else {
			Some(H384::from(&vec))
		}
	}
}

impl ssz_hash::SpecHash for H384 {
	fn spec_hash<H: ::hash_db::Hasher>(&self) -> H::Out {
		let encoded = ssz::Encode::encode(self);
		H::hash(&encoded)
	}
}

impl H384 {
	pub fn into_public(&self) -> Option<bls::Public> {
		bls::Public::from_compressed_bytes(self.as_ref())
	}

	pub fn from_public(public: bls::Public) -> Self {
		H384::from_slice(&public.to_compressed_bytes())
	}
}

impl Into<AuthorityId> for bls::Public {
	fn into(self) -> AuthorityId {
		AuthorityId::from_public(self)
	}
}

impl Into<H256> for H384 {
	fn into(self) -> H256 {
		H256::from_slice(&self[0..32])
	}
}
