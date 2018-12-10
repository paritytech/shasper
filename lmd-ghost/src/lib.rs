extern crate shasper_primitives;
extern crate sr_std as rstd;

use rstd::collections::btree_map::BTreeMap;
use shasper_primitives::{H256, Count, Slot};

pub struct SimplifiedTreeRoute {
	pub retracted: Vec<(Slot, H256, Count)>,
	pub common: (Slot, H256, Count),
	pub enacted: Vec<(Slot, H256, Count)>,
}

pub struct FinalizationState {
	pub finalized: (Slot, H256),
	pub justified: Vec<(Slot, H256)>,
}

impl FinalizationState {
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

#[derive(Default)]
struct VoteCounts(BTreeMap<H256, Count>);

impl VoteCounts {
	pub fn import_target(&mut self, target: SimplifiedTreeRoute) {
		for block in target.retracted {
			*self.0.entry(block.1).or_default() -= 1;
		}
		for block in target.enacted {
			*self.0.entry(block.1).or_default() += 1;
		}
	}

	pub fn get(&self, hash: &H256) -> Option<Count> {
		self.0.get(hash).map(|c| *c)
	}

	pub fn drain(self) -> BTreeMap<H256, Count> {
		self.0
	}
}

struct VotedTreeRoute<'a> {
	vote_counts: &'a VoteCounts,
	route: SimplifiedTreeRoute,
}

impl<'a> VotedTreeRoute<'a> {
	pub fn is_new_best(&self) -> bool {
		let retracted_branch_count = self.route.retracted
			.last()
			.map(|block| self.vote_counts.get(&block.1).unwrap_or(block.2))
			.unwrap_or_default();
		let enacted_branch_count = self.route.enacted
			.first()
			.map(|block| self.vote_counts.get(&block.1).unwrap_or(block.2))
			.unwrap_or_default();

		enacted_branch_count >= retracted_branch_count
	}
}

pub struct ImportRequest {
	pub new_attestation_targets: Vec<SimplifiedTreeRoute>, // Tree route from old attestation target to new attestation target.
	pub finalize: Option<((Slot, H256), Vec<(Slot, H256)>)>,
	pub justify: Option<(Slot, H256)>,
	pub route: SimplifiedTreeRoute,
	pub current_slot: Slot,
}

pub struct ImportResponse {
	pub updated_vote_counts: BTreeMap<H256, Count>,
	pub is_new_best: bool,
}

pub fn process(state: &mut FinalizationState, request: ImportRequest, epoch_length: Slot) -> ImportResponse {
	for finalize in request.finalize {
		state.finalize(finalize.0, finalize.1);
	}
	for justify in request.justify {
		state.justify(justify);
	}

	let mut vote_counts = VoteCounts::default();
	for target in request.new_attestation_targets {
		vote_counts.import_target(target);
	}

	let justified_head = state.justified_head(request.current_slot, epoch_length);
	let is_new_best = if request.route.retracted.iter().any(|block| block.1 == justified_head.1) {
		false
	} else {
		let voted_route = VotedTreeRoute {
			vote_counts: &vote_counts,
			route: request.route,
		};

		voted_route.is_new_best()
	};

	ImportResponse {
		updated_vote_counts: vote_counts.drain(),
		is_new_best,
	}
}
