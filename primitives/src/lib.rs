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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

extern crate parity_codec as codec;
extern crate parity_codec_derive as codec_derive;

mod authority_id;
mod bitfield;
mod signature;
mod attestation;

pub use crypto;
pub use signature::{H768, Signature};
pub use authority_id::{H384, AuthorityId};
pub use bitfield::BitField;
pub use attestation::{UnsignedAttestation, UncheckedAttestation, CheckedAttestation, AttestationContext};

pub use substrate_primitives::{storage, H256, OpaqueMetadata, Blake2Hasher};

#[cfg(feature = "std")]
pub use substrate_primitives::bytes;

/// Shasper validator public key.
pub type ValidatorId = AuthorityId;

/// A hash of some data used by the chain.
pub type Hash = substrate_primitives::H256;

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
