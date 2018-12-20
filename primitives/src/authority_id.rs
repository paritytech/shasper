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

#[cfg(feature = "std")]
use primitives::bytes;

construct_fixed_hash! {
	/// Fixed 384-bit hash.
	pub struct H384(48);
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
		bytes::deserialize_check_len(deserializer, bytes::ExpectedLen::Exact(48))
			.map(|x| H384::from_slice(&x))
	}
}

impl ::parity_codec::Encode for H384 {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.using_encoded(f)
	}
}
impl ::parity_codec::Decode for H384 {
	fn decode<I: ::parity_codec::Input>(input: &mut I) -> Option<Self> {
		<[u8; 48] as ::parity_codec::Decode>::decode(input).map(H384)
	}
}

impl H384 {
	pub fn into_public(&self) -> Option<bls::Public> {
		bls::Public::from_bytes(self.as_ref()).ok()
	}

	pub fn from_public(public: bls::Public) -> Self {
		H384::from_slice(&public.as_bytes())
	}
}

impl Into<AuthorityId> for bls::Public {
	fn into(self) -> AuthorityId {
		AuthorityId::from_public(self)
	}
}
