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

use ssz_derive::Ssz;

#[cfg(feature = "serde")]
use serde_derive::{Serialize, Deserialize};
#[cfg(feature = "parity-codec")]
use codec::{Encode, Decode};

use crate::{Slot, Epoch, Timestamp, ValidatorIndex, Shard};
use crate::primitives::{H256, ValidatorId, Version};
use crate::eth1::{Eth1Data, Eth1DataVote};
use crate::attestation::{
	PendingAttestation, Crosslink,
};
use crate::validator::Validator;
use crate::block::BeaconBlockHeader;

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
#[ssz(no_decode, no_encode)]
/// Beacon state.
pub struct BeaconState {
	// Misc
	/// Current slot.
	pub slot: Slot,
	/// Genesis time.
	pub genesis_time: Timestamp,
	/// For versioning hard forks.
	pub fork: Fork,

	/// Validator registry.
	pub validator_registry: Vec<Validator>,
	/// Validator balances.
	pub validator_balances: Vec<u64>,
	/// Last validator registry update epoch.
	pub validator_registry_update_epoch: Epoch,

	// Randomness and committees
	#[ssz(use_fixed)]
	/// Latest randao mixes, of length `LATEST_RANDAO_MIXES_LENGTH`.
	pub latest_randao_mixes: Vec<H256>,
	/// Previous shuffling start shard.
	pub previous_shuffling_start_shard: Shard,
	/// Current shuffling start shard.
	pub current_shuffling_start_shard: Shard,
	/// Previous shuffling epoch.
	pub previous_shuffling_epoch: Epoch,
	/// Current shuffling epoch.
	pub current_shuffling_epoch: Epoch,
	/// Previous shuffling seed.
	pub previous_shuffling_seed: H256,
	/// Current shuffling seed.
	pub current_shuffling_seed: H256,

	// Finality
	/// Previous epoch attestations.
	pub previous_epoch_attestations: Vec<PendingAttestation>,
	/// Current epoch attestations.
	pub current_epoch_attestations: Vec<PendingAttestation>,
	/// Previous justified epoch.
	pub previous_justified_epoch: Epoch,
	/// Current justified epoch.
	pub current_justified_epoch: Epoch,
	/// Previous justified root.
	pub previous_justified_root: H256,
	/// Current justified root.
	pub current_justified_root: H256,
	/// Justification bitfield.
	pub justification_bitfield: u64,
	/// Finalized epoch.
	pub finalized_epoch: Epoch,
	/// Finalized root.
	pub finalized_root: H256,

	// Recent state
	#[ssz(use_fixed)]
	/// Latest crosslinks, of length `SHARD_COUNT`.
	pub latest_crosslinks: Vec<Crosslink>,
	#[ssz(use_fixed)]
	/// Latest block roots, of length `SLOTS_PER_HISTORICAL_ROOT`.
	pub latest_block_roots: Vec<H256>,
	#[ssz(use_fixed)]
	/// Latest state roots, of length `SLOTS_PER_HISTORICAL_ROOT`.
	pub latest_state_roots: Vec<H256>,
	#[ssz(use_fixed)]
	/// Latest active index roots, of length `LATEST_ACTIVE_INDEX_ROOTS_LENGTH`.
	pub latest_active_index_roots: Vec<H256>,
	#[ssz(use_fixed)]
	/// Balances slashed at every withdrawal period, of length `LATEST_SLASHED_EXIT_LENGTH`.
	pub latest_slashed_balances: Vec<u64>,
	/// Latest block header.
	pub latest_block_header: BeaconBlockHeader,
	/// Historical roots.
	pub historical_roots: Vec<H256>,

	// Ethereum 1.0 chain data
	/// Latest eth1 data.
	pub latest_eth1_data: Eth1Data,
	/// Eth1 data votes.
	pub eth1_data_votes: Vec<Eth1DataVote>,
	/// Deposit index.
	pub deposit_index: u64,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
#[ssz(no_decode)]
/// Historical batch information.
pub struct HistoricalBatch {
	/// Block roots
	#[ssz(use_fixed)]
	pub block_roots: Vec<H256>, //; SLOTS_PER_HISTORICAL_ROOT],
	/// State roots
	#[ssz(use_fixed)]
	pub state_roots: Vec<H256>, //; SLOTS_PER_HISTORICAL_ROOT],
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Fork information.
pub struct Fork {
	/// Previous fork version
	pub previous_version: Version,
	/// Current fork version
	pub current_version: Version,
	/// Fork epoch number
	pub epoch: u64,
}

impl BeaconState {
	/// Get validator index from validator ID.
	pub fn validator_index_by_id(&self, validator_id: &ValidatorId) -> Option<ValidatorIndex> {
		for (i, validator) in self.validator_registry.iter().enumerate() {
			if &validator.pubkey == validator_id {
				return Some(i as u64)
			}
		}

		None
	}

	/// Get active validator indices for given epoch.
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
	fn test_empty_genesis_block() {
		let config = NoVerificationConfig::small();
		let state = genesis_state(Default::default(), 0, Eth1Data {
			block_hash: Default::default(),
			// deposit_count: 0,
			deposit_root: Default::default(),
		}, &config).unwrap();
		assert_eq!(state.current_shuffling_seed.as_ref(), &b">\r\xc3\xf3\x1a\xdd\xb2\x7fu)\xfa1,\\s'=\xf2\xe1\xddZ\xfcW2\xdf\xe1\x83W\x11\xfc[\x95"[..]);
		assert_eq!(state.latest_block_header.block_body_root.as_ref(), &b"\x13\xf2\x00\x1f\xf0\xeeJR\x8b<C\xf6=p\xa9\x97\xae\xfc\xa9\x90\xed\x8e\xad\xa2\">\xe6\xec8\x07\xf7\xcc"[..]);
		assert_eq!(Hashable::<<NoVerificationConfig as Config>::Hasher>::hash(&state).as_ref(), &b"\x9c\xe3*1\xe7\xad\xc1N\xb4Y0=&\xad\xc9\x04\x8d\xa7\xc2<__\x7f\x1e\xd8.a&\x94\x9dH\xff"[..]);
	}
}
