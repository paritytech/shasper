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

use primitives::{H256, ValidatorId, Signature};
use ssz::{Hashable, Encode};
use ssz_derive::Ssz;
use serde_derive::{Serialize, Deserialize};
use crate::consts::DEPOSIT_CONTRACT_TREE_DEPTH;
use crate::util::{Hasher, hash, hash2, bls_verify};

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct Eth1Data {
	/// Root of the deposit tree
	pub deposit_root: H256,
	/// Block hash
	pub block_hash: H256,
}

impl Eth1Data {
	pub fn empty() -> Self {
		Self {
			deposit_root: H256::default(),
			block_hash: H256::default(),
		}
	}
}

#[derive(Ssz)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct Eth1DataVote {
	/// Data being voted for
	pub eth1_data: Eth1Data,
	/// Vote count
	pub vote_count: u64,
}

#[derive(Ssz, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct Deposit {
	/// Branch in the deposit tree
	pub proof: [H256; DEPOSIT_CONTRACT_TREE_DEPTH],
	/// Index in the deposit tree
	pub index: u64,
	/// Data
	pub deposit_data: DepositData,
}

impl Deposit {
	pub fn is_merkle_valid(&self, deposit_root: &H256) -> bool {
		let merkle = MerkleProof {
			leaf: hash(&self.deposit_data.encode()),
			proof: self.proof,
			depth: DEPOSIT_CONTRACT_TREE_DEPTH,
			index: self.index as usize,
			root: *deposit_root,
		};

		merkle.is_valid()
	}

	pub fn is_proof_valid(&self, domain: u64) -> bool {
		bls_verify(
			&self.deposit_data.deposit_input.pubkey,
			&self.deposit_data.deposit_input.truncated_hash::<Hasher>(),
			&self.deposit_data.deposit_input.proof_of_possession,
			domain
		)
	}
}

#[derive(Ssz, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct DepositData {
	/// Amount in Gwei
	pub amount: u64,
	/// Timestamp from deposit contract
	pub timestamp: u64,
	/// Deposit input
	pub deposit_input: DepositInput,
}

#[derive(Ssz, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct DepositInput {
	/// BLS pubkey
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// A BLS signature of this `DepositInput`
	pub proof_of_possession: Signature,
}

pub struct MerkleProof {
	pub leaf: H256,
	pub proof: [H256; DEPOSIT_CONTRACT_TREE_DEPTH],
	pub root: H256,
	pub depth: usize,
	pub index: usize,
}

impl MerkleProof {
	pub fn is_valid(&self) -> bool {
		let mut value = self.leaf;
		for i in 0..self.depth {
			if self.index / (2usize.pow(i as u32) % 2) == 0 {
				value = hash2(self.proof[i].as_ref(), value.as_ref());
			} else {
				value = hash2(value.as_ref(), self.proof[i].as_ref());
			}
		}

		value == self.root
	}
}
