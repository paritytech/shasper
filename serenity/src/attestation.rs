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

use primitives::{BitField, H256, Signature};
use ssz_derive::Ssz;
use crate::consts::GENESIS_EPOCH;
use crate::util::slot_to_epoch;

#[derive(Ssz, Clone, PartialEq, Eq)]
pub struct Crosslink {
	/// Epoch number
	pub epoch: u64,
	/// Shard data since the previous crosslink
	pub crosslink_data_root: H256,
}

impl Default for Crosslink {
	fn default() -> Self {
		Self {
			epoch: GENESIS_EPOCH,
			crosslink_data_root: H256::default(),
		}
	}
}

#[derive(Ssz)]
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

#[derive(Ssz, Clone)]
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
pub struct AttestationData {
	/// Slot number
	pub slot: u64,
	/// Shard number
	pub shard: u64,
	/// Root of the signed beacon block
	pub beacon_block_root: H256,
	/// Root of the ancestor at the epoch boundary
	pub epoch_boundary_root: H256,
	/// Data from the shard since the last attestation
	pub crosslink_data_root: H256,
	/// Last crosslink
	pub latest_crosslink: Crosslink,
	/// Last justified epoch in the beacon state
	pub justified_epoch: u64,
	/// Hash of the last justified beacon block
	pub justified_block_root: H256,
}

impl AttestationData {
	pub fn is_double_vote(&self, other: &AttestationData) -> bool {
		slot_to_epoch(self.slot) == slot_to_epoch(other.slot)
	}

	pub fn is_surround_vote(&self, other: &AttestationData) -> bool {
		self.justified_epoch < other.justified_epoch &&
			slot_to_epoch(other.slot) < slot_to_epoch(self.slot)
	}
}

#[derive(Ssz)]
pub struct AttestationDataAndCustodyBit {
	/// Attestation data
	pub data: AttestationData,
	/// Custody bit
	pub custody_bit: bool,
}
