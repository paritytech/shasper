pub type Hasher = keccak_hasher::KeccakHasher;

use crate::{consts, Slot, ValidatorIndex, Epoch};
use crate::state::Fork;
use hash_db::Hasher as _;
use primitives::{ValidatorId, H256, Signature, crypto::bls};

pub fn bls_verify(pubkey: &ValidatorId, message: &H256, signature: &Signature, domain: u64) -> bool {
	if !consts::VERIFY_SIGNATURE {
		return true
	}

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
	if !consts::VERIFY_SIGNATURE {
		return true
	}

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
	(&mut bytes[0..4]).copy_from_slice(version.as_ref());
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
	let bytes = v.to_le_bytes();
	let mut ret = H256::default();
	(&mut ret[0..bytes.len()]).copy_from_slice(&bytes);
	ret
}

pub fn to_usize(v: &[u8]) -> usize {
	let mut ret = 0usize.to_le_bytes();
	(&mut ret[..]).copy_from_slice(&v[..v.len()]);
	usize::from_le_bytes(ret)
}

pub fn epoch_start_slot(epoch: Epoch) -> Slot {
	epoch.wrapping_mul(crate::consts::SLOTS_PER_EPOCH)
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
		let flip = if pivot >= index { (pivot - index) % len } else { len - (index - pivot) % len };
		let position = core::cmp::max(index, flip);
		let source = hash3(
			&seed[..],
			&round.to_le_bytes()[..1],
			&(position / 256).to_le_bytes()[..4]
		);
		let byte = source.as_ref()[(position % 256) / 8];
		let bit = (byte >> (position % 8 )) % 2;
		index = if bit == 1 { flip } else { index };
	}

	index
}

pub fn split_offset(len: usize, chunks: usize, index: usize) -> usize {
	(len * index) / chunks
}

pub fn compute_committee(validators: &[ValidatorIndex], seed: &H256, index: usize, total_committees: usize) -> Vec<ValidatorIndex> {
	let start_offset = split_offset(validators.len(), total_committees, index);
	let end_offset = split_offset(validators.len(), total_committees, index + 1);

	let mut ret = Vec::new();
	for i in start_offset..end_offset {
		ret.push(permuted_index(i, seed, validators.len(), consts::SHUFFLE_ROUND_COUNT) as ValidatorIndex);
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

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;

	#[test]
	fn test_permuted_index() {
		let seed = H256::from_str("c0c7f226fbd574a8c63dc26864c27833ea931e7c70b34409ba765f3d2031633d").unwrap();

		assert_eq!(permuted_index(0, &seed, 128, 90), 25);
		assert_eq!(permuted_index(1, &seed, 128, 90), 62);
		assert_eq!(permuted_index(2, &seed, 128, 90), 82);
		assert_eq!(permuted_index(3, &seed, 128, 90), 1);
		assert_eq!(permuted_index(4, &seed, 128, 90), 60);
		assert_eq!(permuted_index(5, &seed, 128, 90), 81);
		assert_eq!(permuted_index(6, &seed, 128, 90), 61);
		assert_eq!(permuted_index(7, &seed, 128, 90), 123);
		assert_eq!(permuted_index(8, &seed, 128, 90), 73);
		assert_eq!(permuted_index(9, &seed, 128, 90), 103);
		assert_eq!(permuted_index(10, &seed, 128, 90), 49);
		assert_eq!(permuted_index(11, &seed, 128, 90), 23);
		assert_eq!(permuted_index(12, &seed, 128, 90), 64);
		assert_eq!(permuted_index(13, &seed, 128, 90), 0);
		assert_eq!(permuted_index(14, &seed, 128, 90), 65);
		assert_eq!(permuted_index(15, &seed, 128, 90), 21);
		assert_eq!(permuted_index(16, &seed, 128, 90), 74);
		assert_eq!(permuted_index(17, &seed, 128, 90), 8);
		assert_eq!(permuted_index(18, &seed, 128, 90), 100);
		assert_eq!(permuted_index(19, &seed, 128, 90), 119);
		assert_eq!(permuted_index(20, &seed, 128, 90), 34);
		assert_eq!(permuted_index(21, &seed, 128, 90), 101);
		assert_eq!(permuted_index(22, &seed, 128, 90), 86);
		assert_eq!(permuted_index(23, &seed, 128, 90), 110);
		assert_eq!(permuted_index(24, &seed, 128, 90), 50);
		assert_eq!(permuted_index(25, &seed, 128, 90), 71);
		assert_eq!(permuted_index(26, &seed, 128, 90), 85);
		assert_eq!(permuted_index(27, &seed, 128, 90), 51);
		assert_eq!(permuted_index(28, &seed, 128, 90), 22);
		assert_eq!(permuted_index(29, &seed, 128, 90), 80);
		assert_eq!(permuted_index(30, &seed, 128, 90), 112);
		assert_eq!(permuted_index(31, &seed, 128, 90), 17);
		assert_eq!(permuted_index(32, &seed, 128, 90), 52);
		assert_eq!(permuted_index(33, &seed, 128, 90), 92);
		assert_eq!(permuted_index(34, &seed, 128, 90), 105);
		assert_eq!(permuted_index(35, &seed, 128, 90), 99);
		assert_eq!(permuted_index(36, &seed, 128, 90), 38);
		assert_eq!(permuted_index(37, &seed, 128, 90), 16);
		assert_eq!(permuted_index(38, &seed, 128, 90), 6);
		assert_eq!(permuted_index(39, &seed, 128, 90), 127);
		assert_eq!(permuted_index(40, &seed, 128, 90), 69);
		assert_eq!(permuted_index(41, &seed, 128, 90), 67);
		assert_eq!(permuted_index(42, &seed, 128, 90), 2);
		assert_eq!(permuted_index(43, &seed, 128, 90), 118);
		assert_eq!(permuted_index(44, &seed, 128, 90), 30);
		assert_eq!(permuted_index(45, &seed, 128, 90), 37);
		assert_eq!(permuted_index(46, &seed, 128, 90), 108);
		assert_eq!(permuted_index(47, &seed, 128, 90), 15);
		assert_eq!(permuted_index(48, &seed, 128, 90), 57);
		assert_eq!(permuted_index(49, &seed, 128, 90), 75);
		assert_eq!(permuted_index(50, &seed, 128, 90), 3);
		assert_eq!(permuted_index(51, &seed, 128, 90), 121);
		assert_eq!(permuted_index(52, &seed, 128, 90), 12);
		assert_eq!(permuted_index(53, &seed, 128, 90), 42);
		assert_eq!(permuted_index(54, &seed, 128, 90), 111);
		assert_eq!(permuted_index(55, &seed, 128, 90), 47);
		assert_eq!(permuted_index(56, &seed, 128, 90), 78);
		assert_eq!(permuted_index(57, &seed, 128, 90), 45);
		assert_eq!(permuted_index(58, &seed, 128, 90), 59);
		assert_eq!(permuted_index(59, &seed, 128, 90), 56);
		assert_eq!(permuted_index(60, &seed, 128, 90), 19);
		assert_eq!(permuted_index(61, &seed, 128, 90), 89);
		assert_eq!(permuted_index(62, &seed, 128, 90), 18);
		assert_eq!(permuted_index(63, &seed, 128, 90), 36);
		assert_eq!(permuted_index(64, &seed, 128, 90), 104);
		assert_eq!(permuted_index(65, &seed, 128, 90), 102);
		assert_eq!(permuted_index(66, &seed, 128, 90), 87);
		assert_eq!(permuted_index(67, &seed, 128, 90), 97);
		assert_eq!(permuted_index(68, &seed, 128, 90), 31);
		assert_eq!(permuted_index(69, &seed, 128, 90), 66);
		assert_eq!(permuted_index(70, &seed, 128, 90), 95);
		assert_eq!(permuted_index(71, &seed, 128, 90), 120);
		assert_eq!(permuted_index(72, &seed, 128, 90), 5);
		assert_eq!(permuted_index(73, &seed, 128, 90), 54);
		assert_eq!(permuted_index(74, &seed, 128, 90), 76);
		assert_eq!(permuted_index(75, &seed, 128, 90), 27);
		assert_eq!(permuted_index(76, &seed, 128, 90), 48);
		assert_eq!(permuted_index(77, &seed, 128, 90), 126);
		assert_eq!(permuted_index(78, &seed, 128, 90), 26);
		assert_eq!(permuted_index(79, &seed, 128, 90), 58);
		assert_eq!(permuted_index(80, &seed, 128, 90), 44);
		assert_eq!(permuted_index(81, &seed, 128, 90), 32);
		assert_eq!(permuted_index(82, &seed, 128, 90), 40);
		assert_eq!(permuted_index(83, &seed, 128, 90), 90);
		assert_eq!(permuted_index(84, &seed, 128, 90), 20);
		assert_eq!(permuted_index(85, &seed, 128, 90), 10);
		assert_eq!(permuted_index(86, &seed, 128, 90), 79);
		assert_eq!(permuted_index(87, &seed, 128, 90), 11);
		assert_eq!(permuted_index(88, &seed, 128, 90), 24);
		assert_eq!(permuted_index(89, &seed, 128, 90), 114);
		assert_eq!(permuted_index(90, &seed, 128, 90), 106);
		assert_eq!(permuted_index(91, &seed, 128, 90), 77);
		assert_eq!(permuted_index(92, &seed, 128, 90), 98);
		assert_eq!(permuted_index(93, &seed, 128, 90), 117);
		assert_eq!(permuted_index(94, &seed, 128, 90), 55);
		assert_eq!(permuted_index(95, &seed, 128, 90), 35);
		assert_eq!(permuted_index(96, &seed, 128, 90), 14);
		assert_eq!(permuted_index(97, &seed, 128, 90), 13);
		assert_eq!(permuted_index(98, &seed, 128, 90), 70);
		assert_eq!(permuted_index(99, &seed, 128, 90), 94);
		assert_eq!(permuted_index(100, &seed, 128, 90), 46);
		assert_eq!(permuted_index(101, &seed, 128, 90), 29);
		assert_eq!(permuted_index(102, &seed, 128, 90), 84);
		assert_eq!(permuted_index(103, &seed, 128, 90), 96);
		assert_eq!(permuted_index(104, &seed, 128, 90), 53);
		assert_eq!(permuted_index(105, &seed, 128, 90), 33);
		assert_eq!(permuted_index(106, &seed, 128, 90), 113);
		assert_eq!(permuted_index(107, &seed, 128, 90), 68);
		assert_eq!(permuted_index(108, &seed, 128, 90), 88);
		assert_eq!(permuted_index(109, &seed, 128, 90), 41);
		assert_eq!(permuted_index(110, &seed, 128, 90), 109);
		assert_eq!(permuted_index(111, &seed, 128, 90), 7);
		assert_eq!(permuted_index(112, &seed, 128, 90), 63);
		assert_eq!(permuted_index(113, &seed, 128, 90), 9);
		assert_eq!(permuted_index(114, &seed, 128, 90), 115);
		assert_eq!(permuted_index(115, &seed, 128, 90), 124);
		assert_eq!(permuted_index(116, &seed, 128, 90), 43);
		assert_eq!(permuted_index(117, &seed, 128, 90), 28);
		assert_eq!(permuted_index(118, &seed, 128, 90), 91);
		assert_eq!(permuted_index(119, &seed, 128, 90), 125);
		assert_eq!(permuted_index(120, &seed, 128, 90), 107);
		assert_eq!(permuted_index(121, &seed, 128, 90), 83);
		assert_eq!(permuted_index(122, &seed, 128, 90), 39);
		assert_eq!(permuted_index(123, &seed, 128, 90), 122);
		assert_eq!(permuted_index(124, &seed, 128, 90), 116);
		assert_eq!(permuted_index(125, &seed, 128, 90), 72);
		assert_eq!(permuted_index(126, &seed, 128, 90), 93);
		assert_eq!(permuted_index(127, &seed, 128, 90), 4);
	}
}
