use primitives::H256;
use rstd::prelude::*;

use attestation::AttestationRecord;
use validators::{Validators, ShardAndCommittee};

#[derive(Encode, Decode)]
pub struct CrosslinkRecord {
	pub dynasty: u64,
	pub slot: u64,
	pub hash: H256,
}

#[derive(Encode, Decode)]
pub struct ActiveState {
	pub pending_attestation: Vec<AttestationRecord>,
	pub recent_block_hashes: Vec<H256>,
}

#[derive(Encode, Decode)]
pub struct CrystallizedState {
	pub validators: Validators,
	pub last_state_recalc: u64,
	pub shard_and_committee_for_slots: Vec<Vec<ShardAndCommittee>>,
	pub last_justified_slot: u64,
	pub justified_streak: u64,
	pub last_finalized_slot: u64,
	pub current_dynasty: u64,
	pub crosslink_records: Vec<CrosslinkRecord>,
	pub dynasty_seed: H256,
	pub dynasty_start: u64,
}
