use serde::{Serializer, Deserializer, de::Error as _};
use impl_serde::serialize;
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
