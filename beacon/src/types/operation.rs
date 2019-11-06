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

//! Beacon operations

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use ssz::{Codec, Encode, Decode};
use bm_le::{IntoTree, FromTree, MaxVec};
use vecarray::VecArray;
use crate::*;
use crate::primitives::*;
use crate::types::*;

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Block proposer slashing.
pub struct ProposerSlashing {
	/// Proposer index
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub proposer_index: Uint,
	/// First proposal
	pub header_1: BeaconBlockHeader,
	/// Second proposal
	pub header_2: BeaconBlockHeader,
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Block attester slashing.
pub struct AttesterSlashing<C: Config> {
	/// First slashable attestation
	pub attestation_1: IndexedAttestation<C>,
	/// Second slashable attestation
	pub attestation_2: IndexedAttestation<C>,
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Attestation.
pub struct Attestation<C: Config> {
	/// Attester aggregation bitfield
	#[bm(compact)]
	#[cfg_attr(feature = "serde", serde(serialize_with = "crate::utils::serialize_bitlist"))]
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_bitlist"))]
	pub aggregation_bits: MaxVec<bool, C::MaxValidatorsPerCommittee>,
	/// Attestation data
	pub data: AttestationData,
	/// Custody bitfield
	#[bm(compact)]
	#[cfg_attr(feature = "serde", serde(serialize_with = "crate::utils::serialize_bitlist"))]
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_bitlist"))]
	pub custody_bits: MaxVec<bool, C::MaxValidatorsPerCommittee>,
	/// BLS aggregate signature
	pub signature: Signature,
}

impl<C: Config> From<Attestation<C>> for SigningAttestation<C> {
	fn from(a: Attestation<C>) -> Self {
		Self {
			aggregation_bits: a.aggregation_bits,
			data: a.data,
			custody_bits: a.custody_bits,
		}
	}
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Unsealed attestation.
pub struct SigningAttestation<C: Config> {
	/// Attester aggregation bitfield
	#[bm(compact)]
	#[cfg_attr(feature = "serde", serde(serialize_with = "crate::utils::serialize_bitlist"))]
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_bitlist"))]
	pub aggregation_bits: MaxVec<bool, C::MaxValidatorsPerCommittee>,
	/// Attestation data
	pub data: AttestationData,
	/// Custody bitfield
	#[bm(compact)]
	#[cfg_attr(feature = "serde", serde(serialize_with = "crate::utils::serialize_bitlist"))]
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_bitlist"))]
	pub custody_bits: MaxVec<bool, C::MaxValidatorsPerCommittee>,
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Block deposit.
pub struct Deposit {
	/// Branch in the deposit tree
	pub proof: VecArray<H256, typenum::Sum<consts::DepositContractTreeDepth, typenum::U1>>,
	/// Data
	pub data: DepositData,
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Block voluntary exit.
pub struct VoluntaryExit {
	/// Minimum epoch for processing exit
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub epoch: Uint,
	/// Index of the exiting validator
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub validator_index: Uint,
	/// Validator signature
	pub signature: Signature,
}

impl From<VoluntaryExit> for SigningVoluntaryExit {
	fn from(v: VoluntaryExit) -> Self {
		Self {
			epoch: v.epoch,
			validator_index: v.validator_index,
		}
	}
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Unsealed voluntary exit transaction.
pub struct SigningVoluntaryExit {
	/// Minimum epoch for processing exit
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub epoch: Uint,
	/// Index of the exiting validator
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub validator_index: Uint,
}
