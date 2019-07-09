use crate::{Encode, Decode, Error, Codec};

macro_rules! impl_builtin_uint {
	( $t:ty, $len:ty ) => {
		impl Codec for $t {
			type Size = $len;
		}

		impl Encode for $t {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				let bytes = self.to_le_bytes();
				f(&bytes)
			}
		}

		impl Decode for $t {
			fn decode(value: &[u8]) -> Result<Self, Error> {
				let mut bytes = <$t>::default().to_le_bytes();
				if value.len() != bytes.len() {
					return Err(Error::IncorrectSize)
				}
				bytes.copy_from_slice(value);
				Ok(<$t>::from_le_bytes(bytes))
			}
		}
	}
}

impl_builtin_uint!(u8, typenum::U1);
impl_builtin_uint!(u16, typenum::U2);
impl_builtin_uint!(u32, typenum::U4);
impl_builtin_uint!(u64, typenum::U8);
impl_builtin_uint!(u128, typenum::U16);

impl Codec for bool {
	type Size = typenum::U1;
}

impl Encode for bool {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		let value = match self {
			true => 0x01u8,
			false => 0x00u8,
		};
		value.using_encoded(f)
	}
}

impl Decode for bool {
	fn decode(value: &[u8]) -> Result<Self, Error> {
		let value = u8::decode(value)?;
		match value {
			0x01 => Ok(true),
			0x00 => Ok(false),
			_ => Err(Error::InvalidType),
		}
	}
}
