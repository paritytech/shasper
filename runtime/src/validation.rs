use state::CrystallizedState;
use attestation::AttestationRecord;

pub fn validate_block_pre_processing_conditions() { }

pub fn validate_parent_block_proposer(slot: u64, parent_slot: u64, crystallized_state: &CrystallizedState, attestations: &[AttestationRecord]) {
	if slot == 0 {
		return;
	}

	let (proposer_index_in_committee, shard_id) = crystallized_state.proposer_position(parent_slot);

	assert!(attestations.len() > 0);
	let attestation = &attestations[0];

	assert!(attestation.shard_id == shard_id &&
			attestation.slot == parent_slot &&
			attestation.attester_bitfield.has_voted(proposer_index_in_committee));
}
