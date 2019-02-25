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

use std::sync::Arc;
use std::collections::hash_map::{HashMap, Entry};

use codec::Encode;
use consensus_common::{ForkChoiceStrategy, ImportBlock, BlockImport, ImportResult, Error as ConsensusError, ErrorKind as ConsensusErrorKind};
use primitives::{H256, Blake2Hasher};
use client::{Client, CallExecutor, ChainHead};
use client::blockchain::{Backend as BlockchainBackend};
use client::backend::{Backend, AuxStore};
use runtime_primitives::generic::BlockId;
use runtime_primitives::traits::{self, Header as HeaderT, Block as BlockT, DigestItemFor, ProvideRuntimeApi, One};
use primitives::{Slot, ValidatorId};
use consensus_primitives::api::ShasperApi;
use parking_lot::Mutex;

use super::{find_slot_header, CompatibleExtrinsic, CompatibleDigestItem};

pub struct ShasperBlockImport<B, E, Block: BlockT<Hash=H256>, RA, PRA> {
	client: Arc<Client<B, E, Block, RA>>,
	api: Arc<PRA>,
	latest_attestations: Mutex<LatestAttestations<B, E, Block, RA, PRA>>,
}

impl<B, E, Block: BlockT<Hash=H256>, RA, PRA> ShasperBlockImport<B, E, Block, RA, PRA> where
	B: Backend<Block, Blake2Hasher>,
	E: CallExecutor<Block, Blake2Hasher> + Clone + Send + Sync,
	Block::Extrinsic: CompatibleExtrinsic,
	RA: Send + Sync,
	PRA: ProvideRuntimeApi,
	PRA::Api: ShasperApi<Block>,
	DigestItemFor<Block>: CompatibleDigestItem + traits::DigestItem<AuthorityId=ValidatorId>,
{
	pub fn new(client: Arc<Client<B, E, Block, RA>>, api: Arc<PRA>) -> ::client::error::Result<Self> {
		let latest_attestations = Mutex::new(LatestAttestations::get_or_default(client.clone(), api.clone())?);
		Ok(Self {
			client, latest_attestations, api,
		})
	}
}

impl<B, E, Block: BlockT<Hash=H256>, RA, PRA> BlockImport<Block> for ShasperBlockImport<B, E, Block, RA, PRA> where
	B: Backend<Block, Blake2Hasher>,
	E: CallExecutor<Block, Blake2Hasher> + Clone + Send + Sync,
	Block::Extrinsic: CompatibleExtrinsic,
	RA: Send + Sync,
	PRA: ProvideRuntimeApi,
	PRA::Api: ShasperApi<Block>,
	DigestItemFor<Block>: CompatibleDigestItem + traits::DigestItem<AuthorityId=ValidatorId>,
{
	type Error = ConsensusError;

	fn check_block(&self, hash: Block::Hash, parent_hash: Block::Hash) -> Result<ImportResult, Self::Error> {
		self.client.check_block(hash, parent_hash)
	}

	fn import_block(&self, mut block: ImportBlock<Block>, new_authorities: Option<Vec<ValidatorId>>)
		-> Result<ImportResult, Self::Error>
	{
		let parent_hash = block.header.parent_hash().clone();

		let mut latest_attestations = self.latest_attestations.lock();
		latest_attestations.note_block(&BlockId::Hash(parent_hash), block.body.as_ref().map(|a| &a[..]));

		let best_header = self.client.best_block_header().map_err::<ConsensusError, _>(|e| ConsensusErrorKind::ClientImport(e.to_string()).into())?;
		let is_new_best = latest_attestations.is_new_best(&BlockId::Hash(best_header.hash()), &BlockId::Hash(parent_hash)).map_err::<ConsensusError, _>(|e| ConsensusErrorKind::ClientImport(e.to_string()).into())?;
		block.fork_choice = ForkChoiceStrategy::Custom(is_new_best);

		match self.client.import_block(block, new_authorities) {
			Ok(result) => {
				latest_attestations.save().map_err::<ConsensusError, _>(|e| ConsensusErrorKind::ClientImport(e.to_string()).into())?;
				Ok(result)
			},
			Err(e) => {
				*latest_attestations = LatestAttestations::get_or_default(self.client.clone(), self.api.clone()).map_err::<ConsensusError, _>(|e| ConsensusErrorKind::ClientImport(e.to_string()).into())?;
				Err(e)
			},
		}
	}
}

pub struct LatestAttestations<B, E, Block: BlockT<Hash=H256>, RA, PRA> {
	data: HashMap<ValidatorId, (Slot, H256)>,
	client: Arc<Client<B, E, Block, RA>>,
	api: Arc<PRA>,
}

const LATEST_ATTESTATIONS_SLOT_KEY: &[u8] = b"lmd_latest_attestations";

impl<B, E, Block: BlockT<Hash=H256>, RA, PRA> LatestAttestations<B, E, Block, RA, PRA> where
	B: Backend<Block, Blake2Hasher>,
	E: CallExecutor<Block, Blake2Hasher> + Clone + Send + Sync,
	Block::Extrinsic: CompatibleExtrinsic,
	RA: Send + Sync,
	PRA: ProvideRuntimeApi,
	PRA::Api: ShasperApi<Block>,
	DigestItemFor<Block>: CompatibleDigestItem + traits::DigestItem<AuthorityId=ValidatorId>,
{
	pub fn get_or_default(client: Arc<Client<B, E, Block, RA>>, api: Arc<PRA>) -> ::client::error::Result<Self> {
		use codec::Decode;

		match client.get_aux(LATEST_ATTESTATIONS_SLOT_KEY)? {
			Some(v) => Vec::<(ValidatorId, (Slot, H256))>::decode(&mut &v[..])
				.map(|v| LatestAttestations {
					data: v.into_iter().collect(),
					client,
					api,
				})
				.ok_or_else(|| ::client::error::ErrorKind::Backend(
					format!("Shasper latest attestations kept in invalid format"),
				).into()),
			None => Ok(LatestAttestations {
				data: Default::default(),
				client,
				api,
			}),
		}
	}

	pub fn save(&self) -> ::client::error::Result<()> {
		self.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>().using_encoded(|s| {
			self.client.insert_aux(&[(LATEST_ATTESTATIONS_SLOT_KEY, &s[..])], &[])
		})
	}

	pub fn note_validator_attestation(&mut self, validator_id: ValidatorId, block_slot: Slot, block_hash: H256) {
		let entry = self.data.entry(validator_id);
		match entry {
			Entry::Occupied(mut entry) => {
				if entry.get().0 < block_slot {
					entry.insert((block_slot, block_hash));
				}
			},
			Entry::Vacant(entry) => {
				entry.insert((block_slot, block_hash));
			},
		}
	}

	pub fn note_block(&mut self, parent_id: &BlockId<Block>, body: Option<&[Block::Extrinsic]>) {
		if let Some(extrinsics) = body {
			for extrinsic in extrinsics {
				let map = extrinsic.as_validator_attestation_map(self.api.as_ref(), parent_id).unwrap_or_default();

				for (validator_id, (block_slot, block_hash)) in map {
					self.note_validator_attestation(validator_id, block_slot, block_hash);
				}
			}
		}
	}

	pub fn is_new_best(&self, current: &BlockId<Block>, new_parent: &BlockId<Block>) -> ::client::error::Result<bool> {
		let leaves = self.client.leaves()?;
		let leaves_with_justified_slots: Vec<(Block::Hash, Slot)> = leaves
			.clone()
			.into_iter()
			.map(|leaf| {
				self.api.runtime_api().justified_slot(&BlockId::Hash(leaf)).map(|slot| (leaf, slot))
			})
			.collect::<Result<_, _>>()?;

		let highest_justified_leaf_and_slot = leaves_with_justified_slots.iter().max_by_key(|(_, slot)| slot).cloned();
		let highest_justified_hash = highest_justified_leaf_and_slot
			.map(|(hleaf, hslot)| {
				let header = find_slot_header(
					self.client.as_ref(), self.api.as_ref(),
					hslot,
					&BlockId::Hash(hleaf)
				)?.expect("Highest justified header must exist; qed");

				Ok(header.hash())
			})
			.map_or(Ok(None), |v: ::client::error::Result<Block::Hash>| v.map(Some))?;

		debug!(target: "shasper", "Highest justified slot: {:?}, hash: {:?}", highest_justified_leaf_and_slot.map(|v| v.1), highest_justified_hash);

		let chain_head_hash = self.client.best_block_header()?.hash();
		let last_finalized_slot = self.api.runtime_api().finalized_slot(&BlockId::Hash(chain_head_hash))?;
		let last_finalized_hash = find_slot_header(
			self.client.as_ref(), self.api.as_ref(),
			last_finalized_slot,
			current
		)?.expect("Last justified header must exist; qed").hash();

		if self.client.backend().blockchain().last_finalized()? != last_finalized_hash {
			self.client.finalize_block(BlockId::Hash(last_finalized_hash), None, true)?;
		}

		debug!(target: "shasper", "Last finalized slot: {}, hash: {:?}", last_finalized_slot, last_finalized_hash);

		let start_block_hash = highest_justified_hash.unwrap_or(last_finalized_hash);

		let current_route = ::client::blockchain::tree_route(self.client.as_ref(), BlockId::Hash(start_block_hash), *current)?;
		let new_route = ::client::blockchain::tree_route(self.client.as_ref(), BlockId::Hash(start_block_hash), *new_parent)?;

		let mut current_score = 0;
		let mut new_score = 0;

		for (_, block_hash) in self.data.values() {
			if current_route.enacted().iter().any(|entry| entry.hash == *block_hash) {
				current_score += 1;
			}
			if new_route.enacted().iter().any(|entry| entry.hash == *block_hash) {
				new_score += 1;
			}
		}

		let current_height = *self.client.header(current)?
			.expect("Chain head header must exist; qed").number();
		let new_height = *self.client.header(new_parent)?
			.expect("New parent must exist; qed").number() + One::one();

		Ok(if current_score > new_score {
			false
		} else if current_score < new_score {
			true
		} else {
			new_height > current_height
		})
	}
}
