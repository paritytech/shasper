extern crate substrate_primitives as primitives;
extern crate sr_primitives as runtime_primitives;

use primitives::{H256, U256};
use runtime_primitives::traits::{Block as BlockT};

pub struct AttestationRecord {
    slot: u64,
    shard_id: u16,
    oplique_parent_hashes: Vec<H256>,
    shard_block_hash: H256,
    attester_bitfield: Vec<u8>,
    justified_slot: u64,
    justified_block_hash: H256,
    aggregate_sig: Vec<U256>,
}

pub struct Header {
    parent_hash: H256,
    slot_number: u64,
    randao_reveal: H256,
    attestations: Vec<AttestationRecord>,
    pow_chain_ref: H256,
    active_state_root: H256,
    crystallized_state_root: H256,
}

fn main() {
    println!("Hello, world!");
}
