use crate::{Encode, Decode, Error, KnownSize, SizeFromConfig, SizeType, impl_decode_with_empty_config};
use typenum::Unsigned;

macro_rules! impl_builtin_uint {
	( $t:ty, $len:ty ) => {
		impl SizeType for $t {
			fn is_fixed() -> bool { true }
		}

		impl KnownSize for $t {
			fn size() -> Option<usize> {
				Some(<$len>::to_usize())
			}
		}

		impl<C> SizeFromConfig<C> for $t {
			fn size_from_config(_config: &C) -> Option<usize> {
				<$t>::size()
			}
		}

		impl Encode for $t {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				let bytes = self.to_le_bytes();
				f(&bytes)
			}
		}

		impl_decode_with_empty_config!($t);
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

impl SizeType for bool {
	fn is_fixed() -> bool { true }
}

impl KnownSize for bool {
	fn size() -> Option<usize> {
		Some(typenum::U1::to_usize())
	}
}

impl<C> SizeFromConfig<C> for bool {
	fn size_from_config(_config: &C) -> Option<usize> {
		bool::size()
	}
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

impl_decode_with_empty_config!(bool);
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
