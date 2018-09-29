use primitives::H256;
use rstd::prelude::*;

use super::Address;

#[derive(Clone, PartialEq, Eq, Default, Encode, Decode, SszEncode, SszDecode)]
#[ssz_codec(sorted)]
pub struct ValidatorRecord {
	pub pubkey: H256,
	pub withdrawal_shard: u16,
	pub withdrawal_address: Address,
	pub randao_commitment: H256,
	pub balance: u128,
	pub start_dynasty: u64,
	pub end_dynasty: u64,
}

#[derive(Encode, Decode, SszEncode, SszDecode)]
#[ssz_codec(sorted)]
pub struct ShardAndCommittee {
	pub shard_id: u16,
	pub committee: Vec<u32>,
}
