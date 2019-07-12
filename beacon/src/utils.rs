#[cfg(feature = "serde")]
use serde::{Serializer, Deserializer};
#[cfg(feature = "serde")]
use impl_serde::serialize;
use bm_le::MaxVec;

#[cfg(feature = "serde")]
pub fn serialize_bitlist<ML, S: Serializer>(
	value: &MaxVec<bool, ML>,
	serializer: S
) -> Result<S::Ok, S::Error> {
	let mut bytes = Vec::new();
    bytes.resize((value.0.len() + 7) / 8, 0u8);

    for i in 0..value.0.len() {
        bytes[i / 8] |= (value.0[i] as u8) << (i % 8);
    }
	serialize::serialize(&bytes, serializer)
}

#[cfg(feature = "serde")]
pub fn deserialize_bitlist<'a, 'de, ML, D: Deserializer<'de>>(
	deserializer: D
) -> Result<MaxVec<bool, ML>, D::Error> {
	serialize::deserialize_check_len(deserializer, serialize::ExpectedLen::Any)
		.map(|bytes| {
			let len = (bytes.len() - 1) * 8 +
				(7 - bytes[bytes.len() - 1].leading_zeros() as usize);
			println!("len: {:?}", len);
			let mut ret = Vec::new();
			for i in 0..len {
				ret.push(bytes[i / 8] & (1 << (i % 8)) != 0);
			}
			MaxVec::from(ret)
		})
}
