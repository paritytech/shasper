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

//! Service and ServiceFactory implementation. Specialized wrapper over Substrate service.

#![warn(unused_extern_crates)]

use std::sync::Arc;
use primitives::ed25519;
use runtime_primitives::BasicInherentData;
use transaction_pool::{self, txpool::{Pool as TransactionPool}};
use shasper_primitives::ValidatorId;
use shasper_runtime::{self, GenesisConfig, RuntimeApi, Block};
use substrate_service::{
	FactoryFullConfiguration, LightComponents, FullComponents, FullBackend,
	FullClient, LightClient, LightBackend, FullExecutor, LightExecutor,
	TaskExecutor,
};
use consensus::{import_queue, start_aura, AuraImportQueue, NothingExtra, SlotDuration, LatestAttestations, ShasperBlockImport};
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
			validator_key: Some(bls::Secret::from_bytes(b"Alice").unwrap())
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
					info!("Using authority key {}", ValidatorId::from_public(bls::Public::from_secret(key)));
					let proposer = Arc::new(consensus::ProposerFactory {
						client: service.client(),
						transaction_pool: service.transaction_pool(),
					});

					let client = service.client();
					executor.spawn(start_aura(
						SlotDuration::get_or_compute(&*client)?,
						LatestAttestations::get_or_default::<Block, _>(&*client)?,
						Arc::new(
							bls::Pair::from_secret(key.clone())
						),
						client.clone(),
						Arc::new(ShasperBlockImport::new(client)),
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
					LatestAttestations::get_or_default::<Block, _>(&*client)?,
					client.clone(),
					Arc::new(ShasperBlockImport::new(client)),
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
					LatestAttestations::get_or_default::<Block, _>(&*client)?,
					client.clone(),
					Arc::new(ShasperBlockImport::new(client)),
					NothingExtra,
					::consensus::make_basic_inherent as _,
				))
			},
	}
}
