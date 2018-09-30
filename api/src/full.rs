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

//! Strongly typed API for full Polkadot client.

use client::backend::LocalBackend;
use client::block_builder::BlockBuilder as ClientBlockBuilder;
use client::{Client, LocalCallExecutor};
use shasper_executor::Executor as LocalDispatch;
use substrate_executor::NativeExecutor;

use runtime::{Block, Extrinsic, InherentData, BlockId};
use substrate_primitives::{Blake2Hasher, RlpCodec};
use {BlockBuilder, ShasperApi, LocalShasperApi, ErrorKind, Result};

impl<B: LocalBackend<Block, Blake2Hasher, RlpCodec>> BlockBuilder for ClientBlockBuilder<B, LocalCallExecutor<B, NativeExecutor<LocalDispatch>>, Block, Blake2Hasher, RlpCodec> {
	fn push_extrinsic(&mut self, extrinsic: Extrinsic) -> Result<()> {
		self.push(extrinsic).map_err(Into::into)
	}

	/// Bake the block with provided extrinsics.
	fn bake(self) -> Result<Block> {
		ClientBlockBuilder::bake(self).map_err(Into::into)
	}
}

impl<B: LocalBackend<Block, Blake2Hasher, RlpCodec>> ShasperApi for Client<B, LocalCallExecutor<B, NativeExecutor<LocalDispatch>>, Block> {
	type BlockBuilder = ClientBlockBuilder<B, LocalCallExecutor<B, NativeExecutor<LocalDispatch>>, Block, Blake2Hasher, RlpCodec>;

	fn evaluate_block(&self, at: &BlockId, block: Block) -> Result<bool> {
		let res: Result<()> = self.call_api_at(at, "execute_block", &block).map_err(From::from);
		match res {
			Ok(_) => Ok(true),
			Err(err) => match err.kind() {
				&ErrorKind::Execution(_) => Ok(false),
				_ => Err(err)
			}
		}
	}

	fn build_block(&self, at: &BlockId, inherent_data: InherentData) -> Result<Self::BlockBuilder> {
		let mut block_builder = self.new_block_at(at)?;
		for inherent in self.inherent_extrinsics(at, inherent_data)? {
			block_builder.push(inherent)?;
		}

		Ok(block_builder)
	}

	fn inherent_extrinsics(&self, at: &BlockId, inherent_data: InherentData) -> Result<Vec<Extrinsic>> {
		let runtime_version = self.runtime_version_at(at)?;
		Ok(self.call_api_at(at, "inherent_extrinsics", &(inherent_data, runtime_version.spec_version))?)
	}
}

impl<B: LocalBackend<Block, Blake2Hasher, RlpCodec>> LocalShasperApi for Client<B, LocalCallExecutor<B, NativeExecutor<LocalDispatch>>, Block>
{}
