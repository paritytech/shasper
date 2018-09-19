use primitives::{H256, U256};

#[derive(Clone, PartialEq, Eq, Debug, Decode, Encode, Default, Serialize, Deserialize)]
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
