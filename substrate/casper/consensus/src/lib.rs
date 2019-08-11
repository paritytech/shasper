use std::{sync::Arc, collections::HashMap};
use primitives::{H256, Blake2Hasher};
use sr_primitives::traits::{Block as BlockT, Header, ProvideRuntimeApi};
use sr_primitives::generic::BlockId;
use consensus_common::{
	BlockImport, Error as ConsensusError,
	BlockImportParams, ImportResult, well_known_cache_keys,
};
use casper_primitives::CasperApi;
use client::{Client, CallExecutor, backend::Backend, blockchain::Backend as _};
use log::warn;

pub struct CasperBlockImport<B, E, Block: BlockT<Hash=H256>, RA, PRA> {
	inner: Arc<Client<B, E, Block, RA>>,
	api: Arc<PRA>,
}

impl<B, E, Block: BlockT<Hash=H256>, RA, PRA> CasperBlockImport<B, E, Block, RA, PRA> {
	pub fn new(inner: Arc<Client<B, E, Block, RA>>, api: Arc<PRA>) -> Self {
		Self { inner, api }
	}
}

impl<B, E, Block: BlockT<Hash=H256>, RA, PRA> Clone for CasperBlockImport<B, E, Block, RA, PRA> {
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
			api: self.api.clone(),
		}
	}
}

impl<B, E, Block: BlockT<Hash=H256>, RA, PRA> BlockImport<Block> for
	CasperBlockImport<B, E, Block, RA, PRA> where
	B: Backend<Block, Blake2Hasher> + 'static,
	E: CallExecutor<Block, Blake2Hasher> + 'static + Clone + Send + Sync,
	RA: Send + Sync,
	PRA: ProvideRuntimeApi,
	PRA::Api: CasperApi<Block>,
{
	type Error = ConsensusError;

	fn import_block(
		&mut self,
		block: BlockImportParams<Block>,
		new_cache: HashMap<well_known_cache_keys::Id, Vec<u8>>
	) -> Result<ImportResult, Self::Error> {
		let at = BlockId::hash(*block.header.parent_hash());
		let result = self.inner.import_block(block, new_cache)?;

		match &result {
			ImportResult::Imported(_) => {
				let inner = || {
					let finalized_hash = match self.api.runtime_api().finalized_block(&at) {
						Ok(finalized_hash) => finalized_hash,
						Err(e) => {
							warn!("Failed to fetch the last finalized hash: {:?}", e);
							return
						},
					};
					let last_finalized = match self.inner.backend().blockchain().last_finalized() {
						Ok(finalized_hash) => finalized_hash,
						Err(e) => {
							warn!("Failed to fetch the client's finalized hash: {:?}", e);
							return
						},
					};
					if finalized_hash != last_finalized {
						match self.inner.finalize_block(BlockId::Hash(finalized_hash), None, true) {
							Ok(()) => (),
							Err(e) => {
								warn!("Block finalization failed: {:?}", e);
								return
							},
						}
					}
				};
				inner();
			},
			_ => (),
		}

		Ok(result)
	}

	fn check_block(
		&mut self,
		hash: Block::Hash,
		parent_hash: Block::Hash,
	) -> Result<ImportResult, Self::Error> {
		self.inner.check_block(hash, parent_hash)
	}
}
