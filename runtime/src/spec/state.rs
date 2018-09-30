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

use state::{ActiveState, CrystallizedState};

pub type SpecActiveState = ActiveState;
pub type SpecCrystallizedState = CrystallizedState;

pub trait SpecActiveStateExt {
	fn spec_hash(&self) -> H256;
}

pub trait SpecCrystallizedStateExt {
	fn spec_hash(&self) -> H256;
}

impl SpecActiveStateExt for SpecActiveState {
	fn spec_hash(&self) -> H256 {
		let encoded = ssz::Encode::encode(self);
		let mut blake2 = Blake2b::new_keyed(&[], 64);
		blake2.input(&encoded);
		H256::from(&blake2.result().code()[0..32])
	}
}

impl SpecCrystallizedStateExt for SpecCrystallizedState {
	fn spec_hash(&self) -> H256 {
		let encoded = ssz::Encode::encode(self);
		let mut blake2 = Blake2b::new_keyed(&[], 64);
		blake2.input(&encoded);
		H256::from(&blake2.result().code()[0..32])
	}
}
