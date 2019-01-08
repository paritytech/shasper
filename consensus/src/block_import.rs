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

use consensus_common::{Authorities, ImportBlock, BlockImport, ImportResult, Error as ConsensusError};
use primitives::H256;
use client::ChainHead;
use client::backend::AuxStore;
use client::blockchain::HeaderBackend;
use runtime_primitives::generic::BlockId;
use runtime_primitives::traits::{Block, DigestItem, DigestItemFor, ProvideRuntimeApi, Header, One};
use shasper_primitives::{Slot, ValidatorId};
use aura_primitives::api::AuraApi;
use codec::Encode;

use super::{CompatibleExtrinsic, CompatibleDigestItem};

pub struct ShasperBlockImport<C> {
	client: Arc<C>,
}

impl<C> ShasperBlockImport<C> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client
		}
	}
}

impl<B: Block<Hash=H256>, C> BlockImport<B> for ShasperBlockImport<C> where
	C: Authorities<B> + BlockImport<B, Error=ConsensusError> + ChainHead<B> + HeaderBackend<B> + AuxStore + ProvideRuntimeApi + Send + Sync,
	DigestItemFor<B>: CompatibleDigestItem + DigestItem<AuthorityId=ValidatorId>,
{
	type Error = ConsensusError;

	fn import_block(&self, block: ImportBlock<B>, new_authorities: Option<Vec<ValidatorId>>)
		-> Result<ImportResult, Self::Error>
	{
		self.client.import_block(block, new_authorities)
	}
}


#[derive(Clone, Debug, Default)]
pub struct LatestAttestations(HashMap<ValidatorId, (Slot, H256)>);

const LATEST_ATTESTATIONS_SLOT_KEY: &[u8] = b"lmd_latest_attestations";

impl LatestAttestations {
	pub fn get_or_default<B: Block, C>(client: &C) -> ::client::error::Result<Self> where
		C: ::client::backend::AuxStore
	{
		use codec::Decode;

		match client.get_aux(LATEST_ATTESTATIONS_SLOT_KEY)? {
			Some(v) => Vec::<(ValidatorId, (Slot, H256))>::decode(&mut &v[..])
				.map(|v| LatestAttestations(v.into_iter().collect()))
				.ok_or_else(|| ::client::error::ErrorKind::Backend(
					format!("Aura slot duration kept in invalid format"),
				).into()),
			None => Ok(Default::default()),
		}
	}

	pub fn save<B: Block, C>(&self, client: &C) -> ::client::error::Result<()> where
		C: ::client::backend::AuxStore
	{
		self.0.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>().using_encoded(|s| {
			client.insert_aux(&[(LATEST_ATTESTATIONS_SLOT_KEY, &s[..])], &[])
		})
	}

	pub fn note_validator_attestation(&mut self, validator_id: ValidatorId, block_slot: Slot, block_hash: H256) {
		let entry = self.0.entry(validator_id);
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

	pub fn note_block<B: Block<Hash=H256>, C>(&mut self, client: &C, parent_id: &BlockId<B>, body: Option<&[B::Extrinsic]>) where
		C: ProvideRuntimeApi,
		C::Api: AuraApi<B>,
		B::Extrinsic: CompatibleExtrinsic,
	{
		if let Some(extrinsics) = body {
			for extrinsic in extrinsics {
				let map = extrinsic.as_validator_attestation_map(client, parent_id).unwrap_or_default();

				for (validator_id, (block_slot, block_hash)) in map {
					self.note_validator_attestation(validator_id, block_slot, block_hash);
				}
			}
		}
	}

	pub fn is_new_best<B: Block<Hash=H256>, C>(&self, client: &C, current: &BlockId<B>, new_parent: &BlockId<B>) -> ::client::error::Result<bool> where
		C: ChainHead<B> + HeaderBackend<B> + ProvideRuntimeApi,
		C::Api: AuraApi<B>,
	{
		let leaves = client.leaves()?;
		let leaves_with_justified_slots: Vec<(B::Hash, Slot)> = leaves
			.clone()
			.into_iter()
			.map(|leaf| {
				client.runtime_api().last_justified_slot(&BlockId::Hash(leaf)).map(|slot| (leaf, slot))
			})
			.collect::<Result<_, _>>()?;

		let highest_justified_leaf_and_slot = leaves_with_justified_slots.iter().max_by_key(|(_, slot)| slot).cloned();
		let highest_justified_hash = highest_justified_leaf_and_slot
			.map(|(hleaf, hslot)| {
				let mut header = client.header(BlockId::Hash(hleaf))?
					.expect("Leaf header must exist; qed");
				let mut slot = client.runtime_api().slot(&BlockId::Hash(hleaf))?;

				while slot > hslot {
					header = client.header(BlockId::Hash(*header.parent_hash()))?
						.expect("Leaf's parent must exist; qed");
					slot = client.runtime_api().slot(&BlockId::Hash(header.hash()))?;
				}

				Ok(header.hash())
			})
			.map_or(Ok(None), |v: ::client::error::Result<B::Hash>| v.map(Some))?;

		let chain_head_hash = client.best_block_header()?.hash();
		let last_finalized_slot = client.runtime_api().last_finalized_slot(&BlockId::Hash(chain_head_hash))?;
		let last_finalized_hash = {
			let mut header = client.header(*current)?
				.expect("Chain head header must exist; qed");
			let mut slot = client.runtime_api().slot(&BlockId::Hash(header.hash()))?;

			while slot > last_finalized_slot {
				header = client.header(BlockId::Hash(*header.parent_hash()))?
					.expect("Chain head's parent must exist; qed");
				slot = client.runtime_api().slot(&BlockId::Hash(header.hash()))?;
			}

			header.hash()
		};

		let start_block_hash = highest_justified_hash.unwrap_or(last_finalized_hash);

		let current_route = ::client::blockchain::tree_route(client, BlockId::Hash(start_block_hash), *current)?;
		let new_route = ::client::blockchain::tree_route(client, BlockId::Hash(start_block_hash), *new_parent)?;

		let mut current_score = 0;
		let mut new_score = 0;

		for (_, block_hash) in self.0.values() {
			if current_route.enacted().iter().any(|entry| entry.hash == *block_hash) {
				current_score += 1;
			}
			if new_route.enacted().iter().any(|entry| entry.hash == *block_hash) {
				new_score += 1;
			}
		}

		let current_height = *client.header(*current)?
			.expect("Chain head header must exist; qed").number();
		let new_height = *client.header(*new_parent)?
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
