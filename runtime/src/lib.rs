//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

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
extern crate srml_aura as aura;
extern crate srml_system as system;
extern crate srml_executive as executive;
extern crate srml_consensus as consensus;
extern crate srml_timestamp as timestamp;
extern crate srml_balances as balances;
extern crate srml_upgrade_key as upgrade_key;
extern crate shasper_consensus_primitives as consensus_primitives;

#[cfg(feature = "std")]
mod genesis;
mod storage;

use rstd::prelude::*;
use primitives::{opaque, H256, ValidatorId, BlockNumber, Hash, OpaqueMetadata};
use primitives::storage::well_known_keys;
use runtime_primitives::{
	ApplyResult, transaction_validity::TransactionValidity,
	generic, traits::{Extrinsic as ExtrinsicT, Header as HeaderT, BlakeTwo256, Block as BlockT, GetNodeBlockType, GetRuntimeBlockType},
	BasicInherentData, CheckInherentError, ApplyOutcome,
};
use client::{
	block_builder::api as block_builder_api, runtime_api as client_api
};
use consensus_primitives::api as consensus_api;
use version::RuntimeVersion;
#[cfg(feature = "std")]
use version::NativeVersion;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use runtime_primitives::BuildStorage;
pub use runtime_primitives::{Permill, Perbill};
pub use srml_support::{StorageValue, StorageVec, RuntimeMetadata};
#[cfg(feature = "std")]
pub use genesis::GenesisConfig;

const TIMESTAMP_SET_POSITION: u32 = 0;

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

pub type DigestItem = generic::DigestItem<H256, ValidatorId>;
pub type Log = DigestItem;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
pub type Digest = generic::Digest<DigestItem>;

#[derive(Decode, Encode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum UncheckedExtrinsic {
	Timestamp(u64),
	Consensus(u64),
	Attestation
}

impl ExtrinsicT for UncheckedExtrinsic {
	fn is_signed(&self) -> Option<bool> {
		match self {
			UncheckedExtrinsic::Timestamp(_) => Some(false),
			UncheckedExtrinsic::Consensus(_) => Some(false),
			UncheckedExtrinsic::Attestation => None,
		}
	}
}

struct AuthorityStorageVec;
impl StorageVec for AuthorityStorageVec {
	type Item = ValidatorId;
	const PREFIX: &'static [u8] = well_known_keys::AUTHORITY_PREFIX;
}

pub struct Runtime;

impl GetNodeBlockType for Runtime {
	type NodeBlock = opaque::Block;
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
			AuthorityStorageVec::items()
		}

		fn execute_block(_block: Block) {

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
		fn apply_extrinsic(_extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
			Ok(ApplyOutcome::Success)
		}

		fn finalise_block() -> <Block as BlockT>::Header {
			Header::new(
				<storage::Number>::take(),
				<storage::ExtrinsicsRoot>::take(),
				Hash::from(runtime_io::storage_root()),
				<storage::ParentHash>::take(),
				<storage::Digest>::take()
			)
		}

		fn inherent_extrinsics(data: BasicInherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			let mut inherent = Vec::new();

			inherent.push(
				(TIMESTAMP_SET_POSITION, UncheckedExtrinsic::Timestamp(data.timestamp))
			);

			inherent.as_mut_slice().sort_unstable_by_key(|v| v.0);
			inherent.into_iter().map(|v| v.1).collect()
		}

		fn check_inherents(block: Block, _data: BasicInherentData) -> Result<(), CheckInherentError> {
			// draw timestamp out from extrinsics.
			block.extrinsics()
				.get(TIMESTAMP_SET_POSITION as usize)
				.and_then(|xt: &UncheckedExtrinsic| match xt {
					UncheckedExtrinsic::Timestamp(ref t) => Some(t.clone()),
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
