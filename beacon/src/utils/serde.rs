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

use serde::{Serializer, Deserializer};
use serde::de::{Error, Visitor};
use impl_serde::serialize;
use core::fmt;
use core::convert::TryFrom;

fn serialize_bitseq<T: AsRef<[bool]>, S: Serializer>(
	value: &T,
	serializer: S,
	is_list: bool,
) -> Result<S::Ok, S::Error> {
	let mut bytes = Vec::new();
    bytes.resize((value.as_ref().len() + if is_list { 8 } else { 7 }) / 8, 0u8);

    for i in 0..value.as_ref().len() {
        bytes[i / 8] |= (value.as_ref()[i] as u8) << (i % 8);
    }
	if is_list {
		bytes[value.as_ref().len() / 8] |= 1 << (value.as_ref().len() % 8);
	}

	serialize::serialize(&bytes, serializer)
}

/// Serialize a serde bitlist.
pub fn serialize_bitlist<ML, S: Serializer>(
	value: &bm_le::MaxVec<bool, ML>,
	serializer: S
) -> Result<S::Ok, S::Error> { serialize_bitseq(value, serializer, true) }

/// Serialize a serde bitvector.
pub fn serialize_bitvector<L: typenum::Unsigned, S: Serializer>(
	value: &vecarray::VecArray<bool, L>,
	serializer: S
) -> Result<S::Ok, S::Error> { serialize_bitseq(value, serializer, false) }

fn deserialize_bitseq<'a, 'de, D: Deserializer<'de>>(
	deserializer: D,
	is_list: bool,
) -> Result<Vec<bool>, D::Error> {
	let bytes = serialize::deserialize_check_len(deserializer, serialize::ExpectedLen::Any)?;

	let len = (bytes.len() - 1) * 8 +
		(if is_list { 7 } else { 8 } - bytes[bytes.len() - 1].leading_zeros() as usize);
	let mut ret = Vec::new();
	for i in 0..len {
		ret.push(bytes[i / 8] & (1 << (i % 8)) != 0);
	}
	Ok(ret)
}

/// Deserialize a serde bitlist.
pub fn deserialize_bitlist<'a, 'de, ML, D: Deserializer<'de>>(
	deserializer: D
) -> Result<bm_le::MaxVec<bool, ML>, D::Error> {
	Ok(bm_le::MaxVec::from(deserialize_bitseq(deserializer, true)?))
}

/// Deserialize a serde bitvector.
pub fn deserialize_bitvector<'a, 'de, L: typenum::Unsigned, D: Deserializer<'de>>(
	deserializer: D
) -> Result<vecarray::VecArray<bool, L>, D::Error> {
	let mut seq = deserialize_bitseq(deserializer, false)?;
	while seq.len() < L::to_usize() {
		seq.push(false);
	}

	vecarray::VecArray::try_from(seq).map_err(|_| D::Error::custom("Invalid bitlist"))
}

/// Deserialize u64 or string.
pub fn deserialize_uint<'a, 'de, D: Deserializer<'de>>(
	deserializer: D
) -> Result<u64, D::Error> {
	struct UintVisitor;

	impl<'a> Visitor<'a> for UintVisitor {
		type Value = u64;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			write!(formatter, "a hex encoded or decimal uint")
		}

		fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> where E: Error {
			Ok(value)
		}

		fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: Error {
			let value = match value.len() {
				0 => 0,
				2 if value.starts_with("0x") => 0,
				_ if value.starts_with("0x") => u64::from_str_radix(&value[2..], 16).map_err(|e| {
					Error::custom(format!("Invalid hex value {}: {}", value, e).as_str())
				})?,
				_ => u64::from_str_radix(value, 10).map_err(|e| {
					Error::custom(format!("Invalid decimal value {}: {:?}", value, e).as_str())
				})?
			};

			Ok(value)
		}

		fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: Error {
			self.visit_str(value.as_ref())
		}
	}

	deserializer.deserialize_any(UintVisitor)
}
