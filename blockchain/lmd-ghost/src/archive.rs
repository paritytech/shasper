use std::collections::HashMap;
use core::hash::Hash;
use core::mem;
use blockchain::traits::{Block, Auxiliary, BlockExecutor, AsExternalities};
use blockchain::import::{BlockImporter, RawImporter, ImportAction};
use blockchain::backend::{Store, SharedCommittable, ImportOperation, ChainQuery, ImportLock, Operation};
use crate::JustifiableExecutor;

pub trait AncestorQuery: Store {
	fn ancestor_at(
		&self,
		id: &<Self::Block as Block>::Identifier,
		depth: usize
	) -> Result<<Self::Block as Block>::Identifier, Self::Error>;
}

pub struct NoCacheAncestorQuery<'a, Ba: Store>(&'a Ba);

impl<'a, Ba: Store> NoCacheAncestorQuery<'a, Ba> {
	pub fn new(backend: &'a Ba) -> Self {
		Self(backend)
	}
}

impl<'a, Ba: Store> Store for NoCacheAncestorQuery<'a, Ba> {
	type Block = Ba::Block;
	type State = Ba::State;
	type Auxiliary = Ba::Auxiliary;
	type Error = Ba::Error;
}

impl<'a, Ba: ChainQuery> AncestorQuery for NoCacheAncestorQuery<'a, Ba> {
	fn ancestor_at(
		&self,
		id: &<Self::Block as Block>::Identifier,
		depth: usize
	) -> Result<<Self::Block as Block>::Identifier, Self::Error> {
		let mut current = id.clone();
		while self.0.depth_at(&current)? > depth {
			current = self.0.block_at(&current)?.parent_id()
				.expect("When parent id is None, depth is 0;
                         no value can be greater than 0; while is false; qed");
		}
		Ok(current)
	}
}

pub struct ArchiveGhost<Ba: Store, VI: Eq + Hash> {
	backend: Ba,
	votes: HashMap<VI, <Ba::Block as Block>::Identifier>,
	overlayed_votes: HashMap<VI, <Ba::Block as Block>::Identifier>,
}

impl<Ba: AncestorQuery + ChainQuery, VI: Eq + Hash> ArchiveGhost<Ba, VI> {
	pub fn new(backend: Ba) -> Self {
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
			if self.backend.ancestor_at(target, block_depth)? == *block {
				total += 1;
			}
		}
		for (v, target) in &self.votes {
			if !self.overlayed_votes.keys().any(|k| k == v) &&
				self.backend.ancestor_at(target, block_depth)? == *block
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
		let mut head_depth = self.backend.depth_at(justified)?;
		loop {
			let children = self.backend.children_at(&head)?;
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

pub struct ArchiveGhostImporter<E: BlockExecutor, Ba: Store<Block=E::Block>> where
	E: JustifiableExecutor,
	Ba::Auxiliary: Auxiliary<E::Block>
{
	ghost: ArchiveGhost<Ba, E::ValidatorIndex>,
	import_lock: ImportLock,
	executor: E,
}

impl<E: BlockExecutor, Ba: SharedCommittable + Store<Block=E::Block>> ArchiveGhostImporter<E, Ba> where
	E: JustifiableExecutor,
	Ba: AncestorQuery + ChainQuery,
	Ba::Auxiliary: Auxiliary<E::Block>
{
	pub fn new(executor: E, backend: Ba, import_lock: ImportLock) -> Self {
		Self {
			executor, import_lock,
			ghost: ArchiveGhost::new(backend),
		}
	}
}

impl<E: BlockExecutor, Ba: Store<Block=E::Block>> BlockImporter for ArchiveGhostImporter<E, Ba> where
	E: JustifiableExecutor,
	Ba: ChainQuery + AncestorQuery,
	Ba: SharedCommittable<Operation=Operation<E::Block, <Ba as Store>::State, <Ba as Store>::Auxiliary>>,
	Ba::Auxiliary: Auxiliary<E::Block>,
	Ba::State: AsExternalities<E::Externalities>,
	blockchain::import::Error: From<Ba::Error> + From<E::Error>,
{
	type Block = Ba::Block;
	type Error = blockchain::import::Error;

	fn import_block(&mut self, block: Ba::Block) -> Result<(), Self::Error> {
		let mut state = self.ghost.backend.state_at(
			&block.parent_id().ok_or(blockchain::import::Error::IsGenesis)?
		)?;
		self.executor.execute_block(&block, state.as_externalities())?;

		self.import_raw(ImportOperation { block, state })
	}
}

impl<E: BlockExecutor, Ba: Store<Block=E::Block>> RawImporter for ArchiveGhostImporter<E, Ba> where
	E: JustifiableExecutor,
	Ba: AncestorQuery + ChainQuery,
	Ba: SharedCommittable<Operation=Operation<E::Block, <Ba as Store>::State, <Ba as Store>::Auxiliary>>,
	Ba::Auxiliary: Auxiliary<E::Block>,
	Ba::State: AsExternalities<E::Externalities>,
	blockchain::import::Error: From<Ba::Error> + From<E::Error>,
{
	type Operation = ImportOperation<Ba::Block, Ba::State>;
	type Error = blockchain::import::Error;

	fn import_raw(
		&mut self,
		mut raw: ImportOperation<Ba::Block, Ba::State>
	) -> Result<(), Self::Error> {
		let (justified_active_validators, justified_block_id, votes) = {
			let externalities = raw.state.as_externalities();
			let justified_active_validators =
				self.executor.justified_active_validators(externalities)?;
			let justified_block_id = match self.executor.justified_block_id(externalities)? {
				Some(value) => value,
				None => self.ghost.backend.genesis(),
			};
			let votes = self.executor.votes(&raw. block, externalities)?;

			let mut importer = ImportAction::new(
				&self.executor, &self.ghost.backend, self.import_lock.lock()
			);
			importer.import_raw(raw);
			importer.commit()?;

			(justified_active_validators, justified_block_id, votes)
		};

		for (k, v) in votes {
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

		let mut importer = ImportAction::new(
			&self.executor, &self.ghost.backend, self.import_lock.lock()
		);
		importer.set_head(new_head);

		match importer.commit() {
			Ok(()) => { self.ghost.commit_overlay(); },
			Err(_) => { self.ghost.reset_overlay(); },
		}

		Ok(())
	}
}
