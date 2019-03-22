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

use primitives::{H256, Signature, H768};
use ssz::Hashable;
use ssz_derive::Ssz;
use serde_derive::{Serialize, Deserialize};
use crate::validator::{VoluntaryExit, Transfer};
use crate::attestation::Attestation;
use crate::slashing::{AttesterSlashing, ProposerSlashing};
use crate::eth1::{Deposit, Eth1Data};
use crate::consts::GENESIS_SLOT;
use crate::state::BeaconState;
use crate::error::Error;
use crate::util::Hasher;

#[derive(Ssz)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct BeaconBlock {
	pub slot: u64,
	pub previous_block_root: H256,
	pub state_root: H256,
	/// Body
	pub body: BeaconBlockBody,
	/// Signature
	#[ssz(truncate)]
	pub signature: Signature,
}

impl BeaconBlock {
	pub fn empty() -> Self {
		Self {
			slot: GENESIS_SLOT,
			previous_block_root: H256::default(),
			state_root: H256::default(),
			signature: Signature::default(),
			body: BeaconBlockBody::empty(),
		}
	}

	pub fn genesis(deposits: Vec<Deposit>, genesis_time: u64, latest_eth1_data: Eth1Data) -> Result<(Self, BeaconState), Error> {
		let genesis_state = BeaconState::genesis(deposits, genesis_time, latest_eth1_data)?;
		let mut block = Self::empty();
		block.state_root = genesis_state.hash::<Hasher>();

		Ok((block, genesis_state))
	}
}

#[derive(Ssz, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct BeaconBlockHeader {
    pub slot: u64,
    pub previous_block_root: H256,
    pub state_root: H256,
    pub block_body_root: H256,
	#[ssz(truncate)]
    pub signature: Signature,
}

impl BeaconBlockHeader {
	pub fn with_state_root(block: &BeaconBlock, state_root: H256) -> Self {
		Self {
			slot: block.slot,
			previous_block_root: block.previous_block_root,
			state_root,
			block_body_root: block.body.hash::<Hasher>(),
			signature: block.signature,
		}
	}
}

#[derive(Ssz)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
pub struct BeaconBlockBody {
	pub randao_reveal: H768,
	pub eth1_data: Eth1Data,
	pub proposer_slashings: Vec<ProposerSlashing>,
	pub attester_slashings: Vec<AttesterSlashing>,
	pub attestations: Vec<Attestation>,
	pub deposits: Vec<Deposit>,
	pub voluntary_exits: Vec<VoluntaryExit>,
	pub transfers: Vec<Transfer>,
}

impl BeaconBlockBody {
	pub fn empty() -> Self {
		Self {
			proposer_slashings: Vec::new(),
			attester_slashings: Vec::new(),
			attestations: Vec::new(),
			deposits: Vec::new(),
			voluntary_exits: Vec::new(),
			transfers: Vec::new(),
			randao_reveal: H768::default(),
			eth1_data: Eth1Data::empty(),
		}
	}
}
