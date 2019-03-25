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

use primitives::{H256, ValidatorId, Version};
use ssz_derive::Ssz;
use serde_derive::{Serialize, Deserialize};
use crate::{Slot, Epoch, Timestamp, ValidatorIndex, Shard};
use crate::eth1::{Eth1Data, Eth1DataVote};
use crate::attestation::{
	PendingAttestation, Crosslink,
};
use crate::validator::Validator;
use crate::block::BeaconBlockHeader;

#[derive(Ssz, Clone, Eq, PartialEq)]
#[ssz(no_decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct BeaconState {
	// Misc
	pub slot: Slot,
	pub genesis_time: Timestamp,
	pub fork: Fork, // For versioning hard forks

	// Validator registry
	pub validator_registry: Vec<Validator>,
	pub validator_balances: Vec<u64>,
	pub validator_registry_update_epoch: Epoch,

	// Randomness and committees
	#[ssz(use_fixed)]
	pub latest_randao_mixes: Vec<H256>, //; LATEST_RANDAO_MIXES_LENGTH],
	pub previous_shuffling_start_shard: Shard,
	pub current_shuffling_start_shard: Shard,
	pub previous_shuffling_epoch: Epoch,
	pub current_shuffling_epoch: Epoch,
	pub previous_shuffling_seed: H256,
	pub current_shuffling_seed: H256,

	// Finality
	pub previous_epoch_attestations: Vec<PendingAttestation>,
	pub current_epoch_attestations: Vec<PendingAttestation>,
	pub previous_justified_epoch: Epoch,
	pub current_justified_epoch: Epoch,
	pub previous_justified_root: H256,
	pub current_justified_root: H256,
	pub justification_bitfield: u64,
	pub finalized_epoch: Epoch,
	pub finalized_root: H256,

	// Recent state
	#[ssz(use_fixed)]
	pub latest_crosslinks: Vec<Crosslink>, //; SHARD_COUNT],
	#[ssz(use_fixed)]
	pub latest_block_roots: Vec<H256>, //; SLOTS_PER_HISTORICAL_ROOT],
	#[ssz(use_fixed)]
	pub latest_state_roots: Vec<H256>, //; SLOTS_PER_HISTORICAL_ROOT],
	#[ssz(use_fixed)]
	pub latest_active_index_roots: Vec<H256>, //; LATEST_ACTIVE_INDEX_ROOTS_LENGTH],
	#[ssz(use_fixed)]
	pub latest_slashed_balances: Vec<u64>, //; LATEST_SLASHED_EXIT_LENGTH], // Balances slashed at every withdrawal period
	pub latest_block_header: BeaconBlockHeader,
	pub historical_roots: Vec<H256>,

	// Ethereum 1.0 chain data
	pub latest_eth1_data: Eth1Data,
	pub eth1_data_votes: Vec<Eth1DataVote>,
	pub deposit_index: u64,
}

#[derive(Ssz, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
#[ssz(no_decode)]
pub struct HistoricalBatch {
	/// Block roots
	#[ssz(use_fixed)]
	pub block_roots: Vec<H256>, //; SLOTS_PER_HISTORICAL_ROOT],
	/// State roots
	#[ssz(use_fixed)]
	pub state_roots: Vec<H256>, //; SLOTS_PER_HISTORICAL_ROOT],
}

#[derive(Ssz, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct Fork {
	/// Previous fork version
	pub previous_version: Version,
	/// Current fork version
	pub current_version: Version,
	/// Fork epoch number
	pub epoch: u64,
}

impl BeaconState {
	pub fn validator_index_by_id(&self, validator_id: &ValidatorId) -> Option<ValidatorIndex> {
		for (i, validator) in self.validator_registry.iter().enumerate() {
			if &validator.pubkey == validator_id {
				return Some(i as u64)
			}
		}

		None
	}

	pub fn active_validator_indices(&self, epoch: Epoch) -> Vec<ValidatorIndex> {
		self.validator_registry.iter()
			.enumerate()
			.filter(|(_, v)| v.is_active(epoch))
			.map(|(i, _)| i as u64)
			.collect::<Vec<_>>()
	}
}

#[cfg(test)]
mod tests {
	use ssz::Hashable;

	use super::*;
	use crate::{Config, NoVerificationConfig, genesis_state};

	#[test]
	#[ignore]
	fn test_empty_genesis_block() {
		let config = NoVerificationConfig::small();
		let state = genesis_state(Default::default(), 0, Eth1Data {
			block_hash: Default::default(),
			// deposit_count: 0,
			deposit_root: Default::default(),
		}, &config).unwrap();
		assert_eq!(state.current_shuffling_seed.as_ref(), &b">\r\xc3\xf3\x1a\xdd\xb2\x7fu)\xfa1,\\s'=\xf2\xe1\xddZ\xfcW2\xdf\xe1\x83W\x11\xfc[\x95"[..]);
		assert_eq!(state.latest_block_header.block_body_root.as_ref(), &b"\xd8\xe5\xbaa\xfc\x87\xc2\x8c\xd7\xe6V\x8fl\xa1\xc0\xfd\x03\x18\xca\xd76V\xe6ti\x85I\xc4\x86L\xda#"[..]);
		assert_eq!(state.hash::<<NoVerificationConfig as Config>::Hasher>().as_ref(), &b"\xc8\xcc\x03\x8ah7\xb3l\xc6rD$\x8b\x91/\xf9\x03\xe1\xcb%\x1f)\x8fj.\xba\xc540\xdaq\x85"[..]);
	}
}
