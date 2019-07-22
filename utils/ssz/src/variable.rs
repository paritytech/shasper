use crate::{Encode, Decode, Error, Codec, VariableSize, MaxVec, Compact, CompactRef};
use crate::utils::{encode_list, decode_list};
use typenum::Unsigned;

impl<T, ML> Codec for Compact<MaxVec<T, ML>> {
	type Size = VariableSize;
}

impl<'a, T, ML> Codec for CompactRef<'a, MaxVec<T, ML>> {
	type Size = VariableSize;
}

impl<T, ML> Codec for MaxVec<T, ML> {
	type Size = VariableSize;
}

macro_rules! impl_builtin_variable_uint_list {
	( $( $t:ty ),* ) => { $(
		impl<'a, ML> Encode for CompactRef<'a, MaxVec<$t, ML>> {
			fn encode(&self) -> Vec<u8> {
				encode_list(self.0)
			}
		}

		impl<ML> Encode for Compact<MaxVec<$t, ML>> {
			fn encode(&self) -> Vec<u8> {
				CompactRef(&self.0).encode()
			}
		}

		impl<ML: Unsigned> Decode for Compact<MaxVec<$t, ML>> {
			fn decode(value: &[u8]) -> Result<Self, Error> {
				let decoded = decode_list(value)?;
				if decoded.len() > ML::to_usize() {
					return Err(Error::ListTooLarge)
				}
				Ok(Compact(MaxVec::from(decoded)))
			}
		}
	)* }
}

impl_builtin_variable_uint_list!(u8, u16, u32, u64, u128);

impl<'a, ML> Encode for CompactRef<'a, MaxVec<bool, ML>> {
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

impl<ML> Encode for Compact<MaxVec<bool, ML>> {
	fn encode(&self) -> Vec<u8> {
		CompactRef(&self.0).encode()
	}
}

impl<ML: Unsigned> Decode for Compact<MaxVec<bool, ML>> {
	fn decode(value: &[u8]) -> Result<Self, Error> {
		let max_len = ML::to_usize();
		let len = (value.len() - 1) * 8 + (7 - value[value.len() - 1].leading_zeros() as usize);
		if len > max_len {
			return Err(Error::ListTooLarge)
		}
		let mut ret = Vec::new();
		for i in 0..len {
			if i / 8 >= value.len() {
				return Err(Error::IncorrectSize)
			}
			ret.push(value[i / 8] & (1 << (i % 8)) != 0);
		}
		Ok(Compact(MaxVec::from(ret)))
	}
}

impl<T: Encode, ML> Encode for MaxVec<T, ML> {
	fn encode(&self) -> Vec<u8> {
		encode_list(&self.0)
	}
}

impl<T: Decode, ML: Unsigned> Decode for MaxVec<T, ML> {
	fn decode(value: &[u8]) -> Result<Self, Error> {
		let ret = decode_list::<T>(value)?;

		if ret.len() > ML::to_usize() {
			return Err(Error::ListTooLarge)
		}
		Ok(MaxVec::from(ret))
	}
}
