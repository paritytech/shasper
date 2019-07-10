use crate::{VariableVecRef, VariableVec, SizeType, Encode, Decode, Error, KnownSize, SizeFromConfig, MaxLenFromConfig, DecodeWithConfig, Composite};
use crate::utils::{encode_builtin_list, decode_builtin_list, encode_composite, decode_composite};
use core::marker::PhantomData;
use alloc::vec::Vec;
use typenum::Unsigned;

impl<'a, T, ML> SizeType for VariableVecRef<'a, T, ML> {
	fn is_variable() -> bool { true }
}

impl<T, ML> SizeType for VariableVec<T, ML> {
	fn is_variable() -> bool { true }
}

impl<'a, T, ML> KnownSize for VariableVecRef<'a, T, ML> {
	fn size() -> Option<usize> { None }
}

impl<T, ML> KnownSize for VariableVec<T, ML> {
	fn size() -> Option<usize> { None }
}

impl<'a, C, T, ML> SizeFromConfig<C> for VariableVecRef<'a, T, ML> {
	fn size_from_config(_config: &C) -> Option<usize> { None }
}

impl<C, T, ML> SizeFromConfig<C> for VariableVec<T, ML> {
	fn size_from_config(_config: &C) -> Option<usize> { None }
}

macro_rules! impl_builtin_variable_uint_list {
	( $( $t:ty ),* ) => { $(
		impl<'a, ML> Encode for VariableVecRef<'a, $t, ML> {
			fn encode(&self) -> Vec<u8> {
				encode_builtin_list(self.0)
			}
		}

		impl<ML: Unsigned> Decode for VariableVec<$t, ML> {
			fn decode(value: &[u8]) -> Result<Self, Error> {
				let decoded = decode_builtin_list(value)?;
				if decoded.len() > ML::to_usize() {
					return Err(Error::ListTooLarge)
				}
				Ok(VariableVec(decoded, Some(ML::to_usize()), PhantomData))
			}
		}

		impl<C, ML: MaxLenFromConfig<C>> DecodeWithConfig<C> for VariableVec<$t, ML> {
			fn decode_with_config(value: &[u8], config: &C) -> Result<Self, Error> {
				let decoded = decode_builtin_list(value)?;
				if let Some(max_len) = ML::max_len_from_config(config) {
					if decoded.len() > max_len {
						return Err(Error::InvalidLength)
					}
				}
				Ok(VariableVec(decoded, ML::max_len_from_config(config), PhantomData))
			}
		}
	)* }
}

impl_builtin_variable_uint_list!(u8, u16, u32, u64, u128);

impl<'a, ML> Encode for VariableVecRef<'a, bool, ML> {
	fn encode(&self) -> Vec<u8> {
		let mut bytes = Vec::new();
        bytes.resize((self.0.len() + 1 + 7) / 8, 0u8);

        for i in 0..self.0.len() {
            bytes[i / 8] |= (self.0[i] as u8) << (i % 8);
        }
		bytes[self.0.len() / 8] |= 1 << (self.0.len() % 8);
		bytes
	}
}

fn decode_bool_list<ML>(value: &[u8], max_len: Option<usize>) -> Result<VariableVec<bool, ML>, Error> {
	let len = (value.len() - 1) * 8 + (7 - value[value.len() - 1].leading_zeros() as usize);
	if let Some(max_len) = max_len {
		if len > max_len {
			return Err(Error::ListTooLarge)
		}
	}
	let mut ret = Vec::new();
	for i in 0..len {
		if i / 8 >= value.len() {
			return Err(Error::IncorrectSize)
		}
        ret.push(value[i / 8] & (1 << (i % 8)) != 0);
    }
	Ok(VariableVec(ret, max_len, PhantomData))
}

impl<ML: Unsigned> Decode for VariableVec<bool, ML> {
	fn decode(value: &[u8]) -> Result<Self, Error> {
		let max_len = Some(ML::to_usize());
		decode_bool_list(value, max_len)
	}
}

impl<C, L: MaxLenFromConfig<C>> DecodeWithConfig<C> for VariableVec<bool, L> {
	fn decode_with_config(value: &[u8], config: &C) -> Result<Self, Error> {
		let max_len = L::max_len_from_config(config);
		decode_bool_list(value, max_len)
	}
}

impl<'a, T: Composite + Encode + SizeType, ML> Encode for VariableVecRef<'a, T, ML> {
	fn encode(&self) -> Vec<u8> {
		encode_composite(self.0)
	}
}

impl<'a, T: Composite + Decode + KnownSize, ML: Unsigned> Decode for VariableVec<T, ML> {
	fn decode(value: &[u8]) -> Result<Self, Error> {
		let value_typ = T::size();
		let ret = decode_composite::<T, _>(value, value_typ, |buf| T::decode(buf))?;

		if ret.len() > ML::to_usize() {
			return Err(Error::ListTooLarge)
		}

		Ok(VariableVec(ret, Some(ML::to_usize()), PhantomData))
	}
}

impl<'a, C, T: Composite + DecodeWithConfig<C> + SizeFromConfig<C>, ML: MaxLenFromConfig<C>> DecodeWithConfig<C> for VariableVec<T, ML> {
	fn decode_with_config(value: &[u8], config: &C) -> Result<Self, Error> {
		let value_typ = T::size_from_config(config);
		let max_len = ML::max_len_from_config(config);
		let ret = decode_composite::<T, _>(value, value_typ, |buf| {
			T::decode_with_config(buf, config)
		})?;

		if let Some(max_len) = max_len {
			if ret.len() > max_len {
				return Err(Error::ListTooLarge)
			}
		}

		Ok(VariableVec(ret, max_len, PhantomData))
	}
}

impl<T, L> Encode for VariableVec<T, L> where
	for<'a> VariableVecRef<'a, T, L>: Encode
{
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		VariableVecRef(&self.0, self.1, PhantomData).using_encoded(f)
	}
}
