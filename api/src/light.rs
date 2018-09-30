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

//! Strongly typed API for light Polkadot client.

use std::sync::Arc;
use client::backend::{Backend, RemoteBackend};
use client::{Client, CallExecutor};
use runtime::{Block, Extrinsic, InherentData, BlockId};
use {ShasperApi, RemoteShasperApi, BlockBuilder, Result, ErrorKind};
use substrate_primitives::{Blake2Hasher, RlpCodec};

/// Light block builder. TODO: make this work (efficiently)
#[derive(Clone, Copy)]
pub struct LightBlockBuilder;

impl BlockBuilder for LightBlockBuilder {
	fn push_extrinsic(&mut self, _xt: Extrinsic) -> Result<()> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn bake(self) -> Result<Block> {
		Err(ErrorKind::UnknownRuntime.into())
	}
}

/// Remote polkadot API implementation.
pub struct RemoteShasperApiWrapper<B: Backend<Block, Blake2Hasher, RlpCodec>, E: CallExecutor<Block, Blake2Hasher, RlpCodec>>(pub Arc<Client<B, E, Block>>);

impl<B: Backend<Block, Blake2Hasher, RlpCodec>, E: CallExecutor<Block, Blake2Hasher, RlpCodec>> ShasperApi for RemoteShasperApiWrapper<B, E> {
	type BlockBuilder = LightBlockBuilder;

	fn evaluate_block(&self, _at: &BlockId, _block: Block) -> Result<bool> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn build_block(&self, _at: &BlockId, _inherent: InherentData) -> Result<Self::BlockBuilder> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn inherent_extrinsics(&self, _at: &BlockId, _inherent: InherentData) -> Result<Vec<Extrinsic>> {
		Err(ErrorKind::UnknownRuntime.into())
	}
}

impl<B: RemoteBackend<Block, Blake2Hasher, RlpCodec>, E: CallExecutor<Block, Blake2Hasher, RlpCodec>> RemoteShasperApi for RemoteShasperApiWrapper<B, E> {}
