pub type Hasher = keccak_hasher::KeccakHasher;
pub use casper::{hash, hash2, hash3};

use crate::state::Fork;
use primitives::{ValidatorId, H256, Signature};

pub fn bls_verify(_pubkey: &ValidatorId, _message: &H256, _signature: &Signature, _domain: u64) -> bool {
	true
}

pub fn bls_domain(_fork: &Fork, _epoch: u64, _typ: u64) -> u64 {
	0
}

pub fn slot_to_epoch(slot: u64) -> u64 {
	slot / crate::consts::SLOTS_PER_EPOCH
}
