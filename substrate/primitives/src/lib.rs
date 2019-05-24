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

pub use crypto;
pub use beacon::primitives::{H768, Signature, H384, H256, ValidatorId, BitField, H32, Version};

/// Shasper validator public key.
pub type AuthorityId = ValidatorId;

/// A hash of some data used by the chain.
pub type Hash = H256;

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

pub fn into_public(value: &H384) -> Option<crypto::bls::Public> {
	crypto::bls::Public::from_bytes(value.as_ref()).ok()
}

pub fn from_public(public: crypto::bls::Public) -> H384 {
	H384::from_slice(&public.as_bytes())
}

pub fn into_signature(value: &H768) -> Option<crypto::bls::Signature> {
	crypto::bls::Signature::from_bytes(value.as_ref()).ok()
}

pub fn into_aggregate_signature(value: &H768) -> Option<crypto::bls::AggregateSignature> {
	crypto::bls::AggregateSignature::from_bytes(value.as_ref()).ok()
}

pub fn from_signature(sig: crypto::bls::Signature) -> H768 {
	H768::from_slice(&sig.as_bytes())
}

pub fn from_aggregate_signature(sig: crypto::bls::AggregateSignature) -> H768 {
	H768::from_slice(&sig.as_bytes())
}
