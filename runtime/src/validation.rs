use primitives::H256;
use runtime_support::storage::StorageMap;

use state::{ActiveState, CrystallizedState, BlockVoteInfo};
use attestation::AttestationRecord;
use consts::CYCLE_LENGTH;

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

pub fn validate_attestation<JustifiedBlockHashes: StorageMap<u64, H256, Query=Option<H256>>>(
	slot: u64,
	parent_slot: u64,
	crystallized_state: &CrystallizedState,
	active_state: &ActiveState,
	attestation: &AttestationRecord
) {
	assert!(attestation.slot <= parent_slot);
	assert!(attestation.slot >= parent_slot.saturating_sub(CYCLE_LENGTH as u64 - 1));

	assert!(attestation.justified_slot <= crystallized_state.last_justified_slot);
	assert!(JustifiedBlockHashes::get(attestation.justified_slot).expect("Justified block hash not found, attestation validation failed") == attestation.justified_block_hash);

	let parent_hashes = active_state.signed_parent_block_hashes(slot, attestation);
	let attestation_indices = crystallized_state.attestation_indices(attestation);

	assert!(attestation.attester_bitfield.count() == attestation_indices.len());
	let pubkeys: Vec<_> = attestation_indices
		.iter()
		.enumerate()
		.filter(|(i, _)| attestation.attester_bitfield.has_voted(*i))
		.map(|(_, index)| crystallized_state.validators[*index].pubkey.clone())
		.collect();

	attestation.verify_signatures(&parent_hashes, &pubkeys);
}

pub fn update_block_vote_cache<BlockVoteCache: StorageMap<H256, BlockVoteInfo, Query=Option<BlockVoteInfo>>>(
	slot: u64,
	crystallized_state: &CrystallizedState,
	active_state: &ActiveState,
	attestation: &AttestationRecord
) {
	let parent_hashes = active_state.signed_parent_block_hashes(slot, attestation);
	let attestation_indices = crystallized_state.attestation_indices(attestation);

	for parent_hash in parent_hashes {
		if attestation.oblique_parent_hashes.contains(&parent_hash) {
			continue;
		}

		let mut info = BlockVoteCache::get(&parent_hash).unwrap_or_default();
		for (i, index) in attestation_indices.iter().enumerate() {
			if attestation.attester_bitfield.has_voted(i) && !info.voter_indices.contains(index) {
				info.voter_indices.push(*index);
				info.total_voter_deposits += crystallized_state.validators[*index].balance;
			}
		}
		BlockVoteCache::insert(parent_hash, info);
	}
}

pub fn process_block<JustifiedBlockHashes: StorageMap<u64, H256, Query=Option<H256>>, BlockVoteCache: StorageMap<H256, BlockVoteInfo, Query=Option<BlockVoteInfo>>>(
	slot: u64,
	parent_slot: u64,
	crystallized_state: &CrystallizedState,
	active_state: &mut ActiveState,
	attestations: &[AttestationRecord]
) {
	validate_parent_block_proposer(slot, parent_slot, crystallized_state, attestations);

	for attestation in attestations {
		validate_attestation::<JustifiedBlockHashes>(
			slot,
			parent_slot,
			crystallized_state,
			active_state,
			attestation
		);
		update_block_vote_cache::<BlockVoteCache>(
			slot,
			crystallized_state,
			active_state,
			attestation
		);
	}

	active_state.pending_attestations.append(&mut attestations.iter().cloned().collect());
}
