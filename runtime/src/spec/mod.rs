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
use blake2::{Blake2b, crypto_mac::Mac};
use ssz;
use rstd::prelude::*;

use attestation::AttestationRecord;

mod state;

pub use self::state::{
	SpecActiveState, SpecCrystallizedState,
	SpecActiveStateExt, SpecCrystallizedStateExt
};

#[derive(Clone, PartialEq, Eq, Default, SszEncode, SszDecode)]
#[cfg_attr(feature = "std", derive(Debug))]
#[ssz_codec(sorted)]
pub struct SpecHeader {
	pub parent_hash: H256,
	pub slot_number: u64,
	pub randao_reveal: H256,
	pub attestations: Vec<SpecAttestationRecord>,
	pub pow_chain_ref: H256,
	pub active_state_root: H256,
	pub crystallized_state_root: H256,
}

impl SpecHeader {
	pub fn spec_hash(&self) -> H256 {
		let encoded = ssz::Encode::encode(self);
		let mut blake2 = Blake2b::new_keyed(&[], 64);
		blake2.input(&encoded);
		H256::from(&blake2.result().code()[0..32])
	}
}

pub type SpecAttestationRecord = AttestationRecord;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn spec_header_hash() {
		assert_eq!(SpecHeader::default().hash(), H256::from("0x66cad4289cc03192dc9a0b7583d1075b17bb6b78bd91694cdd3ff5c57e31d744"));
	}
}
