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

extern crate blake2;
extern crate parity_codec as codec;
#[macro_use]
extern crate parity_codec_derive;
extern crate ssz;
#[macro_use]
extern crate ssz_derive;
extern crate ssz_hash;
#[macro_use]
extern crate ssz_hash_derive;
extern crate hash_db;
extern crate plain_hasher;
extern crate tiny_keccak;
extern crate bls;
extern crate bls_aggregates;
extern crate shuffling;
extern crate byteorder;

#[cfg(feature = "std")]
extern crate serde;

#[cfg(feature = "std")]
#[macro_use]
extern crate serde_derive;

extern crate substrate_primitives as primitives;
extern crate sr_std as rstd;
extern crate sr_primitives as runtime_primitives;
extern crate sr_io as runtime_io;
#[macro_use]
extern crate sr_version as runtime_version;
#[macro_use]
extern crate srml_support as runtime_support;
#[macro_use]
extern crate substrate_client as client;

mod attestation;
mod extrinsic;
mod header;
mod utils;
mod state;
mod system;
mod validators;
mod block;
mod bitfield;

pub mod consts;
pub mod spec;
pub mod validation;

pub use attestation::AttestationRecord;
pub use header::{Header, Digest, DigestItem};
pub use extrinsic::Extrinsic;
pub use state::{CrosslinkRecord, ActiveState, CrystallizedState};
pub use validators::{ValidatorRecord, ShardAndCommittee};
pub use block::{Block, BlockId, BlockExt};
pub use bitfield::BitField;

use primitives::{H256, H160};
use rstd::prelude::*;

use runtime_primitives::{ApplyOutcome, ApplyResult};
use runtime_primitives::traits::{GetNodeBlockType, GetRuntimeBlockType};
use client::{runtime_api as client_api, block_builder::api as block_builder_api};
use runtime_version::RuntimeVersion;
#[cfg(feature = "std")] use runtime_version::NativeVersion;

/// Shasper runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: ver_str!("shasper"),
	impl_name: ver_str!("parity-shasper"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: apis_vec!([]),
};

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

pub type Hash = H256;
pub type BlockNumber = u64;
pub type Address = H160;
pub type PublicKey = Vec<u8>;
pub type ShardId = u16;
pub type InherentData = ();
pub type AuthorityId = primitives::AuthorityId;
pub type NullError = ();

pub struct Runtime;

impl GetNodeBlockType for Runtime {
	type NodeBlock = Block;
}

impl GetRuntimeBlockType for Runtime {
	type RuntimeBlock = Block;
}

// FIXME (#26): implement additional APIs via traits.
impl_runtime_apis! {
	impl client_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion { VERSION.clone() }
		fn authorities() -> Vec<AuthorityId> { system::authorities() }
		fn execute_block(block: Block) { system::execute_block(block) }
		fn initialise_block(header: Header) { system::initialise_block(header) }
	}

	impl block_builder_api::BlockBuilder<Block, Extrinsic> for Runtime {
		fn apply_extrinsic(extrinsic: Extrinsic) -> ApplyResult {
			system::apply_extrinsic(extrinsic);
			Ok(ApplyOutcome::Success)
		}
		fn finalise_block() -> Header { system::finalise_block() }
		fn inherent_extrinsics(_extrinsic: Extrinsic) -> Vec<Extrinsic> { system::inherent_extrinsics() }
		fn check_inherents(_block: Block, _data: Extrinsic) -> Result<(), runtime_primitives::CheckInherentError> { Ok(()) }
		fn random_seed() -> H256 { H256::default() }
	}
}
