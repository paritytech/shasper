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

//! Beacon blocks

use ssz_derive::Ssz;
#[cfg(feature = "serde")]
use serde_derive::{Serialize, Deserialize};
#[cfg(feature = "parity-codec")]
use codec::{Encode, Decode};

use crate::primitives::{Uint, Signature, H256, H768};
use crate::types::{VoluntaryExit, Transfer, Deposit, Attestation, Eth1Data, ProposerSlashing, AttesterSlashing};

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
#[ssz(no_decode)]
/// Beacon block body.
pub struct BeaconBlockBody {
	/// Randao reveal.
	pub randao_reveal: H768,
	/// Eth1 data.
	pub eth1_data: Eth1Data,
	/// Graffiti.
	pub graffiti: H256,
	/// Proposer slashings.
	pub proposer_slashings: Vec<ProposerSlashing>,
	/// Attester slashings.
	pub attester_slashings: Vec<AttesterSlashing>,
	/// Attestations.
	pub attestations: Vec<Attestation>,
	/// Deposits.
	pub deposits: Vec<Deposit>,
	/// Voluntary exits.
	pub voluntary_exits: Vec<VoluntaryExit>,
	/// Transfer.
	pub transfers: Vec<Transfer>,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
#[ssz(no_decode)]
/// Beacon block.
pub struct BeaconBlock {
	/// Slot of the block.
	pub slot: Uint,
	/// Previous block root.
	pub previous_block_root: H256,
	/// State root.
	pub state_root: H256,
	/// Body.
	pub body: BeaconBlockBody,
	#[ssz(truncate)]
	/// Signature.
	pub signature: Signature,
}
