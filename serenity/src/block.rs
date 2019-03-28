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
use hash_db::Hasher;

use crate::validator::{VoluntaryExit, Transfer};
use crate::attestation::Attestation;
use crate::slashing::{AttesterSlashing, ProposerSlashing};
use crate::eth1::{Deposit, Eth1Data};

#[derive(Ssz)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
#[ssz(no_decode)]
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
	pub fn with_state_root_no_signature<H: Hasher<Out=H256>>(block: &BeaconBlock, state_root: H256) -> Self {
		Self {
			slot: block.slot,
			previous_block_root: block.previous_block_root,
			state_root,
			block_body_root: block.body.hash::<H>(),
			// signed_root(block) is used for block id purposes so signature is a stub
			signature: Signature::default(),
		}
	}
}

#[derive(Ssz)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug), serde(deny_unknown_fields))]
#[ssz(no_decode)]
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{Config, NoVerificationConfig};
	use std::str::FromStr;
	use ssz::{Encode, Prefixable};

	#[test]
	fn empty_header_serialization() {
		let header = BeaconBlockHeader {
			slot: 0,
			previous_block_root: Default::default(),
			state_root: Default::default(),
			block_body_root: Default::default(),
			signature: Default::default(),
		};

		assert!(!BeaconBlockHeader::prefixed());
		assert_eq!(header.encode(), &b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"[..]);
		assert_eq!(header.hash::<<NoVerificationConfig as Config>::Hasher>().as_ref(), &b"\xe0\x10\x03\xd7*\n\xe4y\xfe\xae'\x1e\x10\xa0\xb0\xb1\xc6#~\xe9h\xd3\xeeZ\x06\x99\xf1\xfb9\x98\xa63"[..]);
	}

	#[test]
	fn basic_header_serialization() {
		let header = BeaconBlockHeader {
			slot: 4294967296,
			previous_block_root: Default::default(),
			state_root: H256::from_str("bdac85b271ed09d9a47a161395cd15d85eca25d9e3dd9e458c8cc08c80180273").unwrap(),
			block_body_root: H256::from_str("13f2001ff0ee4a528b3c43f63d70a997aefca990ed8eada2223ee6ec3807f7cc").unwrap(),
			signature: Default::default(),
		};

		assert_eq!(header.encode(), &b"\x00\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xbd\xac\x85\xb2q\xed\t\xd9\xa4z\x16\x13\x95\xcd\x15\xd8^\xca%\xd9\xe3\xdd\x9eE\x8c\x8c\xc0\x8c\x80\x18\x02s\x13\xf2\x00\x1f\xf0\xeeJR\x8b<C\xf6=p\xa9\x97\xae\xfc\xa9\x90\xed\x8e\xad\xa2\">\xe6\xec8\x07\xf7\xcc\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"[..]);
		assert_eq!(header.hash::<<NoVerificationConfig as Config>::Hasher>().as_ref(), &b"\xda<\x93\x8f\xbc\x97\xb9\xec\xe3\xa2:\"w\xeb\x86J\xd6\x17>!@N}(a\xb7\x91\x1e^\x8brR"[..]);
	}
}
