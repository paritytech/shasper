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

use crate::Config;

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct Eth1Data {
	/// Root of the deposit tree
	pub deposit_root: H256,
	/// Total number of deposits
	// TODO: this field is not present in current test spec.
	// pub deposit_count: u64,
	/// Block hash
	pub block_hash: H256,
}

impl Eth1Data {
	pub fn empty() -> Self {
		Self {
			deposit_root: H256::default(),
			// TODO: this field is not present in current test spec.
			// deposit_count: 0,
			block_hash: H256::default(),
		}
	}
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct Eth1DataVote {
	/// Data being voted for
	pub eth1_data: Eth1Data,
	/// Vote count
	pub vote_count: u64,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
#[ssz(no_decode)]
pub struct Deposit {
	/// Branch in the deposit tree
	#[ssz(use_fixed)]
	pub proof: Vec<H256>,
	/// Index in the deposit tree
	pub index: u64,
	/// Data
	pub deposit_data: DepositData,
}

impl Deposit {
	pub fn is_merkle_valid<C: Config>(&self, deposit_root: &H256, config: &C) -> bool {
		let merkle = MerkleProof {
			leaf: config.hash(&self.deposit_data.encode()),
			proof: self.proof.as_ref(),
			depth: config.deposit_contract_tree_depth(),
			index: self.index as usize,
			root: *deposit_root,
		};

		merkle.is_valid(config)
	}

	pub fn is_proof_valid<C: Config>(&self, domain: u64, config: &C) -> bool {
		config.bls_verify(
			&self.deposit_data.deposit_input.pubkey,
			&self.deposit_data.deposit_input.truncated_hash::<C::Hasher>(),
			&self.deposit_data.deposit_input.proof_of_possession,
			domain
		)
	}
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct DepositData {
	/// Amount in Gwei
	pub amount: u64,
	/// Timestamp from deposit contract
	pub timestamp: u64,
	/// Deposit input
	pub deposit_input: DepositInput,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct DepositInput {
	/// BLS pubkey
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// A BLS signature of this `DepositInput`
	pub proof_of_possession: Signature,
}

pub struct MerkleProof<'a> {
	pub leaf: H256,
	pub proof: &'a [H256],
	pub root: H256,
	pub depth: usize,
	pub index: usize,
}

impl<'a> MerkleProof<'a> {
	pub fn is_valid<C: Config>(&self, config: &C) -> bool {
		if self.proof.len() != config.deposit_contract_tree_depth() {
			return false
		}

		let mut value = self.leaf;
		for i in 0..self.depth {
			if (self.index / 2usize.pow(i as u32)) % 2 != 0 {
				value = config.hash2(self.proof[i].as_ref(), value.as_ref());
			} else {
				value = config.hash2(value.as_ref(), self.proof[i].as_ref());
			}
		}

		value == self.root
	}
}
