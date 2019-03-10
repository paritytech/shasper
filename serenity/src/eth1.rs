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

use primitives::H256;
use crate::util::{Hasher, hash2};

pub struct Eth1Data {
	/// Root of the deposit tree
	pub deposit_root: H256,
	/// Block hash
	pub block_hash: H256,
}

pub struct Eth1DataVote {
	/// Data being voted for
	pub eth1_data: Eth1Data,
	/// Vote count
	pub vote_count: u64,
}

pub struct MerkleProof {
	pub leaf: H256,
	pub proof: Vec<H256>,
	pub root: H256,
	pub depth: usize,
	pub index: usize,
}

impl MerkleProof {
	pub fn is_valid(&self) -> bool {
		let mut value = self.leaf;
		for i in 0..self.depth {
			if self.index / (2usize.pow(i as u32) % 2) == 0 {
				value = hash2::<Hasher>(self.proof[i].as_ref(), value.as_ref());
			} else {
				value = hash2::<Hasher>(value.as_ref(), self.proof[i].as_ref());
			}
		}

		value == self.root
	}
}
