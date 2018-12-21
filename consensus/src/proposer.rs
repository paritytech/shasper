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

use std::sync::Arc;
use std::time;

use AuraConsensusData;
use service::consensus::AuthoringApi;
use client::{self, error};
use client::{block_builder::api::BlockBuilder as BlockBuilderApi};
use codec::{Decode, Encode};
use consensus_common::{self, evaluation};
use runtime_primitives::traits::{Block as BlockT, Hash as HashT, Header as HeaderT, ProvideRuntimeApi, AuthorityIdFor};
use runtime_primitives::generic::BlockId;
use runtime_primitives::BasicInherentData;
use transaction_pool::txpool::{self, Pool as TransactionPool};

type Timestamp = u64;

// block size limit.
const MAX_TRANSACTIONS_SIZE: usize = 4 * 1024 * 1024;

/// Proposer factory.
pub struct ProposerFactory<C, A> where A: txpool::ChainApi {
	/// The client instance.
	pub client: Arc<C>,
	/// The transaction pool.
	pub transaction_pool: Arc<TransactionPool<A>>,
}

impl<C, A, ConsensusData> consensus_common::Environment<<C as AuthoringApi>::Block, ConsensusData> for ProposerFactory<C, A> where
	C: AuthoringApi,
	<C as ProvideRuntimeApi>::Api: BlockBuilderApi<<C as AuthoringApi>::Block, BasicInherentData>,
	A: txpool::ChainApi<Block=<C as AuthoringApi>::Block>,
	client::error::Error: From<<C as AuthoringApi>::Error>,
	Proposer<<C as AuthoringApi>::Block, C, A>: consensus_common::Proposer<<C as AuthoringApi>::Block, ConsensusData>,
{
	type Proposer = Proposer<<C as AuthoringApi>::Block, C, A>;
	type Error = error::Error;

	fn init(
		&self,
		parent_header: &<<C as AuthoringApi>::Block as BlockT>::Header,
		_: &[AuthorityIdFor<<C as AuthoringApi>::Block>],
	) -> Result<Self::Proposer, error::Error> {
		let parent_hash = parent_header.hash();

		let id = BlockId::hash(parent_hash);

		info!("Starting consensus session on top of parent {:?}", parent_hash);

		let proposer = Proposer {
			client: self.client.clone(),
			parent_hash,
			parent_id: id,
			parent_number: *parent_header.number(),
			transaction_pool: self.transaction_pool.clone(),
		};

		Ok(proposer)
	}
}

struct ConsensusData {
	timestamp: Option<u64>,
}

/// The proposer logic.
pub struct Proposer<Block: BlockT, C, A: txpool::ChainApi> {
	client: Arc<C>,
	parent_hash: <Block as BlockT>::Hash,
	parent_id: BlockId<Block>,
	parent_number: <<Block as BlockT>::Header as HeaderT>::Number,
	transaction_pool: Arc<TransactionPool<A>>,
}

impl<Block, C, A> consensus_common::Proposer<<C as AuthoringApi>::Block, AuraConsensusData> for Proposer<Block, C, A> where
	Block: BlockT,
	C: AuthoringApi<Block=Block>,
	<C as ProvideRuntimeApi>::Api: BlockBuilderApi<Block, BasicInherentData>,
	A: txpool::ChainApi<Block=Block>,
	client::error::Error: From<<C as AuthoringApi>::Error>
{
	type Create = Result<<C as AuthoringApi>::Block, error::Error>;
	type Error = error::Error;

	fn propose(&self, consensus_data: AuraConsensusData)
		-> Result<<C as AuthoringApi>::Block, error::Error>
	{
		self.propose_with(ConsensusData { timestamp: Some(consensus_data.timestamp) })
	}
}

impl<Block, C, A> consensus_common::Proposer<<C as AuthoringApi>::Block, ()> for Proposer<Block, C, A> where
	Block: BlockT,
	C: AuthoringApi<Block=Block>,
	<C as ProvideRuntimeApi>::Api: BlockBuilderApi<Block, BasicInherentData>,
	A: txpool::ChainApi<Block=Block>,
	client::error::Error: From<<C as AuthoringApi>::Error>
{
	type Create = Result<<C as AuthoringApi>::Block, error::Error>;
	type Error = error::Error;

	fn propose(&self, _consensus_data: ()) -> Result<<C as AuthoringApi>::Block, error::Error> {
		self.propose_with(ConsensusData { timestamp: None })
	}
}

impl<Block, C, A> Proposer<Block, C, A>	where
	Block: BlockT,
	C: AuthoringApi<Block=Block>,
	<C as ProvideRuntimeApi>::Api: BlockBuilderApi<Block, BasicInherentData>,
	A: txpool::ChainApi<Block=Block>,
	client::error::Error: From<<C as AuthoringApi>::Error>,
{
	fn propose_with(&self, consensus_data: ConsensusData)
		-> Result<<C as AuthoringApi>::Block, error::Error>
	{
		use runtime_primitives::traits::BlakeTwo256;

		let timestamp = consensus_data.timestamp.unwrap_or_else(current_timestamp);
		let inherent_data = BasicInherentData::new(timestamp, 0);

		let block = self.client.build_block(
			&self.parent_id,
			inherent_data,
			|block_builder| {
				let mut unqueue_invalid = Vec::new();
				let mut pending_size = 0;
				let pending_iterator = self.transaction_pool.ready();

				for pending in pending_iterator {
					// TODO [ToDr] Probably get rid of it, and validate in runtime.
					let encoded_size = pending.data.encode().len();
					if pending_size + encoded_size >= MAX_TRANSACTIONS_SIZE { break }

					match block_builder.push_extrinsic(pending.data.clone()) {
						Ok(()) => {
							pending_size += encoded_size;
						}
						Err(e) => {
							trace!(target: "transaction-pool", "Invalid transaction: {}", e);
							unqueue_invalid.push(pending.hash.clone());
						}
					}
				}

				self.transaction_pool.remove_invalid(&unqueue_invalid);
			})?;

		info!("Proposing block [number: {}; hash: {}; parent_hash: {}; extrinsics: [{}]]",
			  block.header().number(),
			  <<C as AuthoringApi>::Block as BlockT>::Hash::from(block.header().hash()),
			  block.header().parent_hash(),
			  block.extrinsics().iter()
			  .map(|xt| format!("{}", BlakeTwo256::hash_of(xt)))
			  .collect::<Vec<_>>()
			  .join(", ")
			 );

		let substrate_block = Decode::decode(&mut block.encode().as_slice())
			.expect("blocks are defined to serialize to substrate blocks correctly; qed");

		assert!(evaluation::evaluate_initial(
			&substrate_block,
			&self.parent_hash,
			self.parent_number,
		).is_ok());

		Ok(substrate_block)
	}
}

fn current_timestamp() -> Timestamp {
	time::SystemTime::now().duration_since(time::UNIX_EPOCH)
		.expect("now always later than unix epoch; qed")
		.as_secs()
}
