use crate::{Encode, Series, SeriesItem, FixedVec, FixedVecRef,
			KnownSize, SizeFromConfig, LenFromConfig, Error, Decode,
			DecodeWithConfig, Composite, SizeType};
use typenum::Unsigned;
use core::marker::PhantomData;
use alloc::vec::Vec;

fn decode_builtin_list<T: KnownSize + Decode, L>(
	value: &[u8],
) -> Result<FixedVec<T, L>, Error> {
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

	Ok(FixedVec(ret, PhantomData))
}

macro_rules! impl_builtin_fixed_uint_vector {
	( $( $t:ty ),* ) => { $(
		impl<'a, L: Unsigned> KnownSize for FixedVecRef<'a, $t, L> {
			fn size() -> Option<usize> {
				<$t>::size().map(|s| s * L::to_usize())
			}
		}

		impl<'a, C, L: LenFromConfig<C>> SizeFromConfig<C> for FixedVecRef<'a, $t, L> {
			fn size_from_config(config: &C) -> Option<usize> {
				let len = L::len_from_config(config);
				<$t>::size().map(|s| s * len)
			}
		}

		impl<'a, L> Encode for FixedVecRef<'a, $t, L> {
			fn encode(&self) -> Vec<u8> {
				let mut series = Series(Default::default());
				for value in self.0 {
					series.0.push(SeriesItem::Fixed(value.encode()));
				}
				series.encode()
			}
		}

		impl<L: Unsigned> Decode for FixedVec<$t, L> {
			fn decode(value: &[u8]) -> Result<Self, Error> {
				let decoded = decode_builtin_list(value)?;
				if decoded.0.len() != L::to_usize() {
					return Err(Error::InvalidLength)
				}
				Ok(decoded)
			}
		}

		impl<C, L: LenFromConfig<C>> DecodeWithConfig<C> for FixedVec<$t, L> {
			fn decode_with_config(value: &[u8], config: &C) -> Result<Self, Error> {
				let decoded = decode_builtin_list(value)?;
				if decoded.0.len() != L::len_from_config(config) {
					return Err(Error::InvalidLength)
				}
				Ok(decoded)
			}
		}
	)* }
}

impl_builtin_fixed_uint_vector!(u8, u16, u32, u64, u128);

impl<'a, L: Unsigned> KnownSize for FixedVecRef<'a, bool, L> {
	fn size() -> Option<usize> {
		Some((L::to_usize() + 7) / 8)
	}
}

impl<'a, C, L: LenFromConfig<C>> SizeFromConfig<C> for FixedVecRef<'a, bool, L> {
	fn size_from_config(config: &C) -> Option<usize> {
		let len = L::len_from_config(config);
		Some((len + 7) / 8)
	}
}

impl<'a, L> Encode for FixedVecRef<'a, bool, L> {
	fn encode(&self) -> Vec<u8> {
		let mut bytes = Vec::new();
        bytes.resize((self.0.len() + 7) / 8, 0u8);

        for i in 0..self.0.len() {
            bytes[i / 8] |= (self.0[i] as u8) << (i % 8);
        }
		bytes
	}
}

fn decode_bool_vector<L>(value: &[u8], len: usize) -> Result<FixedVec<bool, L>, Error> {
	let mut ret = Vec::new();
	for i in 0..len {
		if i / 8 >= value.len() {
			return Err(Error::IncorrectSize)
		}
        ret.push(value[i / 8] & (1 << (i % 8)) != 0);
    }
	Ok(FixedVec(ret, PhantomData))
}

impl<L: Unsigned> Decode for FixedVec<bool, L> {
	fn decode(value: &[u8]) -> Result<Self, Error> {
		let len = L::to_usize();
		decode_bool_vector(value, len)
	}
}

impl<C, L: LenFromConfig<C>> DecodeWithConfig<C> for FixedVec<bool, L> {
	fn decode_with_config(value: &[u8], config: &C) -> Result<Self, Error> {
		let len = L::len_from_config(config);
		decode_bool_vector(value, len)
	}
}

impl<'a, T: Composite + KnownSize, L: Unsigned> KnownSize for FixedVecRef<'a, T, L> {
	fn size() -> Option<usize> {
		T::size().map(|l| l * L::to_usize())
	}
}

impl<'a, C, T: Composite + SizeFromConfig<C>, L: LenFromConfig<C>> SizeFromConfig<C> for FixedVecRef<'a, T, L> {
	fn size_from_config(config: &C) -> Option<usize> {
		T::size_from_config(config).map(|l| l * L::len_from_config(config))
	}
}

impl<'a, T: Composite + Encode + SizeType, L> Encode for FixedVecRef<'a, T, L> {
	fn encode(&self) -> Vec<u8> {
		let mut series = Series(Default::default());
		for value in self.0 {
			if T::is_fixed() {
				series.0.push(SeriesItem::Fixed(value.encode()));
			} else {
				series.0.push(SeriesItem::Variable(value.encode()));
			}
		}
		series.encode()
	}
}

impl<'a, T: Composite + Decode + KnownSize, L: Unsigned> Decode for FixedVec<T, L> {
	fn decode(value: &[u8]) -> Result<Self, Error> {
		let value_typ = T::size();
		let series = Series::decode_list(value, value_typ)?;
		let mut ret = Vec::new();

		for part in series.0 {
			match part {
				SeriesItem::Fixed(fixed) => {
					if value_typ.is_some() {
						ret.push(T::decode(&fixed)?);
					} else {
						return Err(Error::InvalidType);
					}
				},
				SeriesItem::Variable(variable) => {
					if value_typ.is_none() {
						ret.push(T::decode(&variable)?);
					} else {
						return Err(Error::InvalidType);
					}
				},
			}
		}

		if L::to_usize() == ret.len() {
			Ok(FixedVec(ret, PhantomData))
		} else {
			Err(Error::InvalidLength)
		}
	}
}

impl<'a, C, T: Composite + DecodeWithConfig<C> + SizeFromConfig<C>, L: LenFromConfig<C>> DecodeWithConfig<C> for FixedVec<T, L> {
	fn decode_with_config(value: &[u8], config: &C) -> Result<Self, Error> {
		let value_typ = T::size_from_config(config);
		let series = Series::decode_list(value, value_typ)?;
		let mut ret = Vec::new();

		for part in series.0 {
			match part {
				SeriesItem::Fixed(fixed) => {
					if value_typ.is_some() {
						ret.push(T::decode_with_config(&fixed, config)?);
					} else {
						return Err(Error::InvalidType);
					}
				},
				SeriesItem::Variable(variable) => {
					if value_typ.is_none() {
						ret.push(T::decode_with_config(&variable, config)?);
					} else {
						return Err(Error::InvalidType);
					}
				},
			}
		}

		if L::len_from_config(config) == ret.len() {
			Ok(FixedVec(ret, PhantomData))
		} else {
			Err(Error::InvalidLength)
		}
	}
}

impl<'a, T: SizeType, L> SizeType for FixedVecRef<'a, T, L> {
	fn is_fixed() -> bool { T::is_fixed() }
}

impl<T: SizeType, L> SizeType for FixedVec<T, L> {
	fn is_fixed() -> bool { T::is_fixed() }
}

impl<T: SizeType, L> KnownSize for FixedVec<T, L> where
	for<'a> FixedVecRef<'a, T, L>: KnownSize,
{
	fn size() -> Option<usize> {
		FixedVecRef::<T, L>::size()
	}
}

impl<C, T: SizeType, L> SizeFromConfig<C> for FixedVec<T, L> where
	for<'a> FixedVecRef<'a, T, L>: SizeFromConfig<C>,
{
	fn size_from_config(config: &C) -> Option<usize> {
		FixedVecRef::<T, L>::size_from_config(config)
	}
}

impl<T, L> Encode for FixedVec<T, L> where
	for<'a> FixedVecRef<'a, T, L>: Encode
{
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		FixedVecRef(&self.0, PhantomData).using_encoded(f)
	}
}
