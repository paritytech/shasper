use core::marker::PhantomData;
use std::path::Path;
use std::sync::{Arc, RwLock};
use blockchain::traits::{Block, Auxiliary};
use blockchain::backend::{Store, ChainQuery, SharedCommittable, ChainSettlement, Operation};
use parity_codec::{Encode, Decode};
use rocksdb::{DB, Options};

use super::Error;
use super::settlement::RocksSettlement;
use super::utils::*;

pub struct RocksBackend<B: Block, A: Auxiliary<B>, S> {
	db: Arc<DB>,
	head: Arc<RwLock<B::Identifier>>,
	genesis: Arc<B::Identifier>,
	_marker: PhantomData<(B, A, S)>,
}

impl<B: Block, A: Auxiliary<B>, S> RocksBackend<B, A, S> where
	B::Identifier: Decode
{
	fn options() -> Options {
		let mut db_opts = Options::default();
		db_opts.create_missing_column_families(true);
		db_opts.create_if_missing(true);

		db_opts
	}
}

impl<B: Block, A: Auxiliary<B>, S> Clone for RocksBackend<B, A, S> {
	fn clone(&self) -> Self {
		Self {
			db: self.db.clone(),
			head: self.head.clone(),
			genesis: self.genesis.clone(),
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

impl<B: Block, A: Auxiliary<B>, S> ChainQuery for RocksBackend<B, A, S> where
	B::Identifier: Encode + Decode,
	B: Encode + Decode,
	A: Encode + Decode,
	A::Key: Encode + Decode,
	S: Encode + Decode,
{
	fn head(&self) -> B::Identifier {
		*self.head.read().expect("Lock is poisoned")
	}

	fn genesis(&self) -> B::Identifier {
		*self.genesis.as_ref()
	}

	fn contains(
		&self,
		id: &B::Identifier
	) -> Result<bool, Error> {
		Ok(fetch_block_data::<B, S>(&self.db, id)?.is_some())
	}

	fn is_canon(
		&self,
		id: &B::Identifier
	) -> Result<bool, Error> {
		Ok(fetch_block_data::<B, S>(&self.db, id)?.ok_or(Error::NotExist)?.is_canon)
	}

	fn lookup_canon_depth(
		&self,
		depth: usize,
	) -> Result<Option<B::Identifier>, Error> {
		let depth = depth as u64;

		let cf = self.db.cf_handle(COLUMN_CANON_DEPTH_MAPPINGS).ok_or(Error::Corrupted)?;
		match self.db.get_cf(cf, depth.encode())? {
			Some(hash) => Ok(Some(B::Identifier::decode(&mut hash.as_ref()).ok_or(Error::Corrupted)?)),
			None => Ok(None),
		}
	}

	fn auxiliary(
		&self,
		key: &A::Key
	) -> Result<Option<A>, Error> {
		let cf = self.db.cf_handle(COLUMN_AUXILIARIES).ok_or(Error::Corrupted)?;
		match self.db.get_cf(cf, key.encode())? {
			Some(v) => Ok(Some(A::decode(&mut v.as_ref()).ok_or(Error::Corrupted)?)),
			None => Ok(None),
		}
	}

	fn children_at(
		&self,
		id: &B::Identifier,
	) -> Result<Vec<B::Identifier>, Error> {
		Ok(fetch_block_data::<B, S>(&self.db, id)?.ok_or(Error::NotExist)?.children)
	}

	fn depth_at(
		&self,
		id: &B::Identifier
	) -> Result<usize, Error> {
		Ok(fetch_block_data::<B, S>(&self.db, id)?.ok_or(Error::NotExist)?.depth as usize)
	}

	fn block_at(
		&self,
		id: &B::Identifier,
	) -> Result<B, Error> {
		Ok(fetch_block_data::<B, S>(&self.db, id)?.ok_or(Error::NotExist)?.block)
	}

	fn state_at(
		&self,
		id: &B::Identifier,
	) -> Result<Self::State, Error> {
		Ok(fetch_block_data::<B, S>(&self.db, id)?.ok_or(Error::NotExist)?.state)
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
		let mut settlement = RocksSettlement::new(self);
		operation.settle(&mut settlement)?;

		let mut head = self.head.write().expect("Lock is poisoned");
		let new_head = settlement.commit()?;

		if let Some(new_head) = new_head {
			*head = new_head;
		}

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
	pub fn open_or_create<P: AsRef<Path>, F>(path: P, f: F) -> Result<Self, Error> where
		F: FnOnce() -> Result<(B, S), Error>
	{
		let db_opts = Self::options();
		let db = DB::open_cf(&db_opts, path, &[
			COLUMN_BLOCKS, COLUMN_CANON_DEPTH_MAPPINGS, COLUMN_AUXILIARIES, COLUMN_INFO,
		])?;

		let head = fetch_head(&db)?;
		let genesis = fetch_genesis(&db)?;

		match (head, genesis) {
			(Some(head), Some(genesis)) => {
				Ok(Self {
					db: Arc::new(db),
					head: Arc::new(RwLock::new(head)),
					genesis: Arc::new(genesis),
					_marker: PhantomData,
				})
			},
			(None, None) => {
				let (block, state) = f()?;
				assert!(block.parent_id().is_none(),
						"with_genesis must be provided with a genesis block");

				let head = block.id();
				let genesis = head;

				let backend = Self {
					db: Arc::new(db),
					head: Arc::new(RwLock::new(head)),
					genesis: Arc::new(genesis),
					_marker: PhantomData,
				};

				let mut settlement = RocksSettlement::new(&backend);
				settlement.insert_block(
					genesis,
					block,
					state,
					0,
					Vec::new(),
					true
				);
				settlement.insert_canon_depth_mapping(0, genesis);
				settlement.set_genesis(genesis);
				settlement.set_head(genesis);
				settlement.commit()?;

				Ok(backend)
			},
			_ => Err(Error::Corrupted),
		}
	}

	pub fn new_with_genesis<P: AsRef<Path>>(path: P, block: B, state: S) -> Result<Self, Error> {
		let mut created = false;
		let backend = Self::open_or_create(path, || {
			created = true;
			Ok((block, state))
		})?;
		if !created {
			return Err(Error::Corrupted);
		}
		Ok(backend)
	}

	pub fn from_existing<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
		Self::open_or_create(path, || Err(Error::Corrupted))
	}

	pub(crate) fn db(&self) -> &DB {
		self.db.as_ref()
	}
}
