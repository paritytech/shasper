use primitives::{H256, U256};
use rstd::prelude::*;

#[derive(Clone, PartialEq, Eq, Decode, Encode, SszEncode, SszDecode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AttestationRecord {
	pub slot: u64,
	pub shard_id: u16,
	pub oplique_parent_hashes: Vec<H256>,
	pub shard_block_hash: H256,
	pub attester_bitfield: Vec<u8>,
	pub justified_slot: u64,
	pub justified_block_hash: H256,
	pub aggregate_sig: Vec<U256>,
}

impl Default for AttestationRecord {
	fn default() -> Self {
		Self {
			slot: 0,
			shard_id: 0,
			oplique_parent_hashes: vec![],
			shard_block_hash: H256::new(),
			attester_bitfield: vec![],
			justified_slot: 0,
			justified_block_hash: H256::new(),
			aggregate_sig: vec![U256::zero(), U256::zero()],
		}
	}
}
