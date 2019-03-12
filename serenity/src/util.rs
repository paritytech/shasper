pub type Hasher = keccak_hasher::KeccakHasher;

use crate::{Slot, ValidatorIndex, Epoch};
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

pub fn bls_aggregate_pubkeys(_pubkeys: &[ValidatorId]) -> ValidatorId {
	ValidatorId::default()
}

pub fn bls_verify_multiple(_pubkey: &[ValidatorId], _message: &[H256], _signature: &Signature, _domain: u64) -> bool {
	true
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

pub const fn slot_to_epoch(slot: Slot) -> Epoch {
	slot / crate::consts::SLOTS_PER_EPOCH
}

pub fn to_bytes(v: u64) -> H256 {
	H256::from_low_u64_le(v)
}

pub fn to_usize(v: &[u8]) -> usize {
	let mut ret = 0usize.to_le_bytes();
	(&mut ret[..]).copy_from_slice(&v[..v.len()]);
	usize::from_le_bytes(ret)
}

pub fn epoch_start_slot(epoch: Epoch) -> Slot {
	epoch * crate::consts::SLOTS_PER_EPOCH
}

pub fn permuted_index(mut index: usize, seed: &H256, len: usize, round: usize) -> usize {
	if index >= len {
		index = index % len;
	}

	let usize_len = 0usize.to_le_bytes().len();

	for round in 0..round {
		let pivot = to_usize(
			&hash2(&seed[..], &round.to_le_bytes()[..1]).as_ref()[..usize_len]
		) % len;
		let flip = if pivot > index { pivot - index } else { index - pivot } % len;
		let position = core::cmp::max(index, flip);
		let source = hash3(
			&seed[..],
			&round.to_le_bytes()[..1],
			&(position / 256).to_le_bytes()[..4]
		);
		let byte = source.as_ref()[(position % 256) / 8];
		let bit = (byte >> (position % 8 )) % 2;
		index = if bit == 1 { flip } else { index }
	}

	index
}

pub fn split<T>(mut values: Vec<T>, split_count: usize) -> Vec<Vec<T>> {
	let len = values.len();
	values.reverse();

	let mut ret = Vec::new();
	for i in 0..split_count {
		let mut current = Vec::new();
		let v = ((len * (i + 1)) / split_count) - ((len * i) / split_count);

		for _ in 0..v {
			if let Some(value) = values.pop() {
				current.push(value);
			}
		}
		ret.push(current);
	}
	ret
}

pub fn epoch_committee_count(active_validator_count: usize) -> usize {
	use crate::consts::{SHARD_COUNT, SLOTS_PER_EPOCH, TARGET_COMMITTEE_SIZE};

	core::cmp::max(
		1,
		core::cmp::min(
			SHARD_COUNT / SLOTS_PER_EPOCH as usize,
			active_validator_count / SLOTS_PER_EPOCH as usize / TARGET_COMMITTEE_SIZE,
		)
	) * SLOTS_PER_EPOCH as usize
}

pub fn shuffling(seed: &H256, active_validators: Vec<ValidatorIndex>) -> Vec<Vec<ValidatorIndex>> {
	let mut shuffled_indices = Vec::new();
	let len = active_validators.len();

	for i in 0..len {
		shuffled_indices.push(active_validators[permuted_index(i, seed, len, crate::consts::SHUFFLE_ROUND_COUNT)]);
	}

	split(shuffled_indices, epoch_committee_count(len))
}

pub fn is_power_of_two(value: u64) -> bool {
	return (value > 0) && (value & (value - 1) == 0)
}
