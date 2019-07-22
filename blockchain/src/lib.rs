mod pool;
pub mod backend;

pub use pool::AttestationPool;

use beacon::primitives::H256;
use beacon::types::*;
use beacon::{Error as BeaconError, BeaconState, Config, BLSConfig, Inherent, Transaction};
use std::sync::Arc;
use blockchain::traits::{Block as BlockT, BlockExecutor, AsExternalities};
use lmd_ghost::JustifiableExecutor;
use parity_codec::{Encode, Decode};
use bm_le::tree_root;
use core::marker::PhantomData;

use blockchain_rocksdb::RocksState as RocksStateT;

#[derive(Eq, PartialEq, Clone, Debug, Encode, Decode)]
pub struct Block<C: Config>(pub BeaconBlock<C>);

impl<C: Config> BlockT for Block<C> {
	type Identifier = H256;

	fn id(&self) -> H256 {
		let header = BeaconBlockHeader {
			slot: self.0.slot,
			parent_root: self.0.parent_root,
			state_root: self.0.state_root,
			body_root: tree_root::<sha2::Sha256, _>(&self.0.body),
			..Default::default()
		};

		tree_root::<sha2::Sha256, _>(&SigningBeaconBlockHeader::from(header.clone()))
	}

	fn parent_id(&self) -> Option<H256> {
		if self.0.parent_root == H256::default() {
			None
		} else {
			Some(self.0.parent_root)
		}
	}
}

pub trait StateExternalities {
	type Config: Config;

	fn state(&mut self) -> &mut BeaconState<Self::Config>;
}

#[derive(Clone)]
pub struct MemoryState<C: Config> {
	state: BeaconState<C>,
}

impl<C: Config> From<BeaconState<C>> for MemoryState<C> {
	fn from(state: BeaconState<C>) -> Self {
		Self { state }
	}
}

impl<C: Config> Into<BeaconState<C>> for MemoryState<C> {
	fn into(self) -> BeaconState<C> {
		self.state
	}
}

impl<C: Config> StateExternalities for MemoryState<C> {
	type Config = C;

	fn state(&mut self) -> &mut BeaconState<C> {
		&mut self.state
	}
}

impl<C: Config> AsExternalities<dyn StateExternalities<Config=C>> for MemoryState<C> {
	fn as_externalities(&mut self) -> &mut (dyn StateExternalities<Config=C> + 'static) {
		self
	}
}

#[derive(Clone)]
pub struct RocksState<C: Config> {
	state: BeaconState<C>,
}

impl<C: Config> From<BeaconState<C>> for RocksState<C> {
	fn from(state: BeaconState<C>) -> Self {
		Self { state }
	}
}

impl<C: Config> Into<BeaconState<C>> for RocksState<C> {
	fn into(self) -> BeaconState<C> {
		self.state
	}
}

impl<C: Config> StateExternalities for RocksState<C> {
	type Config = C;

	fn state(&mut self) -> &mut BeaconState<C> {
		&mut self.state
	}
}

impl<C: Config> AsExternalities<dyn StateExternalities<Config=C>> for RocksState<C> {
	fn as_externalities(&mut self) -> &mut (dyn StateExternalities<Config=C> + 'static) {
		self
	}
}

impl<C: Config> RocksStateT for RocksState<C> {
	type Raw = BeaconState<C>;

	fn from_raw(state: BeaconState<C>, _db: Arc<::rocksdb::DB>) -> Self {
		Self { state }
	}

	fn into_raw(self) -> BeaconState<C> {
		self.state
	}
}

#[derive(Debug)]
pub enum Error {
	Beacon(BeaconError),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error { }

impl From<BeaconError> for Error {
	fn from(error: BeaconError) -> Error {
		Error::Beacon(error)
	}
}

impl From<Error> for blockchain::import::Error {
	fn from(error: Error) -> blockchain::import::Error {
		blockchain::import::Error::Executor(Box::new(error))
	}
}

#[derive(Clone)]
pub struct Executor<C: Config, BLS: BLSConfig> {
	_marker: PhantomData<(C, BLS)>,
}

impl<C: Config, BLS: BLSConfig> Executor<C, BLS> {
	pub fn new() -> Self {
		Self { _marker: PhantomData }
	}

	pub fn initialize_block(
		&self,
		state: &mut <Self as BlockExecutor>::Externalities,
		target_slot: u64,
	) -> Result<(), Error> {
		Ok(beacon::initialize_block::<C>(state.state(), target_slot)?)
	}

	pub fn apply_inherent(
		&self,
		parent_block: &Block<C>,
		state: &mut <Self as BlockExecutor>::Externalities,
		inherent: Inherent,
	) -> Result<UnsealedBeaconBlock<C>, Error> {
		Ok(beacon::apply_inherent::<C, BLS>(&parent_block.0, state.state(), inherent)?)
	}

	pub fn apply_extrinsic(
		&self,
		block: &mut UnsealedBeaconBlock<C>,
		state: &mut <Self as BlockExecutor>::Externalities,
		extrinsic: Transaction<C>,
	) -> Result<(), Error> {
		Ok(beacon::apply_transaction::<C, BLS>(block, state.state(), extrinsic)?)
	}

	pub fn finalize_block(
		&self,
		block: &mut UnsealedBeaconBlock<C>,
		state: &mut <Self as BlockExecutor>::Externalities,
	) -> Result<(), Error> {
		Ok(beacon::finalize_block::<C, BLS>(block, state.state())?)
	}
}

impl<C: Config, BLS: BLSConfig> BlockExecutor for Executor<C, BLS> {
	type Error = Error;
	type Block = Block<C>;
	type Externalities = dyn StateExternalities<Config=C> + 'static;

	fn execute_block(
		&self,
		block: &Block<C>,
		state: &mut Self::Externalities,
	) -> Result<(), Error> {
		Ok(beacon::execute_block::<C, BLS>(&block.0, state.state())?)
	}
}

impl<C: Config, BLS: BLSConfig> JustifiableExecutor for Executor<C, BLS> {
	type ValidatorIndex = u64;

	fn justified_active_validators(
		&self,
		state: &mut Self::Externalities,
	) -> Result<Vec<Self::ValidatorIndex>, Self::Error> {
		Ok(state.state().justified_active_validators())
	}

	fn justified_block_id(
		&self,
		state: &mut Self::Externalities,
	) -> Result<Option<<Self::Block as BlockT>::Identifier>, Self::Error> {
		let justified_root = state.state().current_justified_checkpoint.root;
		if justified_root == H256::default() {
			Ok(None)
		} else {
			Ok(Some(justified_root))
		}
	}

	fn votes(
		&self,
		block: &Self::Block,
		state: &mut Self::Externalities,
	) -> Result<Vec<(Self::ValidatorIndex, <Self::Block as BlockT>::Identifier)>, Self::Error> {
		Ok(state.state().block_vote_targets(&block.0)?)
	}
}
