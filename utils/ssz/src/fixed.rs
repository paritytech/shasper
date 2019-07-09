use crate::{FixedVec, FixedVecRef, LenFromConfig, Encode, Decode, DecodeWithConfig, Error, KnownSize, SizeFromConfig};
use alloc::vec::Vec;
use core::marker::PhantomData;
use typenum::Unsigned;

fn decode_builtin_vector<T: KnownSize + Decode, L>(
	value: &[u8],
	len: usize
) -> Result<FixedVec<T, L>, Error> {
	let mut ret = Vec::new();
	let single_size = T::size().expect("uint size are fixed known; qed");
	for i in 0..len {
		let start = i * single_size;
		let end = (i + 1) * single_size;
		if end >= value.len() {
			return Err(Error::IncorrectSize)
		}
		ret.push(T::decode(&value[start..end])?);
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
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				let mut ret = Vec::new();
				for value in self.0 {
					value.using_encoded(|buf| ret.extend_from_slice(buf));
				}
				f(&ret)
			}
		}

		impl<L: Unsigned> Decode for FixedVec<$t, L> {
			fn decode(value: &[u8]) -> Result<Self, Error> {
				decode_builtin_vector(value, L::to_usize())
			}
		}

		impl<C, L: LenFromConfig<C>> DecodeWithConfig<C> for FixedVec<$t, L> {
			fn decode_with_config(value: &[u8], config: &C) -> Result<Self, Error> {
				let len = L::len_from_config(config);
				decode_builtin_vector(value, len)
			}
		}
	)* }
}

impl_builtin_fixed_uint_vector!(u8, u16, u32, u64, u128);

impl<T, L> KnownSize for FixedVec<T, L> where
	for<'a> FixedVecRef<'a, T, L>: KnownSize,
{
	fn size() -> Option<usize> {
		FixedVecRef::<T, L>::size()
	}
}

impl<C, T, L> SizeFromConfig<C> for FixedVec<T, L> where
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
