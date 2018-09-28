use primitives::H256;
use blake2::Blake2b;
use blake2::crypto_mac::Mac;
use ssz;
#[allow(unused_imports)]
use rstd::prelude::*;

use attestation::AttestationRecord;

#[derive(Clone, PartialEq, Eq, SszEncode, SszDecode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SpecHeader {
	parent_hash: H256,
	slot_number: u64,
	randao_reveal: H256,
	attestations: Vec<SpecAttestationRecord>,
	pow_chain_ref: H256,
	active_state_root: H256,
	crystallized_state_root: H256,
}

impl SpecHeader {
	pub fn hash(&self) -> H256 {
		let encoded = ssz::Encode::encode(self);
		let mut blake2 = Blake2b::new_keyed(&[], 64);
		blake2.input(&encoded);
		H256::from(&blake2.result().code()[0..32])
	}
}

pub type SpecAttestationRecord = AttestationRecord;
