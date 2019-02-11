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
mod state;
pub mod utils;

use rstd::prelude::*;
use primitives::{BlockNumber, ValidatorId, OpaqueMetadata, UncheckedAttestation, CheckedAttestation};
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
use inherents::{CheckInherentsResult, InherentData};
use runtime_support::storage::StorageValue;
use runtime_support::storage::unhashed::StorageVec;
use consensus_primitives::api as consensus_api;
use runtime_version::RuntimeVersion;
#[cfg(feature = "std")]
use runtime_version::NativeVersion;
use codec::Encode;
use client::impl_runtime_apis;
use casper::store::ValidatorStore;
use state::Store;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use runtime_primitives::BuildStorage;
#[cfg(feature = "std")]
pub use genesis::GenesisConfig;
pub use extrinsic::UncheckedExtrinsic;
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
			let store = Store;

			let current_slot = storage::Number::get();
			store.active_validators(current_slot)
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
			use runtime_primitives::traits::Header;

			storage::Number::put(header.number());
			storage::ParentHash::put(header.parent_hash());
			storage::Digest::put(header.digest.clone());

			storage::note_parent_hash();
		}
	}

	impl client_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Default::default())
		}
	}

	impl block_builder_api::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
			let _extrinsic_index = <storage::UncheckedExtrinsics>::count();

			let mut extrinsics = <storage::UncheckedExtrinsics>::items();
			extrinsics.push(Some(extrinsic.clone()));

			<storage::UncheckedExtrinsics>::set_items(extrinsics);

			match extrinsic {
				UncheckedExtrinsic::Attestation(attestation) => {
					let checked = state::check_attestation(attestation).expect("Extrinsic is invalid.");
					let casper = storage::CasperContext::get();
					if !casper.validate_attestation(&checked) {
						panic!("Extrinsic does not pass casper check.");
					}
					storage::PendingAttestations::set_item(storage::PendingAttestations::count(), &Some(checked));
					storage::CasperContext::put(casper);
				},
			}

			Ok(ApplyOutcome::Success)
		}

		fn finalise_block() -> <Block as BlockT>::Header {
			let mut store = Store;
			let number = <storage::Number>::get();

			if number % consts::CYCLE_LENGTH == 0 {
				let mut casper = storage::CasperContext::get();
				let beacon_rewards = casper::reward::beacon_rewards(&store);
				let casper_rewards = casper::reward::casper_rewards(&casper, &store);
				let actions = casper::reward::default_scheme_rewards(
					&store,
					&beacon_rewards,
					&casper_rewards,
					casper.epoch - casper.finalized_epoch,
					&casper::reward::DefaultSchemeConfig {
						base_reward_quotient: consts::BASE_REWARD_QUOTIENT,
						inactivity_penalty_quotient: consts::INACTIVITY_PENALTY_QUOTIENT,
						includer_reward_quotient: consts::INCLUDER_REWARD_QUOTIENT,
						min_attestation_inclusion_delay: consts::MIN_ATTESTATION_INCLUSION_DELAY,
						whistleblower_reward_quotient: consts::WHISTLEBLOWER_REWARD_QUOTIENT,
					},
				);

				for action in actions {
					use casper::reward::RewardAction;

					match action {
						(validator_id, RewardAction::Add(balance)) =>
							storage::add_balance(&validator_id, balance),
						(validator_id, RewardAction::Sub(balance)) =>
							storage::sub_balance(&validator_id, balance),
						(validator_id, RewardAction::Penalize(balance)) =>
							storage::penalize_validator(&validator_id, balance)
					}
				}

				casper.advance_epoch(&mut store);
			}

			let extrinsics = storage::UncheckedExtrinsics::items()
				.into_iter()
				.filter(|e| e.is_some())
				.map(|e| e.expect("Checked is_some in filter; qed"))
				.collect::<Vec<_>>();
			let extrinsic_data = extrinsics.iter().map(Encode::encode).collect::<Vec<_>>();
			storage::UncheckedExtrinsics::set_count(0);

			storage::Number::take();
			let parent_hash = storage::ParentHash::take();
			let extrinsics_root = BlakeTwo256::enumerated_trie_root(&extrinsic_data.iter().map(Vec::as_slice).collect::<Vec<_>>());
			let digest = <storage::Digest>::take();
			let state_root = BlakeTwo256::storage_root();

			Header {
				number, extrinsics_root, state_root, parent_hash, digest
			}
		}

		fn inherent_extrinsics(_data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
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
		fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			match tx {
				UncheckedExtrinsic::Attestation(attestation) => {
					let checked = state::check_attestation(attestation).expect("Extrinsic is invalid.");
					let casper = storage::CasperContext::get();
					if !casper.validate_attestation(&checked) {
						panic!("Extrinsic does not pass casper check.");
					}
				},
			}

			TransactionValidity::Valid {
				priority: 0,
				requires: Vec::new(),
				provides: Vec::new(),
				longevity: u64::max_value(),
			}
		}
	}

	impl aura_primitives::AuraApi<Block> for Runtime {
		fn slot_duration() -> u64 {
			2
		}
	}

	impl consensus_api::ShasperApi<Block> for Runtime {
		fn finalized_epoch() -> u64 {
			let casper = storage::CasperContext::get();
			casper.finalized_epoch
		}

		fn justified_epoch() -> u64 {
			let casper = storage::CasperContext::get();
			casper.justified_epoch
		}

		fn slot() -> u64 {
			storage::Number::get()
		}

		fn finalized_slot() -> u64 {
			let casper = storage::CasperContext::get();
			utils::epoch_to_slot(casper.finalized_epoch)
		}

		fn justified_slot() -> u64 {
			let casper = storage::CasperContext::get();
			utils::epoch_to_slot(casper.justified_epoch)
		}

		fn check_attestation(unchecked: UncheckedAttestation) -> Option<CheckedAttestation> {
			state::check_attestation(unchecked)
		}

		fn validator_index(validator_id: ValidatorId) -> Option<u32> {
			for (i, record) in storage::Validators::items().into_iter().enumerate() {
				if let Some(record) = record {
					if record.validator_id == validator_id {
						return Some(i as u32);
					}
				}
			}
			None
		}
	}
}
