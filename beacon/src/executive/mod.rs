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

mod helpers;
mod transition;
mod assignment;
mod choice;

pub use self::assignment::*;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use ssz::{Codec, Encode, Decode};
use bm_le::{IntoTree, FromTree, MaxVec};
use vecarray::VecArray;
use crate::*;
use crate::primitives::*;
use crate::types::*;
use crate::consts;

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Beacon state.
pub struct BeaconState<C: Config> {
	// == Versioning ==
	/// Genesis time as Unix timestamp.
	pub genesis_time: Uint,
	/// Current slot.
	pub slot: Uint,
	/// Fork version.
	pub fork: Fork,

	// == History ==
	/// Latest blokc header.
	pub latest_block_header: BeaconBlockHeader,
	/// Past block roots.
	pub block_roots: VecArray<H256, C::SlotsPerHistoricalRoot>,
	/// Past state roots.
	pub state_roots: VecArray<H256, C::SlotsPerHistoricalRoot>,
	/// Past historical roots.
	pub historical_roots: MaxVec<H256, C::HistoricalRootsLimit>,

	// == Eth1 ==
	/// Last accepted eth1 data.
	pub eth1_data: Eth1Data,
	/// Votes on eth1 data.
	pub eth1_data_votes: MaxVec<Eth1Data, C::SlotsPerEth1VotingPeriod>,
	/// Eth1 data deposit index.
	pub eth1_deposit_index: Uint,

	// == Registry ==
	/// Validator registry.
	pub validators: MaxVec<Validator, C::ValidatorRegistryLimit>,
	#[bm(compact)]
	/// Balance of the validators.
	pub balances: MaxVec<Uint, C::ValidatorRegistryLimit>,

	// == Shuffling ==
	/// Start shard for shuffling.
	pub start_shard: Uint,
	/// Past randao mixes.
	pub randao_mixes: VecArray<H256, C::EpochsPerHistoricalVector>,
	/// Past active index roots.
	pub active_index_roots: VecArray<H256, C::EpochsPerHistoricalVector>,
	/// Past compact committees roots.
	pub compact_committees_roots: VecArray<H256, C::EpochsPerHistoricalVector>,

	// == Slashings ==
	#[bm(compact)]
	/// Past slashings.
	pub slashings: VecArray<Uint, C::EpochsPerSlashingsVector>,

	// == Attestations ==
	/// Attestations on previous epoch.
	pub previous_epoch_attestations: MaxVec<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,
	/// Attestations on current epoch.
	pub current_epoch_attestations: MaxVec<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,

	// == Crosslinks ==
	/// Previous crosslinks.
	pub previous_crosslinks: VecArray<Crosslink, C::ShardCount>,
	/// Current crosslinks.
	pub current_crosslinks: VecArray<Crosslink, C::ShardCount>,

	// == Finality ==
	#[bm(compact)]
	#[cfg_attr(feature = "serde", serde(serialize_with = "crate::utils::serialize_bitvector"))]
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_bitvector"))]
	/// Justification bits for Casper.
	pub justification_bits: VecArray<bool, consts::JustificationBitsLength>,
	/// Previous justified checkpoint.
	pub previous_justified_checkpoint: Checkpoint,
	/// Current justified checkpoint.
	pub current_justified_checkpoint: Checkpoint,
	/// Last finalized checkpoint.
	pub finalized_checkpoint: Checkpoint,
}
