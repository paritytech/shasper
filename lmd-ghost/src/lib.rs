// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! LMD-GHOST fork choice rule implementation. Caller needs to first create a
//! `FinalizationState`. When processing blocks, create an `ImportRequest` and
//! pass it to `process`, and apply `ImportResult` to blockchain.

extern crate shasper_primitives;
extern crate sr_std as rstd;

use rstd::collections::btree_map::BTreeMap;
use shasper_primitives::{H256, Count, Slot};

/// Simplified version of Substrate tree route.
pub struct SimplifiedTreeRoute {
	/// Retracted block slot, block hash, and current count metadata.
	pub retracted: Vec<(Slot, H256, Count)>,
	/// Common block slot, block hash, and current count metadata.
	pub common: (Slot, H256, Count),
	/// Enacted block slot, block hash, and current count metadata.
	pub enacted: Vec<(Slot, H256, Count)>,
}

/// State related to finalization.
pub struct FinalizationState {
	/// Last finalized block slot and block hash.
	pub finalized: (Slot, H256),
	/// Justified block slots and block hashes.
	pub justified: Vec<(Slot, H256)>,
}

impl FinalizationState {
	fn finalize(&mut self, block: (Slot, H256), retracted: Vec<(Slot, H256)>) {
		self.justified.retain(|b| b.0 > block.0 && !retracted.iter().any(|c| c.1 == b.1));
		self.finalized = block;
	}

	fn justify(&mut self, block: (Slot, H256)) {
		self.justified.push(block);
		self.justified.sort_by_key(|b| b.0);
	}

	fn justified_head(&self, current_slot: Slot, epoch_length: Slot) -> (Slot, H256) {
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

/// An import request.
pub struct ImportRequest {
	/// New attestation targets. The caller should keep track of the current validators'
	/// attestation targets. When a new block is imported, a tree route should be calculated
	/// from the current attestation target to new attestation target.
	pub new_attestation_targets: Vec<SimplifiedTreeRoute>,
	/// Finalizing block, if any, and retracted blocks due to finalization.
	pub finalize: Option<((Slot, H256), Vec<(Slot, H256)>)>,
	/// Justifying block, if any.
	pub justify: Option<(Slot, H256)>,
	/// Tree route from current head to new block.
	pub route: SimplifiedTreeRoute,
	/// New block's slot.
	pub current_slot: Slot,
}

/// An import response. The caller should apply this to blockchain database.
pub struct ImportResponse {
	/// All updated vote counts to be applied to block metadata.
	pub updated_vote_counts: BTreeMap<H256, Count>,
	/// Whether the new block is the best block.
	pub is_new_best: bool,
}

/// Process a new block given state and import request.
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
