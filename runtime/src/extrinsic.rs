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

use primitives::H256;
use rstd::prelude::*;

use attestation::AttestationRecord;
use spec::SpecHeader;

#[derive(Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct Extrinsic {
	pub slot_number: u64,
	pub randao_reveal: H256,
	pub pow_chain_ref: H256,
	pub attestations: Vec<AttestationRecord>,
}

impl Extrinsic {
	pub fn spec_hash(&self, parent_hash: H256, active_state_root: H256, crystallized_state_root: H256) -> H256 {
		let spec_header = SpecHeader {
			parent_hash: parent_hash,
			slot_number: self.slot_number,
			randao_reveal: self.randao_reveal,
			attestations: self.attestations.clone(),
			pow_chain_ref: self.pow_chain_ref,
			active_state_root: active_state_root,
			crystallized_state_root: crystallized_state_root,
		};

		spec_header.spec_hash()
	}
}
