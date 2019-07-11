use crate::{Codec, Encode, Decode, Error, SeriesItem, Series, Size};
use alloc::vec::Vec;

pub fn encode_list<T: Encode>(
	values: &[T]
) -> Vec<u8> {
	let mut series = Series(Default::default());
	for value in values {
		if <T as Codec>::Size::is_fixed() {
			series.0.push(SeriesItem::Fixed(value.encode()));
		} else {
			series.0.push(SeriesItem::Variable(value.encode()));
		}
	}
	series.encode()
}

pub fn decode_list<T: Decode>(
	value: &[u8],
) -> Result<Vec<T>, Error> {
	let value_typ = <T as Codec>::Size::size();
	let series = Series::decode_list(value, value_typ)?;
	let mut ret = Vec::new();

	for part in series.0 {
		match part {
			SeriesItem::Fixed(fixed) => {
				if <T as Codec>::Size::is_fixed() {
					ret.push(T::decode(&fixed)?);
				} else {
					return Err(Error::InvalidType)
				}
			},
			SeriesItem::Variable(variable) => {
				if <T as Codec>::Size::is_variable() {
					ret.push(T::decode(&variable)?);
				} else {
					return Err(Error::InvalidType)
				}
			},
		}
	}

	Ok(ret)
}
