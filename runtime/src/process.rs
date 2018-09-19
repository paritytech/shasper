use runtime_support::storage::StorageValue;

use super::{BlockNumber, Hash};
use header::Header;
use state::{ActiveState, CrystallizedState};

storage_items! {
	Number: b"sys:num" => required BlockNumber;
	ParentHash: b"sys:pha" => required Hash;
	Active: b"sys:active" => required ActiveState;
	Crystallized: b"sys:crystallized" => required CrystallizedState;
}

pub fn initialise_block(header: Header) {
	<Number>::put(&header.number);
	<ParentHash>::put(&header.parent_hash);
}
