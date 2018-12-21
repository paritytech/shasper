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

use primitives::{H256, ValidatorId, EthereumAddress};
use rstd::prelude::*;

#[derive(Clone, PartialEq, Eq, Default, Encode, Decode, SszEncode, SszDecode, SszHash)]
#[ssz_codec(sorted)]
pub struct ValidatorRecord {
	pub pubkey: ValidatorId,
	pub withdrawal_shard: u16,
	pub withdrawal_address: EthereumAddress,
	pub randao_commitment: H256,
	pub balance: u128,
	pub start_dynasty: u64,
	pub end_dynasty: u64,
}

#[derive(Clone, Encode, Decode, SszEncode, SszDecode, SszHash)]
#[ssz_codec(sorted)]
pub struct ShardAndCommittee {
	pub shard_id: u16,
	pub committee: Vec<u32>,
}
