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
use crate::types::{Fork, Validator, BeaconBlockHeader, Eth1Data, Crosslink, PendingAttestation, CompactCommittee, Checkpoint};
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
	pub justification_bits: u32,
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
			genesis_time: Default::default(),
			slot: Default::default(),
			fork: Default::default(),

			latest_block_header: Default::default(),
			block_roots: fixed_vec(config.slots_per_historical_root()),
			state_roots: fixed_vec(config.slots_per_historical_root()),
			historical_roots: Default::default(),

			eth1_data: Default::default(),
			eth1_data_votes: Default::default(),
			eth1_deposit_index: Default::default(),

			validators: Default::default(),
			balances: Default::default(),

			start_shard: Default::default(),
			randao_mixes: fixed_vec(config.epochs_per_historical_vector()),
			active_index_roots: fixed_vec(config.epochs_per_historical_vector()),
			compact_committees_roots: fixed_vec(config.epochs_per_historical_vector()),

			slashings: fixed_vec(config.epochs_per_slashings_vector()),

			previous_epoch_attestations: Default::default(),
			current_epoch_attestations: Default::default(),

			previous_crosslinks: fixed_vec(config.shard_count()),
			current_crosslinks: fixed_vec(config.shard_count()),

			justification_bits: Default::default(),
			previous_justified_checkpoint: Default::default(),
			current_justified_checkpoint: Default::default(),
			finalized_checkpoint: Default::default(),
		}
	}

	/// Get validator public key.
	pub fn validator_pubkey(&self, index: u64) -> Option<ValidatorId> {
		if index as usize >= self.validators.len() {
			return None
		}

		let validator = &self.validators[index as usize];
		Some(validator.pubkey.clone())
	}

	/// Get validator index from public key.
	pub fn validator_index(&self, pubkey: &ValidatorId) -> Option<u64> {
		let validator_pubkeys = self.validators.iter()
			.map(|v| v.pubkey.clone()).collect::<Vec<_>>();
		validator_pubkeys.iter().position(|v| v == pubkey).map(|v| v as u64)
	}
}
