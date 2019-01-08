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

use consensus_common::{Authorities, ImportBlock, BlockImport, ImportResult, Error as ConsensusError};
use primitives::H256;
use client::ChainHead;
use client::backend::AuxStore;
use client::blockchain::HeaderBackend;
use runtime_primitives::traits::{Block, DigestItem, DigestItemFor, ProvideRuntimeApi};
use shasper_primitives::ValidatorId;

use super::CompatibleDigestItem;

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
