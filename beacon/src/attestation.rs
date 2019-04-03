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

use crate::primitives::{BitField, H256, Signature};
use crate::Config;

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Crosslink {
	/// Epoch number
	pub epoch: u64,
	/// Shard data since the previous crosslink
	pub crosslink_data_root: H256,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Attestation {
	/// Attester aggregation bitfield
	pub aggregation_bitfield: BitField,
	/// Attestation data
	pub data: AttestationData,
	/// Custody bitfield
	pub custody_bitfield: BitField,
	/// BLS aggregate signature
	pub aggregate_signature: Signature,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PendingAttestation {
	/// Attester aggregation bitfield
	pub aggregation_bitfield: BitField,
	/// Attestation data
	pub data: AttestationData,
	/// Custody bitfield
	pub custody_bitfield: BitField,
	/// Inclusion slot
	pub inclusion_slot: u64,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AttestationData {
	// LMD GHOST vote
	/// Slot number
	pub slot: u64,
	/// Root of the signed beacon block
	pub beacon_block_root: H256,

	// FFG vote
	/// Last justified epoch in the beacon state
	pub source_epoch: u64,
	/// Hash of the last justified beacon block
	pub source_root: H256,
	/// Root of the ancestor at the epoch boundary
	pub target_root: H256,

	// Crosslink vote
	/// Shard number
	pub shard: u64,
	/// Last crosslink
	pub previous_crosslink: Crosslink,
	/// Data from the shard since the last attestation
	pub crosslink_data_root: H256,
}

impl AttestationData {
	pub fn is_double_vote<C: Config>(&self, other: &AttestationData, config: &C) -> bool {
		config.slot_to_epoch(self.slot) == config.slot_to_epoch(other.slot)
	}

	pub fn is_surround_vote<C: Config>(&self, other: &AttestationData, config: &C) -> bool {
		self.source_epoch < other.source_epoch &&
			config.slot_to_epoch(other.slot) < config.slot_to_epoch(self.slot)
	}
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AttestationDataAndCustodyBit {
	/// Attestation data
	pub data: AttestationData,
	/// Custody bit
	pub custody_bit: bool,
}
