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

mod misc;
mod operation;
mod block;

pub use self::misc::{
	Fork, Checkpoint, Validator, AttestationData, AttestationDataAndCustodyBit,
	IndexedAttestation, SigningIndexedAttestation, PendingAttestation, Eth1Data,
	HistoricalBatch, DepositData, SigningDepositData, BeaconBlockHeader,
	SigningBeaconBlockHeader,
};
pub use self::operation::{
	ProposerSlashing, AttesterSlashing, Attestation, SigningAttestation,
	Deposit, VoluntaryExit, SigningVoluntaryExit
};
pub use self::block::{
	BeaconBlockBody, BeaconBlock, UnsealedBeaconBlock, Block,
};
