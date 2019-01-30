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

use primitives::{H256, BlockNumber, Hash, ValidatorId};
use primitives::storage::well_known_keys;
use runtime_support::storage_items;
use runtime_support::storage::unhashed;
use crate::state::{ActiveState, CrystallizedState, BlockVoteInfo};
use crate::{UncheckedExtrinsic, Digest as DigestT, AttestationRecord};

storage_items! {
	pub Number: b"sys:num" => default BlockNumber;
	pub ParentHash: b"sys:parenthash" => default Hash;
	pub ExtrinsicsRoot: b"sys:extrinsicsroot" => default Hash;
	pub Digest: b"sys:digest" => default DigestT;
	pub Timestamp: b"sys:timestamp" => default u64;
	pub Slot: b"sys:slot" => default u64;
	pub ParentSlot: b"sys:parentslot" => default u64;
	pub LastHeaderHash: b"sys:lasthash" => default H256;
	pub RandaoReveal: b"sys:randaoreveal" => default H256;
	pub PowChainRef: b"sys:powchainref" => default H256;

	pub BlockHashesBySlot: b"sys:blockhashesbyslot" => map [ u64 => H256 ];
	pub Active: b"sys:active" => default ActiveState;
	pub ActiveRoot: b"sys:activeroot" => default H256;
	pub Crystallized: b"sys:crystallized" => default CrystallizedState;
	pub CrystallizedRoot: b"sys:crystallizedroot" => default H256;
	pub BlockVoteCache: b"sys:blockvotecache" => default map [ H256 => BlockVoteInfo ];
}

pub struct UncheckedExtrinsics;
impl unhashed::StorageVec for UncheckedExtrinsics {
	type Item = UncheckedExtrinsic;
	const PREFIX: &'static [u8] = b"sys:extrinsics";
}

pub struct Authorities;
impl unhashed::StorageVec for Authorities {
	type Item = ValidatorId;
	const PREFIX: &'static [u8] = well_known_keys::AUTHORITY_PREFIX;
}

pub struct Attestations;
impl unhashed::StorageVec for Attestations {
	type Item = AttestationRecord;
	const PREFIX: &'static [u8] = b"sys:attestations";
}
