use core::mem;
use std::collections::HashMap;
use blockchain::traits::{Block, Auxiliary, Backend, ChainQuery};

fn log2(v: usize) -> Option<usize> {
	if v == 0 {
		None
	} else {
		Some(mem::size_of::<usize>() * 8 - v.leading_zeros() as usize - 1)
	}
}

#[derive(Debug)]
pub struct AncestorCache<B: Block> {
	cache: HashMap<(B::Identifier, usize), Option<B::Identifier>>,
	ancestors: [HashMap<B::Identifier, (Option<B::Identifier>, usize)>; 16],
	blocks: Vec<(B::Identifier, usize, Option<B::Identifier>)>,
	min_depth: usize,
}

impl<B: Block> AncestorCache<B> {
	pub fn new() -> Self {
		Self {
			cache: Default::default(),
			ancestors: [
				Default::default(), Default::default(), Default::default(), Default::default(),
				Default::default(), Default::default(), Default::default(), Default::default(),
				Default::default(), Default::default(), Default::default(), Default::default(),
				Default::default(), Default::default(), Default::default(), Default::default(),
			],
			blocks: Default::default(),
			min_depth: 0,
		}
	}

	pub fn push_block(
		&mut self,
		block: B::Identifier,
		block_depth: usize,
		parent: Option<B::Identifier>
	) {
		for i in 0..16 {
			if (block_depth - self.min_depth) % (1 << i) == 0 {
				self.ancestors[i].insert(block, (parent, block_depth - 1));
			} else {
				if let Some(parent) = parent {
					if let Some(ancestor) = self.ancestors[i].get(&parent).map(|v| v.clone()) {
						self.ancestors[i].insert(block, ancestor);
					}
				}
			}
		}
		self.blocks.push((block, block_depth, parent));
	}

	pub fn lookup(
		&mut self,
		block: &B::Identifier,
		block_depth: usize,
		target_depth: usize
	) -> Option<B::Identifier> {
		if target_depth > block_depth {
			return None
		} else if target_depth == block_depth {
			return Some(*block)
		}

		let cache_key = (*block, target_depth);
		if let Some(ancestor) = self.cache.get(&cache_key) {
			return ancestor.clone()
		}

		let (skip_block, skip_depth) = self.ancestors
			[log2(block_depth - target_depth - 1).unwrap_or(0)]
			.get(&block)
			.expect("Ancestors data is invalid").clone();
		let skip_block = match skip_block {
			Some(skip_block) => skip_block,
			None => return None,
		};
		let ret = self.lookup(&skip_block, skip_depth, target_depth);

		self.cache.insert(cache_key, ret);
		ret
	}

	pub fn prune(
		&mut self,
		min_depth: usize
	) {
		self.cache.retain(|(_, depth), _| *depth >= min_depth);
		for i in 0..16 {
			self.ancestors[i].retain(|_, (_, depth)| *depth >= min_depth);
		}
		self.blocks.retain(|(_, depth, _)| *depth >= min_depth);
		self.min_depth = min_depth;
		for (block, block_depth, parent) in self.blocks.clone() {
			self.push_block(block, block_depth, parent);
		}
	}

	pub fn min_depth(&self) -> usize { self.min_depth }
}

pub struct PruningGhost<B: Block> {
	ancestor_cache: AncestorCache<B>,
	latest_scores: HashMap<(B::Identifier, usize), usize>,
}

impl<B: Block> PruningGhost<B> {
	pub fn new() -> Self {
		Self {
			ancestor_cache: AncestorCache::new(),
			latest_scores: Default::default(),
		}
	}

	pub fn ancestor_cache(&mut self) -> &mut AncestorCache<B> {
		&mut self.ancestor_cache
	}

	pub fn vote_count(&mut self, block: &B::Identifier, block_depth: usize) -> usize {
		let mut total = 0;
		for ((target, target_depth), votes) in &self.latest_scores {
			if self.ancestor_cache.lookup(target, *target_depth, block_depth) == Some(*block) {
				total += votes;
			}
		}
		total
	}

	pub fn apply_scores(&mut self, scores: &[((B::Identifier, usize), isize)]) {
		for ((target, target_depth), change) in scores {
			if *target_depth >= self.ancestor_cache.min_depth() {
				self.latest_scores.entry((*target, *target_depth))
					.and_modify(|v| {
						if *change > 0 {
							*v += *change as usize;
						} else {
							*v -= (-*change) as usize;
						}
					})
					.or_insert(*change as usize);
			}
		}
		self.latest_scores.retain(|_, score| *score > 0);
	}

	pub fn head<Ba>(
		&mut self,
		backend: &Ba,
		justified: &B::Identifier,
		justified_depth: usize,
	) -> Result<B::Identifier, Ba::Error> where
		Ba: Backend<Block=B> + ChainQuery,
		Ba::Auxiliary: Auxiliary<B>
	{
		let mut head = *justified;
		let mut head_depth = justified_depth;
		loop {
			let children = backend.children_at(&head)?;
			if children.len() == 0 {
				return Ok(head)
			}
			let mut best = children[0];
			let mut best_score = 0;
			for child in children {
				let vote_count = self.vote_count(&child, head_depth + 1);
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

#[cfg(test)]
mod tests {
	use super::*;
	use blockchain::traits::{Operation, ImportOperation};
	use blockchain::backend::{MemoryBackend, KeyValueMemoryState, MemoryLikeBackend};

	#[derive(Clone, Debug)]
	struct DummyBlock {
		id: usize,
		parent_id: Option<usize>
	}

	impl Block for DummyBlock {
		type Identifier = usize;

		fn id(&self) -> usize {
			self.id
		}

		fn parent_id(&self) -> Option<usize> {
			self.parent_id
		}
	}

    #[test]
	fn test_log2() {
		assert_eq!(log2(1), Some(0));
		assert_eq!(log2(2), Some(1));
		assert_eq!(log2(4), Some(2));
	}

	#[test]
	fn test_ancestor_cache() {
		let mut cache = AncestorCache::<DummyBlock>::new();

		cache.push_block(1, 1, None);
		cache.push_block(2, 2, Some(1));
		assert_eq!(cache.lookup(&2, 2, 2), Some(2));
		cache.push_block(3, 3, Some(2));
		assert_eq!(cache.lookup(&3, 3, 2), Some(2));
		assert_eq!(cache.lookup(&3, 3, 1), Some(1));
		assert_eq!(cache.lookup(&3, 3, 0), None);
	}

	#[test]
	fn test_ghost() {
		let genesis = DummyBlock { id: 0, parent_id: None };
		let mut backend = MemoryBackend::<DummyBlock, (), KeyValueMemoryState>::new_with_genesis(genesis, Default::default());
		let mut ghost = PruningGhost::<DummyBlock>::new();

		backend.commit(Operation {
			import_block: vec![
				ImportOperation {
					block: DummyBlock { id: 1, parent_id: Some(0) },
					state: Default::default(),
				},
				ImportOperation {
					block: DummyBlock { id: 2, parent_id: Some(1) },
					state: Default::default(),
				},
				ImportOperation {
					block: DummyBlock { id: 3, parent_id: Some(2) },
					state: Default::default(),
				},
				ImportOperation {
					block: DummyBlock { id: 4, parent_id: Some(1) },
					state: Default::default(),
				},
			],
			set_head: None,
			insert_auxiliaries: Vec::new(),
			remove_auxiliaries: Vec::new(),
		}).unwrap();
		ghost.ancestor_cache().push_block(1, 1, Some(0));
		ghost.ancestor_cache().push_block(2, 2, Some(1));
		ghost.ancestor_cache().push_block(3, 3, Some(2));
		ghost.ancestor_cache().push_block(4, 2, Some(1));

		ghost.apply_scores(&[
			((3, 3), 2),
			((4, 2), 1),
		]);

		assert_eq!(ghost.head(&backend, &1, 1).unwrap(), 3);

		ghost.apply_scores(&[
			((3, 3), -1),
			((4, 2), 1),
		]);

		assert_eq!(ghost.head(&backend, &1, 1).unwrap(), 4);
	}
}
