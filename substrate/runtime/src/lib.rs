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

extern crate parity_codec as codec;
extern crate parity_codec_derive as codec_derive;
extern crate substrate_client as client;

mod extrinsic;
mod storage;
#[cfg(feature = "std")]
mod genesis;

pub use extrinsic::Extrinsic;
#[cfg(feature = "std")]
pub use genesis::GenesisConfig;

use primitives::{sr25519, H256, OpaqueMetadata};
use codec::Encode;
use runtime_primitives::{
	create_runtime_str, ApplyOutcome, ApplyResult,
	traits::{GetNodeBlockType, GetRuntimeBlockType, BlakeTwo256, Verify, Hash as HashT,
			 Block as BlockT, Header as HeaderT},
	transaction_validity::TransactionValidity,
};
use runtime_support::storage::{StorageValue, unhashed::StorageVec};
use inherents::{InherentData, CheckInherentsResult};
use client::{
	impl_runtime_apis,
	block_builder::api as block_builder_api,
	runtime_api as client_api,
};
#[cfg(feature = "std")]
use runtime_version::{RuntimeVersion, NativeVersion};

/// Shasper runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("shasper"),
	impl_name: create_runtime_str!("shasper-substrate"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// Proof of authority signature.
pub type AuthoritySignature = sr25519::Signature;
/// The identity type used by authorities.
pub type AuthorityId = <AuthoritySignature as Verify>::Signer;
/// The signature type used by accounts/transactions.
pub type AccountSignature = sr25519::Signature;
/// An identifier for an account on this system.
pub type AccountId = <AccountSignature as Verify>::Signer;
/// A simple hash type for all our hashing.
pub type Hash = H256;
/// The block number type used in this runtime.
pub type BlockNumber = u64;
/// Index of a transaction.
pub type Index = u64;
/// The item of a block digest.
pub type DigestItem = runtime_primitives::generic::DigestItem<H256, AuthorityId, AuthoritySignature>;
/// The digest of a block.
pub type Digest = runtime_primitives::generic::Digest<DigestItem>;
/// A test block.
pub type Block = runtime_primitives::generic::Block<Header, Extrinsic>;
/// A test block's header.
pub type Header = runtime_primitives::generic::Header<BlockNumber, BlakeTwo256, DigestItem>;

/// Shasper runtime.
pub struct Runtime;

impl GetNodeBlockType for Runtime {
	type NodeBlock = Block;
}

impl GetRuntimeBlockType for Runtime {
	type RuntimeBlock = Block;
}

impl_runtime_apis! {
	impl client_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion { VERSION }
		fn execute_block(block: Block) {
			let (_header, extrinsics) = block.deconstruct();

			match &extrinsics[0] {
				Extrinsic::BeaconBlock(block) => {
					let mut state = storage::State::get().expect("State has been initialized");
					let config = storage::Config::get().expect("Config has been initialized");

					beacon::execute_block(block, &mut state, &config)
						.expect("Executing block failed");
					storage::State::put(state);
				}
			}
		}
		fn initialize_block(header: &<Block as BlockT>::Header) {
			storage::Number::put(header.number());
			storage::ParentHash::put(header.parent_hash());
			storage::Digest::put(header.digest.clone());
		}
		fn authorities() -> Vec<AuthorityId> {
			panic!("Deprecated, please use `AuthoritiesApi`.")
		}
	}

	impl block_builder_api::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
			let mut extrinsics = storage::Extrinsics::items();
			extrinsics.push(Some(extrinsic.clone()));
			storage::Extrinsics::set_items(extrinsics);

			match extrinsic {
				Extrinsic::BeaconBlock(block) => {
					let mut state = storage::State::get().expect("State has been initialized");
					let config = storage::Config::get().expect("Config has been initialized");

					beacon::execute_block(&block, &mut state, &config)
						.expect("Executing block failed");
					storage::State::put(state);
				}
			}

			Ok(ApplyOutcome::Success)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			let extrinsics = storage::Extrinsics::items()
				.into_iter()
				.filter(|e| e.is_some())
				.map(|e| e.expect("Checked is_some in filter; qed"))
				.collect::<Vec<_>>();
			let extrinsic_data = extrinsics.iter().map(Encode::encode).collect::<Vec<_>>();
			storage::Extrinsics::set_count(0);

			let number = storage::Number::take();
			let parent_hash = storage::ParentHash::take();
			let extrinsics_root = BlakeTwo256::enumerated_trie_root(&extrinsic_data.iter().map(Vec::as_slice).collect::<Vec<_>>());
			let digest = storage::Digest::take();
			let state_root = BlakeTwo256::storage_root();

			Header {
				number, extrinsics_root, state_root, parent_hash, digest
			}
		}

		fn inherent_extrinsics(_data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			vec![]
		}

		fn check_inherents(_block: Block, _data: InherentData) -> CheckInherentsResult {
			CheckInherentsResult::new()
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			Default::default()
		}
	}

	impl client_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Default::default())
		}
	}

	impl client_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(_tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			TransactionValidity::Invalid(0)
		}
	}

	impl consensus_authorities::AuthoritiesApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityId> {
			let authority = storage::Authority::get();
			vec![authority]
		}
	}

	impl aura_primitives::AuraApi<Block> for Runtime {
		fn slot_duration() -> u64 {
			6
		}
	}
}
