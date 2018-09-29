use primitives::H256;
use rstd::prelude::*;

use attestation::AttestationRecord;

#[derive(Encode, Decode)]
pub struct Extrinsic {
	pub slot_number: u64,
	pub randao_reveal: H256,
	pub pow_chain_ref: H256,
	pub attestations: Vec<AttestationRecord>,
}
