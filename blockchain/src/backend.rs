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
use blockchain::traits::{Block, Auxiliary};
use blockchain::backend::{Store, SharedCommittable, ChainQuery, Operation};
use lmd_ghost::archive::{AncestorQuery, NoCacheAncestorQuery};

pub struct ShasperBackend<Ba>(Ba);

impl<Ba> ShasperBackend<Ba> {
	pub fn new(backend: Ba) -> Self {
		Self(backend)
	}
}

impl<Ba: Clone> Clone for ShasperBackend<Ba> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<Ba: Store> Store for ShasperBackend<Ba> {
	type Block = Ba::Block;
	type State = Ba::State;
	type Auxiliary = Ba::Auxiliary;
	type Error = Ba::Error;
}

impl<Ba: ChainQuery> ChainQuery for ShasperBackend<Ba> {
	fn genesis(&self) -> <Self::Block as Block>::Identifier {
		self.0.genesis()
	}
	fn head(&self) -> <Self::Block as Block>::Identifier {
		self.0.head()
	}
	fn contains(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<bool, Self::Error> {
		Ok(self.0.contains(hash)?)
	}
	fn is_canon(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<bool, Self::Error> {
		Ok(self.0.is_canon(hash)?)
	}
	fn lookup_canon_depth(
		&self,
		depth: usize,
	) -> Result<Option<<Self::Block as Block>::Identifier>, Self::Error> {
		Ok(self.0.lookup_canon_depth(depth)?)
	}
	fn auxiliary(
		&self,
		key: &<Self::Auxiliary as Auxiliary<Self::Block>>::Key,
	) -> Result<Option<Self::Auxiliary>, Self::Error> {
		Ok(self.0.auxiliary(key)?)
	}
	fn depth_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<usize, Self::Error> {
		Ok(self.0.depth_at(hash)?)
	}
	fn children_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<Vec<<Self::Block as Block>::Identifier>, Self::Error> {
		Ok(self.0.children_at(hash)?)
	}
	fn state_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<Self::State, Self::Error> {
		Ok(self.0.state_at(hash)?)
	}
	fn block_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<Self::Block, Self::Error> {
		Ok(self.0.block_at(hash)?)
	}
}

impl<Ba: ChainQuery> AncestorQuery for ShasperBackend<Ba> {
	fn ancestor_at(
		&self,
		id: &<Self::Block as Block>::Identifier,
		depth: usize
	) -> Result<<Self::Block as Block>::Identifier, Self::Error> {
		NoCacheAncestorQuery::new(&self.0).ancestor_at(id, depth)
	}
}

impl<Ba> SharedCommittable for ShasperBackend<Ba> where
	Ba: SharedCommittable<Operation=Operation<Self::Block, Self::State, Self::Auxiliary>>
{
	type Operation = Operation<Self::Block, Self::State, Self::Auxiliary>;

	fn commit(
		&self,
		operation: Operation<Self::Block, Self::State, Self::Auxiliary>,
	) -> Result<(), Self::Error> {
		self.0.commit(operation)
	}
}
