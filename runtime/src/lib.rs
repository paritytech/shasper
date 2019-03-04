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

#![warn(missing_docs)]

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

use primitives::BlockNumber;
use runtime_primitives::{generic, traits::{GetNodeBlockType, GetRuntimeBlockType, BlakeTwo256}};
#[cfg(feature = "std")]
use runtime_version::NativeVersion;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use runtime_primitives::BuildStorage;
#[cfg(feature = "std")]
pub use genesis::GenesisConfig;
pub use extrinsic::UncheckedExtrinsic;
pub use digest::DigestItem;
pub use apis::{VERSION, RuntimeApi};
#[cfg(feature = "std")]
pub use apis::dispatch;

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// Block digest item.
pub type Log = DigestItem;
/// Block digest collection.
pub type Digest = generic::Digest<Log>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// Shasper runtime.
pub struct Runtime;

impl GetNodeBlockType for Runtime {
	type NodeBlock = Block;
}

impl GetRuntimeBlockType for Runtime {
	type RuntimeBlock = Block;
}

#[allow(missing_docs)]
mod apis {
	use rstd::prelude::*;
	use primitives::{Slot, ValidatorId, OpaqueMetadata, UncheckedAttestation, CheckedAttestation};
	use client::block_builder::api::runtime_decl_for_BlockBuilder::BlockBuilder;
	use runtime_primitives::{
		ApplyResult, transaction_validity::{TransactionValidity, TransactionLongevity},
		traits::{Block as BlockT, BlakeTwo256, Hash as HashT},
		ApplyOutcome, RuntimeString,
	};
	use client::{
		block_builder::api as block_builder_api,
		runtime_api as client_api
	};
	use inherents::{CheckInherentsResult, InherentData, MakeFatalError};
	use runtime_support::storage::StorageValue;
	use runtime_support::storage::unhashed::StorageVec;
	use consensus_primitives::api as consensus_api;
	use runtime_version::RuntimeVersion;
	use codec::Encode;
	use client::impl_runtime_apis;
	use casper::store::ValidatorStore;
	use super::{Block, Runtime};
	use crate::{
		state::{self, Store}, storage, utils, consts, extrinsic::UncheckedExtrinsic,
		Header,
	};

	#[cfg(feature = "std")]
	pub use self::api::dispatch;

	/// This runtime version.
	pub const VERSION: RuntimeVersion = RuntimeVersion {
		spec_name: runtime_primitives::create_runtime_str!("shasper"),
		impl_name: runtime_primitives::create_runtime_str!("shasper"),
		authoring_version: 1,
		spec_version: 1,
		impl_version: 0,
		apis: RUNTIME_API_VERSIONS,
	};

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
				extrinsics.push(Some(extrinsic.clone()));

				<storage::UncheckedExtrinsics>::set_items(extrinsics);

				match &extrinsic {
					UncheckedExtrinsic::Slot(slot)
						if extrinsic_index == consts::SLOT_INHERENT_EXTRINSIC_INDEX =>
					{
						storage::Slot::put(slot);
						storage::note_parent_hash();
					},
					UncheckedExtrinsic::Randao(reveal)
						if extrinsic_index == consts::RANDAO_INHERENT_EXTRINSIC_INDEX =>
					{
						let store = Store;
						let last_slot = storage::LastSlot::get();
						let slot = storage::Slot::get();

						let authorities = store.active_validators(slot);
						let idx = slot % (authorities.len() as u64);
						let proposer = authorities[idx as usize];

						let mut randao = storage::Randao::get();
						randao.mix(&reveal);

						let mut validators = storage::Validators::items();
						for record in &mut validators  {
							if let Some(record) = record.as_mut() {
								if record.validator_id == proposer {
									assert!(record.randao_commitment.reveal(&reveal, (slot - last_slot) as usize));
								}
							}
						}
						storage::Validators::set_items(validators);

						storage::Randao::put(randao);
					},
					UncheckedExtrinsic::Attestation(ref attestation)
						if extrinsic_index >= consts::ATTESTATION_EXTRINSIC_START_INDEX =>
					{
						let checked = state::check_attestation(attestation.clone(), true)
							.expect("Extrinsic is invalid.");
						let casper = storage::CasperContext::get();
						if !casper.validate_attestation(&checked) {
							panic!("Extrinsic does not pass casper check.");
						}

						let mut pending_attestations = storage::PendingAttestations::items();
						pending_attestations.push(Some(checked));
						storage::PendingAttestations::set_items(pending_attestations);
					},
					_ => panic!("Extrinsic order is incorrect"),
				}

				Ok(ApplyOutcome::Success)
			}

			fn finalise_block() -> <Block as BlockT>::Header {
				let mut store = Store;
				let mut last_slot = storage::LastSlot::get();
				let slot = storage::Slot::get();

				while last_slot < slot {
					if last_slot % consts::CYCLE_LENGTH == 0 {
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
						storage::CasperContext::put(casper);
					}
					last_slot += 1;
					storage::LastSlot::put(last_slot);
				}

				let extrinsics = storage::UncheckedExtrinsics::items()
					.into_iter()
					.filter(|e| e.is_some())
					.map(|e| e.expect("Checked is_some in filter; qed"))
					.collect::<Vec<_>>();
				assert!(extrinsics.len() >= consts::ATTESTATION_EXTRINSIC_START_INDEX as usize);
				let extrinsic_data = extrinsics.iter().map(Encode::encode).collect::<Vec<_>>();
				storage::UncheckedExtrinsics::set_count(0);

				let number = storage::Number::take();
				let parent_hash = storage::ParentHash::take();
				let extrinsics_root = BlakeTwo256::enumerated_trie_root(&extrinsic_data.iter().map(Vec::as_slice).collect::<Vec<_>>());
				let digest = storage::Digest::take();
				let state_root = BlakeTwo256::storage_root();

				Header {
					number, extrinsics_root, state_root, parent_hash, digest
				}
			}

			fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
				let timestamp = match data.get_data::<consensus_primitives::TimestampInherentData>(
					&consensus_primitives::TIMESTAMP_INHERENT_IDENTIFIER
				) {
					Ok(Some(data)) => data,
					_ => panic!("Decode timestamp inherent failed"),
				};
				let randao = match data.get_data::<consensus_primitives::RandaoInherentData>(
					&consensus_primitives::RANDAO_INHERENT_IDENTIFIER
				) {
					Ok(Some(data)) => data,
					_ => panic!("Decode randao inherent failed"),
				};

				let mut ret = Vec::new();
				ret.push(UncheckedExtrinsic::Slot(timestamp.slot));
				ret.push(UncheckedExtrinsic::Randao(randao.randao_reveal));
				ret
			}

			fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
				let mut result = CheckInherentsResult::default();

				let slot = match data.get_data::<consensus_primitives::TimestampInherentData>(
					&consensus_primitives::TIMESTAMP_INHERENT_IDENTIFIER
				) {
					Ok(Some(data)) => data.slot,
					_ => {
						result.put_error(
							consensus_primitives::TIMESTAMP_INHERENT_IDENTIFIER,
							&MakeFatalError::from(RuntimeString::from("Slot decode failed"))
						).expect("Putting error failed");
						return result;
					},
				};

				if (block.extrinsics.len() as u32) < consts::ATTESTATION_EXTRINSIC_START_INDEX {
					result.put_error(
						consensus_primitives::TIMESTAMP_INHERENT_IDENTIFIER,
						&MakeFatalError::from(RuntimeString::from("Slot extrinsic missing"))
					).expect("Putting error failed");
					return result;
				}

				match block.extrinsics[0] {
					UncheckedExtrinsic::Slot(block_slot) if block_slot == slot => (),
					_ => {
						result.put_error(
							consensus_primitives::TIMESTAMP_INHERENT_IDENTIFIER,
							&MakeFatalError::from(RuntimeString::from("Incorrect block slot"))
						).expect("Putting error failed");
						return result;
					},
				}

				match block.extrinsics[1] {
					UncheckedExtrinsic::Randao(_) => (),
					_ => {
						result.put_error(
							consensus_primitives::RANDAO_INHERENT_IDENTIFIER,
							&MakeFatalError::from(RuntimeString::from("Incorrect randao extrinsic"))
						).expect("Putting error failed");
						return result;
					},
				}

				result
			}

			fn random_seed() -> <Block as BlockT>::Hash {
				Default::default()
			}
		}

		impl client_api::TaggedTransactionQueue<Block> for Runtime {
			fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
				let checked = match tx {
					UncheckedExtrinsic::Attestation(attestation) => {
						let checked = match state::check_attestation(attestation, false) {
							Some(checked) => checked,
							None => return TransactionValidity::Invalid(0),
						};
						let casper = storage::CasperContext::get();
						if !casper.validate_attestation(&checked) {
							return TransactionValidity::Invalid(1)
						}
						if storage::PendingAttestations::items().contains(&Some(checked.clone())) {
							return TransactionValidity::Invalid(2)
						}
						checked
					},
					_ => return TransactionValidity::Invalid(3),
				};

				let target_epoch = checked.data.target_epoch;
				TransactionValidity::Valid {
					priority: 0,
					requires: Vec::new(),
					provides: checked.validator_ids.into_iter()
						.map(|validator_id| {
							(validator_id, target_epoch).encode()
						})
						.collect(),
					longevity: TransactionLongevity::max_value(),
				}
			}
		}

		impl aura_primitives::AuraApi<Block> for Runtime {
			fn slot_duration() -> u64 {
				consts::SLOT_DURATION
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

			fn finalized_slot() -> u64 {
				let casper = storage::CasperContext::get();
				utils::epoch_to_slot(casper.finalized_epoch)
			}

			fn justified_slot() -> u64 {
				let casper = storage::CasperContext::get();
				utils::epoch_to_slot(casper.justified_epoch)
			}

			fn slot() -> Slot {
				storage::Slot::get()
			}

			fn proposer(slot: Slot) -> ValidatorId {
				let store = Store;
				let authorities = store.active_validators(slot);

				let idx = slot % (authorities.len() as u64);
				authorities[idx as usize]
			}

			fn genesis_slot() -> Slot {
				storage::GenesisSlot::get()
			}

			fn check_attestation(unchecked: UncheckedAttestation) -> Option<CheckedAttestation> {
				state::check_attestation(unchecked, false)
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
}
