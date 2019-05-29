use core::marker::PhantomData;
use std::path::Path;
use std::{fmt, error as stderror};
use std::sync::Arc;
use blockchain::traits::{Block, Auxiliary, Backend, ChainQuery};
use blockchain::backend::SharedDatabase;
use parity_codec::{Encode, Decode};
use rocksdb::{DB, Options, WriteOptions};

const COLUMN_BLOCKS: &str = "blocks";
const COLUMN_CANON_DEPTH_MAPPINGS: &str = "canon_depth_mappings";
const COLUMN_AUXILIARIES: &str = "auxiliaries";
const COLUMN_INFO: &str = "info";
const KEY_HEAD: &str = "head";
const KEY_GENESIS: &str = "genesis";

#[derive(Debug)]
pub enum Error {
	NotExist,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl stderror::Error for Error { }

pub struct RocksDatabase<B: Block, A: Auxiliary<B>, S> {
	db: Arc<DB>,
	_marker: PhantomData<(B, A, S)>,
}

impl<B: Block, A: Auxiliary<B>, S> RocksDatabase<B, A, S> {
	pub fn open<P: AsRef<Path>>(path: P) -> Self {
		let db = DB::open_cf(&Options::default(), path, &[
			COLUMN_BLOCKS, COLUMN_CANON_DEPTH_MAPPINGS, COLUMN_AUXILIARIES, COLUMN_INFO,
		]).unwrap();

		Self {
			db: Arc::new(db),
			_marker: PhantomData,
		}
	}
}

impl<B: Block, A: Auxiliary<B>, S> Clone for RocksDatabase<B, A, S> {
	fn clone(&self) -> Self {
		Self {
			db: self.db.clone(),
			_marker: PhantomData,
		}
	}
}

impl<B: Block, A: Auxiliary<B>, S> Backend for RocksDatabase<B, A, S> {
	type Block = B;
	type Auxiliary = A;
	type State = S;
	type Error = Error;
}

#[derive(Encode, Decode)]
struct BlockData<B: Block, S> {
	block: B,
	state: S,
	depth: usize,
	children: Vec<B::Identifier>,
	is_canon: bool,
}

impl<B: Block, A: Auxiliary<B>, S> SharedDatabase for RocksDatabase<B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
	S: Encode + Decode,
{
	fn insert_block(
		&self,
		id: <Self::Block as Block>::Identifier,
		block: Self::Block,
		state: Self::State,
		depth: usize,
		children: Vec<<Self::Block as Block>::Identifier>,
		is_canon: bool
	) {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).unwrap();
		self.db.put_cf_opt(cf, id.encode(), BlockData {
			block, state, depth, children, is_canon
		}.encode(), &WriteOptions::default()).unwrap();
	}
	fn push_child(
		&self,
		id: <Self::Block as Block>::Identifier,
		child: <Self::Block as Block>::Identifier,
	) {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).unwrap();
		let mut data = BlockData::<B, S>::decode(
			&mut self.db.get_cf(cf, id.encode()).unwrap().unwrap().as_ref()
		).unwrap();
		data.children.push(child);
		self.db.put_cf_opt(cf, id.encode(), data.encode(), &WriteOptions::default()).unwrap();
	}
	fn set_canon(
		&self,
		id: <Self::Block as Block>::Identifier,
		is_canon: bool
	) {
		let cf = self.db.cf_handle(COLUMN_BLOCKS).unwrap();
		let mut data = BlockData::<B, S>::decode(
			&mut self.db.get_cf(cf, id.encode()).unwrap().unwrap().as_ref()
		).unwrap();
		data.is_canon = is_canon;
		self.db.put_cf_opt(cf, id.encode(), data.encode(), &WriteOptions::default()).unwrap();
	}
	fn insert_canon_depth_mapping(
		&self,
		depth: usize,
		id: <Self::Block as Block>::Identifier,
	) {
		let depth = depth as u64;

		let cf = self.db.cf_handle(COLUMN_CANON_DEPTH_MAPPINGS).unwrap();
		self.db.put_cf_opt(cf, depth.encode(), id.encode(), &WriteOptions::default()).unwrap();
	}
	fn remove_canon_depth_mapping(
		&self,
		depth: &usize
	) {
		let depth = *depth as u64;

		let cf = self.db.cf_handle(COLUMN_CANON_DEPTH_MAPPINGS).unwrap();
		self.db.delete_cf_opt(cf, depth.encode(), &WriteOptions::default()).unwrap();
	}
	fn insert_auxiliary(
		&self,
		key: <Self::Auxiliary as Auxiliary<Self::Block>>::Key,
		value: Self::Auxiliary
	) {
		let cf = self.db.cf_handle(COLUMN_AUXILIARIES).unwrap();
		self.db.put_cf_opt(cf, key.encode(), value.encode(), &WriteOptions::default()).unwrap();
	}
	fn remove_auxiliary(
		&self,
		key: &<Self::Auxiliary as Auxiliary<Self::Block>>::Key,
	) {
		let cf = self.db.cf_handle(COLUMN_AUXILIARIES).unwrap();
		self.db.delete_cf_opt(cf, key.encode(), &WriteOptions::default()).unwrap();
	}
	fn set_head(
		&self,
		head: <Self::Block as Block>::Identifier
	) {
		let cf = self.db.cf_handle(COLUMN_INFO).unwrap();
		self.db.put_cf_opt(cf, KEY_HEAD.encode(), head.encode(), &WriteOptions::default()).unwrap();
	}
}

impl<B: Block, A: Auxiliary<B>, S> ChainQuery for RocksDatabase<B, A, S> where
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
		Ok(data.ok_or(Error::NotExist)?.depth)
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
