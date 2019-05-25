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

use runtime_support::storage_items;
use runtime_support::storage::unhashed;
use crypto::bls;
use beacon::ParameteredConfig;
use beacon::types::BeaconState;
use crate::{BlockNumber, Hash, Extrinsic};

storage_items! {
	pub State: b"sys:state" => BeaconState;
	pub Config: b"sys:config" => ParameteredConfig<bls::Verification>;
	pub Authority: b"sys:authority" => default super::AuthorityId;

	pub Number: b"sys:num" => default BlockNumber;
	pub ParentHash: b"sys:parenthash" => default Hash;
	pub Digest: b"sys:digest" => default super::Digest;
}

#[allow(dead_code)]
pub struct Extrinsics;
impl unhashed::StorageVec for Extrinsics {
	type Item = Option<Extrinsic>;
	const PREFIX: &'static [u8] = b"sys:extrinsics";
}
