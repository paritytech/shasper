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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod utils;
mod basic;
mod series;
mod fixed;
mod variable;

pub use bm_le::{FixedVec, FixedVecRef, VariableVec, VariableVecRef,
				LenFromConfig, MaxLenFromConfig};
pub use series::{Series, SeriesItem};

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
}

/// Type of this size.
pub trait SizeType {
	/// Whether this value is fixed.
	fn is_fixed() -> bool { !Self::is_variable() }
	/// Whether this value is variable.
	fn is_variable() -> bool { !Self::is_fixed() }
}

/// Trait for fetching size from config.
pub trait SizeFromConfig<C>: SizeType {
	/// Get the size of this type with given config.
	fn size_from_config(config: &C) -> Option<usize>;
}

/// Trait for type with known size.
pub trait KnownSize: SizeType {
	/// Size of this type.
	fn size() -> Option<usize>;
}

/// Trait that allows zero-copy write of value-references to slices in ssz format.
///
/// Implementations should override `using_encoded` for value types and `encode_to` and `size_hint` for allocating types.
/// Wrapper types should override all methods.
pub trait Encode {
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
pub trait Decode: Sized {
	/// Attempt to deserialise the value from input.
	fn decode(value: &[u8]) -> Result<Self, Error>;
}

/// Trait for composite values.
pub trait Composite { }

pub trait DecodeWithConfig<C>: Sized {
	fn decode_with_config(value: &[u8], config: &C) -> Result<Self, Error>;
}

#[macro_export]
macro_rules! impl_decode_with_empty_config {
	( $t:ty ) => {
		impl<C> $crate::DecodeWithConfig<C> for $t {
			fn decode_with_config(value: &[u8], _config: &C) -> Result<Self, $crate::Error> {
				<Self as $crate::Decode>::decode(value)
			}
		}
	}
}
