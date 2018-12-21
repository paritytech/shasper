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
use runtime_support::storage::StorageMap;
use rstd::prelude::*;
use rstd::collections::btree_map::BTreeMap;

use state::{ActiveState, CrystallizedState, BlockVoteInfo, CrosslinkRecord};
use attestation::AttestationRecord;
use consts::{CYCLE_LENGTH, WEI_PER_ETH, BASE_REWARD_QUOTIENT, SQRT_E_DROP_TIME, SLOT_DURATION, MIN_DYNASTY_LENGTH, SHARD_COUNT};
use utils::sqrt;
use primitives::ShardId;

pub fn validate_block_pre_processing_conditions() { }

pub fn validate_parent_block_proposer(slot: u64, parent_slot: u64, crystallized_state: &CrystallizedState, attestations: &[AttestationRecord]) {
	if slot == 0 {
		return;
	}

	let (proposer_index_in_committee, shard_id) = crystallized_state.proposer_position(parent_slot);

	if attestations.len() == 0 {
		return;
	}
	let attestation = &attestations[0];

	assert!(attestation.shard_id == shard_id &&
			attestation.slot == parent_slot &&
			attestation.attester_bitfield.has_voted(proposer_index_in_committee));
}

pub fn validate_attestation<BlockHashesBySlot: StorageMap<u64, H256, Query=Option<H256>>>(
	slot: u64,
	parent_slot: u64,
	crystallized_state: &CrystallizedState,
	active_state: &ActiveState,
	attestation: &AttestationRecord
) {
	assert!(attestation.slot <= parent_slot);
	assert!(attestation.slot >= parent_slot.saturating_sub(CYCLE_LENGTH as u64 - 1));

	assert!(attestation.justified_slot <= crystallized_state.last_justified_slot);
	assert!(BlockHashesBySlot::get(attestation.justified_slot).expect("Justified block hash not found, attestation validation failed") == attestation.justified_block_hash);

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

pub fn process_block<BlockHashesBySlot: StorageMap<u64, H256, Query=Option<H256>>, BlockVoteCache: StorageMap<H256, BlockVoteInfo, Query=BlockVoteInfo>>(
	slot: u64,
	parent_slot: u64,
	crystallized_state: &CrystallizedState,
	active_state: &mut ActiveState,
	attestations: &[AttestationRecord]
) {
	validate_parent_block_proposer(slot, parent_slot, crystallized_state, attestations);

	for attestation in attestations {
		validate_attestation::<BlockHashesBySlot>(
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

pub fn calculate_ffg_rewards<BlockHashesBySlot: StorageMap<u64, H256, Query=Option<H256>>, BlockVoteCache: StorageMap<H256, BlockVoteInfo, Query=BlockVoteInfo>>(
	slot: u64,
	crystallized_state: &CrystallizedState,
) -> Vec<i128> {
	let active_validator_indices = crystallized_state.active_validator_indices();
	let mut rewards_and_penalties: Vec<i128> = crystallized_state.validators.iter().map(|_| 0).collect();

	let time_since_finality = slot - crystallized_state.last_finalized_slot;

	let total_deposits = crystallized_state.total_deposits();
	let total_deposits_in_eth = total_deposits / WEI_PER_ETH;

	if total_deposits_in_eth == 0 {
		return rewards_and_penalties;
	}

	let reward_quotient = BASE_REWARD_QUOTIENT * sqrt(total_deposits_in_eth);
	let quadratic_penalty_quotient = (SQRT_E_DROP_TIME / SLOT_DURATION).pow(2);

	let last_state_recalc = crystallized_state.last_state_recalc;

	for slot in last_state_recalc.saturating_sub(CYCLE_LENGTH as u64)..last_state_recalc {
		let block_hash = BlockHashesBySlot::get(slot);

		let (total_participated_deposits, voter_indices) = match block_hash {
			Some(block_hash) => {
				let cache = BlockVoteCache::get(block_hash);
				(cache.total_voter_deposits, cache.voter_indices)
			},
			None => {
				(0, Vec::new())
			},
		};

		let participating_validator_indices: Vec<_> = active_validator_indices
			.iter()
			.filter(|index| voter_indices.contains(index))
			.cloned()
			.collect();

		let non_participating_validator_indices: Vec<_> = active_validator_indices
			.iter()
			.filter(|index| !voter_indices.contains(index))
			.cloned()
			.collect();

		if time_since_finality <= 3 * CYCLE_LENGTH as u64 {
			for index in participating_validator_indices {
				let balance = crystallized_state.validators[index].balance;

				rewards_and_penalties[index] += (
					balance /
						reward_quotient *
						(2 * total_participated_deposits - total_deposits) /
						total_deposits
				) as i128;
			}
			for index in non_participating_validator_indices {
				let balance = crystallized_state.validators[index].balance;

				rewards_and_penalties[index] -= (
					balance /
						reward_quotient
				) as i128;
			}
		} else {
			for index in non_participating_validator_indices {
				let balance = crystallized_state.validators[index].balance;

				rewards_and_penalties[index] -= (
					(balance / reward_quotient) +
						(balance * time_since_finality as u128 / quadratic_penalty_quotient)
				) as i128;
			}
		}
	}

	rewards_and_penalties
}

pub fn calculate_crosslink_rewards(
	_slot: u64,
	crystallized_state: &CrystallizedState,
) -> Vec<i128> {
	crystallized_state.validators.iter().map(|_| 0).collect()
}

pub fn apply_rewards_and_penalties<BlockHashesBySlot: StorageMap<u64, H256, Query=Option<H256>>, BlockVoteCache: StorageMap<H256, BlockVoteInfo, Query=BlockVoteInfo>>(
	slot: u64,
	crystallized_state: &mut CrystallizedState,
) {
	let ffg_rewards = calculate_ffg_rewards::<BlockHashesBySlot, BlockVoteCache>(slot, crystallized_state);
	let crosslink_rewards = calculate_crosslink_rewards(slot, crystallized_state);

	let active_validator_indices = crystallized_state.active_validator_indices();

	for index in active_validator_indices {
		if ffg_rewards[index] > 0 {
			crystallized_state.validators[index].balance += ffg_rewards[index] as u128;
		}
		if ffg_rewards[index] < 0 {
			crystallized_state.validators[index].balance = crystallized_state.validators[index].balance.saturating_sub((-ffg_rewards[index]) as u128);
		}

		if crosslink_rewards[index] > 0 {
			crystallized_state.validators[index].balance += crosslink_rewards[index] as u128;
		}
		if crosslink_rewards[index] < 0 {
			crystallized_state.validators[index].balance = crystallized_state.validators[index].balance.saturating_sub((-crosslink_rewards[index]) as u128);
		}
	}
}

pub fn initialize_new_cycle<BlockHashesBySlot: StorageMap<u64, H256, Query=Option<H256>>, BlockVoteCache: StorageMap<H256, BlockVoteInfo, Query=BlockVoteInfo>>(
	slot: u64,
	crystallized_state: &mut CrystallizedState,
	active_state: &mut ActiveState
) {
	let last_state_recalc = crystallized_state.last_state_recalc;
	let total_deposits = crystallized_state.total_deposits();

	let mut last_justified_slot = crystallized_state.last_justified_slot;
	let mut last_finalized_slot = crystallized_state.last_finalized_slot;
	let mut justified_streak = crystallized_state.justified_streak;

	for i in 0..CYCLE_LENGTH {
		let slot = i as u64 + last_state_recalc.saturating_sub(CYCLE_LENGTH as u64);

		let block_hash = active_state.recent_block_hashes[i];
		let vote_balance = BlockVoteCache::get(&block_hash).total_voter_deposits;

		if 3 * vote_balance >= 2 * total_deposits {
			last_justified_slot = ::rstd::cmp::max(last_justified_slot, slot);
			justified_streak += 1;
		} else {
			justified_streak = 0;
		}

		if justified_streak >= CYCLE_LENGTH as u64 + 1 {
			last_finalized_slot = ::rstd::cmp::max(last_finalized_slot, slot.saturating_sub(CYCLE_LENGTH as u64 - 1));
		}
	}

	process_updated_crosslinks(crystallized_state, active_state);

	active_state.pending_attestations = active_state.pending_attestations.clone()
		.into_iter()
		.filter(|a| a.slot >= last_state_recalc)
		.collect();

	apply_rewards_and_penalties::<BlockHashesBySlot, BlockVoteCache>(slot, crystallized_state);

	let mut new_shards_and_committees_for_slots: Vec<_> = crystallized_state.shards_and_committees_for_slots[CYCLE_LENGTH..].iter().cloned().collect();
	let mut copied_shards_and_committees_for_slots = new_shards_and_committees_for_slots.clone();
	new_shards_and_committees_for_slots.append(&mut copied_shards_and_committees_for_slots);

	crystallized_state.last_state_recalc = last_state_recalc + CYCLE_LENGTH as u64;
	crystallized_state.shards_and_committees_for_slots = new_shards_and_committees_for_slots;
	crystallized_state.last_justified_slot = last_justified_slot;
	crystallized_state.justified_streak = justified_streak;
	crystallized_state.last_finalized_slot = last_finalized_slot;
}

pub fn is_ready_for_dynasty_transition(
	slot: u64,
	crystallized_state: &CrystallizedState
) -> bool {
	let slots_since_last_dynasty_change = slot - crystallized_state.dynasty_start;
	if slots_since_last_dynasty_change < MIN_DYNASTY_LENGTH {
		return false;
	}

	if crystallized_state.last_finalized_slot <= crystallized_state.dynasty_start {
		return false;
	}

	let mut required_shards = Vec::new();
	for shards_and_committees_for_slot in &crystallized_state.shards_and_committees_for_slots {
		for shard_and_committee in shards_and_committees_for_slot {
			required_shards.push(shard_and_committee.shard_id);
		}
	}

	for (shard_id, crosslink) in crystallized_state.crosslink_records.iter().enumerate() {
		if required_shards.contains(&(shard_id as u16)) {
			if crosslink.slot <= crystallized_state.dynasty_start {
				return false;
			}
		}
	}

	return true;
}

pub fn process_dynasty_transition(
	parent_hash: H256,
	crystallized_state: &mut CrystallizedState
) {
	let new_start_shard = (crystallized_state.shards_and_committees_for_slots.last().expect("There must be at least one shard_and_committee").last().expect("There must be at least one shard_and_committee").shard_id + 1) % SHARD_COUNT;

	let mut new_shards_and_committees: Vec<_> = crystallized_state.shards_and_committees_for_slots[CYCLE_LENGTH..].iter().cloned().collect();
	new_shards_and_committees.append(&mut crystallized_state.new_shuffling(parent_hash, new_start_shard));
	crystallized_state.shards_and_committees_for_slots = new_shards_and_committees;

	crystallized_state.current_dynasty += 1;
	crystallized_state.dynasty_start = crystallized_state.last_state_recalc;
}

pub fn process_cycle_transitions<BlockHashesBySlot: StorageMap<u64, H256, Query=Option<H256>>, BlockVoteCache: StorageMap<H256, BlockVoteInfo, Query=BlockVoteInfo>>(
	slot: u64,
	parent_hash: H256,
	crystallized_state: &mut CrystallizedState,
	active_state: &mut ActiveState
) {
	while slot >= crystallized_state.last_state_recalc + CYCLE_LENGTH as u64 {
		initialize_new_cycle::<BlockHashesBySlot, BlockVoteCache>(slot, crystallized_state, active_state);

		if is_ready_for_dynasty_transition(slot, crystallized_state) {
			process_dynasty_transition(parent_hash, crystallized_state);
		}
	}
}
