use core::marker::PhantomData;
use std::collections::HashMap;
use std::path::Path;
use std::{fmt, error as stderror};
use std::sync::Arc;
use blockchain::traits::{Block, Auxiliary};
use blockchain::backend::{Store, ChainQuery, OperationError, SharedCommittable, ChainSettlement, Operation};
use parity_codec::{Encode, Decode};
use rocksdb::{DB, Options, WriteBatch};

const COLUMN_BLOCKS: &str = "blocks";
const COLUMN_CANON_DEPTH_MAPPINGS: &str = "canon_depth_mappings";
const COLUMN_AUXILIARIES: &str = "auxiliaries";
const COLUMN_INFO: &str = "info";
const KEY_HEAD: &str = "head";
const KEY_GENESIS: &str = "genesis";

#[derive(Debug)]
/// RocksDB backend errors
pub enum Error {
	/// Invalid Operation
	InvalidOperation,
	/// Trying to import a block that is genesis
	IsGenesis,
	/// Query does not exist
	NotExist,
	/// Corrupted database,
	Corrupted,
	/// RocksDB errors
	Rocks(rocksdb::Error),
}

impl From<rocksdb::Error> for Error {
	fn from(error: rocksdb::Error) -> Error {
		Error::Rocks(error)
	}
}

impl OperationError for Error {
	fn invalid_operation() -> Self {
		Error::InvalidOperation
	}

	fn block_is_genesis() -> Self {
		Error::IsGenesis
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl stderror::Error for Error { }

impl From<Error> for blockchain::import::Error {
	fn from(error: Error) -> Self {
		match error {
			Error::IsGenesis => blockchain::import::Error::IsGenesis,
			error => blockchain::import::Error::Backend(Box::new(error)),
		}
	}
}

pub struct RocksBackend<B: Block, A: Auxiliary<B>, S> {
	db: Arc<DB>,
	_marker: PhantomData<(B, A, S)>,
}

struct RocksSettlement<'a, B: Block, A: Auxiliary<B>, S> {
	backend: &'a RocksBackend<B, A, S>,
	changes: HashMap<(&'static str, Vec<u8>), Option<Vec<u8>>>,
	last_error: Option<Error>,
}

impl<B: Block, A: Auxiliary<B>, S> RocksBackend<B, A, S> {
	pub fn open<P: AsRef<Path>>(path: P) -> Self {
		let mut db_opts = Options::default();
		db_opts.create_missing_column_families(true);
		db_opts.create_if_missing(true);

		let db = DB::open_cf(&db_opts, path, &[
			COLUMN_BLOCKS, COLUMN_CANON_DEPTH_MAPPINGS, COLUMN_AUXILIARIES, COLUMN_INFO,
		]).unwrap();

		Self {
			db: Arc::new(db),
			_marker: PhantomData,
		}
	}
}

impl<B: Block, A: Auxiliary<B>, S> Clone for RocksBackend<B, A, S> {
	fn clone(&self) -> Self {
		Self {
			db: self.db.clone(),
			_marker: PhantomData,
		}
	}
}

impl<B: Block, A: Auxiliary<B>, S> Store for RocksBackend<B, A, S> {
	type Block = B;
	type Auxiliary = A;
	type State = S;
	type Error = Error;
}

impl<'a, B: Block, A: Auxiliary<B>, S> Store for RocksSettlement<'a, B, A, S> {
	type Block = B;
	type Auxiliary = A;
	type State = S;
	type Error = Error;
}

#[derive(Encode, Decode)]
struct BlockData<B: Block, S> {
	block: B,
	state: S,
	depth: u64,
	children: Vec<B::Identifier>,
	is_canon: bool,
}

impl<'a, B: Block, A: Auxiliary<B>, S> ChainQuery for RocksSettlement<'a, B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
	S: Encode + Decode,
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

impl<'a, B: Block, A: Auxiliary<B>, S> ChainSettlement for RocksSettlement<'a, B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
	S: Encode + Decode,
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
			block, state, depth: depth as u64, children, is_canon
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

		let mut data = match self.backend.fetch_block_data(id) {
			Ok(data) => data,
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

		let mut data = match self.backend.fetch_block_data(id) {
			Ok(data) => data,
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

		self.changes.insert((COLUMN_INFO, KEY_HEAD.encode()), Some(head.encode()));
	}
}

impl<'a, B: Block, A: Auxiliary<B>, S> RocksSettlement<'a, B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
	S: Encode + Decode,
{
	fn set_genesis(
		&mut self,
		genesis: B::Identifier
	) {
		if self.last_error.is_some() {
			return
		}

		self.changes.insert((COLUMN_INFO, KEY_GENESIS.encode()), Some(genesis.encode()));
	}

	fn commit(self) -> Result<(), Error> {
		if let Some(error) = self.last_error {
			return Err(error)
		}

		let mut batch = WriteBatch::default();

		for ((column, key), value) in self.changes {
			let cf = self.backend.db.cf_handle(column).ok_or(Error::Corrupted)?;
			match value {
				Some(value) => {
					batch.put_cf(cf, key, value)?;
				},
				None => {
					batch.delete_cf(cf, key)?;
				},
			}
		}

		self.backend.db.write(batch)?;
		Ok(())
	}
}

impl<B: Block, A: Auxiliary<B>, S> ChainQuery for RocksBackend<B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
	S: Encode + Decode,
{
	fn head(&self) -> B::Identifier {
		let cf = self.db.cf_handle(COLUMN_INFO).unwrap();
		B::Identifier::decode(
			&mut self.db.get_cf(cf, KEY_HEAD.encode()).unwrap().unwrap().as_ref()
		).unwrap()
	}

	fn genesis(&self) -> B::Identifier {
		let cf = self.db.cf_handle(COLUMN_INFO).unwrap();
		B::Identifier::decode(
			&mut self.db.get_cf(cf, KEY_GENESIS.encode()).unwrap().unwrap().as_ref()
		).unwrap()
	}

	fn contains(
		&self,
		id: &B::Identifier
	) -> Result<bool, Error> {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).unwrap();
		let data = self.db.get_cf(cf, id.encode()).unwrap().map(
			|v| BlockData::<B, S>::decode(&mut v.as_ref()).unwrap()
		);
		Ok(data.is_some())
	}

	fn is_canon(
		&self,
		id: &B::Identifier
	) -> Result<bool, Error> {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).unwrap();
		let data = self.db.get_cf(cf, id.encode()).unwrap().map(
			|v| BlockData::<B, S>::decode(&mut v.as_ref()).unwrap()
		);
		Ok(data.ok_or(Error::NotExist)?.is_canon)
	}

	fn lookup_canon_depth(
		&self,
		depth: usize,
	) -> Result<Option<B::Identifier>, Error> {
		let depth = depth as u64;

		let cf = self.db.cf_handle(COLUMN_CANON_DEPTH_MAPPINGS).unwrap();
		let hash = self.db.get_cf(cf, depth.encode()).unwrap().map(
			|v| B::Identifier::decode(&mut v.as_ref()).unwrap()
		);
		Ok(hash)
	}

	fn auxiliary(
		&self,
		key: &A::Key
	) -> Result<Option<A>, Error> {
		let cf = self.db.cf_handle(COLUMN_AUXILIARIES).unwrap();
		let auxiliary = self.db.get_cf(cf, key.encode()).unwrap().map(
			|v| A::decode(&mut v.as_ref()).unwrap()
		);
		Ok(auxiliary)
	}

	fn children_at(
		&self,
		id: &B::Identifier,
	) -> Result<Vec<B::Identifier>, Error> {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).unwrap();
		let data = self.db.get_cf(cf, id.encode()).unwrap().map(
			|v| BlockData::<B, S>::decode(&mut v.as_ref()).unwrap()
		);
		Ok(data.ok_or(Error::NotExist)?.children)
	}

	fn depth_at(
		&self,
		id: &B::Identifier
	) -> Result<usize, Error> {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).unwrap();
		let data = self.db.get_cf(cf, id.encode()).unwrap().map(
			|v| BlockData::<B, S>::decode(&mut v.as_ref()).unwrap()
		);
		Ok(data.ok_or(Error::NotExist)?.depth as usize)
	}

	fn block_at(
		&self,
		id: &B::Identifier,
	) -> Result<B, Error> {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).unwrap();
		let data = self.db.get_cf(cf, id.encode()).unwrap().map(
			|v| BlockData::<B, S>::decode(&mut v.as_ref()).unwrap()
		);
		Ok(data.ok_or(Error::NotExist)?.block)
	}

	fn state_at(
		&self,
		id: &B::Identifier,
	) -> Result<Self::State, Error> {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).unwrap();
		let data = self.db.get_cf(cf, id.encode()).unwrap().map(
			|v| BlockData::<B, S>::decode(&mut v.as_ref()).unwrap()
		);
		Ok(data.ok_or(Error::NotExist)?.state)
	}
}

impl<B: Block, A: Auxiliary<B>, S> SharedCommittable for RocksBackend<B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
	S: Encode + Decode,
{
	type Operation = Operation<Self::Block, Self::State, Self::Auxiliary>;

	fn commit(
		&self,
		operation: Operation<Self::Block, Self::State, Self::Auxiliary>,
	) -> Result<(), Self::Error> {
		let mut settlement = RocksSettlement {
			backend: self,
			changes: Default::default(),
			last_error: None,
		};
		operation.settle(&mut settlement)?;
		settlement.commit()?;

		Ok(())
	}
}

impl<B: Block, A: Auxiliary<B>, S> RocksBackend<B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
	S: Encode + Decode,
{
	fn fetch_block_data(&self, id: B::Identifier) -> Result<BlockData<B, S>, Error> {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).ok_or(Error::Corrupted)?;
		let raw = match self.db.get_cf(cf, id.encode())? {
			Some(raw) => raw,
			None => return Err(Error::Corrupted),
		};
		Ok(BlockData::decode(&mut raw.as_ref()).ok_or(Error::Corrupted)?)
	}

	pub fn new_with_genesis<P: AsRef<Path>>(path: P, block: B, state: S) -> Self {
		assert!(block.parent_id().is_none(), "with_genesis must be provided with a genesis block");

		let db = Self::open(path);
		let genesis_id = block.id();

		let mut settlement = RocksSettlement {
			backend: &db,
			changes: Default::default(),
			last_error: None,
		};
		settlement.insert_block(
			genesis_id,
			block,
			state,
			0,
			Vec::new(),
			true
		);
		settlement.insert_canon_depth_mapping(0, genesis_id);
		settlement.set_genesis(genesis_id);
		settlement.set_head(genesis_id);
		settlement.commit().unwrap();

		db
	}

	pub fn from_existing<P: AsRef<Path>>(path: P) -> Self {
		Self::open(path)
	}
}
