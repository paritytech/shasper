mod helpers;
mod transition;

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
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BeaconState<C: Config> {
	// == Versioning ==
	pub genesis_time: Uint,
	pub slot: Uint,
	pub fork: Fork,

	// == History ==
	pub latest_block_header: BeaconBlockHeader,
	pub block_roots: VecArray<H256, C::SlotsPerHistoricalRoot>,
	pub state_roots: VecArray<H256, C::SlotsPerHistoricalRoot>,
	pub historical_roots: MaxVec<H256, C::HistoricalRootsLimit>,

	// == Eth1 ==
	pub eth1_data: Eth1Data,
	pub eth1_data_votes: MaxVec<Eth1Data, C::SlotsPerEth1VotingPeriod>,
	pub eth1_deposit_index: Uint,

	// == Registry ==
	pub validators: MaxVec<Validator, C::ValidatorRegistryLimit>,
	#[bm(compact)]
	pub balances: MaxVec<Uint, C::ValidatorRegistryLimit>,

	// == Shuffling ==
	pub start_shard: Uint,
	pub randao_mixes: VecArray<H256, C::EpochsPerHistoricalVector>,
	pub active_index_roots: VecArray<H256, C::EpochsPerHistoricalVector>,
	pub compact_committees_roots: VecArray<H256, C::EpochsPerHistoricalVector>,

	// == Slashings ==
	#[bm(compact)]
	pub slashings: VecArray<Uint, C::EpochsPerSlashingsVector>,

	// == Attestations ==
	pub previous_epoch_attestations: MaxVec<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,
	pub current_epoch_attestations: MaxVec<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,

	// == Crosslinks ==
	pub previous_crosslinks: VecArray<Crosslink, C::ShardCount>,
	pub current_crosslinks: VecArray<Crosslink, C::ShardCount>,

	// == Finality ==
	#[bm(compact)]
	#[cfg_attr(feature = "serde", serde(serialize_with = "crate::utils::serialize_bitvector"))]
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_bitvector"))]
	pub justification_bits: VecArray<bool, consts::JustificationBitsLength>,
	pub previous_justified_checkpoint: Checkpoint,
	pub current_justified_checkpoint: Checkpoint,
	pub finalized_checkpoint: Checkpoint,
}
