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

use primitives::H256;
use rstd::prelude::Vec;
use super::AttestationRecord;

use ssz_derive::{SszEncode, SszDecode};
use ssz_hash_derive::SszHash;

#[derive(Clone, PartialEq, Eq, Default, SszEncode, SszDecode, SszHash)]
#[cfg_attr(feature = "std", derive(Debug))]
#[ssz_codec(sorted)]
pub struct SpecHeader {
	pub parent_hash: H256,
	pub slot_number: u64,
	pub randao_reveal: H256,
	pub attestations: Vec<AttestationRecord>,
	pub pow_chain_ref: H256,
	pub active_state_root: H256,
	pub crystallized_state_root: H256,
}
