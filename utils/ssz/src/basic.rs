use crate::{Encode, Decode, Input, Error, Codec, FixedSize};

macro_rules! impl_builtin_uint {
	( $( $t:ty ),* ) => { $(
		impl Codec for $t {
			type Size = FixedSize;
		}

		impl Encode for $t {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				let bytes = self.to_le_bytes();
				f(&bytes)
			}
		}

		impl Decode for $t {
			fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
				let mut bytes = <$t>::default().to_le_bytes();
				value.read(&mut bytes)?;
				Ok(<$t>::from_le_bytes(bytes))
			}
		}
	)* }
}

impl_builtin_uint!(u8, u16, u32, u64, u128);

impl Codec for bool {
	type Size = FixedSize;
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
	fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
		let value = u8::decode(value)?;
		match value {
			0x01 => Ok(true),
			0x00 => Ok(false),
			_ => Err(Error::InvalidType),
		}
	}
}
