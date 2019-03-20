pub type Hasher = keccak_hasher::KeccakHasher;

use crate::{Slot, ValidatorIndex, Epoch};
use crate::state::Fork;
use hash_db::Hasher as _;
use primitives::{ValidatorId, H256, Signature, crypto::bls};

pub fn bls_verify(pubkey: &ValidatorId, message: &H256, signature: &Signature, domain: u64) -> bool {
	pubkey.into_public()
		.map(|public| {
			signature.into_signature().map(|signature| {
				signature.verify(&message[..], domain, &public)
			}).unwrap_or(false)
		})
		.unwrap_or(false)
}

pub fn bls_aggregate_pubkeys(pubkeys: &[ValidatorId]) -> Option<ValidatorId> {
	let mut aggregated_pubkey = bls::AggregatePublic::new();
	for pubkey in pubkeys {
		let blskey = pubkey.into_public()?;
		aggregated_pubkey.add(&blskey);
	}
	Some((bls::Public {
		point: aggregated_pubkey.point
	}).into())
}

pub fn bls_verify_multiple(pubkeys: &[ValidatorId], messages: &[H256], signature: &Signature, domain: u64) -> bool {
	let mut aggregated_pubkeys = Vec::new();
	for key in pubkeys {
		let blskey = match key.into_public() {
			Some(k) => k,
			None => return false,
		};
		aggregated_pubkeys.push(bls::AggregatePublic {
			point: blskey.point
		});
	}

	let mut aggregated_message = Vec::new();
	for message in messages {
		aggregated_message.append(&mut (&message[..]).to_vec());
	}

	let blssig = match signature.into_signature() {
		Some(s) => s,
		None => return false,
	};
	let aggregated_signature = bls::AggregateSignature {
		point: blssig.point,
	};

	aggregated_signature.verify_multiple(
		&aggregated_message,
		domain,
		&aggregated_pubkeys[..].iter().collect::<Vec<_>>()
	)
}

pub fn bls_domain(fork: &Fork, epoch: u64, typ: u64) -> u64 {
	let version = if epoch < fork.epoch {
		&fork.previous_version
	} else {
		&fork.current_version
	};

	let mut bytes = [0u8; 8];
	(&mut bytes[0..4]).copy_from_slice(version);
	(&mut bytes[4..8]).copy_from_slice(&typ.to_le_bytes()[0..4]);

	u64::from_le_bytes(bytes)
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

pub fn compare_hash(a: &H256, b: &H256) -> core::cmp::Ordering {
	for i in 0..32 {
		if a[i] > b[i] {
			return core::cmp::Ordering::Greater
		} else if a[i] < b[i] {
			return core::cmp::Ordering::Less
		}
	}
	core::cmp::Ordering::Equal
}

pub fn integer_squareroot(n: u64) -> u64 {
	let mut x = n;
	let mut y = (x + 1) / 2;
	while y < x {
		x = y;
		y = (x + n / x) / 2
	}
	x
}
