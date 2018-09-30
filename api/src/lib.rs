// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Strongly typed API for Polkadot based around the locally-compiled native
//! runtime.

extern crate shasper_executor;
extern crate shasper_runtime as runtime;
extern crate parity_codec as codec;
extern crate sr_io as runtime_io;
extern crate substrate_client as client;
extern crate substrate_executor as substrate_executor;
extern crate substrate_primitives;
extern crate sr_primitives as runtime_primitives;

#[macro_use]
extern crate error_chain;

extern crate log;

pub mod full;
pub mod light;

use runtime::{InherentData, Extrinsic, Block, BlockId};

error_chain! {
	errors {
		/// Unknown runtime code.
		UnknownRuntime {
			description("Unknown runtime code")
			display("Unknown runtime code")
		}
		/// Unknown block ID.
		UnknownBlock(b: String) {
			description("Unknown block")
			display("Unknown block {}", b)
		}
		/// Execution error.
		Execution(e: String) {
			description("Execution error")
			display("Execution error: {}", e)
		}
		/// Some other error.
		// TODO: allow to be specified as associated type of PolkadotApi
		Other(e: Box<::std::error::Error + Send>) {
			description("Other error")
			display("Other error: {}", e.description())
		}
	}
}

impl From<client::error::Error> for Error {
	fn from(e: client::error::Error) -> Error {
		match e {
			client::error::Error(client::error::ErrorKind::UnknownBlock(b), _) => Error::from_kind(ErrorKind::UnknownBlock(b)),
			client::error::Error(client::error::ErrorKind::Execution(e), _) =>
				Error::from_kind(ErrorKind::Execution(format!("{}", e))),
			other => Error::from_kind(ErrorKind::Other(Box::new(other) as Box<_>)),
		}
	}
}

/// Build new blocks.
pub trait BlockBuilder {
	/// Push an extrinsic onto the block. Fails if the extrinsic is invalid.
	fn push_extrinsic(&mut self, extrinsic: Extrinsic) -> Result<()>;

	/// Bake the block with provided extrinsics.
	fn bake(self) -> Result<Block>;
}

/// Trait encapsulating the Shasper API.
///
/// All calls should fail when the exact runtime is unknown.
pub trait ShasperApi {
	/// The block builder for this API type.
	type BlockBuilder: BlockBuilder;

	/// Evaluate a block. Returns true if the block is good, false if it is known to be bad,
	/// and an error if we can't evaluate for some reason.
	fn evaluate_block(&self, at: &BlockId, block: Block) -> Result<bool>;

	/// Build a block on top of the given, with inherent extrinsics pre-pushed.
	fn build_block(&self, at: &BlockId, inherent_data: InherentData) -> Result<Self::BlockBuilder>;

	/// Attempt to produce the (encoded) inherent extrinsics for a block being built upon the given.
	/// This may vary by runtime and will fail if a runtime doesn't follow the same API.
	fn inherent_extrinsics(&self, at: &BlockId, inherent_data: InherentData) -> Result<Vec<Extrinsic>>;
}

/// Mark for all Shasper API implementations, that are making use of state data, stored locally.
pub trait LocalShasperApi: ShasperApi {}

/// Mark for all Shasper API implementations, that are fetching required state data from remote nodes.
pub trait RemoteShasperApi: ShasperApi {}
