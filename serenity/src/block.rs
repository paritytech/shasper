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

use primitives::{H256, Signature};
use crate::validator::{Deposit, VoluntaryExit, Transfer};
use crate::attestation::Attestation;
use crate::slashing::{AttesterSlashing, ProposerSlashing};
use crate::eth1::Eth1Data;

pub struct BeaconBlock {
	// Header
	pub slot: u64,
	pub parent_root: H256,
	pub state_root: H256,
	pub randao_reveal: Signature,
	pub eth1_data: Eth1Data,

	/// Body
	pub body: BeaconBlockBody,
	/// Signature
	pub signature: Signature,
}

pub struct BeaconBlockHeader {
    pub slot: u64,
    pub previous_block_root: H256,
    pub state_root: H256,
    pub block_body_root: H256,
    pub signature: Signature,
}

pub struct BeaconBlockBody {
	pub proposer_slashings: Vec<ProposerSlashing>,
	pub attester_slashings: Vec<AttesterSlashing>,
	pub attestations: Vec<Attestation>,
	pub deposits: Vec<Deposit>,
	pub voluntary_exits: Vec<VoluntaryExit>,
	pub transfers: Vec<Transfer>,
}
