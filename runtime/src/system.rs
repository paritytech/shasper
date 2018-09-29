use primitives::H256;
use runtime_support::storage::StorageValue;
use rstd::prelude::*;

use super::{BlockNumber, Hash, Block};
use header::Header;
use state::{ActiveState, CrystallizedState, BlockVoteInfo};
use validation;

storage_items! {
	Number: b"sys:num" => required BlockNumber;
	// We set parent hash and parent slot to current hash at the end of the block. Not sure whether there're better ways
	// to handle this state transition.
	ParentHash: b"sys:parenthash" => required Hash;
	ParentSlot: b"sys:parentslot" => required u64;
	BlockHashesBySlot: b"sys:blockhashesbyslot" => map [ u64 => H256 ];
	Active: b"sys:active" => required ActiveState;
	Crystallized: b"sys:crystallized" => required CrystallizedState;
	BlockVoteCache: b"sys:blockvotecache" => required map [ H256 => BlockVoteInfo ];
}

pub fn initialise_block(header: Header) {
	assert_eq!(ParentHash::get(), header.parent_hash);

	Number::put(&header.number);
}

pub fn execute_block(block: Block) {
	let extrinsic = &block.extrinsics[0];
	let slot = extrinsic.slot_number;
	let parent_hash = ParentHash::get();
	let parent_slot = ParentSlot::get();
	let attestations = &extrinsic.attestations;

	let mut active_state = Active::get();
	let mut crystallized_state = Crystallized::get();

	validation::validate_block_pre_processing_conditions();
	active_state.update_recent_block_hashes(parent_slot, slot, parent_hash);

	validation::process_block::<BlockHashesBySlot, BlockVoteCache>(
		slot,
		parent_slot,
		&crystallized_state,
		&mut active_state,
		attestations
	);

	validation::process_cycle_transitions::<BlockHashesBySlot, BlockVoteCache>(
		slot,
		parent_hash,
		&mut crystallized_state,
		&mut active_state
	);

	ParentSlot::put(&slot);
	// TODO: Update ParentHash
}
