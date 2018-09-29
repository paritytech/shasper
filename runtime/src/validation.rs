use primitives::H256;
use runtime_support::storage::StorageMap;
use rstd::collections::btree_map::BTreeMap;

use state::{ActiveState, CrystallizedState, BlockVoteInfo, CrosslinkRecord};
use attestation::AttestationRecord;
use consts::CYCLE_LENGTH;
use ::ShardId;

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

pub fn update_block_vote_cache<BlockVoteCache: StorageMap<H256, BlockVoteInfo, Query=BlockVoteInfo>>(
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

		let mut info = BlockVoteCache::get(&parent_hash);
		for (i, index) in attestation_indices.iter().enumerate() {
			if attestation.attester_bitfield.has_voted(i) && !info.voter_indices.contains(index) {
				info.voter_indices.push(*index);
				info.total_voter_deposits += crystallized_state.validators[*index].balance;
			}
		}
		BlockVoteCache::insert(parent_hash, info);
	}
}

pub fn process_block<JustifiedBlockHashes: StorageMap<u64, H256, Query=Option<H256>>, BlockVoteCache: StorageMap<H256, BlockVoteInfo, Query=BlockVoteInfo>>(
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

pub fn process_updated_crosslinks(
	crystallized_state: &mut CrystallizedState,
	active_state: &ActiveState
) {
	let mut total_attestation_balance: BTreeMap<(ShardId, H256), u128> = Default::default();

	for attestation in &active_state.pending_attestations {
		let shard_tuple = (attestation.shard_id, attestation.shard_block_hash);

		let attestation_indices = crystallized_state.attestation_indices(attestation);
		let total_committee_balance = attestation_indices
			.iter()
			.fold(0, |acc, index| {
				acc + crystallized_state.validators[*index].balance
			});

		*total_attestation_balance.entry(shard_tuple).or_insert(0) += attestation_indices
			.iter()
			.enumerate()
			.filter(|(in_cycle_slot_height, _)| {
				attestation.attester_bitfield.has_voted(*in_cycle_slot_height)
			})
			.fold(0, |acc, (_, index)| {
				acc + crystallized_state.validators[*index].balance
			});

		if 3 * *total_attestation_balance.entry(shard_tuple).or_insert(0) >= 2 * total_committee_balance && crystallized_state.current_dynasty > crystallized_state.crosslink_records[attestation.shard_id as usize].dynasty {
			crystallized_state.crosslink_records[attestation.shard_id as usize] = CrosslinkRecord {
				dynasty: crystallized_state.current_dynasty,
				slot: crystallized_state.last_state_recalc + CYCLE_LENGTH as u64,
				hash: attestation.shard_block_hash
			};
		}
	}
}
