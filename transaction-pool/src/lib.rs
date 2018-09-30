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

use primitives::H256;
use runtime::{Extrinsic, Block, BlockId};
use transaction_pool::{VerifiedFor, Readiness, ExtrinsicFor};
use transaction_pool::scoring::{Change, Choice};

use std::cmp::Ordering;

pub type TransactionPool = transaction_pool::Pool<ChainApi>;

#[derive(Debug, Clone)]
pub struct Verified {
	pub extrinsic: Extrinsic,
	hash: H256,
	sender: H256
}

impl transaction_pool::VerifiedTransaction for Verified {
	type Hash = H256;
	type Sender = H256;

	fn hash(&self) -> &H256 { &self.hash }
	fn sender(&self) -> &H256 { &self.sender }
	fn mem_usage(&self) -> usize { 0 }
}

/// A simple transaction pool API that only allows one extrinsic in the pool at a given time.
pub struct ChainApi;

impl transaction_pool::ChainApi for ChainApi {
	type Block = Block;
	type Hash = H256;
	type Sender = H256;
	type VEx = Verified;
	type Ready = ();
	type Error = transaction_pool::Error;
	type Score = u8;
	type Event = ();

	fn verify_transaction(&self, _at: &BlockId, xt: &ExtrinsicFor<Self>) -> Result<Self::VEx, Self::Error> {
		Ok(Verified {
			extrinsic: xt.clone(),
			hash: H256::new(),
			sender: H256::new()
		})
	}

	fn ready(&self) -> Self::Ready { () }
	fn is_ready(&self, _at: &BlockId, _known_nonces: &mut (), _xt: &VerifiedFor<Self>) -> Readiness { Readiness::Ready }
	fn compare(_old: &VerifiedFor<Self>, _other: &VerifiedFor<Self>) -> Ordering { Ordering::Equal }
	fn choose(_old: &VerifiedFor<Self>, _new: &VerifiedFor<Self>) -> Choice { Choice::ReplaceOld }
	fn update_scores(
		_xts: &[transaction_pool::Transaction<VerifiedFor<Self>>],
		_scores: &mut [Self::Score],
		_change: Change<()>
	) { }
	fn should_replace(_old: &VerifiedFor<Self>, _new: &VerifiedFor<Self>) -> Choice { Choice::ReplaceOld }
}
