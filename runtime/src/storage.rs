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

use primitives::{BlockNumber, Hash, Epoch, Balance, ValidatorId};
use runtime_support::storage_items;
use runtime_support::storage::StorageValue;
use runtime_support::storage::unhashed::{self, StorageVec};
use crate::state::ValidatorRecord;
use crate::attestation::CheckedAttestation;
use crate::{UncheckedExtrinsic, Digest as DigestT, utils};

storage_items! {
	pub Number: b"sys:num" => default BlockNumber;
	pub ParentHash: b"sys:parenthash" => default Hash;
	pub ExtrinsicsRoot: b"sys:extrinsicsroot" => default Hash;
	pub Digest: b"sys:digest" => default DigestT;
	pub CasperContext: b"sys:caspercontext" => default casper::CasperContext<Epoch>;
}

pub struct UncheckedExtrinsics;
impl unhashed::StorageVec for UncheckedExtrinsics {
	type Item = Option<UncheckedExtrinsic>;
	const PREFIX: &'static [u8] = b"sys:extrinsics";
}

pub struct LatestStorageRoots;
impl unhashed::StorageVec for LatestStorageRoots {
	type Item = Option<Hash>;
	const PREFIX: &'static [u8] = b"sys:lateststorageroots";
}

pub struct PendingAttestations;
impl unhashed::StorageVec for PendingAttestations {
	type Item = Option<CheckedAttestation>;
	const PREFIX: &'static [u8] = b"sys:pendingattestations";
}

pub fn note_parent_hash() {
	let slot = Number::get() - 1;
	let hash = ParentHash::get();
	assert!(LatestStorageRoots::count() < slot as u32);
	for i in LatestStorageRoots::count()..(slot as u32) {
		LatestStorageRoots::set_item(i, &None);
	}
	LatestStorageRoots::set_item(slot as u32, &Some(hash));
}

pub struct Validators;
impl unhashed::StorageVec for Validators {
	type Item = Option<ValidatorRecord>;
	const PREFIX: &'static [u8] = b"sys:validators";
}

pub fn add_balance(validator_id: &ValidatorId, balance: Balance) {
	if let Some((index, Some(mut record))) = Validators::items().into_iter()
		.enumerate()
		.find(|(_, record)| record.as_ref().map(|r| &r.validator_id == validator_id).unwrap_or(false))
	{
		record.balance += balance;
		Validators::set_item(index as u32, &Some(record));
	}
}

pub fn sub_balance(validator_id: &ValidatorId, balance: Balance) {
	if let Some((index, Some(mut record))) = Validators::items().into_iter()
		.enumerate()
		.find(|(_, record)| record.as_ref().map(|r| &r.validator_id == validator_id).unwrap_or(false))
	{
		record.balance -= balance;
		Validators::set_item(index as u32, &Some(record));
	}
}

pub fn penalize_validator(validator_id: &ValidatorId, balance: Balance) {
	if let Some((index, Some(mut record))) = Validators::items().into_iter()
		.enumerate()
		.find(|(_, record)| record.as_ref().map(|r| &r.validator_id == validator_id).unwrap_or(false))
	{
		record.balance -= balance;
		record.valid_to = utils::slot_to_epoch(Number::get());
		Validators::set_item(index as u32, &Some(record));
	}
}
