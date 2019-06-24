mod pool;
pub mod backend;

pub use pool::AttestationPool;

use beacon::primitives::H256;
use beacon::types::{BeaconState, BeaconBlock, UnsealedBeaconBlock, BeaconBlockHeader};
use beacon::{Error as BeaconError, Executive, Config, Inherent, Transaction};
use std::sync::Arc;
use blockchain::traits::{Block as BlockT, BlockExecutor, AsExternalities};
use lmd_ghost::JustifiableExecutor;
use parity_codec::{Encode, Decode};
use ssz::Digestible;

use blockchain_rocksdb::RocksState as RocksStateT;

#[derive(Eq, PartialEq, Clone, Debug, Encode, Decode)]
pub struct Block(pub BeaconBlock);

impl BlockT for Block {
	type Identifier = H256;

	fn id(&self) -> H256 {
		let header = BeaconBlockHeader {
			slot: self.0.slot,
			parent_root: self.0.parent_root,
			state_root: self.0.state_root,
			body_root: H256::from_slice(
				Digestible::<sha2::Sha256>::hash(&self.0.body).as_slice()
			),
			..Default::default()
		};

		H256::from_slice(
			Digestible::<sha2::Sha256>::truncated_hash(&header).as_slice()
		)
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
	fn state(&mut self) -> &mut BeaconState;
}

#[derive(Clone)]
pub struct MemoryState {
	state: BeaconState,
}

impl From<BeaconState> for MemoryState {
	fn from(state: BeaconState) -> Self {
		Self { state }
	}
}

impl Into<BeaconState> for MemoryState {
	fn into(self) -> BeaconState {
		self.state
	}
}

impl StateExternalities for MemoryState {
	fn state(&mut self) -> &mut BeaconState {
		&mut self.state
	}
}

impl AsExternalities<dyn StateExternalities> for MemoryState {
	fn as_externalities(&mut self) -> &mut (dyn StateExternalities + 'static) {
		self
	}
}

#[derive(Clone)]
pub struct RocksState {
	state: BeaconState,
}

impl From<BeaconState> for RocksState {
	fn from(state: BeaconState) -> Self {
		Self { state }
	}
}

impl Into<BeaconState> for RocksState {
	fn into(self) -> BeaconState {
		self.state
	}
}

impl StateExternalities for RocksState {
	fn state(&mut self) -> &mut BeaconState {
		&mut self.state
	}
}

impl AsExternalities<dyn StateExternalities> for RocksState {
	fn as_externalities(&mut self) -> &mut (dyn StateExternalities + 'static) {
		self
	}
}

impl RocksStateT for RocksState {
	type Raw = BeaconState;

	fn from_raw(state: BeaconState, _db: Arc<::rocksdb::DB>) -> Self {
		Self { state }
	}

	fn into_raw(self) -> BeaconState {
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
pub struct Executor<C: Config> {
	config: C,
}

impl<C: Config> Executor<C> {
	pub fn new(config: C) -> Self {
		Self { config }
	}

	pub fn executive<'state, 'config>(
		&'config self,
		state: &'state mut <Self as BlockExecutor>::Externalities,
	) -> Executive<'state, 'config, C> {
		Executive {
			state: state.state(),
			config: &self.config,
		}
	}

	pub fn initialize_block(
		&self,
		state: &mut <Self as BlockExecutor>::Externalities,
		target_slot: u64,
	) -> Result<(), Error> {
		Ok(beacon::initialize_block(state.state(), target_slot, &self.config)?)
	}

	pub fn apply_inherent(
		&self,
		parent_block: &Block,
		state: &mut <Self as BlockExecutor>::Externalities,
		inherent: Inherent,
	) -> Result<UnsealedBeaconBlock, Error> {
		Ok(beacon::apply_inherent(&parent_block.0, state.state(), inherent, &self.config)?)
	}

	pub fn apply_extrinsic(
		&self,
		block: &mut UnsealedBeaconBlock,
		state: &mut <Self as BlockExecutor>::Externalities,
		extrinsic: Transaction,
	) -> Result<(), Error> {
		Ok(beacon::apply_transaction(block, state.state(), extrinsic, &self.config)?)
	}

	pub fn finalize_block(
		&self,
		block: &mut UnsealedBeaconBlock,
		state: &mut <Self as BlockExecutor>::Externalities,
	) -> Result<(), Error> {
		Ok(beacon::finalize_block(block, state.state(), &self.config)?)
	}
}

impl<C: Config> BlockExecutor for Executor<C> {
	type Error = Error;
	type Block = Block;
	type Externalities = dyn StateExternalities + 'static;

	fn execute_block(
		&self,
		block: &Block,
		state: &mut Self::Externalities,
	) -> Result<(), Error> {
		Ok(beacon::execute_block(&block.0, state.state(), &self.config)?)
	}
}

impl<C: Config> JustifiableExecutor for Executor<C> {
	type ValidatorIndex = u64;

	fn justified_active_validators(
		&self,
		state: &mut Self::Externalities,
	) -> Result<Vec<Self::ValidatorIndex>, Self::Error> {
		Ok(self.executive(state).justified_active_validators())
	}

	fn justified_block_id(
		&self,
		state: &mut Self::Externalities,
	) -> Result<Option<<Self::Block as BlockT>::Identifier>, Self::Error> {
		let justified_root = state.state().current_justified_root;
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
		Ok(self.executive(state).block_vote_targets(&block.0)?)
	}
}
