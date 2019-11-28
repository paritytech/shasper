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

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use ssz::{Codec, Encode, Decode};
use bm_le::{IntoTree, FromTree, MaxVec};
use vecarray::VecArray;
use crate::Config;
use crate::components::Checkpoint as CheckpointT;
use crate::primitives::{Version, Uint, H256, ValidatorId, Signature};

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Fork information.
pub struct Fork {
	/// Previous fork version
	pub previous_version: Version,
	/// Current fork version
	pub current_version: Version,
	/// Fork epoch number
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub epoch: Uint,
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Checkpoint
pub struct Checkpoint {
	/// Epoch
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub epoch: Uint,
	/// Root of the checkpoint
	pub root: H256,
}

impl CheckpointT for Checkpoint {
	fn epoch(&self) -> u64 {
		self.epoch
	}
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Validator record.
pub struct Validator {
	/// BLS public key
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Effective balance
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub effective_balance: Uint,
	/// Was the validator slashed
	pub slashed: bool,

	// == Status epochs ==
	/// Epoch when became eligible for activation
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub activation_eligibility_epoch: Uint,
	/// Epoch when validator activated
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub activation_epoch: Uint,
	/// Epoch when validator exited
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub exit_epoch: Uint,
	/// Epoch when validator is eligible to withdraw
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub withdrawable_epoch: Uint,
}

impl Validator {
	/// Whether it is active validator.
	pub fn is_active(&self, epoch: Uint) -> bool {
		self.activation_epoch <= epoch && epoch < self.exit_epoch
	}

	/// Whether it is slashable.
	pub fn is_slashable(&self, epoch: Uint) -> bool {
		self.slashed == false &&
			self.activation_epoch <= epoch && epoch < self.withdrawable_epoch
	}
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Attestation data.
pub struct AttestationData {
	/// Voting slot.
	pub slot: Uint,
	/// Voting committee index.
	pub index: Uint,

	// == LMD-GHOST vote ==
	/// Root of the signed beacon block
	pub beacon_block_root: H256,

	// == FFG vote ==
	/// Source
	pub source: Checkpoint,
	/// Target
	pub target: Checkpoint,
}

impl AttestationData {
	/// Is slashable.
	pub fn is_slashable(&self, other: &AttestationData) -> bool {
		(self != other && self.target.epoch == other.target.epoch) ||
			(self.source.epoch < other.source.epoch &&
			 other.target.epoch < self.target.epoch)
	}
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Attestation data with custody bit.
pub struct AttestationDataAndCustodyBit {
	/// Attestation data
	pub data: AttestationData,
	/// Custody bit
	pub custody_bit: bool,
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(bound = "C: Config"))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Indexed attestation.
pub struct IndexedAttestation<C: Config> {
	/// Validator indices of custody bit 0.
	#[bm(compact)]
	pub custody_bit_0_indices: MaxVec<Uint, C::MaxValidatorsPerCommittee>,
	/// Validator indices of custody bit 1
	#[bm(compact)]
	pub custody_bit_1_indices: MaxVec<Uint, C::MaxValidatorsPerCommittee>,
	/// Attestation data
	pub data: AttestationData,
	/// Aggregate signature
	pub signature: Signature,
}

impl<C: Config> From<IndexedAttestation<C>> for SigningIndexedAttestation<C> {
	fn from(indexed: IndexedAttestation<C>) -> Self {
		Self {
			custody_bit_0_indices: indexed.custody_bit_0_indices,
			custody_bit_1_indices: indexed.custody_bit_1_indices,
			data: indexed.data
		}
	}
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(bound = "C: Config"))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Signing indexed attestation.
pub struct SigningIndexedAttestation<C: Config> {
	/// Validator indices of custody bit 0.
	#[bm(compact)]
	pub custody_bit_0_indices: MaxVec<Uint, C::MaxValidatorsPerCommittee>,
	/// Validator indices of custody bit 1
	#[bm(compact)]
	pub custody_bit_1_indices: MaxVec<Uint, C::MaxValidatorsPerCommittee>,
	/// Attestation data
	pub data: AttestationData,
}

#[derive(Codec, Encode, Decode, FromTree, IntoTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(bound = "C: Config"))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Pending attestation.
pub struct PendingAttestation<C: Config> {
	/// Attester aggregation bitfield
	#[bm(compact)]
	#[cfg_attr(feature = "serde", serde(serialize_with = "crate::utils::serialize_bitlist"))]
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_bitlist"))]
	pub aggregation_bits: MaxVec<bool, C::MaxValidatorsPerCommittee>,
	/// Attestation data
	pub data: AttestationData,
	/// Inclusion delay
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub inclusion_delay: Uint,
	/// Proposer index
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub proposer_index: Uint,
}

#[derive(Codec, Encode, Decode, FromTree, IntoTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Eth1 data.
pub struct Eth1Data {
	/// Root of the deposit tree
	pub deposit_root: H256,
	/// Total number of deposits
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub deposit_count: Uint,
	/// Block hash
	pub block_hash: H256,
}

#[derive(Codec, Encode, Decode, FromTree, IntoTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(bound = "C: Config"))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Historical batch information.
pub struct HistoricalBatch<C: Config> {
	/// Block roots
	pub block_roots: VecArray<H256, C::SlotsPerHistoricalRoot>,
	/// State roots
	pub state_roots: VecArray<H256, C::SlotsPerHistoricalRoot>,
}

#[derive(Codec, Encode, Decode, FromTree, IntoTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Deposit data.
pub struct DepositData {
	/// BLS pubkey
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Amount in Gwei
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub amount: Uint,
	/// Container self-signature
	pub signature: Signature,
}

impl From<DepositData> for SigningDepositData {
	fn from(data: DepositData) -> Self {
		Self {
			pubkey: data.pubkey,
			withdrawal_credentials: data.withdrawal_credentials,
			amount: data.amount,
		}
	}
}

#[derive(Codec, Encode, Decode, FromTree, IntoTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Deposit data.
pub struct SigningDepositData {
	/// BLS pubkey
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Amount in Gwei
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
	pub amount: Uint,
}

#[derive(Codec, Encode, Decode, FromTree, IntoTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Beacon block header.
pub struct BeaconBlockHeader {
	/// Slot of the block.
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
    pub slot: Uint,
	/// Previous block root.
    pub parent_root: H256,
	/// State root.
    pub state_root: H256,
	/// Block body root.
    pub body_root: H256,
	/// Signature.
    pub signature: Signature,
}

impl From<BeaconBlockHeader> for SigningBeaconBlockHeader {
	fn from(header: BeaconBlockHeader) -> Self {
		Self {
			slot: header.slot,
			parent_root: header.parent_root,
			state_root: header.state_root,
			body_root: header.body_root,
		}
	}
}

#[derive(Codec, Encode, Decode, FromTree, IntoTree, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(parity_codec::Encode, parity_codec::Decode))]
/// Beacon block header.
pub struct SigningBeaconBlockHeader {
	/// Slot of the block.
	#[cfg_attr(feature = "serde", serde(deserialize_with = "crate::utils::deserialize_uint"))]
    pub slot: Uint,
	/// Previous block root.
    pub parent_root: H256,
	/// State root.
    pub state_root: H256,
	/// Block body root.
    pub body_root: H256,
}
