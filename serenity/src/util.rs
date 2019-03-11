pub type Hasher = keccak_hasher::KeccakHasher;

use crate::state::Fork;
use hash_db::Hasher as _;
use primitives::{ValidatorId, H256, Signature};

pub fn bls_verify(pubkey: &ValidatorId, message: &H256, signature: &Signature, _domain: u64) -> bool {
	pubkey.into_public()
		.map(|public| {
			signature.into_signature().map(|signature| {
				public.verify(&message[..], &signature)
			}).unwrap_or(false)
		})
		.unwrap_or(false)
}

pub fn bls_domain(_fork: &Fork, _epoch: u64, _typ: u64) -> u64 {
	0
}

/// Hash bytes with a hasher.
pub fn hash(seed: &[u8]) -> H256 {
	Hasher::hash(seed)
}

/// Hash two bytes with a hasher.
pub fn hash2(seed: &[u8], a: &[u8]) -> H256 {
	let mut v = seed.to_vec();
	let mut a = a.to_vec();
	v.append(&mut a);
	Hasher::hash(&v)
}

/// Hash three bytes with a hasher.
pub fn hash3(seed: &[u8], a: &[u8], b: &[u8]) -> H256 {
	let mut v = seed.to_vec();
	let mut a = a.to_vec();
	let mut b = b.to_vec();
	v.append(&mut a);
	v.append(&mut b);
	Hasher::hash(&v)
}

pub const fn slot_to_epoch(slot: u64) -> u64 {
	slot / crate::consts::SLOTS_PER_EPOCH
}

pub fn to_bytes(v: u64) -> H256 {
	H256::from_low_u64_le(v)
}
