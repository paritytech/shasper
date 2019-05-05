// Copyright 2017-2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

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

//! Primitives

mod authority_id;
mod bitfield;
mod signature;
mod version;

pub use authority_id::{H384, ValidatorId};
pub use bitfield::BitField;
pub use signature::{H768, Signature};
pub use version::{H32, Version};
pub use primitive_types::H256;

/// Alias to u64.
pub type Uint = u64;
/// A slot number.
pub type Slot = Uint;
/// An epoch number.
pub type Epoch = Uint;
/// A shard number.
pub type Shard = Uint;
/// A validator registry index.
pub type ValidatorIndex = Uint;
/// An amount in Gwei.
pub type Gwei = Uint;
