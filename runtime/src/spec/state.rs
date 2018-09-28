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
