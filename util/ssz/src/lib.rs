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

#![cfg_attr(not(feature = "std"), no_std, feature(alloc), feature(alloc_prelude), feature(prelude_import))]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub mod prelude {
	pub use core::prelude::v1::*;
	pub use alloc::prelude::v1::*;
}

#[cfg(feature = "std")]
#[doc(hidden)]
pub mod prelude {
	pub use std::prelude::v1::*;
}

#[doc(hidden)]
pub use hash_db;

#[cfg(not(feature = "std"))]
#[allow(unused)]
#[prelude_import]
use crate::prelude::*;

mod codec;
#[doc(hidden)]
pub mod hash;
#[cfg(test)]
mod tests;

pub use self::codec::{Input, Output, Encode, Decode, Prefixable, Fixed};
pub use self::hash::{Hashable, Composite};

/// Trait that allows zero-copy read/write of value-references to/from slices in LE format.
pub trait Ssz: Decode + Encode + Hashable {}

impl<S: Encode + Decode + Hashable> Ssz for S {}
