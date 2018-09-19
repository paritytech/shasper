use primitives::H256;

use attestation::AttestationRecord;

pub struct Extrinsic {
	pub randao_reveal: H256,
	pub pow_chain_ref: H256,
	pub attestations: Vec<AttestationRecord>,
}
