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

pub use cli::{VersionInfo, IntoExit, error};

use service;
use futures::{future, Future, sync::oneshot};
use std::cell::RefCell;
use tokio::runtime::Runtime;
use service::{ServiceFactory, Roles as ServiceRoles};
use std::ops::Deref;
use cli::{NoCustom, impl_augment_clap};
use log::info;
use client::ExecutionStrategies;
use state_machine::ExecutionStrategy;
use structopt::StructOpt;
use crypto::bls;
use crate::chain_spec;

/// Node specific parameters
#[derive(Debug, Clone, StructOpt)]
pub struct NodeParams {
	/// Validator key for Shasper.
	#[structopt(short = "k", long = "validator-key")]
	validator_key: Option<String>,
}

impl_augment_clap!(NodeParams);

/// Parse command line arguments into service configuration.
pub fn run<I, T, E>(args: I, exit: E, version: VersionInfo) -> error::Result<()> where
	I: IntoIterator<Item = T>,
	T: Into<std::ffi::OsString> + Clone,
	E: IntoExit,
{
	cli::parse_and_execute::<crate::service::Factory, NoCustom, NodeParams, _, _, _, _, _>(
		load_spec, &version, "shasper-node", args, exit,
		|exit, custom_args, mut config| {
			info!("{}", version.name);
			info!("  version {}", config.full_version());
			info!("  by Parity Technologies, 2017-2019");
			info!("Chain specification: {}", config.chain_spec.name());
			info!("Node name: {}", config.name);
			info!("Roles: {:?}", config.roles);

			config.custom.validator_key = custom_args.validator_key.map(|k| {
				bls::Secret::from_bytes(k.as_bytes()).expect("Validator key provided is invalid")
			});

			// Make this a native-only runtime.
			config.execution_strategies = ExecutionStrategies {
				syncing: ExecutionStrategy::NativeWhenPossible,
				importing: ExecutionStrategy::NativeWhenPossible,
				block_construction: ExecutionStrategy::NativeWhenPossible,
				other: ExecutionStrategy::NativeWhenPossible,
			};

			let runtime = Runtime::new().map_err(|e| format!("{:?}", e))?;
			let executor = runtime.executor();
			match config.roles {
				ServiceRoles::LIGHT => run_until_exit(
					runtime,
					crate::service::Factory::new_light(config, executor).map_err(|e| format!("{:?}", e))?,
					exit
				),
				_ => run_until_exit(
					runtime,
					crate::service::Factory::new_full(config, executor).map_err(|e| format!("{:?}", e))?,
					exit
				),
			}.map_err(|e| format!("{:?}", e))
		}
	).map_err(Into::into).map(|_| ())
}

fn load_spec(id: &str) -> Result<Option<chain_spec::ChainSpec>, String> {
	Ok(match chain_spec::Alternative::from(id) {
		Some(spec) => Some(spec.load()?),
		None => None,
	})
}

fn run_until_exit<T, C, E>(
	mut runtime: Runtime,
	service: T,
	e: E,
) -> error::Result<()>
	where
		T: Deref<Target=service::Service<C>>,
		C: service::Components,
		E: IntoExit,
{
	let (exit_send, exit) = exit_future::signal();

	let executor = runtime.executor();
	cli::informant::start(&service, exit.clone(), executor.clone());

	let _ = runtime.block_on(e.into_exit());
	exit_send.fire();

	// we eagerly drop the service so that the internal exit future is fired,
	// but we need to keep holding a reference to the global telemetry guard
	let _telemetry = service.telemetry();
	drop(service);

	// TODO [andre]: timeout this future #1318
	let _ = runtime.shutdown_on_idle().wait();

	Ok(())
}

// handles ctrl-c
pub struct Exit;
impl IntoExit for Exit {
	type Exit = future::MapErr<oneshot::Receiver<()>, fn(oneshot::Canceled) -> ()>;
	fn into_exit(self) -> Self::Exit {
		// can't use signal directly here because CtrlC takes only `Fn`.
		let (exit_send, exit) = oneshot::channel();

		let exit_send_cell = RefCell::new(Some(exit_send));
		ctrlc::set_handler(move || {
			if let Some(exit_send) = exit_send_cell.try_borrow_mut().expect("signal handler not reentrant; qed").take() {
				exit_send.send(()).expect("Error sending exit notification");
			}
		}).expect("Error setting Ctrl-C handler");

		exit.map_err(drop)
	}
}
