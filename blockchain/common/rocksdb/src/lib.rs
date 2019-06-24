mod utils;
mod settlement;
mod backend;

pub use self::backend::RocksBackend;

use std::{fmt, error as stderror};
use std::sync::Arc;
use parity_codec::{Encode, Decode};
use rocksdb::DB;
use blockchain::backend::OperationError;

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

pub trait RocksState {
	type Raw: Encode + Decode;

	fn from_raw(raw: Self::Raw, db: Arc<DB>) -> Self;
	fn into_raw(self) -> Self::Raw;
}
