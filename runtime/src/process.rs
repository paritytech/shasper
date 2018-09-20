use primitives::H256;
use runtime_support::storage::{StorageValue, StorageMap};
use rstd::prelude::*;

use super::{BlockNumber, Hash, Block};
use attestation::AttestationRecord;
use header::Header;
use state::{ActiveState, CrystallizedState};
use consts::CYCLE_LENGTH;

storage_items! {
	Number: b"sys:num" => required BlockNumber;
	// We set parent hash and parent slot to current hash at the end of the block. Not sure whether there're better ways
	// to handle this state transition.
	ParentHash: b"sys:parenthash" => required Hash;
	ParentSlot: b"sys:parentslot" => required u64;
	JustifiedBlockHashes: b"sys:justifiedblockhashes" => required map [ u64 => H256 ];
	Active: b"sys:active" => required ActiveState;
	Crystallized: b"sys:crystallized" => required CrystallizedState;
}

pub fn initialise_block(header: Header) {
	assert_eq!(<ParentHash>::get(), header.parent_hash);

	<Number>::put(&header.number);
}

pub fn execute_block(block: Block) {
	let ref header = block.header;

	let mut active = <Active>::get();
	let mut crystallized = <Crystallized>::get();

	let parent_hash = block.header.parent_hash;
	let parent_slot = <ParentSlot>::get();
	let slot_number = block.extrinsics[0].slot_number().expect("Expect index 0 to be slot number");
	let randao_reveal = block.extrinsics[1].randao_reveal().expect("Expect index 1 to be randao reveal");
	let pow_chain_ref = block.extrinsics[2].pow_chain_ref().expect("Expect index 2 to be pow chain ref");

	assert!(slot_number > parent_slot);

	update_recent_block_hashes(&mut active, parent_slot, slot_number, parent_hash);

	for i in 3..block.extrinsics.len() {
		verify_attestation(
			&block.extrinsics[i].attestation().expect("Expect index 3+ to be attestation"),
			&active,
			&crystallized,
			parent_slot
		);
	}
}

fn update_recent_block_hashes(active: &mut ActiveState, parent_slot: u64, current_slot: u64, parent_hash: H256) {
	let d = (current_slot - parent_slot) as usize;
	let mut recent_block_hashes: Vec<H256> = active.recent_block_hashes[d..].iter().cloned().collect();
	for _ in 0..::rstd::cmp::min(d, active.recent_block_hashes.len()) {
		recent_block_hashes.push(parent_hash);
	}
	active.recent_block_hashes = recent_block_hashes;
}

fn verify_attestation(
	attestation: &AttestationRecord,
	active: &ActiveState,
	crystallized: &CrystallizedState,
	parent_slot: u64
) {
	assert!(attestation.slot <= parent_slot);
	assert!(attestation.slot >= parent_slot.saturating_sub(CYCLE_LENGTH as u64 - 1));

	assert!(attestation.justified_slot <= crystallized.last_justified_slot);
	assert_eq!(attestation.justified_block_hash, <JustifiedBlockHashes>::get(attestation.justified_slot));

	let attestation_indices = crystallized.shard_and_committee_for_slots[attestation.slot as usize].iter().find(|v| v.shard_id == attestation.shard_id).expect("Attestation must exist").committee.clone();
	assert_eq!(attestation.attester_bitfield.len(), (attestation_indices.len() + 7) / 8);

	let mut _group_public_key = H256::new();
	for (i, validator_index) in attestation_indices.into_iter().enumerate() {
		if (attestation.attester_bitfield[i / 8] >> (7 - (i % 8))) % 2 == 1 {
			_group_public_key = _group_public_key | crystallized.validators.0[validator_index as usize].pubkey;
		}
	}

	// TODO: actually verify the message via blake2s.
	assert!(true);
}
