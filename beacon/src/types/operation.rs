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

use ssz_derive::Ssz;
#[cfg(feature = "serde")]
use serde_derive::{Serialize, Deserialize};
#[cfg(feature = "parity-codec")]
use codec::{Encode, Decode};

use crate::primitives::{Uint, BitField, Signature, H256, ValidatorId};
use crate::types::{BeaconBlockHeader, IndexedAttestation, AttestationData, DepositData};
use crate::utils::fixed_vec;
use crate::Config;

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
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

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Block attester slashing.
pub struct AttesterSlashing {
	/// First slashable attestation
	pub attestation_1: IndexedAttestation,
	/// Second slashable attestation
	pub attestation_2: IndexedAttestation,
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Attestation.
pub struct Attestation {
	/// Attester aggregation bitfield
	pub aggregation_bits: BitField,
	/// Attestation data
	pub data: AttestationData,
	/// Custody bitfield
	pub custody_bits: BitField,
	#[ssz(truncate)]
	/// BLS aggregate signature
	pub signature: Signature,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
#[ssz(no_decode)]
/// Block deposit.
pub struct Deposit {
	/// Branch in the deposit tree
	#[ssz(use_fixed)]
	pub proof: Vec<H256>,
	/// Data
	pub data: DepositData,
}

impl Deposit {
	/// Default deposit from config.
	pub fn default_with_config<C: Config>(config: &C) -> Self {
		Self {
			proof: fixed_vec(consts::DEPOSIT_CONTRACT_TREE_DEPTH + 1),
			data: Default::default(),
		}
	}
}

#[derive(Ssz, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Block voluntary exit.
pub struct VoluntaryExit {
	/// Minimum epoch for processing exit
	pub epoch: Uint,
	/// Index of the exiting validator
	pub validator_index: Uint,
	/// Validator signature
	#[ssz(truncate)]
	pub signature: Signature,
}

#[derive(Ssz, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
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
	#[ssz(truncate)]
	pub signature: Signature,
}
