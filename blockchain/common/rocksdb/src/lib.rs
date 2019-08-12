// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.







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
