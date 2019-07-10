use crate::{KnownSize, Encode, Decode, Composite, SizeType, Error, SeriesItem, Series};
use alloc::vec::Vec;

pub fn encode_builtin_list<T: KnownSize + Encode>(
	values: &[T]
) -> Vec<u8> {
	let mut series = Series(Default::default());
	for value in values {
		if T::is_fixed() {
			series.0.push(SeriesItem::Fixed(value.encode()));
		} else {
			series.0.push(SeriesItem::Variable(value.encode()));
		}
	}
	series.encode()
}


pub fn decode_builtin_list<T: KnownSize + Decode>(
	value: &[u8],
) -> Result<Vec<T>, Error> {
	let series = Series::decode_list(value, T::size())?;
	let mut ret = Vec::new();

	for part in series.0 {
		match part {
			SeriesItem::Fixed(fixed) => {
				ret.push(T::decode(&fixed)?);
			},
			SeriesItem::Variable(_) => {
				return Err(Error::InvalidType);
			},
		}
	}

	Ok(ret)
}

pub fn encode_composite<T: Composite + Encode + SizeType>(
	values: &[T]
) -> Vec<u8> {
	let mut series = Series(Default::default());
	for value in values {
		if T::is_fixed() {
			series.0.push(SeriesItem::Fixed(value.encode()));
		} else {
			series.0.push(SeriesItem::Variable(value.encode()));
		}
	}
	series.encode()
}

pub fn decode_composite<T: Composite + SizeType, F: Fn(&[u8]) -> Result<T, Error>>(
	value: &[u8],
	value_typ: Option<usize>,
	f: F
) -> Result<Vec<T>, Error> {
	let series = Series::decode_list(value, value_typ)?;
	let mut ret = Vec::new();

	for part in series.0 {
		match part {
			SeriesItem::Fixed(fixed) => {
				if T::is_fixed() {
					ret.push(f(&fixed)?);
				} else {
					return Err(Error::InvalidType)
				}
			},
			SeriesItem::Variable(variable) => {
				if T::is_variable() {
					ret.push(f(&variable)?);
				} else {
					return Err(Error::InvalidType)
				}
			},
		}
	}

	Ok(ret)
}
