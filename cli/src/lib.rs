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

//! Substrate CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

extern crate tokio;

extern crate substrate_cli as cli;
extern crate substrate_service;
extern crate shasper_service as service;
extern crate exit_future;
extern crate structopt;

#[macro_use]
extern crate log;

pub use cli::error;

use std::ops::Deref;
use tokio::runtime::Runtime;
use structopt::StructOpt;

pub use service::{Components as ServiceComponents, Service, ServiceFactory};
pub use cli::{VersionInfo, IntoExit, CoreParams};

/// Extend params for Node
#[derive(Debug, StructOpt)]
pub struct NodeParams {
	#[structopt(flatten)]
	core: CoreParams
}

/// The chain specification option.
#[derive(Clone, Debug)]
pub enum ChainSpec {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,
}

/// Get a chain config from a spec setting.
impl ChainSpec {
	pub(crate) fn load(self) -> Result<service::ChainSpec, String> {
		Ok(match self {
			ChainSpec::Development => service::chain_spec::development_config(),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(ChainSpec::Development),
			_ => None,
		}
	}
}

fn load_spec(id: &str) -> Result<Option<service::ChainSpec>, String> {
	Ok(match ChainSpec::from(id) {
		Some(spec) => Some(spec.load()?),
		None => None,
	})
}

/// Parse command line arguments into service configuration.
pub fn run<I, T, E>(args: I, exit: E, version: cli::VersionInfo) -> error::Result<()> where
	I: IntoIterator<Item = T>,
	T: Into<std::ffi::OsString> + Clone,
	E: IntoExit,
{
	let matches = match NodeParams::clap()
		.name(version.executable_name)
		.author(version.author)
		.about(version.description)
		.get_matches_from_safe(args) {
			Ok(m) => m,
			Err(e) => e.exit(),
		};

	let (spec, config) = cli::parse_matches::<service::Factory, _>(
		load_spec, version, "shasper-node", &matches
	)?;

	match cli::execute_default::<service::Factory, _>(spec, exit, &matches, &config)? {
		cli::Action::ExecutedInternally => (),
		cli::Action::RunService(exit) => {
			info!("Substrate Shasper Node");
			info!("  version {}", config.full_version());
			info!("  by Parity Technologies, 2017, 2018");
			info!("Chain specification: {}", config.chain_spec.name());
			info!("Node name: {}", config.name);
			info!("Roles: {:?}", config.roles);
			let mut runtime = Runtime::new()?;
			let executor = runtime.executor();
			match config.roles == service::Roles::LIGHT {
				true => run_until_exit(&mut runtime, service::Factory::new_light(config, executor)?, exit)?,
				false => run_until_exit(&mut runtime, service::Factory::new_full(config, executor)?, exit)?,
			}
		}
	}
	Ok(())
}

fn run_until_exit<T, C, E>(
	runtime: &mut Runtime,
	service: T,
	e: E,
) -> error::Result<()>
	where
	    T: Deref<Target=substrate_service::Service<C>>,
		C: substrate_service::Components,
		E: IntoExit,
{
	let (exit_send, exit) = exit_future::signal();

	let executor = runtime.executor();
	cli::informant::start(&service, exit.clone(), executor.clone());

	let _ = runtime.block_on(e.into_exit());
	exit_send.fire();
	Ok(())
}
