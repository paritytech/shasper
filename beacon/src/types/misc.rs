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

//! Misc dependencies

use ssz_derive::Ssz;
#[cfg(feature = "serde")]
use serde_derive::{Serialize, Deserialize};
#[cfg(feature = "parity-codec")]
use codec::{Encode, Decode};

use crate::primitives::{Uint, H256, Version, Signature, ValidatorId, BitField};
use crate::utils::fixed_vec;
use crate::Config;

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
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
	pub epoch: Uint,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Checkpoint
pub struct Checkpoint {
	/// Epoch
	pub epoch: Uint,
	/// Root of the checkpoint
	pub root: H256,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Validator record.
pub struct Validator {
	/// BLS public key
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Effective balance
	pub effective_balance: Uint,
	/// Was the validator slashed
	pub slashed: bool,

	// == Status epochs ==
	/// Epoch when became eligible for activation
	pub activation_eligibility_epoch: Uint,
	/// Epoch when validator activated
	pub activation_epoch: Uint,
	/// Epoch when validator exited
	pub exit_epoch: Uint,
	/// Epoch when validator is eligible to withdraw
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

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Crosslink.
pub struct Crosslink {
	/// Shard number
	pub shard: Uint,
	/// Root of the previous crosslink
	pub parent_root: H256,

	// == Crosslinking data ==
	/// Crosslinking data from epoch start
	pub start_epoch: Uint,
	/// Crosslinking data to epoch end
	pub end_epoch: Uint,
	/// Root of the crosslinked shard data since the previous crosslink
	pub data_root: H256,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Attestation data.
pub struct AttestationData {
	// == LMD-GHOST vote ==
	/// Root of the signed beacon block
	pub beacon_block_root: H256,

	// == FFG vote ==
	/// Source
	pub source: Checkpoint,
	/// Target
	pub target: Checkpoint,

	/// Crosslink vote
	pub crosslink: Crosslink,
}

impl AttestationData {
	/// Is slashable.
	pub fn is_slashable(&self, other: &AttestationData) -> bool {
		(self != other && self.target.epoch == other.target.epoch) ||
			(self.source.epoch < other.source.epoch &&
			 other.target.epoch < self.target.epoch)
	}
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Attestation data with custody bit.
pub struct AttestationDataAndCustodyBit {
	/// Attestation data
	pub data: AttestationData,
	/// Custody bit
	pub custody_bit: bool,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Indexed attestation.
pub struct IndexedAttestation {
	/// Validator indices of custody bit 0.
	pub custody_bit_0_indices: Vec<Uint>,
	/// Validator indices of custody bit 1
	pub custody_bit_1_indices: Vec<Uint>,
	/// Attestation data
	pub data: AttestationData,
	#[ssz(truncate)]
	/// Aggregate signature
	pub signature: Signature,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Pending attestation.
pub struct PendingAttestation {
	/// Attester aggregation bitfield
	pub aggregation_bits: BitField,
	/// Attestation data
	pub data: AttestationData,
	/// Inclusion delay
	pub inclusion_delay: Uint,
	/// Proposer index
	pub proposer_index: Uint,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Eth1 data.
pub struct Eth1Data {
	/// Root of the deposit tree
	pub deposit_root: H256,
	/// Total number of deposits
	pub deposit_count: Uint,
	/// Block hash
	pub block_hash: H256,
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
	pub block_roots: Vec<H256>,
	/// State roots
	#[ssz(use_fixed)]
	pub state_roots: Vec<H256>,
}

impl HistoricalBatch {
	/// Default historical batch from config.
	pub fn default_with_config<C: Config>(config: &C) -> Self {
		Self {
			block_roots: fixed_vec(config.slots_per_historical_root()),
			state_roots: fixed_vec(config.slots_per_historical_root()),
		}
	}
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Deposit data.
pub struct DepositData {
	/// BLS pubkey
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Amount in Gwei
	pub amount: Uint,
	#[ssz(truncate)]
	/// Container self-signature
	pub signature: Signature,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Compact committee
pub struct CompactCommittee {
	/// BLS pubkeys
	pub pubkeys: Vec<ValidatorId>,
	/// Compact validators
	pub compact_validators: Vec<Uint>,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Beacon block header.
pub struct BeaconBlockHeader {
	/// Slot of the block.
    pub slot: Uint,
	/// Previous block root.
    pub parent_root: H256,
	/// State root.
    pub state_root: H256,
	/// Block body root.
    pub body_root: H256,
	#[ssz(truncate)]
	/// Signature.
    pub signature: Signature,
}
