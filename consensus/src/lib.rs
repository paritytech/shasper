// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

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

//! Shasper consensus engine.

extern crate parity_codec as codec;
extern crate substrate_client as client;

#[macro_use]
extern crate log;

mod block_import;

pub use consensus_primitives::*;
pub use block_import::{ShasperBlockImport, LatestAttestations};
pub use consensus_common::SyncOracle;
pub use aura_slots::SlotDuration;

use std::sync::Arc;
use std::time::Duration;
use std::collections::hash_map::{HashMap};

use codec::Encode;
use consensus_common::{Authorities, BlockImport, Environment, Proposer, ImportBlock, BlockOrigin, ForkChoiceStrategy, Error as ConsensusError};
use consensus_common::import_queue::{Verifier, BasicQueue};
use client::{blockchain::HeaderBackend, ChainHead};
use client::backend::AuxStore;
use client::block_builder::api::BlockBuilder as BlockBuilderApi;
use runtime::UncheckedExtrinsic;
use runtime::utils::epoch_to_slot;
use runtime_primitives::{generic::BlockId, Justification, RuntimeString};
use runtime_primitives::traits::{Block, Header, Digest, DigestItemFor, DigestItem, ProvideRuntimeApi, NumberFor};
use primitives::{ValidatorId, H256, Slot, Epoch, BlockNumber, UnsignedAttestation};
use aura_slots::{SlotCompatible, CheckedHeader, SlotWorker, SlotInfo};
use inherents::InherentDataProviders;
use casper::Attestation;
use transaction_pool::txpool::{ChainApi as PoolChainApi, Pool};

use futures::{Future, IntoFuture, future};
use tokio::timer::Timeout;
use parking_lot::Mutex;
use api::ShasperApi;
use crypto::bls;

/// A digest item which is usable with aura consensus.
pub trait CompatibleDigestItem: Sized {
	/// Construct a digest item which is a slot number and a signature on the
	/// hash.
	fn shasper_seal(slot_number: u64, signature: bls::Signature) -> Self;

	/// If this item is an Aura seal, return the slot number and signature.
	fn as_shasper_seal(&self) -> Option<(u64, bls::Signature)>;
}

impl CompatibleDigestItem for runtime::DigestItem {
	fn shasper_seal(slot_number: u64, signature: bls::Signature) -> Self {
		let signature_bytes: Vec<_> = signature.to_compressed_bytes().into_iter().cloned().collect();
		runtime::DigestItem::Seal(slot_number, signature_bytes)
	}

	fn as_shasper_seal(&self) -> Option<(u64, bls::Signature)> {
		match self {
			runtime::DigestItem::Seal(slot_number, signature_bytes) => {
				if let Some(signature) = bls::Signature::from_compressed_bytes(&signature_bytes) {
					Some((*slot_number, signature))
				} else {
					None
				}
			}
			_ => None
		}
	}
}

pub trait CompatibleExtrinsic: Sized {
	fn as_validator_attestation_map<B: Block<Hash=H256>, C>(&self, client: &C, id: &BlockId<B>) -> Option<HashMap<ValidatorId, (Slot, H256)>> where
		C: ProvideRuntimeApi,
		C::Api: ShasperApi<B>;
}

impl CompatibleExtrinsic for runtime::UncheckedExtrinsic {
	fn as_validator_attestation_map<B: Block<Hash=H256>, C>(&self, client: &C, id: &BlockId<B>) -> Option<HashMap<ValidatorId, (Slot, H256)>> where
		C: ProvideRuntimeApi,
		C::Api: ShasperApi<B>
	{
		match self {
			&runtime::UncheckedExtrinsic::Attestation(ref attestation) => {
				let checked = match client.runtime_api().check_attestation(id, attestation.clone()) {
					Ok(checked) => checked?,
					Err(_) => return None,
				};

				Some(checked
					 .validator_ids()
					 .into_iter()
					 .map(|v| (v, (epoch_to_slot(attestation.data.target_epoch), attestation.data.target_epoch_block_hash)))
					 .collect())
			},
		}
	}
}

fn inherent_to_common_error(err: RuntimeString) -> consensus_common::Error {
	consensus_common::ErrorKind::InherentData(err.into()).into()
}

fn client_to_common_error(err: client::error::Error) -> consensus_common::Error {
	consensus_common::ErrorKind::Other(Box::new(err)).into()
}

/// Register the shasper inherent data provider.
pub fn register_shasper_inherent_data_provider(
	inherent_data_providers: &InherentDataProviders,
	slot_duration: u64,
) -> Result<(), consensus_common::Error> {
	if !inherent_data_providers.has_provider(&consensus_primitives::INHERENT_IDENTIFIER) {
		inherent_data_providers
			.register_provider(consensus_primitives::InherentDataProvider::new(slot_duration))
			.map_err(inherent_to_common_error)
	} else {
		Ok(())
	}
}

struct ShasperSlotCompatible;

impl SlotCompatible for ShasperSlotCompatible {
	fn extract_timestamp_and_slot(
		data: &inherents::InherentData
	) -> Result<(u64, u64), consensus_common::Error> {
		match data.get_data::<consensus_primitives::InherentData>(
			&consensus_primitives::INHERENT_IDENTIFIER
		) {
			Ok(Some(data)) => Ok((data.timestamp, data.slot)),
			_ => Err(consensus_common::ErrorKind::InherentData("Decode inherent failed".into()).into()),
		}
	}
}

fn replace_inherent_data_slot(
	data: &mut inherents::InherentData,
	slot: Slot,
) -> Result<(), consensus_common::Error> {
	let mut inherent_data = match data.get_data::<consensus_primitives::InherentData>(
		&consensus_primitives::INHERENT_IDENTIFIER
	) {
		Ok(Some(data)) => data,
		_ => return Err(consensus_common::ErrorKind::InherentData("Decode inherent failed".into()).into()),
	};

	inherent_data.slot = slot;
	data.replace_data(consensus_primitives::INHERENT_IDENTIFIER, &inherent_data);

	Ok(())
}

/// Start the shasper worker. The returned future should be run in a tokio runtime.
pub fn start_shasper<B, C, E, I, SO, P, Error, OnExit>(
	slot_duration: SlotDuration,
	local_key: Arc<bls::Pair>,
	client: Arc<C>,
	block_import: Arc<I>,
	env: Arc<E>,
	sync_oracle: SO,
	pool: Arc<Pool<P>>,
	on_exit: OnExit,
	inherent_data_providers: InherentDataProviders,
) -> Result<impl Future<Item=(), Error=()>, consensus_common::Error> where
	B: Block<Hash=H256, Extrinsic=UncheckedExtrinsic>,
	NumberFor<B>: From<BlockNumber>,
	C: Authorities<B> + ChainHead<B> + HeaderBackend<B> + AuxStore + ProvideRuntimeApi,
	C::Api: ShasperApi<B>,
	B::Extrinsic: CompatibleExtrinsic,
	E: Environment<B, Error=Error>,
	E::Proposer: Proposer<B, Error=Error>,
	<<E::Proposer as Proposer<B>>::Create as IntoFuture>::Future: Send + 'static,
	I: BlockImport<B> + Send + Sync + 'static,
	Error: From<C::Error> + From<I::Error>,
	SO: SyncOracle + Send + Clone,
	P: PoolChainApi<Block=B>,
	OnExit: Future<Item=(), Error=()> + Send + 'static,
	DigestItemFor<B>: CompatibleDigestItem + DigestItem<AuthorityId=ValidatorId>,
	Error: ::std::error::Error + Send + 'static + From<::consensus_common::Error>,
{
	let worker = ShasperWorker {
		client: client.clone(),
		block_import,
		env,
		local_key,
		last_proposed_epoch: Default::default(),
		pool,
		inherent_data_providers: inherent_data_providers.clone(),
	};

	aura_slots::start_slot_worker::<_, _, _, _, ShasperSlotCompatible, _>(
		slot_duration,
		client,
		Arc::new(worker),
		sync_oracle,
		on_exit,
		inherent_data_providers
	)
}

struct ShasperWorker<C, E, I, P: PoolChainApi> {
	client: Arc<C>,
	block_import: Arc<I>,
	env: Arc<E>,
	local_key: Arc<bls::Pair>,
	last_proposed_epoch: Mutex<Epoch>,
	inherent_data_providers: InherentDataProviders,
	pool: Arc<Pool<P>>,
}

impl<B: Block<Hash=H256, Extrinsic=UncheckedExtrinsic>, C, E, I, P, Error> SlotWorker<B> for ShasperWorker<C, E, I, P> where
	C: Authorities<B> + ChainHead<B> + HeaderBackend<B> + ProvideRuntimeApi,
	C::Api: ShasperApi<B>,
	NumberFor<B>: From<BlockNumber>,
	E: Environment<B, Error=Error>,
	E::Proposer: Proposer<B, Error=Error>,
	<<E::Proposer as Proposer<B>>::Create as IntoFuture>::Future: Send + 'static,
	I: BlockImport<B> + Send + Sync + 'static,
	P: PoolChainApi<Block=B>,
	Error: From<C::Error> + From<I::Error>,
	DigestItemFor<B>: CompatibleDigestItem + DigestItem<AuthorityId=ValidatorId>,
	Error: ::std::error::Error + Send + 'static + From<::consensus_common::Error>,
{
	type OnSlot = Box<Future<Item=(), Error=consensus_common::Error> + Send>;

	fn on_start(
		&self,
		slot_duration: u64
	) -> Result<(), consensus_common::Error> {
		register_shasper_inherent_data_provider(&self.inherent_data_providers, slot_duration)?;

		let chain_head_hash = self.client.best_block_header().map_err(client_to_common_error)?.hash();
		let current_epoch = runtime::utils::slot_to_epoch(
			self.client.runtime_api().slot(&BlockId::Hash(chain_head_hash)).map_err(client_to_common_error)? - 1
		);
		*self.last_proposed_epoch.lock() = current_epoch;

		Ok(())
	}

	fn on_slot(
		&self,
		chain_head: B::Header,
		slot_info: SlotInfo,
	) -> Self::OnSlot {
		let public_key = self.local_key.public.clone();
		let (timestamp, slot_num, slot_duration) =
			(slot_info.timestamp, slot_info.number, slot_info.duration);

		let authorities = match self.client.authorities(&BlockId::Hash(chain_head.hash())) {
			Ok(authorities) => authorities,
			Err(e) => {
				warn!("Unable to fetch authorities at\
					   block {:?}: {:?}", chain_head.hash(), e);
				return Box::new(future::ok(()));
			}
		};

		let chain_head = match self.client.best_block_header() {
			Ok(header) => header,
			Err(_) => {
				warn!("Unable to fetch chain head");
				return Box::new(future::ok(()));
			}
		};
		let current_slot = match self.client.runtime_api().slot(&BlockId::Hash(chain_head.hash())) {
			Ok(slot) => slot - 1,
			Err(_) => {
				warn!("Unable to get current slot");
				return Box::new(future::ok(()));
			},
		};
		let current_epoch = runtime::utils::slot_to_epoch(current_slot);

		if *self.last_proposed_epoch.lock() < current_epoch {
			debug!(target: "shasper", "Last proposed epoch {} is less than current epoch {}, submitting a new attestation", *self.last_proposed_epoch.lock(), current_epoch);
			let validator_id = ValidatorId::from_public(public_key.clone());
			let validator_index = match self.client.runtime_api().validator_index(&BlockId::Hash(chain_head.hash()), validator_id) {
				Ok(validator_index) => validator_index,
				Err(_) => {
					warn!("Fetching validator index failed");
					return Box::new(future::ok(()));
				},
			};

			if let Some(validator_index) = validator_index {
				let justified_epoch = match self.client.runtime_api().justified_epoch(&BlockId::Hash(chain_head.hash())) {
					Ok(v) => v,
					Err(_) => {
						warn!("Fetching justified epoch failed");
						return Box::new(future::ok(()));
					},
				};
				let justified_header = match self.client.header(BlockId::Number(runtime::utils::epoch_to_slot(justified_epoch).into())) {
					Ok(Some(v)) => v,
					Err(_) | Ok(None) => {
						warn!("Fetching justified header failed");
						return Box::new(future::ok(()));
					},
				};
				let target_header = match self.client.header(BlockId::Number(runtime::utils::epoch_to_slot(current_epoch).into())) {
					Ok(Some(v)) => v,
					Err(_) | Ok(None) => {
						warn!("Fetching current header failed");
						return Box::new(future::ok(()));
					},
				};

				let unsigned = UnsignedAttestation {
					slot: current_slot,
					slot_block_hash: chain_head.hash(),
					source_epoch: justified_epoch,
					source_epoch_block_hash: justified_header.hash(),
					target_epoch: current_epoch,
					target_epoch_block_hash: target_header.hash(),
					validator_index,
				};
				let signed = unsigned.sign_with(&self.local_key.secret);

				if self.pool.submit_one(&BlockId::Hash(chain_head.hash()), UncheckedExtrinsic::Attestation(signed)).is_err() {
					warn!("Submitting attestation failed");
					return Box::new(future::ok(()));
				}

				*self.last_proposed_epoch.lock() = current_epoch;
			} else {
				debug!(target: "shasper", "Given public key {} is not in the validator set", validator_id);
			}
		}

		let proposal_work = match utils::slot_author(slot_num, &authorities) {
			None => return Box::new(future::ok(())),
			Some(author) => if author == ValidatorId::from_public(public_key.clone()) {
				debug!(target: "aura", "Starting authorship at slot {}; timestamp = {}",
					   slot_num, timestamp);

				// we are the slot author. make a block and sign it.
				let proposer = match self.env.init(&chain_head, &authorities) {
					Ok(p) => p,
					Err(e) => {
						warn!("Unable to author block in slot {:?}: {:?}", slot_num, e);
						return Box::new(future::ok(()))
					}
				};

				let remaining_duration = slot_info.remaining_duration();
				// deadline our production to approx. the end of the
				// slot
				Timeout::new(
					proposer.propose(slot_info.inherent_data, remaining_duration).into_future(),
					utils::time_until_next(Duration::from_secs(timestamp), slot_duration),
				)
			} else {
				return Box::new(future::ok(()));
			}
		};

		let block_import = self.block_import.clone();
		let pair = self.local_key.clone();
		Box::new(
			proposal_work
				.map(move |b| {
					let (header, body) = b.deconstruct();
					let pre_hash = header.hash();
					let parent_hash = header.parent_hash().clone();

					// sign the pre-sealed hash of the block and then
					// add it to a digest item.
					let to_sign = (slot_num, pre_hash).encode();
					let signature = pair.secret.sign(&to_sign[..]);
					let item = <DigestItemFor<B> as CompatibleDigestItem>::shasper_seal(
						slot_num,
						signature,
					);

					let import_block = ImportBlock {
						origin: BlockOrigin::Own,
						header,
						justification: None,
						post_digests: vec![item],
						body: Some(body),
						finalized: false,
						auxiliary: Vec::new(),
						fork_choice: ForkChoiceStrategy::LongestChain,
					};

					if let Err(e) = block_import.import_block(import_block, None) {
						warn!(target: "aura", "Error with block built on {:?}: {:?}",
							  parent_hash, e);
					}
				})
				.map_err(|e| consensus_common::ErrorKind::ClientImport(format!("{:?}", e)).into())
		)
	}
}

/// check a header has been signed by the right key. If the slot is too far in the future, an error will be returned.
/// if it's successful, returns the pre-header, the slot number, and the signat.
//
// FIXME: needs misbehavior types - https://github.com/paritytech/substrate/issues/1018
fn check_header<B: Block>(slot_now: u64, mut header: B::Header, hash: B::Hash, authorities: &[ValidatorId]) -> Result<CheckedHeader<B::Header, bls::Signature>, String>
	where DigestItemFor<B>: CompatibleDigestItem
{
	let digest_item = match header.digest_mut().pop() {
		Some(x) => x,
		None => return Err(format!("Header {:?} is unsealed", hash)),
	};
	let (slot_num, sig) = match digest_item.as_shasper_seal() {
		Some(x) => x,
		None => return Err(format!("Header {:?} is unsealed", hash)),
	};

	if slot_num > slot_now {
		header.digest_mut().push(digest_item);
		Ok(CheckedHeader::Deferred(header, slot_num))
	} else {
		// check the signature is valid under the expected authority and
		// chain state.

		let expected_author = match utils::slot_author(slot_num, &authorities) {
			None => return Err("Slot Author not found".to_string()),
			Some(author) => author
		};

		let pre_hash = header.hash();
		let to_sign = (slot_num, pre_hash).encode();
		let public = if let Some(public) = expected_author.into_public() {
			public
		} else {
			return Err("Bad public key for header author".to_string())
		};


		if public.verify(&to_sign[..], &sig) {
			Ok(CheckedHeader::Checked(header, slot_num, sig))
		} else {
			Err(format!("Bad signature on {:?}", hash))
		}
	}
}

/// A verifier for Aura blocks.
pub struct ShasperVerifier<C> {
	client: Arc<C>,
	inherent_data_providers: InherentDataProviders,
}

impl<C> ShasperVerifier<C> {
	fn check_inherents<B: Block>(
		&self,
		block: B,
		block_id: BlockId<B>,
		inherent_data: inherents::InherentData,
	) -> Result<(), String> where C: ProvideRuntimeApi, C::Api: BlockBuilderApi<B> {
		let inherent_res = self.client.runtime_api().check_inherents(
			&block_id,
			block,
			inherent_data,
		).map_err(|e| format!("{:?}", e))?;

		if !inherent_res.ok() {
			Err("Inherent data checking error".into())
		} else {
			Ok(())
		}
	}
}

impl<B: Block<Hash=H256>, C> Verifier<B> for ShasperVerifier<C> where
	C: Authorities<B> + BlockImport<B> + ChainHead<B> + HeaderBackend<B> + AuxStore + ProvideRuntimeApi + Send + Sync,
	C::Api: BlockBuilderApi<B> + ShasperApi<B>,
	B::Extrinsic: CompatibleExtrinsic,
	DigestItemFor<B>: CompatibleDigestItem + DigestItem<AuthorityId=ValidatorId>,
{
	fn verify(
		&self,
		origin: BlockOrigin,
		header: B::Header,
		justification: Option<Justification>,
		mut body: Option<Vec<B::Extrinsic>>,
	) -> Result<(ImportBlock<B>, Option<Vec<ValidatorId>>), String> {
		let mut inherent_data = self.inherent_data_providers.create_inherent_data().map_err(String::from)?;
		let slot_now = ShasperSlotCompatible::extract_timestamp_and_slot(&inherent_data)
			.map(|v| v.1)
			.map_err(|e| format!("Could not extract timestamp and slot: {:?}", e))?;
		let hash = header.hash();
		let parent_hash = *header.parent_hash();
		let authorities = self.client.authorities(&BlockId::Hash(parent_hash))
			.map_err(|e| format!("Could not fetch authorities at {:?}: {:?}", parent_hash, e))?;

		// we add one to allow for some small drift.
		// FIXME: in the future, alter this queue to allow deferring of headers
		// https://github.com/paritytech/substrate/issues/1019
		let checked_header = check_header::<B>(slot_now + 1, header, hash, &authorities[..])?;
		match checked_header {
			CheckedHeader::Checked(pre_header, slot_num, sig) => {
				let item = <DigestItemFor<B>>::shasper_seal(slot_num, sig);

				// if the body is passed through, we need to use the runtime
				// to check that the internally-set timestamp in the inherents
				// actually matches the slot set in the seal.
				if let Some(inner_body) = body.take() {
					replace_inherent_data_slot(&mut inherent_data, slot_num)
						.map_err(|e| format!("{:?}", e))?;
					let block = B::new(pre_header.clone(), inner_body);

					self.check_inherents(
						block.clone(),
						BlockId::Hash(parent_hash),
						inherent_data,
					)?;

					let (_, inner_body) = block.deconstruct();
					body = Some(inner_body);
				}

				trace!(target: "aura", "Checked {:?}; importing.", pre_header);

				let import_block = ImportBlock {
					origin,
					header: pre_header,
					post_digests: vec![item],
					body,
					finalized: false,
					justification,
					auxiliary: Vec::new(),
					fork_choice: ForkChoiceStrategy::LongestChain,
				};

				// FIXME #1019 extract authorities
				Ok((import_block, None))
			}
			CheckedHeader::Deferred(a, b) => {
				debug!(target: "aura", "Checking {:?} failed; {:?}, {:?}.", hash, a, b);
				Err(format!("Header {:?} rejected: too far in the future", hash))
			}
		}
	}
}

/// The Aura import queue type.
pub type ShasperImportQueue<B, C> = BasicQueue<B, ShasperVerifier<C>>;

/// Start an import queue for the Aura consensus algorithm.
pub fn import_queue<B, C, I>(
	slot_duration: SlotDuration,
	client: Arc<C>,
	block_import: Arc<I>,
	inherent_data_providers: InherentDataProviders,
) -> Result<ShasperImportQueue<B, C>, consensus_common::Error> where
	B: Block<Hash=H256>,
	C: Authorities<B> + BlockImport<B, Error=::consensus_common::Error> + ChainHead<B> + HeaderBackend<B> + AuxStore + ProvideRuntimeApi + Send + Sync,
	C::Api: BlockBuilderApi<B> + ShasperApi<B>,
	B::Extrinsic: CompatibleExtrinsic,
	DigestItemFor<B>: CompatibleDigestItem + DigestItem<AuthorityId=ValidatorId>,
	I: 'static + BlockImport<B, Error=ConsensusError> + Send + Sync,
{
	register_shasper_inherent_data_provider(&inherent_data_providers, slot_duration.get())?;

	let verifier = Arc::new(ShasperVerifier { client: client.clone(), inherent_data_providers });
	Ok(BasicQueue::new(verifier, block_import, None))
}
