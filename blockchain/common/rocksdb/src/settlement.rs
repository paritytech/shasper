// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.







use std::collections::HashMap;
use blockchain::traits::{Block, Auxiliary};
use blockchain::backend::{Store, ChainQuery, ChainSettlement};
use parity_codec::{Encode, Decode};
use rocksdb::WriteBatch;

use super::{RocksBackend, RocksState, Error};
use super::utils::*;

pub struct RocksSettlement<'a, B: Block, A: Auxiliary<B>, S> {
	backend: &'a RocksBackend<B, A, S>,
	changes: HashMap<(&'static str, Vec<u8>), Option<Vec<u8>>>,
	new_head: Option<B::Identifier>,
	last_error: Option<Error>,
}

impl<'a, B: Block, A: Auxiliary<B>, S> Store for RocksSettlement<'a, B, A, S> {
	type Block = B;
	type Auxiliary = A;
	type State = S;
	type Error = Error;
}

impl<'a, B: Block, A: Auxiliary<B>, S: RocksState> ChainQuery for RocksSettlement<'a, B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
{
	fn genesis(&self) -> <Self::Block as Block>::Identifier {
		self.backend.genesis()
	}
	fn head(&self) -> <Self::Block as Block>::Identifier {
		self.backend.head()
	}
	fn contains(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<bool, Self::Error> {
		Ok(self.backend.contains(hash)?)
	}
	fn is_canon(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<bool, Self::Error> {
		Ok(self.backend.is_canon(hash)?)
	}
	fn lookup_canon_depth(
		&self,
		depth: usize,
	) -> Result<Option<<Self::Block as Block>::Identifier>, Self::Error> {
		Ok(self.backend.lookup_canon_depth(depth)?)
	}
	fn auxiliary(
		&self,
		key: &<Self::Auxiliary as Auxiliary<Self::Block>>::Key,
	) -> Result<Option<Self::Auxiliary>, Self::Error> {
		Ok(self.backend.auxiliary(key)?)
	}
	fn depth_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<usize, Self::Error> {
		Ok(self.backend.depth_at(hash)?)
	}
	fn children_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<Vec<<Self::Block as Block>::Identifier>, Self::Error> {
		Ok(self.backend.children_at(hash)?)
	}
	fn state_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<Self::State, Self::Error> {
		Ok(self.backend.state_at(hash)?)
	}
	fn block_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<Self::Block, Self::Error> {
		Ok(self.backend.block_at(hash)?)
	}
}

impl<'a, B: Block, A: Auxiliary<B>, S: RocksState> ChainSettlement for RocksSettlement<'a, B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
{
	fn insert_block(
		&mut self,
		id: <Self::Block as Block>::Identifier,
		block: Self::Block,
		state: Self::State,
		depth: usize,
		children: Vec<<Self::Block as Block>::Identifier>,
		is_canon: bool
	) {
		if self.last_error.is_some() {
			return
		}

		self.changes.insert((COLUMN_BLOCKS, id.encode()), Some(BlockData {
			block, state: state.into_raw(), depth: depth as u64, children, is_canon
		}.encode()));
	}

	fn push_child(
		&mut self,
		id: <Self::Block as Block>::Identifier,
		child: <Self::Block as Block>::Identifier,
	) {
		if self.last_error.is_some() {
			return
		}

		let mut data = match fetch_block_data::<B, S::Raw>(self.backend.db(), &id) {
			Ok(Some(data)) => data,
			Ok(None) => {
				self.last_error = Some(Error::Corrupted);
				return
			},
			Err(error) => {
				self.last_error = Some(error);
				return
			},
		};

		data.children.push(child);
		self.changes.insert((COLUMN_BLOCKS, id.encode()), Some(data.encode()));
	}

	fn set_canon(
		&mut self,
		id: <Self::Block as Block>::Identifier,
		is_canon: bool
	) {
		if self.last_error.is_some() {
			return
		}

		let mut data = match fetch_block_data::<B, S::Raw>(self.backend.db(), &id) {
			Ok(Some(data)) => data,
			Ok(None) => {
				self.last_error = Some(Error::Corrupted);
				return
			},
			Err(error) => {
				self.last_error = Some(error);
				return
			},
		};

		data.is_canon = is_canon;
		self.changes.insert((COLUMN_BLOCKS, id.encode()), Some(data.encode()));
	}

	fn insert_canon_depth_mapping(
		&mut self,
		depth: usize,
		id: <Self::Block as Block>::Identifier,
	) {
		if self.last_error.is_some() {
			return
		}

		let depth = depth as u64;
		self.changes.insert((COLUMN_CANON_DEPTH_MAPPINGS, depth.encode()), Some(id.encode()));
	}

	fn remove_canon_depth_mapping(
		&mut self,
		depth: &usize
	) {
		if self.last_error.is_some() {
			return
		}

		let depth = *depth as u64;
		self.changes.insert((COLUMN_CANON_DEPTH_MAPPINGS, depth.encode()), None);
	}

	fn insert_auxiliary(
		&mut self,
		key: <Self::Auxiliary as Auxiliary<Self::Block>>::Key,
		value: Self::Auxiliary
	) {
		if self.last_error.is_some() {
			return
		}

		self.changes.insert((COLUMN_AUXILIARIES, key.encode()), Some(value.encode()));
	}

	fn remove_auxiliary(
		&mut self,
		key: &<Self::Auxiliary as Auxiliary<Self::Block>>::Key,
	) {
		if self.last_error.is_some() {
			return
		}

		self.changes.insert((COLUMN_AUXILIARIES, key.encode()), None);
	}

	fn set_head(
		&mut self,
		head: <Self::Block as Block>::Identifier
	) {
		if self.last_error.is_some() {
			return
		}

		self.new_head = Some(head);
		self.changes.insert((COLUMN_INFO, KEY_HEAD.encode()), Some(head.encode()));
	}
}

impl<'a, B: Block, A: Auxiliary<B>, S: RocksState> RocksSettlement<'a, B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
{
	pub fn new(backend: &'a RocksBackend<B, A, S>) -> Self {
		Self {
			backend,
			changes: Default::default(),
			last_error: None,
			new_head: None,
		}
	}

	pub(crate) fn set_genesis(
		&mut self,
		genesis: B::Identifier
	) {
		if self.last_error.is_some() {
			return
		}

		self.changes.insert((COLUMN_INFO, KEY_GENESIS.encode()), Some(genesis.encode()));
	}

	pub(crate) fn commit(self) -> Result<Option<B::Identifier>, Error> {
		if let Some(error) = self.last_error {
			return Err(error)
		}

		let mut batch = WriteBatch::default();

		for ((column, key), value) in self.changes {
			let cf = self.backend.db().cf_handle(column).ok_or(Error::Corrupted)?;
			match value {
				Some(value) => {
					batch.put_cf(cf, key, value)?;
				},
				None => {
					batch.delete_cf(cf, key)?;
				},
			}
		}

		self.backend.db().write(batch)?;
		Ok(self.new_head)
	}
}
