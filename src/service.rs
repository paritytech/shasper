//! Service and ServiceFactory implementation. Specialized wrapper over Substrate service.

#![warn(unused_extern_crates)]

use std::sync::Arc;
use primitives::ed25519;
use runtime_primitives::BasicInherentData;
use transaction_pool::{self, txpool::{Pool as TransactionPool}};
use shasper_primitives::{ValidatorId, opaque::Block};
use shasper_runtime::{self, GenesisConfig, RuntimeApi};
use substrate_service::{
	FactoryFullConfiguration, LightComponents, FullComponents, FullBackend,
	FullClient, LightClient, LightBackend, FullExecutor, LightExecutor,
	TaskExecutor,
};
use consensus::{import_queue, start_aura, AuraImportQueue, NothingExtra, SlotDuration};
use client;
use crypto::bls;

pub use substrate_executor::NativeExecutor;
// Our native executor instance.
native_executor_instance!(
	pub Executor,
	shasper_runtime::api::dispatch,
	shasper_runtime::native_version,
	include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/shasper_runtime.compact.wasm")
);

construct_simple_protocol! {
	/// Demo protocol attachment for substrate.
	pub struct NodeProtocol where Block = Block { }
}

/// Node specific configuration
pub struct NodeConfig {
	pub validator_key: Option<bls::Secret>,
}

impl Default for NodeConfig {
	fn default() -> Self {
		Self {
			validator_key: Some(bls::Secret::from_bytes(b"Alice                                           ").unwrap())
		}
	}
}

construct_service_factory! {
	struct Factory {
		Block = Block,
		RuntimeApi = RuntimeApi,
		NetworkProtocol = NodeProtocol { |config| Ok(NodeProtocol::new()) },
		RuntimeDispatch = Executor,
		FullTransactionPoolApi = transaction_pool::ChainApi<client::Client<FullBackend<Self>, FullExecutor<Self>, Block, RuntimeApi>, Block>
			{ |config, client| Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client))) },
		LightTransactionPoolApi = transaction_pool::ChainApi<client::Client<LightBackend<Self>, LightExecutor<Self>, Block, RuntimeApi>, Block>
			{ |config, client| Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client))) },
		Genesis = GenesisConfig,
		Configuration = NodeConfig,
		FullService = FullComponents<Self>
			{ |config: FactoryFullConfiguration<Self>, executor: TaskExecutor|
				FullComponents::<Factory>::new(config, executor) },
		AuthoritySetup = {
			|service: Self::FullService, executor: TaskExecutor, _: Option<Arc<ed25519::Pair>>| {
				if let Some(ref key) = service.config.custom.validator_key {
					info!("Using authority key {}", ValidatorId::from_public(bls::Public::from_secret_key(key)));
					let proposer = Arc::new(consensus::ProposerFactory {
						client: service.client(),
						transaction_pool: service.transaction_pool(),
					});

					let client = service.client();
					executor.spawn(start_aura(
						SlotDuration::get_or_compute(&*client)?,
						Arc::new(
							bls::Pair {
								pk: bls::Public::from_secret_key(key),
								sk: key.clone(),
							}
						),
						client.clone(),
						client,
						proposer,
						service.network(),
					));
				}

				Ok(service)
			}
		},
		LightService = LightComponents<Self>
			{ |config, executor| <LightComponents<Factory>>::new(config, executor) },
		FullImportQueue = AuraImportQueue<
			Self::Block,
			FullClient<Self>,
			NothingExtra,
			::consensus::InherentProducingFn<BasicInherentData>,
		>
			{ |config: &mut FactoryFullConfiguration<Self>, client: Arc<FullClient<Self>>| {
				Ok(import_queue(
					SlotDuration::get_or_compute(&*client)?,
					client,
					NothingExtra,
					::consensus::make_basic_inherent as _,
				))
			}},
		LightImportQueue = AuraImportQueue<
			Self::Block,
			LightClient<Self>,
			NothingExtra,
			::consensus::InherentProducingFn<BasicInherentData>,
		>
			{ |config: &mut FactoryFullConfiguration<Self>, client: Arc<LightClient<Self>>|
				Ok(import_queue(
					SlotDuration::get_or_compute(&*client)?,
					client,
					NothingExtra,
					::consensus::make_basic_inherent as _,
				))
			},
	}
}
