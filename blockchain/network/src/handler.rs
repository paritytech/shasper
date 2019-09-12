use core::marker::PhantomData;
use blockchain::{Auxiliary, AsExternalities};
use blockchain::backend::{Store, SharedCommittable, ChainQuery, ImportLock};
use beacon::{Config, types::BeaconBlock, primitives::H256};
use network_messages::{HelloMessage, BeaconBlocksRequest};
use shasper_runtime::{StateExternalities, Block};

pub struct Handler<C, Ba> {
	import_lock: ImportLock,
	backend: Ba,
	_marker: PhantomData<C>,
}

impl<C, Ba> Handler<C, Ba> where
	C: Config,
	Ba: Store<Block=Block<C>> + SharedCommittable + ChainQuery,
	Ba::State: StateExternalities + AsExternalities<dyn StateExternalities<Config=C>>,
	Ba::Auxiliary: Auxiliary<Block<C>>,
{
	pub fn new(backend: Ba, import_lock: ImportLock) -> Self {
		Self {
			import_lock, backend,
			_marker: PhantomData,
		}
	}

	pub fn status(&self) -> HelloMessage {
		let head_hash = self.backend.head();
		let head_state = self.backend.state_at(&head_hash).unwrap();
		let head_slot = head_state.state().slot;
		let finalized_root = head_state.state().finalized_checkpoint.root;
		let finalized_epoch = head_state.state().finalized_checkpoint.epoch;
		let fork_version = head_state.state().fork.current_version.clone();

		HelloMessage {
			fork_version,
			finalized_root,
			finalized_epoch,
			head_root: head_hash,
			head_slot,
		}
	}

	pub fn head_request(&self, count: usize) -> BeaconBlocksRequest {
		let head_hash = self.backend.head();
		let head_state = self.backend.state_at(&head_hash).unwrap();
		let head_slot = head_state.state().slot;

		BeaconBlocksRequest {
			head_block_root: head_hash,
			start_slot: head_slot,
			count: count as u64,
			step: 1
		}
	}

	fn blocks_by_depth_no_lock(&self, start_depth: usize, count: usize) -> Vec<BeaconBlock<C>> {
		let mut ret = Vec::new();
		for d in start_depth..(start_depth + count) {
			match self.backend.lookup_canon_depth(d as usize) {
				Ok(Some(hash)) => {
					let block = self.backend.block_at(&hash)
						.expect("Found hash cannot fail");
					ret.push(block);
				},
				_ => break,
			}
		}
		ret.into_iter().map(Into::into).collect()
	}

	pub fn blocks_by_depth(&self, start_depth: usize, count: usize) -> Vec<BeaconBlock<C>> {
		let _ = self.import_lock.lock();
		self.blocks_by_depth_no_lock(start_depth, count)
	}

	pub fn blocks_by_slot(
		&self, start_hash: H256, start_slot: u64, count: usize
	) -> Vec<BeaconBlock<C>> {
		let _ = self.import_lock.lock();

		if !self.backend.is_canon(&start_hash).unwrap() {
			return Vec::new();
		}

		let start_state = match self.backend.state_at(&start_hash) {
			Ok(state) => state,
			Err(_) => return Vec::new(),
		};

		if start_state.state().slot != start_slot {
			return Vec::new()
		}

		let start_depth = self.backend.depth_at(&start_hash).unwrap();

		self.blocks_by_depth_no_lock(start_depth, count)
	}
}
