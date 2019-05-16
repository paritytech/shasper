use std::collections::HashMap;
use core::hash::Hash;
use core::mem;
use blockchain::traits::{Backend, Operation, Block, ChainQuery, Auxiliary, BlockExecutor, ImportBlock, AsExternalities};
use blockchain::backend::{MemoryLikeBackend, SharedBackend};
use crate::{LmdGhostExternalities, VotedBlock};

pub trait AncestorQuery: ChainQuery {
	fn ancestor_at(
		&self,
		id: &<Self::Block as Block>::Identifier,
		depth: usize
	) -> Result<<Self::Block as Block>::Identifier, Self::Error>;
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
	) -> Result<<Self::Block as Block>::Identifier, Self::Error> {
		let mut current = id.clone();
		while self.depth_at(&current)? > depth {
			current = self.block_at(&current)?.parent_id()
				.expect("When parent id is None, depth is 0;
                         no value can be greater than 0; while is false; qed");
		}
		Ok(current)
	}
}

pub struct ArchiveGhost<Ba: Backend, VI: Eq + Hash> {
	backend: SharedBackend<Ba>,
	votes: HashMap<VI, <Ba::Block as Block>::Identifier>,
	overlayed_votes: HashMap<VI, <Ba::Block as Block>::Identifier>,
}

impl<Ba: AncestorQuery, VI: Eq + Hash> ArchiveGhost<Ba, VI> {
	pub fn new(backend: SharedBackend<Ba>) -> Self {
		Self {
			backend,
			votes: Default::default(),
			overlayed_votes: Default::default(),
		}
	}

	pub fn update_overlay(
		&mut self,
		validator_id: VI,
		target_root: <Ba::Block as Block>::Identifier
	) {
		self.overlayed_votes.insert(validator_id, target_root);
	}

	pub fn commit_overlay(
		&mut self
	) {
		let mut overlayed_votes = HashMap::new();
		mem::swap(&mut overlayed_votes, &mut self.overlayed_votes);

		for (k, v) in overlayed_votes {
			self.votes.insert(k, v);
		}
	}

	pub fn reset_overlay(
		&mut self
	) {
		self.overlayed_votes = HashMap::new();
	}

	pub fn update_active(
		&mut self,
		active_validators: &[VI]
	) {
		self.votes.retain(|v, _| {
			active_validators.contains(v)
		});
	}

	pub fn vote_count(
		&self,
		block: &<Ba::Block as Block>::Identifier,
		block_depth: usize
	) -> Result<usize, Ba::Error> {
		let mut total = 0;
		for (_, target) in &self.overlayed_votes {
			if self.backend.read().ancestor_at(target, block_depth)? == *block {
				total += 1;
			}
		}
		for (v, target) in &self.votes {
			if !self.overlayed_votes.keys().any(|k| k == v) &&
				self.backend.read().ancestor_at(target, block_depth)? == *block
			{
				total += 1;
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

pub struct ArchiveGhostImporter<E: BlockExecutor, Ba: Backend<Block=E::Block>> where
	Ba::Block: VotedBlock,
	Ba::Auxiliary: Auxiliary<E::Block>
{
	ghost: ArchiveGhost<Ba, <Ba::Block as VotedBlock>::ValidatorIdentifier>,
	executor: E,
}

impl<E: BlockExecutor, Ba: Backend<Block=E::Block>> ArchiveGhostImporter<E, Ba> where
	Ba: AncestorQuery,
	Ba::Block: VotedBlock,
	Ba::Auxiliary: Auxiliary<E::Block>
{
	pub fn new(executor: E, backend: SharedBackend<Ba>) -> Self {
		Self {
			executor,
			ghost: ArchiveGhost::new(backend),
		}
	}
}

impl<E: BlockExecutor, Ba: Backend<Block=E::Block>> ImportBlock for ArchiveGhostImporter<E, Ba> where
	Ba: AncestorQuery,
	E::Block: VotedBlock,
	Ba::Auxiliary: Auxiliary<E::Block>,
	Ba::State: AsExternalities<E::Externalities>,
	E::Externalities: LmdGhostExternalities<<Ba::Block as Block>::Identifier, <Ba::Block as VotedBlock>::ValidatorIdentifier>,
	blockchain::chain::Error: From<Ba::Error> + From<E::Error>,
{
	type Block = Ba::Block;
	type Error = blockchain::chain::Error;

	fn import_block(&mut self, block: Ba::Block) -> Result<(), Self::Error> {
		let (justified_active_validators, justified_block_id) = {
			let mut importer = self.ghost.backend.begin_import(&self.executor);
			let mut operation = importer.execute_block(block.clone())?;
			let justified_active_validators = operation.state.as_externalities().justified_active_validators();
			let justified_block_id = operation.state.as_externalities().justified_block_id();

			importer.import_raw(operation);
			importer.commit()?;

			(justified_active_validators, justified_block_id)
		};

		for (k, v) in block.votes() {
			self.ghost.update_overlay(k, v);
		}
		self.ghost.update_active(&justified_active_validators);
		let new_head = match self.ghost.head(&justified_block_id) {
			Ok(value) => value,
			Err(e) => {
				self.ghost.reset_overlay();
				return Err(e.into())
			},
		};

		let mut importer = self.ghost.backend.begin_import(&self.executor);
		importer.set_head(new_head);

		match importer.commit() {
			Ok(()) => { self.ghost.commit_overlay(); },
			Err(_) => { self.ghost.reset_overlay(); },
		}

		Ok(())
	}
}
