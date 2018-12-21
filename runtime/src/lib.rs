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

extern crate sr_std as rstd;
extern crate sr_io as runtime_io;
#[macro_use]
extern crate substrate_client as client;
#[macro_use]
extern crate srml_support;
extern crate sr_primitives as runtime_primitives;
#[cfg(feature = "std")]
#[macro_use]
extern crate serde_derive;
extern crate shasper_primitives as primitives;
extern crate parity_codec;
#[macro_use]
extern crate parity_codec_derive;
#[macro_use]
extern crate sr_version as version;
extern crate shasper_consensus_primitives as consensus_primitives;
extern crate keccak_hasher;
extern crate ssz;
#[macro_use]
extern crate ssz_derive;
extern crate ssz_hash;
#[macro_use]
extern crate ssz_hash_derive;
extern crate byteorder;
extern crate hash_db;
extern crate shasper_crypto as crypto;
extern crate shuffling;
extern crate srml_support as runtime_support;

#[cfg(feature = "std")]
mod genesis;
mod storage;
mod consts;
mod attestation;
pub mod spec;
mod extrinsic;
mod validators;
mod state;
mod utils;
pub mod validation;

use rstd::prelude::*;
use primitives::{H256, ValidatorId, OpaqueMetadata};
use client::block_builder::api::runtime_decl_for_BlockBuilder::BlockBuilder;
use runtime_primitives::{
	ApplyResult, transaction_validity::TransactionValidity, generic,
	traits::{self, Block as BlockT, GetNodeBlockType, GetRuntimeBlockType, BlakeTwo256, Hash as HashT},
	BasicInherentData, CheckInherentError, ApplyOutcome,
};
use client::{
	block_builder::api as block_builder_api, runtime_api as client_api
};
use srml_support::StorageMap;
use srml_support::storage::unhashed::StorageVec;
use consensus_primitives::api as consensus_api;
use version::RuntimeVersion;
#[cfg(feature = "std")]
use version::NativeVersion;
use parity_codec::Encode;
use keccak_hasher::KeccakHasher;
use spec::SpecHeader;
use ssz_hash::SpecHash;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use runtime_primitives::BuildStorage;
pub use runtime_primitives::{Permill, Perbill};
pub use srml_support::{StorageValue, RuntimeMetadata};
#[cfg(feature = "std")]
pub use genesis::GenesisConfig;
pub use attestation::AttestationRecord;
pub use extrinsic::UncheckedExtrinsic;
pub use primitives::BlockNumber;
pub use validators::{ValidatorRecord, ShardAndCommittee};
pub use state::{CrosslinkRecord, ActiveState, BlockVoteInfo, CrystallizedState};

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("shasper"),
	impl_name: create_runtime_str!("shasper"),
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

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize))]
pub enum DigestItem {
	/// System digest item announcing that authorities set has been changed
	/// in the block. Contains the new set of authorities.
	AuthoritiesChange(Vec<ValidatorId>),
	/// System digest item that contains the root of changes trie at given
	/// block. It is created for every block iff runtime supports changes
	/// trie creation.
	ChangesTrieRoot(H256),
	/// Put a Seal on it
	Seal(u64, Vec<u8>),
	/// Any 'non-system' digest item, opaque to the native code.
	Other(Vec<u8>),
}

impl traits::DigestItem for DigestItem {
	type Hash = H256;
	type AuthorityId = ValidatorId;

	fn as_authorities_change(&self) -> Option<&[Self::AuthorityId]> {
		match self {
			DigestItem::AuthoritiesChange(ref validators) => Some(validators),
			_ => None,
		}
	}

	fn as_changes_trie_root(&self) -> Option<&H256> {
		match self {
			DigestItem::ChangesTrieRoot(ref root) => Some(root),
			_ => None,
		}
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
			<storage::Authorities>::items()
		}

		fn execute_block(block: Block) {
			let (header, extrinsics) = block.deconstruct();
			Runtime::initialise_block(header);
			extrinsics.into_iter().for_each(|e| {
				Runtime::apply_extrinsic(e).ok().expect("Extrinsic in block execution must be valid");
			});

			Runtime::finalise_block();
		}

		fn initialise_block(header: <Block as BlockT>::Header) {
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

	impl block_builder_api::BlockBuilder<Block, BasicInherentData> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
			let extrinsic_index = <storage::UncheckedExtrinsics>::count();

			if extrinsic_index == consts::TIMESTAMP_POSITION {
				<storage::Timestamp>::put(extrinsic.clone().timestamp().expect("Invalid timestamp"));
			} else if extrinsic_index == consts::SLOT_POSITION {
				let parent_slot = <storage::Slot>::get();
				<storage::ParentSlot>::put(parent_slot);
				<storage::Slot>::put(extrinsic.clone().slot().expect("Invalid slot"));
			} else if extrinsic_index == consts::RANDAO_REVEAL_POSITION {
				<storage::RandaoReveal>::put(extrinsic.clone().randao_reveal().expect("Invalid randao reveal"));
			} else if extrinsic_index == consts::POW_CHAIN_REF_POSITION {
				<storage::PowChainRef>::put(extrinsic.clone().pow_chain_ref().expect("Invalid pow chain ref"));
			} else {
				let attestation = extrinsic.clone().attestation().expect("Invalid attestation");
				let mut attestations = <storage::Attestations>::items();
				attestations.push(attestation);
				<storage::Attestations>::set_items(attestations);
			}

			let mut extrinsics = <storage::UncheckedExtrinsics>::items();
			extrinsics.push(extrinsic);

			let extrinsics_data: Vec<Vec<u8>> = extrinsics.iter().map(Encode::encode).collect();
			let extrinsics_root = BlakeTwo256::enumerated_trie_root(&extrinsics_data.iter().map(Vec::as_slice).collect::<Vec<_>>());
			<storage::ExtrinsicsRoot>::put(H256::from(extrinsics_root));

			<storage::UncheckedExtrinsics>::set_items(extrinsics);

			Ok(ApplyOutcome::Success)
		}

		fn finalise_block() -> <Block as BlockT>::Header {
			<storage::UncheckedExtrinsics>::set_count(0);

			let number = <storage::Number>::take();
			let extrinsics_root = <storage::ExtrinsicsRoot>::take();
			let parent_hash = <storage::ParentHash>::take();
			let digest = <storage::Digest>::take();
			let _timestamp = <storage::Timestamp>::take();
			let slot = <storage::Slot>::get();
			let parent_slot = <storage::ParentSlot>::get();
			let parent_header_hash = <storage::LastHeaderHash>::get();
			let randao_reveal = <storage::RandaoReveal>::take();
			let pow_chain_ref = <storage::PowChainRef>::take();
			let attestations = <storage::Attestations>::items();

			<storage::Attestations>::set_count(0);

			let mut active_state = <storage::Active>::get();
			let mut crystallized_state = <storage::Crystallized>::get();

			if number == 1 {
				crystallized_state.last_state_recalc = slot;
			}

			validation::validate_block_pre_processing_conditions();
			active_state.update_recent_block_hashes(parent_slot, slot, parent_header_hash);

			validation::process_block::<storage::BlockHashesBySlot, storage::BlockVoteCache>(
				slot,
				parent_slot,
				&crystallized_state,
				&mut active_state,
				&attestations
			);

			validation::process_cycle_transitions::<storage::BlockHashesBySlot, storage::BlockVoteCache>(
				slot,
				parent_header_hash,
				&mut crystallized_state,
				&mut active_state
			);

			let active_state_root = active_state.spec_hash::<KeccakHasher>();
			let crystallized_state_root = crystallized_state.spec_hash::<KeccakHasher>();

			let spec_header = SpecHeader {
				randao_reveal, attestations, pow_chain_ref,
				active_state_root, crystallized_state_root,
				slot_number: slot,
				parent_hash: parent_header_hash,
			};
			let block_hash = ssz_hash::SpecHash::spec_hash::<KeccakHasher>(&spec_header);

			<storage::BlockHashesBySlot>::insert(slot, block_hash);
			<storage::Active>::put(&active_state);
			<storage::ActiveRoot>::put(&active_state_root);
			<storage::Crystallized>::put(&crystallized_state);
			<storage::CrystallizedRoot>::put(&crystallized_state_root);
			<storage::LastHeaderHash>::put(&block_hash);

			let state_root = BlakeTwo256::storage_root();

			Header {
				number, extrinsics_root, state_root, parent_hash, digest
			}
		}

		fn inherent_extrinsics(data: BasicInherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			let mut inherent = Vec::new();

			inherent.push(
				(consts::TIMESTAMP_POSITION, UncheckedExtrinsic::Timestamp(data.timestamp))
			);

			inherent.push(
				(consts::SLOT_POSITION, UncheckedExtrinsic::Slot(data.aura_expected_slot))
			);

			inherent.push(
				(consts::RANDAO_REVEAL_POSITION, UncheckedExtrinsic::RandaoReveal(Default::default()))
			);

			inherent.push(
				(consts::POW_CHAIN_REF_POSITION, UncheckedExtrinsic::PowChainRef(Default::default()))
			);

			inherent.as_mut_slice().sort_unstable_by_key(|v| v.0);
			inherent.into_iter().map(|v| v.1).collect()
		}

		fn check_inherents(block: Block, _data: BasicInherentData) -> Result<(), CheckInherentError> {
			block.extrinsics()
				.get(consts::TIMESTAMP_POSITION as usize)
				.and_then(|xt: &UncheckedExtrinsic| match xt {
					UncheckedExtrinsic::Timestamp(ref t) => Some(t.clone()),
					_ => None,
				})
				.ok_or_else(|| CheckInherentError::Other("No valid timestamp in block.".into()))?;

			block.extrinsics()
				.get(consts::SLOT_POSITION as usize)
				.and_then(|xt: &UncheckedExtrinsic| match xt {
					UncheckedExtrinsic::Slot(ref t) => Some(t.clone()),
					_ => None,
				})
				.ok_or_else(|| CheckInherentError::Other("No valid timestamp in block.".into()))?;

			block.extrinsics()
				.get(consts::RANDAO_REVEAL_POSITION as usize)
				.and_then(|xt: &UncheckedExtrinsic| match xt {
					UncheckedExtrinsic::RandaoReveal(ref t) => Some(t.clone()),
					_ => None,
				})
				.ok_or_else(|| CheckInherentError::Other("No valid timestamp in block.".into()))?;

			block.extrinsics()
				.get(consts::POW_CHAIN_REF_POSITION as usize)
				.and_then(|xt: &UncheckedExtrinsic| match xt {
					UncheckedExtrinsic::PowChainRef(ref t) => Some(t.clone()),
					_ => None,
				})
				.ok_or_else(|| CheckInherentError::Other("No valid timestamp in block.".into()))?;

			Ok(())
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			Default::default()
		}
	}

	impl client_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(_tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			TransactionValidity::Valid {
				priority: 0,
				requires: Vec::new(),
				provides: Vec::new(),
				longevity: u64::max_value(),
			}
		}
	}

	impl consensus_api::AuraApi<Block> for Runtime {
		fn slot_duration() -> u64 {
			10
		}
	}
}
