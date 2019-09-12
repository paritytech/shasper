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

//! SimpleSerialization crate written in Rust.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod utils;
mod basic;
mod series;
mod size;
mod fixed;
mod variable;

pub use bm_le::{Compact, CompactRef, MaxVec};
pub use series::{Series, SeriesItem};
pub use ssz_derive::{Codec, Encode, Decode};

pub use crate::size::{Size, VariableSize, Add, Mul, Div};

use alloc::vec::Vec;

#[derive(Debug)]
/// Error type for encoding and decoding.
pub enum Error {
	/// Incorrect size.
	IncorrectSize,
	/// Invalid type.
	InvalidType,
	/// Vector length is incorrect.
	InvalidLength,
	/// List length is too large.
	ListTooLarge,
	/// Other errors.
	Other(&'static str),
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
	fn from(_: std::io::Error) -> Error {
		Error::Other("io error")
	}
}

/// Base trait for ssz encoding and decoding.
pub trait Codec {
	/// Size of the current type, also indicates whether it is fixed-sized or
	/// variable-sized.
	type Size: Size;
}

/// Trait that allows zero-copy write of value-references to slices in ssz format.
///
/// Implementations should override `using_encoded` for value types and `encode_to` and `size_hint` for allocating types.
/// Wrapper types should override all methods.
pub trait Encode: Codec {
	/// Convert self to an owned vector.
	fn encode(&self) -> Vec<u8> {
		let mut r = Vec::new();
		self.using_encoded(|buf| r.extend_from_slice(buf));
		r
	}

	/// Convert self to a slice and then invoke the given closure with it.
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		f(&self.encode())
	}
}

/// Trait that allows zero-copy read of value-references from slices in ssz format.
pub trait Decode: Codec + Sized {
	/// Attempt to deserialise the value from input.
	fn decode(value: &[u8]) -> Result<Self, Error>;
}

/// Type for length offset used for variable-sized item placeholder.
pub type LengthOffset = u32;
