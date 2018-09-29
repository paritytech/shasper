use primitives::H256;
use rstd::prelude::*;

use consts::CYCLE_LENGTH;
use attestation::AttestationRecord;
use validators::{ValidatorRecord, ShardAndCommittee};

#[derive(Encode, Decode, Default, SszEncode, SszDecode)]
#[ssz_codec(sorted)]
pub struct CrosslinkRecord {
	pub dynasty: u64,
	pub slot: u64,
	pub hash: H256,
}

#[derive(Encode, Decode, Default, SszEncode, SszDecode)]
#[ssz_codec(sorted)]
pub struct ActiveState {
	pub pending_attestation: Vec<AttestationRecord>,
	pub recent_block_hashes: Vec<H256>,
}

impl ActiveState {
	pub fn block_hash(&self, current_slot: u64, target_slot: u64) -> H256 {
		let current_slot = current_slot as usize;
		let target_slot = target_slot as usize;

		let sback = current_slot.saturating_sub(CYCLE_LENGTH * 2);
		assert!(sback <= target_slot && target_slot > sback + CYCLE_LENGTH * 2);
		self.recent_block_hashes[target_slot - sback]
	}

	pub fn block_hashes(&self, current_slot: u64, target_from_slot: u64, target_to_slot: u64) -> Vec<H256> {
		let mut ret = Vec::new();
		for target_slot in target_from_slot..(target_to_slot + 1) {
			ret.push(self.block_hash(current_slot, target_slot));
		}
		ret
	}

	pub fn block_hashes_to_sign(&self, current_slot: u64, current_hash: H256) -> Vec<H256> {
		let mut ret = self.block_hashes(
			current_slot,
			current_slot.saturating_sub(CYCLE_LENGTH as u64 - 1),
			current_slot.saturating_sub(1)
		);
		ret.push(current_hash);
		ret
	}
}

#[derive(Encode, Decode, Default, SszEncode, SszDecode)]
#[ssz_codec(sorted)]
pub struct CrystallizedState {
	pub validators: Vec<ValidatorRecord>,
	pub last_state_recalc: u64,
	pub shards_and_committees_for_slots: Vec<Vec<ShardAndCommittee>>,
	pub last_justified_slot: u64,
	pub justified_streak: u64,
	pub last_finalized_slot: u64,
	pub current_dynasty: u64,
	pub crosslink_records: Vec<CrosslinkRecord>,
	pub dynasty_seed: H256,
	pub dynasty_start: u64,
}

impl CrystallizedState {
	pub fn shards_and_committees_for_slot(&self, slot: u64) -> &[ShardAndCommittee] {
		let slot = slot as usize;
		let start = (self.last_state_recalc as usize).saturating_sub(CYCLE_LENGTH);
		assert!(start <= slot && slot > start + CYCLE_LENGTH * 2);
		&self.shards_and_committees_for_slots[slot - start]
	}

	pub fn proposer_position(&self, parent_slot: u64) -> (usize, u16) {
		let shard_and_committee = &self.shards_and_committees_for_slot(parent_slot)[0];

		assert!(shard_and_committee.committee.len() > 0);
		let proposer_index_in_committee = parent_slot as usize % shard_and_committee.committee.len();

		(proposer_index_in_committee, shard_and_committee.shard_id)
	}
}
