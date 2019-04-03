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

use crate::primitives::{Signature, BitField};
use crate::attestation::AttestationData;
use crate::block::BeaconBlockHeader;

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Block proposer slashing.
pub struct ProposerSlashing {
	/// Proposer index
	pub proposer_index: u64,
	/// First proposal
	#[cfg_attr(feature = "serde", serde(rename = "header_1"))]
	pub header_a: BeaconBlockHeader,
	/// Second proposal
	#[cfg_attr(feature = "serde", serde(rename = "header_2"))]
	pub header_b: BeaconBlockHeader,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Block attester slashing.
pub struct AttesterSlashing {
	/// First slashable attestation
	pub slashable_attestation_a: SlashableAttestation,
	/// Second slashable attestation
	pub slashable_attestation_b: SlashableAttestation,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Slashable attestation.
pub struct SlashableAttestation {
	/// Validator indices
	pub validator_indices: Vec<u64>,
	/// Attestation data
	pub data: AttestationData,
	/// Custody bitfield
	pub custody_bitfield: BitField,
	/// Aggregate signature
	pub aggregate_signature: Signature,
}
