//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

extern crate futures;
#[macro_use]
extern crate error_chain;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate sr_primitives as runtime_primitives;
extern crate substrate_cli;
extern crate substrate_primitives as primitives;
extern crate shasper_consensus as consensus;
extern crate substrate_client as client;
#[macro_use]
extern crate substrate_network as network;
#[macro_use]
extern crate substrate_executor;
extern crate substrate_transaction_pool as transaction_pool;
#[macro_use]
extern crate substrate_service;
extern crate shasper_runtime;
extern crate shasper_primitives;
extern crate structopt;

mod chain_spec;
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn run() -> cli::error::Result<()> {
	let version = VersionInfo {
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "shasper",
		author: "Parity Technologies <admin@parity.io>",
		description: "Substrate Shasper",
	};
	cli::run(::std::env::args(), cli::Exit, version)
}

quick_main!(run);
