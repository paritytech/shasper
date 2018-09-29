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

}
