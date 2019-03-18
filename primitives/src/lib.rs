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

mod authority_id;
mod bitfield;
mod signature;
mod attestation;

pub use crypto;
pub use keccak_hasher::KeccakHasher;
pub use signature::{H768, Signature};
pub use authority_id::{H384, AuthorityId};
pub use bitfield::BitField;
pub use attestation::{UnsignedAttestation, UncheckedAttestation, CheckedAttestation, AttestationContext};

pub use primitive_types::H256;

/// Shasper validator public key.
pub type ValidatorId = AuthorityId;

/// A hash of some data used by the chain.
pub type Hash = primitive_types::H256;

/// Index of a block number in the chain.
pub type BlockNumber = u64;

/// Count value in Shasper.
pub type Count = u64;

/// Slot value in Shasper.
pub type Slot = u64;

/// Epoch value in Shasper.
pub type Epoch = u64;

/// Balance value in Shasper.
pub type Balance = u128;

/// Validator index in Shasper.
pub type ValidatorIndex = u32;

/// Timestamp value in Shasper.
pub type Timestamp = u64;
