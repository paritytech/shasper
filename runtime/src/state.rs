use primitives::H256;

use super::Address;
use attestation::AttestationRecord;

pub struct ValidatorRecord {
	pub pubkey: H256,
	pub withdrawal_shard: u16,
	pub withdrawal_address: Address,
	pub randao_commitment: H256,
	pub balance: u128,
	pub start_dynasty: u64,
	pub end_dynasty: u64,
}

pub struct CrosslinkRecord {
	pub dynasty: u64,
	pub slot: u64,
	pub hash: H256,
}

pub struct ShardAndCommittee {
	pub shard_id: u16,
	pub committee: Vec<u32>,
}

pub struct ActiveState {
	pub pending_attestation: Vec<AttestationRecord>,
	pub recent_block_hashes: Vec<H256>,
}

pub struct CrystallizedState {
	pub validators: Vec<ValidatorRecord>,
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
