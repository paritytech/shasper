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

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use ssz::{Codec, Encode, Decode};
use bm_le::{IntoTree, FromTree, MaxVec};
use crate::*;
use crate::primitives::*;
use crate::types::*;

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(bound = "C: Config + Serialize + Clone + DeserializeOwned + 'static"))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Beacon block body.
pub struct BeaconBlockBody<C: Config> {
	/// Randao reveal.
	pub randao_reveal: H768,
	/// Eth1 data.
	pub eth1_data: Eth1Data,
	/// Graffiti.
	pub graffiti: H256,
	/// Proposer slashings.
	pub proposer_slashings: MaxVec<ProposerSlashing, C::MaxProposerSlashings>,
	/// Attester slashings.
	pub attester_slashings: MaxVec<AttesterSlashing<C>, C::MaxAttesterSlashings>,
	/// Attestations.
	pub attestations: MaxVec<Attestation<C>, C::MaxAttestations>,
	/// Deposits.
	pub deposits: MaxVec<Deposit, C::MaxDeposits>,
	/// Voluntary exits.
	pub voluntary_exits: MaxVec<VoluntaryExit, C::MaxVoluntaryExits>,
	/// Transfer.
	pub transfers: MaxVec<Transfer, C::MaxTransfers>,
}

/// Sealed or unsealed block.
pub trait Block {
	type Config: Config;

	/// Slot of the block.
	fn slot(&self) -> u64;
	/// Previous block root.
	fn parent_root(&self) -> &H256;
	/// State root.
	fn state_root(&self) -> &H256;
	/// Body.
	fn body(&self) -> &BeaconBlockBody<Self::Config>;
	/// Signature of the block. None for unsealed block.
	fn signature(&self) -> Option<&Signature>;
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(bound = "C: Config + Serialize + Clone + DeserializeOwned + 'static"))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Beacon block.
pub struct BeaconBlock<C: Config> {
	/// Slot of the block.
	pub slot: Uint,
	/// Previous block root.
	pub parent_root: H256,
	/// State root.
	pub state_root: H256,
	/// Body.
	pub body: BeaconBlockBody<C>,
	/// Signature.
	pub signature: Signature,
}

impl<C: Config> Block for BeaconBlock<C> {
	type Config = C;

	fn slot(&self) -> u64 { self.slot }
	fn parent_root(&self) -> &H256 { &self.parent_root }
	fn state_root(&self) -> &H256 { &self.state_root }
	fn body(&self) -> &BeaconBlockBody<C> { &self.body }
	fn signature(&self) -> Option<&Signature> { Some(&self.signature) }
}

impl<'a, C: Config, T: Block<Config=C>> From<&'a T> for UnsealedBeaconBlock<C> {
	fn from(t: &'a T) -> UnsealedBeaconBlock<C> {
		UnsealedBeaconBlock {
			slot: t.slot(),
			parent_root: t.parent_root().clone(),
			state_root: t.state_root().clone(),
			body: t.body().clone(),
		}
	}
}

#[derive(Codec, Encode, Decode, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(bound = "C: Config + Serialize + Clone + DeserializeOwned + 'static"))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Unsealed Beacon block.
pub struct UnsealedBeaconBlock<C: Config> {
	/// Slot of the block.
	pub slot: Uint,
	/// Previous block root.
	pub parent_root: H256,
	/// State root.
	pub state_root: H256,
	/// Body.
	pub body: BeaconBlockBody<C>,
}

impl<C: Config> Block for UnsealedBeaconBlock<C> {
	type Config = C;

	fn slot(&self) -> u64 { self.slot }
	fn parent_root(&self) -> &H256 { &self.parent_root }
	fn state_root(&self) -> &H256 { &self.state_root }
	fn body(&self) -> &BeaconBlockBody<C> { &self.body }
	fn signature(&self) -> Option<&Signature> { None }
}

impl<C: Config> UnsealedBeaconBlock<C> {
	/// Fake sealing a beacon block, with empty signature.
	pub fn fake_seal(self) -> BeaconBlock<C> {
		BeaconBlock {
			slot: self.slot,
			parent_root: self.parent_root,
			state_root: self.state_root,
			body: self.body,
			signature: Default::default(),
		}
	}
}
