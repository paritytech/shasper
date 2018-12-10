extern crate shasper_primitives;
extern crate sr_std as rstd;

use rstd::collections::btree_map::BTreeMap;
use shasper_primitives::{H384, H256, Count, Slot};

pub struct SimplifiedTreeRoute {
	pub retracted: Vec<(Slot, H256)>,
	pub common: (Slot, H256),
	pub enacted: Vec<(Slot, H256)>,
}

pub struct GlobalMetadata {
	pub latest_attestation_targets: BTreeMap<H384, (Slot, H256)>,
	pub finalized: (Slot, H256),
	pub justified: Vec<(Slot, H256)>,
}

impl GlobalMetadata {
	pub fn finalize(&mut self, block: (Slot, H256), retracted: Vec<(Slot, H256)>) {
		self.justified.retain(|b| b.0 > block.0 && !retracted.iter().any(|c| c.1 == b.1));
		self.finalized = block;
	}

	pub fn justify(&mut self, block: (Slot, H256)) {
		self.justified.push(block);
		self.justified.sort_by_key(|b| b.0);
	}

	pub fn justified_head(&self, current_slot: Slot, epoch_length: Slot) -> (Slot, H256) {
		self.justified.last()
			.and_then(|last| {
				if current_slot >= last.0 + epoch_length {
					Some(last.clone())
				} else {
					None
				}
			})
			.unwrap_or(self.finalized.clone())
	}
}

pub type BlockMetadata = Count;

pub struct ImportRequest {
	pub new_attestation_targets: BTreeMap<H384, (Slot, H256)>,
	pub finalize: Option<((Slot, H256), Vec<(Slot, H256)>)>,
	pub justify: Option<(Slot, H256)>,
}
