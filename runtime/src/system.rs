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

use primitives::{self, H256, Blake2Hasher};
use runtime_support::storage::{StorageValue, StorageMap};
use rstd::prelude::*;

use super::{BlockNumber, Hash, Block};
use ssz_hash::SpecHash;
use header::{Header, Digest};
use state::{ActiveState, CrystallizedState, BlockVoteInfo};
use extrinsic::Extrinsic;
use validation;
use runtime_io;

storage_items! {
	Number: b"sys:num" => default BlockNumber;
	ParentNumber: b"sys:parentnumber" => default BlockNumber;
	ParentHash: b"sys:parenthash" => default Hash;
	ParentSlot: b"sys:parentslot" => default u64;
	BlockHashesBySlot: b"sys:blockhashesbyslot" => map [ u64 => H256 ];
	Active: b"sys:active" => default ActiveState;
	ActiveRoot: b"sys:activeroot" => default H256;
	Crystallized: b"sys:crystallized" => default CrystallizedState;
	CrystallizedRoot: b"sys:crystallizedroot" => default H256;
	BlockVoteCache: b"sys:blockvotecache" => default map [ H256 => BlockVoteInfo ];
}

pub fn authorities() -> Vec<primitives::AuthorityId> {
	Vec::new()
}

pub fn initialise_block(header: Header) {
	Number::put(&header.number);
}

pub fn apply_extrinsic(extrinsic: Extrinsic) {
	state_transition(extrinsic)
}

pub fn execute_block(mut block: Block) {
	Number::put(&block.header.number);

	state_transition(block.extrinsics.remove(0))
}

// FIXME #27: fix header fields.
pub fn finalise_block() -> Header {
	Header {
		state_root: H256::from_slice(runtime_io::storage_root().as_ref()),
		digest: Digest {
			logs: Vec::new(),
		},
		extrinsics_root: H256::default(),
		number: Number::get(),
		parent_hash: ParentHash::get(),
	}
}

pub fn inherent_extrinsics() -> Vec<Extrinsic> {
	Vec::new()
}

fn state_transition(extrinsic: Extrinsic) {
	assert_eq!(Number::get(), ParentNumber::get() + 1);

	let number = Number::get();
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

	let active_state_root = active_state.spec_hash::<Blake2Hasher>();
	let crystallized_state_root = crystallized_state.spec_hash::<Blake2Hasher>();
	let block_hash = extrinsic.header_spec_hash(parent_hash, active_state_root, crystallized_state_root);

	ParentNumber::put(&number);
	ParentHash::put(&block_hash);
	ParentSlot::put(&slot);
	BlockHashesBySlot::insert(slot, block_hash);
	Active::put(&active_state);
	ActiveRoot::put(&active_state_root);
	Crystallized::put(&crystallized_state);
	CrystallizedRoot::put(&crystallized_state_root);
}
