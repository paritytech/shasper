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

use codec::{Encode, Decode};
#[cfg(feature = "std")]
use impl_serde::serialize as bytes;
#[cfg(feature = "std")]
use serde::{Serialize, Serializer, Deserialize, Deserializer};

// TODO: Validate bitfield trailing bits in encoding/decoding.

#[derive(Clone, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BitField(pub Vec<u8>);

#[cfg(feature = "std")]
impl Serialize for BitField {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		bytes::serialize(&self.0, serializer)
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for BitField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		bytes::deserialize_check_len(deserializer, bytes::ExpectedLen::Any)
			.map(|x| BitField(x))
	}
}

impl ssz::Prefixable for BitField {
	fn prefixed() -> bool {
		<Vec<u8>>::prefixed()
	}
}


impl ssz::Encode for BitField {
	fn encode_to<W: ssz::Output>(&self, dest: &mut W) {
		ssz::Encode::encode_to(&self.0, dest)
	}
}

impl ssz::Decode for BitField {
	fn decode_as<I: ssz::Input>(input: &mut I) -> Option<(Self, usize)> {
		<Vec<u8>>::decode_as(input).map(|(s, u)| (BitField(s), u))
	}
}

impl ssz::Composite for BitField { }

impl ssz::Hashable for BitField {
	fn hash<H: hash_db::Hasher>(&self) -> H::Out {
		self.0.hash::<H>()
	}
}

impl BitField {
	pub fn has_voted(&self, index: usize) -> bool {
		(self.0[index / 8] >> (index % 8)) == 1
	}

	pub fn verify(&self, size: usize) -> bool {
		if self.0.len() != (size + 7) / 8 {
			return false
		}

		for i in size..(self.0.len() * 8) {
			if self.has_voted(i) {
				return false
			}
		}

		true
	}
}
