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

use core::ops::{BitOr, BitOrAssign, BitAnd, BitAndAssign};
use core::cmp::min;
#[cfg(feature = "parity-codec")]
use codec::{Encode, Decode};
#[cfg(feature = "serde")]
use impl_serde::serialize as bytes;
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer, Deserialize, Deserializer};

#[derive(Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Variable length bitfield.
pub struct BitField(pub Vec<u8>);

#[cfg(feature = "serde")]
impl Serialize for BitField {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		bytes::serialize(&self.0, serializer)
	}
}

#[cfg(feature = "serde")]
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

impl<H: hash_db::Hasher> ssz::Hashable<H> for BitField {
	fn hash(&self) -> H::Out {
		ssz::Hashable::<H>::hash(&self.0)
	}
}

impl<D: digest::Digest> ssz::Digestible<D> for BitField {
	fn hash(&self) -> generic_array::GenericArray<u8, D::OutputSize> {
		ssz::Digestible::<D>::hash(&self.0)
	}
}

impl BitOr for BitField {
	type Output = Self;

	fn bitor(mut self, rhs: Self) -> Self {
		self.bitor_assign(rhs);
		self
	}
}

impl BitOrAssign for BitField {
	fn bitor_assign(&mut self, rhs: Self) {
		for i in 0..min(self.0.len(), rhs.0.len()) {
			self.0[i] |= rhs.0[i];
		}
	}
}

impl BitAnd for BitField {
	type Output = Self;

	fn bitand(mut self, rhs: Self) -> Self {
		self.bitand_assign(rhs);
		self
	}
}

impl BitAndAssign for BitField {
	fn bitand_assign(&mut self, rhs: Self) {
		for i in 0..min(self.0.len(), rhs.0.len()) {
			self.0[i] &= rhs.0[i];
		}
	}
}

impl BitField {
	/// New with given length.
	pub fn new(length: usize) -> Self {
		let vec_len = (length + 7) / 8;
		let mut vec = Vec::new();
		vec.resize(vec_len, 0);
		Self(vec)
	}

	/// Get bit at index.
	pub fn get_bit(&self, index: usize) -> bool {
		(self.0[index / 8] >> (index % 8)) % 2 == 1
	}

	/// Set bit at index.
	pub fn set_bit(&mut self, index: usize, bit: bool) {
		if bit == true {
			self.0[index / 8] = self.0[index / 8] | (1 << (index % 8));
		} else {
			self.0[index / 8] = self.0[index / 8] & (!(1 << (index % 8)));
		}
	}

	/// Verify that the bitfield is of given size.
	pub fn verify(&self, size: usize) -> bool {
		if self.0.len() != (size + 7) / 8 {
			return false
		}

		for i in size..(self.0.len() * 8) {
			if self.get_bit(i) == true {
				return false
			}
		}

		true
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_set_bit() {
		let mut bitfield = BitField::new(22);
		assert_eq!(bitfield.get_bit(18), false);
		assert_eq!(bitfield.get_bit(19), false);
		assert_eq!(bitfield.get_bit(20), false);
		assert_eq!(bitfield.get_bit(21), false);

		bitfield.set_bit(19, true);
		assert_eq!(bitfield.get_bit(18), false);
		assert_eq!(bitfield.get_bit(19), true);
		assert_eq!(bitfield.get_bit(20), false);
		assert_eq!(bitfield.get_bit(21), false);

		bitfield.set_bit(19, false);
		assert_eq!(bitfield.get_bit(18), false);
		assert_eq!(bitfield.get_bit(19), false);
		assert_eq!(bitfield.get_bit(20), false);
		assert_eq!(bitfield.get_bit(21), false);
	}
}
