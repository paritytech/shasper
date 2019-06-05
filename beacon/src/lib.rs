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

//! Minimal beacon chain state transition implementation for Serenity.

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

#[cfg(feature = "parity-codec")]
extern crate parity_codec as codec;

/// Version of ethereum/eth2.0-specs.
pub const VERSION: &str = "v0.6.3";
/// Commit of ethereum/eth2.0-specs.
pub const COMMIT: &str = "cb9301a9fece8864d97b6ff6b0bb3a662fa21484";

mod config;
mod utils;
mod error;
mod executive;

pub mod primitives;
pub mod types;
pub use crate::config::*;
pub use crate::executive::*;
pub use crate::error::Error;
