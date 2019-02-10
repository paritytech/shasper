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

//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
mod service;
mod cli;

pub use ::cli::{VersionInfo, IntoExit, error};

use error_chain::quick_main;

fn run() -> cli::error::Result<()> {
	let version = VersionInfo {
		name: "Substrate Shasper Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "shasper",
		author: "Parity Technologies <admin@parity.io>",
		description: "Substrate Shasper",
		support_url: "https://github.com/paritytech/shasper",
	};
	cli::run(::std::env::args(), cli::Exit, version)
}

quick_main!(run);
