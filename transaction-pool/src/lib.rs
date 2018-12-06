// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

extern crate shasper_runtime as runtime;

extern crate substrate_primitives as primitives;
extern crate substrate_transaction_pool as transaction_pool;

pub type TransactionPool<B, E> = transaction_pool::txpool::Pool<ChainApi<B, E>>;

/// A simple transaction pool API that only allows one extrinsic in the pool at a given time.
pub type ChainApi<B, E> = transaction_pool::ChainApi<B, E>;
