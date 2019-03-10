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
use crypto::bls;
use fixed_hash::construct_fixed_hash;
#[cfg(feature = "std")]
use impl_serde::serialize as bytes;

const SIZE: usize = 96;

construct_fixed_hash! {
	/// Fixed 384-bit hash.
	pub struct H768(SIZE);
}

pub type Signature = H768;

#[cfg(feature = "std")]
impl Serialize for H768 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		bytes::serialize(&self.0, serializer)
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for H768 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		bytes::deserialize_check_len(deserializer, bytes::ExpectedLen::Exact(SIZE))
			.map(|x| H768::from_slice(&x))
	}
}

impl codec::Encode for H768 {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.using_encoded(f)
	}
}
impl codec::Decode for H768 {
	fn decode<I: codec::Input>(input: &mut I) -> Option<Self> {
		<[u8; SIZE] as codec::Decode>::decode(input).map(H768)
	}
}

impl ssz::Encode for H768 {
	fn encode_to<W: ::ssz::Output>(&self, dest: &mut W) {
		dest.write(self.as_ref())
	}
}
impl ssz::Decode for H768 {
	fn decode<I: ::ssz::Input>(input: &mut I) -> Option<Self> {
		let mut vec = [0u8; SIZE];
		if input.read(&mut vec[..SIZE]) != SIZE {
			None
		} else {
			Some(H768::from(&vec))
		}
	}
}

impl ssz::Hashable for H768 {
	fn hash<H: ::hash_db::Hasher>(&self) -> H::Out {
		let encoded = ssz::Encode::encode(self);
		H::hash(&encoded)
	}
}

impl H768 {
	pub fn into_signature(&self) -> Option<bls::Signature> {
		bls::Signature::from_compressed_bytes(self.as_ref())
	}

	pub fn into_aggregate_signature(&self) -> Option<bls::AggregateSignature> {
		bls::AggregateSignature::from_compressed_bytes(self.as_ref())
	}

	pub fn from_signature(sig: bls::Signature) -> Self {
		H768::from_slice(&sig.to_compressed_bytes())
	}

	pub fn from_aggregate_signature(sig: bls::AggregateSignature) -> Self {
		H768::from_slice(&sig.to_compressed_bytes())
	}
}

impl Into<Signature> for bls::Signature {
	fn into(self) -> Signature {
		Signature::from_signature(self)
	}
}

impl Into<Signature> for bls::AggregateSignature {
	fn into(self) -> Signature {
		Signature::from_aggregate_signature(self)
	}
}
