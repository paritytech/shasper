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

extern crate shasper_runtime as runtime;
extern crate shasper_network as network;
extern crate shasper_executor as executor;
extern crate shasper_transaction_pool as transaction_pool;
extern crate sr_primitives as runtime_primitives;
extern crate substrate_primitives as primitives;
extern crate substrate_client as client;
extern crate substrate_service as service;
extern crate substrate_consensus_common as consensus_common;
extern crate parity_codec as codec;
extern crate tokio;

pub mod chain_spec;

use consensus_common::{ImportBlock, BlockOrigin};
use runtime::{Block, Header, Extrinsic, RuntimeApi};
use network::Protocol;
use network::import_queue::{Verifier, BasicQueue};
use transaction_pool::TransactionPool;
use runtime_primitives::StorageMap;
use tokio::runtime::TaskExecutor;

use std::sync::Arc;

pub type ChainSpec = service::ChainSpec<StorageMap>;

/// All configuration for the node.
pub type Configuration = service::FactoryFullConfiguration<Factory>;
pub use service::{
	Roles, PruningMode, TransactionPoolOptions, ServiceFactory,
	ErrorKind, Error, ComponentBlock, LightComponents, FullComponents, Components,
	FullBackend, LightBackend, FullExecutor, LightExecutor,
	FactoryFullConfiguration, LightClient, FullClient,
};

pub struct NullVerifier;

impl Verifier<Block> for NullVerifier {
	fn verify(
		&self,
		origin: BlockOrigin,
		header: Header,
		_justification: Vec<u8>,
		body: Option<Vec<Extrinsic>>
	) -> Result<(ImportBlock<Block>, Option<Vec<primitives::AuthorityId>>), String> {
		Ok((
			ImportBlock {
				header, body, origin,
				external_justification: Default::default(),
				post_runtime_digests: Vec::new(),
				finalized: false,
				auxiliary: Vec::new()
			},
			None
		))
	}
}

pub type NullQueue = BasicQueue<Block, NullVerifier>;

pub struct Factory;

impl service::ServiceFactory for Factory {
	type Block = Block;
	type NetworkProtocol = Protocol;
	type RuntimeApi = RuntimeApi;
	type RuntimeDispatch = executor::Executor;
	type FullTransactionPoolApi = transaction_pool::ChainApi<client::Client<FullBackend<Self>, FullExecutor<Self>, Block, RuntimeApi>, Block>;
	type LightTransactionPoolApi = transaction_pool::ChainApi<client::Client<LightBackend<Self>, LightExecutor<Self>, Block, RuntimeApi>, Block>;
	type Genesis = StorageMap;
	type Configuration = ();
	type FullService = Service<service::FullComponents<Self>>;
	type LightService = Service<service::LightComponents<Self>>;
	type FullImportQueue = NullQueue;
	type LightImportQueue = NullQueue;

	fn build_full_transaction_pool(config: TransactionPoolOptions, client: Arc<service::FullClient<Self>>)
		-> Result<TransactionPool<client::Client<FullBackend<Self>, FullExecutor<Self>, Block, RuntimeApi>, Block>, service::Error>
	{
		Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client)))
	}

	fn build_light_transaction_pool(config: TransactionPoolOptions, client: Arc<service::LightClient<Self>>)
		-> Result<TransactionPool<client::Client<LightBackend<Self>, LightExecutor<Self>, Block, RuntimeApi>, Block>, service::Error>
	{
		Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client)))
	}

	fn build_network_protocol(_config: &Configuration)
		-> Result<Protocol, service::Error>
	{
		Ok(Protocol::new())
	}

	fn new_light(config: Configuration, executor: TaskExecutor)
		-> Result<Service<service::LightComponents<Factory>>, service::Error>
	{
		Ok(Service(service::Service::<service::LightComponents<Factory>>::new(config, executor.clone())?))
	}

	fn new_full(config: Configuration, executor: TaskExecutor)
		-> Result<Service<service::FullComponents<Factory>>, service::Error>
	{
		Ok(Service(service::Service::<service::FullComponents<Factory>>::new(config, executor.clone())?))
	}

	fn build_full_import_queue(
		_config: &FactoryFullConfiguration<Self>,
		_client: Arc<FullClient<Self>>
	) -> Result<Self::FullImportQueue, service::Error> {
		Ok(NullQueue::new(Arc::new(NullVerifier)))
	}

	fn build_light_import_queue(
		_config: &FactoryFullConfiguration<Self>,
		_client: Arc<LightClient<Self>>
	) -> Result<Self::LightImportQueue, service::Error> {
		Ok(NullQueue::new(Arc::new(NullVerifier)))
	}
}

pub struct Service<C: service::Components>(service::Service<C>);

impl<C: service::Components> ::std::ops::Deref for Service<C> {
	type Target = service::Service<C>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
