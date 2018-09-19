use runtime_support::storage::StorageValue;

use super::{BlockNumber, Hash};
use header::Header;

storage_items! {
	Number: b"sys:num" => required BlockNumber;
	ParentHash: b"sys:pha" => required Hash;
}

pub fn initialise_block(header: Header) {
	<Number>::put(&header.number);
	<ParentHash>::put(&header.parent_hash);
}
