extern crate shasper_runtime as runtime;
extern crate shasper_network as network;
extern crate shasper_executor as executor;
extern crate shasper_transaction_pool as transaction_pool;
extern crate sr_primitives as runtime_primitives;
extern crate substrate_primitives as primitives;
extern crate substrate_client as client;
extern crate substrate_service as service;
extern crate parity_codec as codec;
extern crate tokio;

pub mod chain_spec;

use runtime::Block;
use network::Protocol;
use primitives::H256;
use transaction_pool::TransactionPool;
use runtime_primitives::StorageMap;
use tokio::runtime::TaskExecutor;

use std::sync::Arc;

pub type ChainSpec = service::ChainSpec<StorageMap>;

/// All configuration for the node.
pub type Configuration = service::FactoryFullConfiguration<Factory>;
pub use service::{
	Roles, PruningMode, TransactionPoolOptions, ServiceFactory,
	ErrorKind, Error, ComponentBlock, LightComponents, FullComponents, Components};

pub struct Factory;

impl service::ServiceFactory for Factory {
	type Block = Block;
	type ExtrinsicHash = H256;
	type NetworkProtocol = Protocol;
	type RuntimeDispatch = executor::Executor;
	type FullTransactionPoolApi = transaction_pool::ChainApi;
	type LightTransactionPoolApi = transaction_pool::ChainApi;
	type Genesis = StorageMap;
	type Configuration = ();
	type FullService = Service<service::FullComponents<Self>>;
	type LightService = Service<service::LightComponents<Self>>;

	fn build_full_transaction_pool(config: TransactionPoolOptions, _client: Arc<service::FullClient<Self>>)
		-> Result<TransactionPool, service::Error>
	{
		Ok(TransactionPool::new(config, transaction_pool::ChainApi))
	}

	fn build_light_transaction_pool(config: TransactionPoolOptions, _client: Arc<service::LightClient<Self>>)
		-> Result<TransactionPool, service::Error>
	{
		Ok(TransactionPool::new(config, transaction_pool::ChainApi))
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
}

pub struct Service<C: service::Components>(service::Service<C>);

impl<C: service::Components> ::std::ops::Deref for Service<C> {
	type Target = service::Service<C>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
