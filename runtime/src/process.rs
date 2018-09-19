use primitives::H256;
use runtime_support::storage::StorageValue;

use super::{BlockNumber, Hash, Block};
use header::Header;
use state::{ActiveState, CrystallizedState};

storage_items! {
	Number: b"sys:num" => required BlockNumber;
	// We set parent hash and parent slot to current hash at the end of the block. Not sure whether there're better ways
	// to handle this state transition.
	ParentHash: b"sys:parenthash" => required Hash;
	ParentSlot: b"sys:parentslot" => required u64;
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
}

fn update_recent_block_hashes(active: &mut ActiveState, parent_slot: u64, current_slot: u64, parent_hash: H256) {
	let d = (current_slot - parent_slot) as usize;
	let mut recent_block_hashes: Vec<H256> = active.recent_block_hashes[d..].iter().cloned().collect();
	for _ in 0..::std::cmp::min(d, active.recent_block_hashes.len()) {
		recent_block_hashes.push(parent_hash);
	}
	active.recent_block_hashes = recent_block_hashes;
}
