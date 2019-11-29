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
mod choice;
mod assignment;

pub use self::assignment::CommitteeAssignment;

use core::ops::Deref;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use ssz::{Codec, Encode, Decode};
use bm_le::{IntoTree, FromTree, MaxVec};
use vecarray::VecArray;
use crate::{Config, Error};
use crate::primitives::{H256, Uint, ValidatorIndex, Gwei};
use crate::types::{
	BeaconBlockHeader, Validator, Eth1Data, PendingAttestation, Checkpoint, Fork,
};
use crate::components::{Registry, Checkpoint as CheckpointT};
use crate::consts;

#[derive(PartialEq, Eq, Debug)]
/// Beacon executive. Cached data for intermediate state transition.
pub struct BeaconExecutive<'a, C: Config> {
	state: &'a mut BeaconState<C>,

	active_validator_indices: Option<Vec<ValidatorIndex>>,
	total_active_balance: Option<Gwei>,
}

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Create an executive from a mutable state reference.
	pub fn new(state: &'a mut BeaconState<C>) -> Self {
		Self {
			state,

			active_validator_indices: None,
			total_active_balance: None,
		}
	}
}

impl<'a, C: Config> Deref for BeaconExecutive<'a, C> {
	type Target = BeaconState<C>;

	fn deref(&self) -> &BeaconState<C> {
		&self.state
	}
}

macro_rules! unslashed_balance {
	( $attestations:tt, $epoch:expr, $self:expr ) => ({
		let attestations = $self.$attestations($epoch)?;
		let unslashed_attesting_indices =
			$self.unslashed_attesting_indices(&attestations)?;

		Ok($self.total_balance(&unslashed_attesting_indices))
	})
}

macro_rules! unslashed_validators {
	( $attestations:tt, $epoch:expr, $self:expr ) => ({
		let attestations = $self.$attestations($epoch)?;
		let unslashed_attesting_indices =
			$self.unslashed_attesting_indices(&attestations)?;

		Ok(Box::new(unslashed_attesting_indices.into_iter().map(move |index| {
			(index, &$self.validators[index as usize])
		})))
	})
}

impl<'a, C: Config> Registry for BeaconExecutive<'a, C> {
	type Error = Error;
	type Checkpoint = Checkpoint;
	type Validator = Validator;
	type Attestation = PendingAttestation<C>;

	fn total_active_balance(&self) -> u64 {
		self.total_active_balance()
	}

	fn attesting_target_balance(
		&self,
		checkpoint: &Self::Checkpoint
	) -> Result<u64, Self::Error> {
		self.attesting_balance(&self.matching_target_attestations(checkpoint.epoch)?)
	}

	fn min_inclusion_delay_attestation(
		&self,
		source_checkpoint: &Self::Checkpoint,
		index: u64,
	) -> Result<Option<Self::Attestation>, Self::Error> {
		let matching_source_attestations =
			self.matching_source_attestations(source_checkpoint.epoch())?;

		Ok(Some(matching_source_attestations.iter()
			.map(|a| Ok((
				a,
				self.attesting_indices(&a.data, &a.aggregation_bits)?.contains(&index)
			)))
			.collect::<Result<Vec<_>, _>>()?
			.into_iter()
			.filter(|(_, c)| *c)
			.map(|(a, _)| a)
			.fold(matching_source_attestations[0].clone(), |a, b| {
				if a.inclusion_delay < b.inclusion_delay { a } else { b.clone() }
			})))
	}

	fn unslashed_attesting_balance(
		&self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<u64, Self::Error> {
		unslashed_balance!(matching_source_attestations, source_checkpoint.epoch(), self)
	}

	fn unslashed_attesting_validators<'b>(
		&'b self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<Box<dyn Iterator<Item=(u64, &Self::Validator)> + 'b>, Self::Error> {
		unslashed_validators!(matching_source_attestations, source_checkpoint.epoch(), self)
	}

	fn unslashed_attesting_target_balance(
		&self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<u64, Self::Error> {
		unslashed_balance!(matching_target_attestations, source_checkpoint.epoch(), self)
	}

	fn unslashed_attesting_target_validators<'b>(
		&'b self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<Box<dyn Iterator<Item=(u64, &Self::Validator)> + 'b>, Self::Error> {
		unslashed_validators!(matching_target_attestations, source_checkpoint.epoch(), self)
	}

	fn unslashed_attesting_matching_head_balance(
		&self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<u64, Self::Error> {
		unslashed_balance!(matching_head_attestations, source_checkpoint.epoch(), self)
	}

	fn unslashed_attesting_matching_head_validators<'b>(
		&'b self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<Box<dyn Iterator<Item=(u64, &Self::Validator)> + 'b>, Self::Error> {
		unslashed_validators!(matching_head_attestations, source_checkpoint.epoch(), self)
	}

	fn balance(
		&self,
		index: u64,
	) -> Result<u64, Self::Error> {
		self.balances.get(index as usize).cloned().ok_or(Error::IndexOutOfRange)
	}

	fn effective_balance(
		&self,
		index: u64,
	) -> Result<u64, Self::Error> {
		self.validators.get(index as usize).map(|v| v.effective_balance)
			.ok_or(Error::IndexOutOfRange)
	}

	fn increase_balance(
		&mut self,
		index: u64,
		value: u64,
	) {
		self.increase_balance(index, value)
	}

	fn decrease_balance(
		&mut self,
		index: u64,
		value: u64,
	) {
		self.decrease_balance(index, value)
	}

	fn validators<'b>(
		&'b self,
	) -> Result<Box<dyn Iterator<Item=(u64, &Self::Validator)> + 'b>, Self::Error> {
		Ok(Box::new(self.validators.iter().enumerate().map(|(i, v)| (i as u64, v))))
	}
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Beacon state.
pub struct BeaconState<C: Config> {
	// == Versioning ==
	/// Genesis time as Unix timestamp.
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub genesis_time: Uint,
	/// Current slot.
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
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
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub eth1_deposit_index: Uint,

	// == Registry ==
	/// Validator registry.
	pub validators: MaxVec<Validator, C::ValidatorRegistryLimit>,
	#[bm(compact)]
	/// Balance of the validators.
	pub balances: MaxVec<Uint, C::ValidatorRegistryLimit>,

	// == Shuffling ==
	/// Past randao mixes.
	pub randao_mixes: VecArray<H256, C::EpochsPerHistoricalVector>,

	// == Slashings ==
	#[bm(compact)]
	/// Past slashings.
	pub slashings: VecArray<Uint, C::EpochsPerSlashingsVector>,

	// == Attestations ==
	/// Attestations on previous epoch.
	pub previous_epoch_attestations: MaxVec<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,
	/// Attestations on current epoch.
	pub current_epoch_attestations: MaxVec<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,

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
