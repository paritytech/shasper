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

extern crate parity_codec;
#[macro_use]
extern crate parity_codec_derive;
extern crate substrate_primitives as primitives;
extern crate sr_primitives as runtime_primitives;
#[macro_use]
extern crate fixed_hash;
extern crate sr_std as rstd;
#[cfg(feature = "std")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "std")]
extern crate serde;
extern crate ssz;
#[macro_use]
extern crate ssz_derive;
extern crate ssz_hash;
#[macro_use]
extern crate ssz_hash_derive;
extern crate hash_db;
extern crate keccak_hasher;
extern crate byteorder;
pub extern crate shasper_crypto as crypto;

mod authority_id;
mod bitfield;
mod attestation;

use runtime_primitives::generic;
use runtime_primitives::traits::{BlakeTwo256, Extrinsic as ExtrinsicT};

pub use authority_id::{H384, AuthorityId};
pub use bitfield::BitField;
pub use attestation::AttestationRecord;

pub use primitives::{storage, H256, OpaqueMetadata};

#[cfg(feature = "std")]
pub use primitives::bytes;

pub type ShardId = u16;

/// Shasper validator public key.
pub type ValidatorId = AuthorityId;

/// Alias to Ed25519 pubkey that identifies an account on the chain.
pub type AccountId = primitives::H256;

/// A hash of some data used by the chain.
pub type Hash = primitives::H256;

/// Index of a block number in the chain.
pub type BlockNumber = u64;

/// Index of an account's extrinsic in the chain.
pub type Nonce = u64;

/// Count value in Shasper.
pub type Count = u64;

/// Slot value in Shapser.
pub type Slot = u64;

pub type EthereumAddress = primitives::H160;
