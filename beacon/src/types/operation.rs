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
#[cfg_attr(feature = "std", derive(Debug))]
/// Block proposer slashing.
pub struct ProposerSlashing {
	/// Proposer index
	pub proposer_index: Uint,
	/// First proposal
	pub header_1: BeaconBlockHeader,
	/// Second proposal
	pub header_2: BeaconBlockHeader,
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
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
#[cfg_attr(feature = "std", derive(Debug))]
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
#[cfg_attr(feature = "std", derive(Debug))]
/// Block voluntary exit.
pub struct VoluntaryExit {
	/// Minimum epoch for processing exit
	pub epoch: Uint,
	/// Index of the exiting validator
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
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SigningVoluntaryExit {
	/// Minimum epoch for processing exit
	pub epoch: Uint,
	/// Index of the exiting validator
	pub validator_index: Uint,
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Block transfer.
pub struct Transfer {
	/// Sender index
	pub sender: Uint,
	/// Recipient index
	pub recipient: Uint,
	/// Amount in Gwei
	pub amount: Uint,
	/// Fee in Gwei for block proposer
	pub fee: Uint,
	/// Inclusion slot
	pub slot: Uint,
	/// Sender withdrawal pubkey
	pub pubkey: ValidatorId,
	/// Sender signature
	pub signature: Signature,
}

impl From<Transfer> for SigningTransfer {
	fn from(t: Transfer) -> Self {
		Self {
			sender: t.sender,
			recipient: t.recipient,
			amount: t.amount,
			fee: t.fee,
			slot: t.slot,
			pubkey: t.pubkey,
		}
	}
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SigningTransfer {
	/// Sender index
	pub sender: Uint,
	/// Recipient index
	pub recipient: Uint,
	/// Amount in Gwei
	pub amount: Uint,
	/// Fee in Gwei for block proposer
	pub fee: Uint,
	/// Inclusion slot
	pub slot: Uint,
	/// Sender withdrawal pubkey
	pub pubkey: ValidatorId,
}
