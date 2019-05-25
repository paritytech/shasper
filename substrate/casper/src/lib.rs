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

//! Generic structs and traits for the Casper FFG.

#![cfg_attr(not(feature = "std"), no_std, feature(alloc), feature(alloc_prelude), feature(prelude_import))]

#![warn(missing_docs)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
pub(crate) mod prelude {
	pub use core::prelude::v1::*;
	pub use alloc::prelude::v1::*;
}

#[cfg(not(feature = "std"))]
#[allow(unused)]
#[prelude_import]
use crate::prelude::*;

extern crate parity_codec as codec;
extern crate parity_codec_derive as codec_derive;

pub mod randao;
pub mod casper;
pub mod reward;
pub mod store;
pub mod context;
pub mod committee;
mod util;

pub use crate::casper::CasperProcess;
pub use crate::util::{hash, hash2, hash3};
