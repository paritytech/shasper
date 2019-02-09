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

//! The Substrate Shasper runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

extern crate parity_codec as codec;
extern crate parity_codec_derive as codec_derive;
extern crate substrate_client as client;

#[cfg(feature = "std")]
mod genesis;
mod storage;
mod consts;
mod extrinsic;
mod digest;

use rstd::prelude::*;
use primitives::{Slot, H256, ValidatorId, OpaqueMetadata};
use client::block_builder::api::runtime_decl_for_BlockBuilder::BlockBuilder;
use runtime_primitives::{
	ApplyResult, transaction_validity::TransactionValidity, generic,
	traits::{Block as BlockT, GetNodeBlockType, GetRuntimeBlockType, BlakeTwo256, Hash as HashT},
	ApplyOutcome,
};
use client::{
	block_builder::api as block_builder_api,
	runtime_api as client_api
};
use inherents::{CheckInherentsResult, InherentData, MakeFatalError};
use runtime_support::StorageMap;
use runtime_support::storage::unhashed::StorageVec;
use consensus_primitives::api as consensus_api;
use runtime_version::RuntimeVersion;
#[cfg(feature = "std")]
use runtime_version::NativeVersion;
use codec::Encode;
use keccak_hasher::KeccakHasher;
use ssz_hash::SpecHash;
use client::impl_runtime_apis;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use runtime_primitives::BuildStorage;
pub use runtime_primitives::{Permill, Perbill};
pub use runtime_support::StorageValue;
#[cfg(feature = "std")]
pub use genesis::GenesisConfig;
pub use extrinsic::UncheckedExtrinsic;
pub use primitives::BlockNumber;
pub use digest::DigestItem;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: runtime_primitives::create_runtime_str!("shasper"),
	impl_name: runtime_primitives::create_runtime_str!("shasper"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

pub type Log = DigestItem;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
pub type Digest = generic::Digest<DigestItem>;

pub struct Runtime;

impl GetNodeBlockType for Runtime {
	type NodeBlock = Block;
}

impl GetRuntimeBlockType for Runtime {
	type RuntimeBlock = Block;
}

// Implement our runtime API endpoints. This is just a bunch of proxying.
impl_runtime_apis! {
	impl client_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn authorities() -> Vec<ValidatorId> {
			unimplemented!()
		}

		fn execute_block(block: Block) {
			let (header, extrinsics) = block.deconstruct();
			Runtime::initialise_block(&header);
			extrinsics.into_iter().for_each(|e| {
				Runtime::apply_extrinsic(e).ok().expect("Extrinsic in block execution must be valid");
			});

			Runtime::finalise_block();
		}

		fn initialise_block(header: &<Block as BlockT>::Header) {
			<storage::Number>::put(header.number);
			<storage::ParentHash>::put(header.parent_hash);
			<storage::ExtrinsicsRoot>::put(header.extrinsics_root);
			<storage::Digest>::put(header.digest.clone());
		}
	}

	impl client_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Default::default())
		}
	}

	impl block_builder_api::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
			let extrinsic_index = <storage::UncheckedExtrinsics>::count();

			let mut extrinsics = <storage::UncheckedExtrinsics>::items();
			extrinsics.push(extrinsic);

			let extrinsics_data: Vec<Vec<u8>> = extrinsics.iter().map(Encode::encode).collect();
			let extrinsics_root = BlakeTwo256::enumerated_trie_root(&extrinsics_data.iter().map(Vec::as_slice).collect::<Vec<_>>());
			<storage::ExtrinsicsRoot>::put(H256::from(extrinsics_root));

			<storage::UncheckedExtrinsics>::set_items(extrinsics);

			unimplemented!();

			Ok(ApplyOutcome::Success)
		}

		fn finalise_block() -> <Block as BlockT>::Header {
			<storage::UncheckedExtrinsics>::set_count(0);

			let number = <storage::Number>::take();
			let parent_hash = <storage::ParentHash>::take();
			let extrinsics_root = <storage::ExtrinsicsRoot>::take();
			let digest = <storage::Digest>::take();

			unimplemented!()
		}

		fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			Default::default()
		}

		fn check_inherents(_block: Block, _data: InherentData) -> CheckInherentsResult {
			CheckInherentsResult::new()
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			Default::default()
		}
	}

	impl client_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(_tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			unimplemented!()
		}
	}

	impl aura_primitives::AuraApi<Block> for Runtime {
		fn slot_duration() -> u64 {
			10
		}
	}

	impl consensus_api::ShasperApi<Block> for Runtime {
		fn finalized_slot() -> u64 {
			unimplemented!()
		}

		fn justified_slot() -> u64 {
			unimplemented!()
		}
	}
}
