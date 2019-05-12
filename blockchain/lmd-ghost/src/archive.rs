use std::collections::HashMap;
use blockchain::traits::{Backend, Operation, Block, ChainQuery, Auxiliary};
use blockchain::backend::{MemoryLikeBackend, SharedBackend};

pub trait AncestorQuery: ChainQuery {
	fn ancestor_at(
		&self,
		id: &<Self::Block as Block>::Identifier,
		depth: usize
	) -> Result<Option<<Self::Block as Block>::Identifier>, Self::Error>;
}

pub struct NoCacheAncestorBackend<Ba: Backend>(Ba);

impl<Ba: Backend> Backend for NoCacheAncestorBackend<Ba> {
	type Block = Ba::Block;
	type State = Ba::State;
	type Auxiliary = Ba::Auxiliary;
	type Error = Ba::Error;

	fn commit(
		&mut self,
		operation: Operation<Self::Block, Self::State, Self::Auxiliary>,
	) -> Result<(), Self::Error> {
		self.0.commit(operation)
	}
}

impl<Ba: ChainQuery> ChainQuery for NoCacheAncestorBackend<Ba> {
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
		self.0.contains(hash)
	}

	fn is_canon(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<bool, Self::Error> {
		self.0.is_canon(hash)
	}

	fn lookup_canon_depth(
		&self,
		depth: usize,
	) -> Result<Option<<Self::Block as Block>::Identifier>, Self::Error> {
		self.0.lookup_canon_depth(depth)
	}

	fn auxiliary(
		&self,
		key: &<Self::Auxiliary as Auxiliary<Self::Block>>::Key,
	) -> Result<Option<Self::Auxiliary>, Self::Error> {
		self.0.auxiliary(key)
	}

	fn depth_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<usize, Self::Error> {
		self.0.depth_at(hash)
	}

	fn children_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<Vec<<Self::Block as Block>::Identifier>, Self::Error> {
		self.0.children_at(hash)
	}

	fn state_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<Self::State, Self::Error> {
		self.0.state_at(hash)
	}

	fn block_at(
		&self,
		hash: &<Self::Block as Block>::Identifier,
	) -> Result<Self::Block, Self::Error> {
		self.0.block_at(hash)
	}
}

impl<Ba: MemoryLikeBackend> MemoryLikeBackend for NoCacheAncestorBackend<Ba> {
	fn new_with_genesis(block: Ba::Block, genesis_state: Ba::State) -> Self {
		Self(Ba::new_with_genesis(block, genesis_state))
	}
}

impl<Ba: ChainQuery> AncestorQuery for NoCacheAncestorBackend<Ba> {
	fn ancestor_at(
		&self,
		id: &<Self::Block as Block>::Identifier,
		depth: usize
	) -> Result<Option<<Self::Block as Block>::Identifier>, Self::Error> {
		let mut current = Some(id.clone());
		while current.is_some() &&
			self.depth_at(current.as_ref().expect("Checked current is some"))? > depth
		{
			current = self.block_at(current.as_ref().expect("Checked current is some"))?.parent_id();
		}
		Ok(current)
	}
}

pub struct ArchiveGhost<Ba: Backend> {
	backend: SharedBackend<Ba>,
	latest_scores: HashMap<<Ba::Block as Block>::Identifier, usize>,
}

impl<Ba: AncestorQuery> ArchiveGhost<Ba> {
	pub fn new(backend: SharedBackend<Ba>) -> Self {
		Self {
			backend,
			latest_scores: Default::default(),
		}
	}

	pub fn apply_scores(
		&mut self,
		scores: &[(<Ba::Block as Block>::Identifier, isize)]
	) {
		for (target, change) in scores {
			self.latest_scores.entry(*target)
				.and_modify(|v| {
					if *change > 0 {
						*v += *change as usize;
					} else {
						*v -= (-*change) as usize;
					}
				})
				.or_insert(*change as usize);
		}
		self.latest_scores.retain(|_, score| *score > 0);
	}

	pub fn vote_count(
		&self,
		block: &<Ba::Block as Block>::Identifier,
		block_depth: usize
	) -> Result<usize, Ba::Error> {
		let mut total = 0;
		for (target, votes) in &self.latest_scores {
			if self.backend.read().ancestor_at(target, block_depth)? == Some(*block) {
				total += votes;
			}
		}
		Ok(total)
	}

	pub fn head(
		&self,
		justified: &<Ba::Block as Block>::Identifier,
	) -> Result<<Ba::Block as Block>::Identifier, Ba::Error> {
		let mut head = *justified;
		let mut head_depth = self.backend.read().depth_at(justified)?;
		loop {
			let children = self.backend.read().children_at(&head)?;
			if children.len() == 0 {
				return Ok(head)
			}
			let mut best = children[0];
			let mut best_score = 0;
			for child in children {
				let vote_count = self.vote_count(&child, head_depth + 1)?;
				if vote_count > best_score {
					best = child;
					best_score = vote_count;
				}
			}
			head = best;
			head_depth += 1;
		}
	}
}
