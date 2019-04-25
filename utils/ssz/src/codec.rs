// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use core::{mem, slice};
use arrayvec::ArrayVec;
use primitive_types::{H160, H256, U256};

/// Trait that allows reading of data into a slice.
pub trait Input {
	/// Read into the provided input slice. Returns the number of bytes read.
	fn read(&mut self, into: &mut [u8]) -> usize;

	/// Read a single byte from the input.
	fn read_byte(&mut self) -> Option<u8> {
		let mut buf = [0u8];
		match self.read(&mut buf[..]) {
			0 => None,
			1 => Some(buf[0]),
			_ => unreachable!(),
		}
	}
}

#[cfg(not(feature = "std"))]
impl<'a> Input for &'a [u8] {
	fn read(&mut self, into: &mut [u8]) -> usize {
		let len = ::core::cmp::min(into.len(), self.len());
		into[..len].copy_from_slice(&self[..len]);
		*self = &self[len..];
		len
	}
}

#[cfg(feature = "std")]
impl<R: ::std::io::Read> Input for R {
	fn read(&mut self, into: &mut [u8]) -> usize {
		match (self as &mut ::std::io::Read).read_exact(into) {
			Ok(()) => into.len(),
			Err(_) => 0,
		}
	}
}

/// Trait that allows writing of data.
pub trait Output: Sized {
	/// Write to the output.
	fn write(&mut self, bytes: &[u8]);

	/// Push a single byte to the output.
	fn push_byte(&mut self, byte: u8) {
		self.write(&[byte]);
	}

	/// Push a value as encoded by Ssz to the output.
	fn push<V: Encode + ?Sized>(&mut self, value: &V) {
		value.encode_to(self);
	}
}

#[cfg(not(feature = "std"))]
impl Output for Vec<u8> {
	fn write(&mut self, bytes: &[u8]) {
		self.extend(bytes);
	}
}

#[cfg(feature = "std")]
impl<W: ::std::io::Write> Output for W {
	fn write(&mut self, bytes: &[u8]) {
		(self as &mut ::std::io::Write).write_all(bytes).expect("Codec outputs are infallible");
	}
}

/// Note whether the item is prefixed.
pub trait Prefixable {
	/// Whether the item is prefixed.
	fn prefixed() -> bool;
}

/// Trait that allows zero-copy write of value-references to slices in SSZ format.
/// Implementations should override `using_encoded` for value types and `encode_to` for allocating types.
pub trait Encode: Prefixable {
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
pub trait Decode: Prefixable + Sized {
	/// Attempt to deserialise the value from input. Return the number of bytes read as the second parameter.
	fn decode_as<I: Input>(value: &mut I) -> Option<(Self, usize)>;

	/// Attempt to deserialise the value from input.
	fn decode<I: Input>(value: &mut I) -> Option<Self> {
		Self::decode_as(value).map(|v| v.0)
	}
}

macro_rules! impl_array {
	( $( $n:expr )* ) => { $(
		impl<T: Prefixable> Prefixable for [T; $n] {
			fn prefixed() -> bool {
				T::prefixed()
			}
		}

		impl<T: Encode> Encode for [T; $n] {
			fn encode_to<W: Output>(&self, dest: &mut W) {
				for item in self.iter() {
					item.encode_to(dest);
				}
			}
		}

		impl<T: Decode> Decode for [T; $n] {
			fn decode_as<I: Input>(input: &mut I) -> Option<(Self, usize)> {
				let mut r = ArrayVec::new();
				let mut len = 0;
				for _ in 0..$n {
					let (item, l) = T::decode_as(input)?;
					r.push(item);
					len += l;
				}
				r.into_inner().ok().map(|v| (v, len))
			}
		}
	)* }
}

impl_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
			40 48 56 64 72 96 128 160 192 224 256 1024 8192);

#[doc(hidden)]
pub struct Fixed<'a, T>(pub &'a [T]);

impl<'a, T: Prefixable> Prefixable for Fixed<'a, T> {
	fn prefixed() -> bool {
		T::prefixed()
	}
}

impl<'a, T: Encode> Encode for Fixed<'a, T> {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		for item in self.0.iter() {
			item.encode_to(dest);
		}
	}
}

impl<T: Prefixable> Prefixable for Box<T> {
	fn prefixed() -> bool {
		T::prefixed()
	}
}

impl<T: Encode> Encode for Box<T> {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		self.as_ref().encode_to(dest)
	}
}

impl<T: Decode> Decode for Box<T> {
	fn decode_as<I: Input>(input: &mut I) -> Option<(Self, usize)> {
		let (item, l) = T::decode_as(input)?;
		Some((Box::new(item), l))
	}
}

impl Prefixable for [u8] {
	fn prefixed() -> bool {
		true
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

impl Prefixable for Vec<u8> {
	fn prefixed() -> bool {
		true
	}
}

impl Encode for Vec<u8> {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		self.as_slice().encode_to(dest)
	}
}

impl Decode for Vec<u8> {
	fn decode_as<I: Input>(input: &mut I) -> Option<(Self, usize)> {
		let (len, l) = u32::decode_as(input)?;
		let len = len as usize;

		let mut vec = vec![0; len];
		if input.read(&mut vec[..len]) != len {
			None
		} else {
			Some((vec, len + l))
		}
	}
}

#[cfg(feature = "std")]
impl Prefixable for String {
	fn prefixed() -> bool {
		true
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
	fn decode_as<I: Input>(input: &mut I) -> Option<(Self, usize)> {
		let (item, l) = Vec::decode_as(input)?;
		Some((Self::from_utf8_lossy(&item).into(), l))
	}
}

impl<T: Prefixable> Prefixable for [T] {
	fn prefixed() -> bool {
		true
	}
}

impl<T: Encode> Encode for [T] {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		let mut bytes = Vec::new();
		for item in self {
			item.encode_to(&mut bytes);
		}

		let len = bytes.len();
		(len as u32).encode_to(dest);
		dest.write(&bytes);
	}
}

impl<T: Prefixable> Prefixable for Vec<T> {
	fn prefixed() -> bool {
		true
	}
}

impl<T: Encode> Encode for Vec<T> {
	fn encode_to<W: Output>(&self, dest: &mut W) {
		self.as_slice().encode_to(dest)
	}
}

impl<T: Decode> Decode for Vec<T> {
	fn decode_as<I: Input>(input: &mut I) -> Option<(Self, usize)> {
		let (len, l) = u32::decode_as(input)?;
		let len = len as usize;

		let mut r = Vec::new();
		let mut i = 0;
		while i < len {
			let (item, l) = T::decode_as(input)?;
			r.push(item);
			i += l;
		}
		if i != len {
			None
		} else {
			Some((r, i + l))
		}
	}
}

macro_rules! tuple_impl {
	($one:ident,) => {
		impl<$one: Prefixable> Prefixable for ($one,) {
			fn prefixed() -> bool {
				$one::prefixed()
			}
		}

		impl<$one: Encode> Encode for ($one,) {
			fn encode_to<T: Output>(&self, dest: &mut T) {
				if Self::prefixed() {
					let bytes = self.0.encode();
					(bytes.len() as u32).encode_to(dest);
					dest.write(&bytes);
				} else {
					self.0.encode_to(dest);
				}
			}
		}

		impl<$one: Decode> Decode for ($one,) {
			fn decode_as<I: Input>(input: &mut I) -> Option<(Self, usize)> {
				let mut l = 0;
				let len = if Self::prefixed() {
					let (len, i) = u32::decode_as(input)?;
					l += i;
					Some(len as usize)
				} else {
					None
				};

				match $one::decode_as(input) {
					None => None,
					Some(($one, i)) => {
						if let Some(len) = len {
							if i != len {
								return None
							}
						}

						l += i;
						Some((($one,), l))
					}
				}
			}
		}
	};
	($first:ident, $($rest:ident,)+) => {
		impl<$first: Prefixable, $($rest: Prefixable),+>
			Prefixable for
			($first, $($rest),+)
		{
			fn prefixed() -> bool {
				let mut prefixed = $first::prefixed();
				$(
					prefixed = prefixed || $rest::prefixed();
				)+
				prefixed
			}
		}

		impl<$first: Encode, $($rest: Encode),+>
			Encode for
			($first, $($rest),+)
		{
			fn encode_to<T: Output>(&self, dest: &mut T) {
				if Self::prefixed() {
					let mut bytes = Vec::new();

					let (
						ref $first,
						$(ref $rest),+
					) = *self;

					$first.encode_to(&mut bytes);
					$($rest.encode_to(&mut bytes);)+

					(bytes.len() as u32).encode_to(dest);
					dest.write(&bytes);
				} else {
					let (
						ref $first,
						$(ref $rest),+
					) = *self;

					$first.encode_to(dest);
					$($rest.encode_to(dest);)+
				}
			}
		}

		impl<$first: Decode, $($rest: Decode),+>
			Decode for
			($first, $($rest),+)
		{
			fn decode_as<INPUT: Input>(input: &mut INPUT) -> Option<(Self, usize)> {
				let mut l = 0;
				let len = if Self::prefixed() {
					let (len, i) = u32::decode_as(input)?;
					l += i;
					Some(len as usize)
				} else {
					None
				};
				let mut il = 0;

				let value = (
					match $first::decode_as(input) {
						Some((x, i)) => {
							l += i;
							il += i;
							x
						},
						None => return None,
					},
					$(match $rest::decode_as(input) {
						Some((x, i)) => {
							l += i;
							il += i;
							x
						},
						None => return None,
					},)+
				);

				if let Some(len) = len {
					if il != len {
						return None
					}
				}

				Some((value, l))
			}
		}

		tuple_impl!($($rest,)+);
	}
}

#[allow(non_snake_case)]
mod inner_tuple_impl {
	use super::{Input, Output, Decode, Encode, Prefixable};
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

		impl Prefixable for $t {
			fn prefixed() -> bool {
				false
			}
		}

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
			fn decode_as<I: Input>(input: &mut I) -> Option<(Self, usize)> {
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
				Some((val.from_le(), size))
			}
		}
	)* }
}
macro_rules! impl_non_endians {
	( $( $t:ty ),* ) => { $(
		impl EndianSensitive for $t {}

		impl Prefixable for $t {
			fn prefixed() -> bool {
				false
			}
		}

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
			fn decode_as<I: Input>(input: &mut I) -> Option<(Self, usize)> {
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
				Some((val.from_le(), size))
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
		impl Prefixable for $name {
			fn prefixed() -> bool {
				false
			}
		}

		impl Encode for $name {
			fn encode_to<W: Output>(&self, dest: &mut W) {
				dest.write(self.as_ref())
			}
		}

		impl Decode for $name {
			fn decode_as<I: Input>(input: &mut I) -> Option<(Self, usize)> {
				let mut vec = [0u8; $len];
				if input.read(&mut vec[..$len]) != $len {
					None
				} else {
					Some(($name::from(&vec), $len))
				}
			}
		}
	}
}

impl_hash!(H160, 20);
impl_hash!(H256, 32);

macro_rules! impl_uint {
	($name: ident, $len: expr) => {
		impl Prefixable for $name {
			fn prefixed() -> bool {
				false
			}
		}

		impl Encode for $name {
			fn encode_to<W: Output>(&self, dest: &mut W) {
				let mut bytes = [0u8; $len * 8];
				self.to_little_endian(&mut bytes);
				dest.write(&bytes)
			}
		}

		impl Decode for $name {
			fn decode_as<I: crate::codec::Input>(input: &mut I) -> Option<(Self, usize)> {
				<[u8; $len * 8] as crate::codec::Decode>::decode(input)
					.map(|b| ($name::from_little_endian(&b), $len * 8))
			}
		}
	}
}

impl_uint!(U256, 4);
