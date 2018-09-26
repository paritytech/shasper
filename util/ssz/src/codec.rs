use alloc::vec::Vec;
use alloc::boxed::Box;
use core::{mem, slice};
use arrayvec::ArrayVec;
use primitives::{H160, H256, U256};

pub use parity_codec::{Input, Output};

/// Trait that allows zero-copy write of value-references to slices in SSZ format.
/// Implementations should override `using_encoded` for value types and `encode_to` for allocating types.
pub trait Encode {
	/// Convert self to a slice and append it to the destination.
	fn encode_to<T: Output>(&self, dest: &mut T) {
		self.using_encoded(|buf| dest.write(buf));
	}

	/// Convert self to an owned vector.
	fn encode(&self) -> Vec<u8> {
		let mut r = Vec::new();
		self.encode_to(&mut r);
		r
	}

	/// Convert self to a slice and then invoke the given closure with it.
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		f(&self.encode())
	}
}

/// Trait that allows zero-copy read of value-references from slices in LE format.
pub trait Decode: Sized {
	/// Attempt to deserialise the value from input.
	fn decode<I: Input>(value: &mut I) -> Option<Self>;
}

/// Trait that allows zero-copy read/write of value-references to/from slices in LE format.
pub trait Codec: Decode + Encode {}

impl<S: Encode + Decode> Codec for S {}

macro_rules! impl_array {
	( $( $n:expr )* ) => { $(
		impl<T: Encode> Encode for [T; $n] {
			fn encode_to<W: Output>(&self, dest: &mut W) {
				for item in self.iter() {
					item.encode_to(dest);
				}
			}
		}

		impl<T: Decode> Decode for [T; $n] {
			fn decode<I: Input>(input: &mut I) -> Option<Self> {
				let mut r = ArrayVec::new();
				for _ in 0..$n {
					r.push(T::decode(input)?);
				}
				r.into_inner().ok()
			}
		}
	)* }
}

impl_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
	40 48 56 64 72 96 128 160 192 224 256);

impl<T: Encode> Encode for Box<T> {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		self.as_ref().encode_to(dest)
	}
}

impl<T: Decode> Decode for Box<T> {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		Some(Box::new(T::decode(input)?))
	}
}

impl Encode for [u8] {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		let len = self.len();
		assert!(len <= u32::max_value() as usize, "Attempted to serialize a collection with too many elements.");
		(len as u32).encode_to(dest);
		dest.write(self)
	}
}

impl Encode for Vec<u8> {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		self.as_slice().encode_to(dest)
	}
}

impl Decode for Vec<u8> {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		u32::decode(input).and_then(move |len| {
			let len = len as usize;
			let mut vec = vec![0; len];
			if input.read(&mut vec[..len]) != len {
				None
			} else {
				Some(vec)
			}
		})
	}
}

#[cfg(feature = "std")]
impl Encode for String {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		self.as_bytes().encode_to(dest)
	}
}

#[cfg(feature = "std")]
impl Decode for String {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		Some(Self::from_utf8_lossy(&Vec::decode(input)?).into())
	}
}

impl<T: Encode> Encode for [T] {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		let len = self.len();
		assert!(len <= u32::max_value() as usize, "Attempted to serialize a collection with too many elements.");
		(len as u32).encode_to(dest);
		for item in self {
			item.encode_to(dest);
		}
	}
}

impl<T: Encode> Encode for Vec<T> {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		self.as_slice().encode_to(dest)
	}
}

impl<T: Decode> Decode for Vec<T> {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		<u32>::decode(input).and_then(move |len| {
			let mut r = Vec::with_capacity(len as usize);
			for _ in 0..len {
				r.push(T::decode(input)?);
			}
			Some(r)
		})
	}
}

macro_rules! tuple_impl {
	($one:ident,) => {
		impl<$one: Encode> Encode for ($one,) {
			fn encode_to<T: Output>(&self, dest: &mut T) {
				self.0.encode_to(dest);
			}
		}

		impl<$one: Decode> Decode for ($one,) {
			fn decode<I: Input>(input: &mut I) -> Option<Self> {
				match $one::decode(input) {
					None => None,
					Some($one) => Some(($one,)),
				}
			}
		}
	};
	($first:ident, $($rest:ident,)+) => {
		impl<$first: Encode, $($rest: Encode),+>
		Encode for
		($first, $($rest),+) {
			fn encode_to<T: Output>(&self, dest: &mut T) {
				let (
					ref $first,
					$(ref $rest),+
				) = *self;

				$first.encode_to(dest);
				$($rest.encode_to(dest);)+
			}
		}

		impl<$first: Decode, $($rest: Decode),+>
		Decode for
		($first, $($rest),+) {
			fn decode<INPUT: Input>(input: &mut INPUT) -> Option<Self> {
				Some((
					match $first::decode(input) {
						Some(x) => x,
						None => return None,
					},
					$(match $rest::decode(input) {
						Some(x) => x,
						None => return None,
					},)+
				))
			}
		}

		tuple_impl!($($rest,)+);
	}
}

#[allow(non_snake_case)]
mod inner_tuple_impl {
	use super::{Input, Output, Decode, Encode};
	tuple_impl!(A, B, C, D, E, F, G, H, I, J, K,);
}

/// Trait to allow conversion to a know endian representation when sensitive.
/// Types implementing this trait must have a size > 0.
// note: the copy bound and static lifetimes are necessary for safety of `Codec` blanket
// implementation.
trait EndianSensitive: Copy + 'static {
	fn to_le(self) -> Self { self }
	fn to_be(self) -> Self { self }
	fn from_le(self) -> Self { self }
	fn from_be(self) -> Self { self }
	fn as_be_then<T, F: FnOnce(&Self) -> T>(&self, f: F) -> T { f(&self) }
	fn as_le_then<T, F: FnOnce(&Self) -> T>(&self, f: F) -> T { f(&self) }
}

macro_rules! impl_endians {
	( $( $t:ty ),* ) => { $(
		impl EndianSensitive for $t {
			fn to_le(self) -> Self { <$t>::to_le(self) }
			fn to_be(self) -> Self { <$t>::to_be(self) }
			fn from_le(self) -> Self { <$t>::from_le(self) }
			fn from_be(self) -> Self { <$t>::from_be(self) }
			fn as_be_then<T, F: FnOnce(&Self) -> T>(&self, f: F) -> T { let d = self.to_be(); f(&d) }
			fn as_le_then<T, F: FnOnce(&Self) -> T>(&self, f: F) -> T { let d = self.to_le(); f(&d) }
		}

		impl Encode for $t {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				self.as_be_then(|le| {
					let size = mem::size_of::<$t>();
					let value_slice = unsafe {
						let ptr = le as *const _ as *const u8;
						if size != 0 {
							slice::from_raw_parts(ptr, size)
						} else {
							&[]
						}
					};

					f(value_slice)
				})
			}
		}

		impl Decode for $t {
			fn decode<I: Input>(input: &mut I) -> Option<Self> {
				let size = mem::size_of::<$t>();
				assert!(size > 0, "EndianSensitive can never be implemented for a zero-sized type.");
				let mut val: $t = unsafe { mem::zeroed() };

				unsafe {
					let raw: &mut [u8] = slice::from_raw_parts_mut(
						&mut val as *mut $t as *mut u8,
						size
					);
					if input.read(raw) != size { return None }
				}
				Some(val.from_be())
			}
		}
	)* }
}
macro_rules! impl_non_endians {
	( $( $t:ty ),* ) => { $(
		impl EndianSensitive for $t {}

		impl Encode for $t {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				self.as_le_then(|le| {
					let size = mem::size_of::<$t>();
					let value_slice = unsafe {
						let ptr = le as *const _ as *const u8;
						if size != 0 {
							slice::from_raw_parts(ptr, size)
						} else {
							&[]
						}
					};

					f(value_slice)
				})
			}
		}

		impl Decode for $t {
			fn decode<I: Input>(input: &mut I) -> Option<Self> {
				let size = mem::size_of::<$t>();
				assert!(size > 0, "EndianSensitive can never be implemented for a zero-sized type.");
				let mut val: $t = unsafe { mem::zeroed() };

				unsafe {
					let raw: &mut [u8] = slice::from_raw_parts_mut(
						&mut val as *mut $t as *mut u8,
						size
					);
					if input.read(raw) != size { return None }
				}
				Some(val.from_le())
			}
		}
	)* }
}

impl_endians!(u16, u32, u64, u128, usize, i16, i32, i64, i128, isize);
impl_non_endians!(
	i8, [u8; 1], [u8; 2], [u8; 3], [u8; 4], [u8; 5], [u8; 6], [u8; 7], [u8; 8],
	[u8; 10], [u8; 12], [u8; 14], [u8; 16], [u8; 20], [u8; 24], [u8; 28], [u8; 32], [u8; 40],
	[u8; 48], [u8; 56], [u8; 64], [u8; 80], [u8; 96], [u8; 112], [u8; 128], bool);

macro_rules! impl_hash {
	($name: ident, $len: expr) => {
		impl Encode for $name {
			fn encode_to<W: Output>(&self, dest: &mut W) {
				dest.write(&self)
			}
		}
		impl Decode for $name {
			fn decode<I: Input>(input: &mut I) -> Option<Self> {
				let mut vec = [0u8; $len];
				if input.read(&mut vec[..$len]) != $len {
					None
				} else {
					Some($name::from(vec.as_ref()))
				}
			}
		}
	}
}

impl_hash!(H160, 20);
impl_hash!(H256, 32);

macro_rules! impl_uint {
	($name: ident, $len: expr) => {
		impl ::codec::Encode for $name {
			fn encode_to<W: Output>(&self, dest: &mut W) {
				let mut bytes = [0u8; $len * 8];
				self.to_big_endian(&mut bytes);
				dest.write(&bytes)
			}
		}

		impl ::codec::Decode for $name {
			fn decode<I: ::codec::Input>(input: &mut I) -> Option<Self> {
				<[u8; $len * 8] as ::codec::Decode>::decode(input)
					.map(|b| $name::from_big_endian(&b))
			}
		}
	}
}

impl_uint!(U256, 4);

#[cfg(test)]
mod tests {
	use super::*;
	use primitives::{H256, H160, U256};

	#[test]
	fn examples() {
		assert_eq!(5u32.encode(), vec![0, 0, 0, 5u8]);
		assert_eq!(u32::decode(&mut [0, 0, 0, 5u8].as_ref()).unwrap(), 5u32);
		assert_eq!(vec![99u8, 111u8, 119u8].encode(), vec![0u8, 0u8, 0u8, 3u8, 99u8, 111u8, 119u8]);
		assert_eq!(Vec::<u8>::decode(&mut [0u8, 0u8, 0u8, 3u8, 99u8, 111u8, 119u8].as_ref()).unwrap(),
				   vec![99u8, 111u8, 119u8]);
		assert_eq!(H160::from([5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
							   5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
							   5u8, 5u8, 5u8, 5u8].as_ref()).encode(),
				   vec![5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
						5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
						5u8, 5u8, 5u8, 5u8]);
		assert_eq!(H160::decode(&mut [5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
									  5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
									  5u8, 5u8, 5u8, 5u8].as_ref()).unwrap(),
				   H160::from([5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
							   5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
							   5u8, 5u8, 5u8, 5u8].as_ref()));
		assert_eq!(H256::from([5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
							   5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
							   5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
							   5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8].as_ref()).encode(),
				   vec![5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
						5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
						5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8,
						5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8, 5u8]);
	}

	#[test]
	fn test_u256() {
		assert_eq!(U256::from(5).encode(), vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
												0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
												0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
												0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 5u8]);
		assert_eq!(U256::decode(&mut [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
									  0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
									  0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
									  0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 5u8].as_ref()).unwrap(),
				   U256::from(5));
		}
}
