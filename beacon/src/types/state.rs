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

//! Beacon state

use ssz_derive::Ssz;
#[cfg(feature = "serde")]
use serde_derive::{Serialize, Deserialize};
#[cfg(feature = "parity-codec")]
use codec::{Encode, Decode};

use crate::primitives::{Uint, H256, ValidatorId};
use crate::types::{Fork, Validator, BeaconBlockHeader, Eth1Data, Crosslink, PendingAttestation};
use crate::utils::fixed_vec;
use crate::Config;

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
#[ssz(no_decode, no_encode)]
/// Beacon state.
pub struct BeaconState {
	// == Versioning ==
	/// Genesis time.
	pub genesis_time: Uint,
	/// Current slot.
	pub slot: Uint,
	/// For versioning hard forks.
	pub fork: Fork,

	// == History ==
	/// Latest block header.
	pub latest_block_header: BeaconBlockHeader,
	#[ssz(use_fixed)]
	/// Latest block roots, of length `SLOTS_PER_HISTORICAL_ROOT`.
	pub block_roots: Vec<H256>,
	#[ssz(use_fixed)]
	/// Latest state roots, of length `SLOTS_PER_HISTORICAL_ROOT`.
	pub state_roots: Vec<H256>,
	/// Historical roots.
	pub historical_roots: Vec<H256>,

	// == Eth1 ==
	/// Latest eth1 data.
	pub eth1_data: Eth1Data,
	/// Eth1 data votes.
	pub eth1_data_votes: Vec<Eth1Data>,
	/// Deposit index.
	pub eth1_deposit_index: Uint,

	// == Validator registry ==
	/// Validator registry.
	pub validators: Vec<Validator>,
	/// Validator balances.
	pub balances: Vec<u64>,

	// == Randomness and committees ==
	/// Latest start shard.
	pub start_shard: Uint,
	#[ssz(use_fixed)]
	/// Latest randao mixes, of length `EPOCHS_PER_HISTORICAL_VECTOR`.
	pub randao_mixes: Vec<H256>,
	#[ssz(use_fixed)]
	/// Latest active index roots, of length `EPOCHS_PER_HISTORICAL_VECTOR`.
	pub active_index_roots: Vec<H256>,
	#[ssz(use_fixed)]
	/// Compact committees roots, of length `EPOCHS_PER_HISTORICAL_VECTOR`.
	pub compact_committees_roots: Vec<H256>,

	// == Slashings ==
	#[ssz(use_fixed)]
	/// Balances slashed at every withdrawal period, of length `EPOCHS_PER_SLASHINGS_VECTOR`.
	pub slashings: Vec<u64>,

	// == Attestations ==
	/// Previous epoch attestations.
	pub previous_epoch_attestations: Vec<PendingAttestation>,
	/// Current epoch attestations.
	pub current_epoch_attestations: Vec<PendingAttestation>,

	// == Crosslinks ==
	#[ssz(use_fixed)]
	/// Previous crosslinks, of length `SHARD_COUNT`.
	pub previous_crosslinks: Vec<Crosslink>,
	#[ssz(use_fixed)]
	/// Current crosslinks, of length `SHARD_COUNT`.
	pub current_crosslinks: Vec<Crosslink>,

	// == Finality ==
	/// Justification bits.
	pub justification_bits: Uint,
	/// Previous justified checkpoint.
	pub previous_justified_checkpoint: Checkpoint,
	/// Current justified checkpoint.
	pub current_justified_checkpoint: Checkpoint,
	/// Finalized checkpoint.
	pub finalized_checkpoint: Checkpoint,
}

impl BeaconState {
	/// Default value from config.
	pub fn default_with_config<C: Config>(config: &C) -> Self {
		Self {
			slot: Default::default(),
			genesis_time: Default::default(),
			fork: Default::default(),
			validator_registry: Default::default(),
			balances: Default::default(),
			latest_randao_mixes: fixed_vec(config.latest_randao_mixes_length()),
			latest_start_shard: Default::default(),
			previous_epoch_attestations: Default::default(),
			current_epoch_attestations: Default::default(),
			previous_justified_epoch: Default::default(),
			current_justified_epoch: Default::default(),
			previous_justified_root: Default::default(),
			current_justified_root: Default::default(),
			justification_bitfield: Default::default(),
			finalized_epoch: Default::default(),
			finalized_root: Default::default(),
			current_crosslinks: fixed_vec(config.shard_count()),
			previous_crosslinks: fixed_vec(config.shard_count()),
			latest_block_roots: fixed_vec(config.slots_per_historical_root()),
			latest_state_roots: fixed_vec(config.slots_per_historical_root()),
			latest_active_index_roots: fixed_vec(config.latest_active_index_roots_length()),
			latest_slashed_balances: fixed_vec(config.latest_slashed_exit_length()),
			latest_block_header: Default::default(),
			historical_roots: Default::default(),
			latest_eth1_data: Default::default(),
			eth1_data_votes: Default::default(),
			deposit_index: Default::default(),
		}
	}

	/// Get validator public key.
	pub fn validator_pubkey(&self, index: u64) -> Option<ValidatorId> {
		if index as usize >= self.validator_registry.len() {
			return None
		}

		let validator = &self.validator_registry[index as usize];
		Some(validator.pubkey.clone())
	}

	/// Get validator index from public key.
	pub fn validator_index(&self, pubkey: &ValidatorId) -> Option<u64> {
		let validator_pubkeys = self.validator_registry.iter()
			.map(|v| v.pubkey.clone()).collect::<Vec<_>>();
		validator_pubkeys.iter().position(|v| v == pubkey).map(|v| v as u64)
	}
}
