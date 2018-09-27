use primitives::H256;

use attestation::AttestationRecord;

pub struct SpecHeader {
	parent_hash: H256,
	slot_number: u64,
	randao_reveal: H256,
	attestations: Vec<SpecAttestationRecord>,
	pow_chain_ref: H256,
	active_state_root: H256,
	crystallized_state_root: H256,
}

pub type SpecAttestationRecord = AttestationRecord;
