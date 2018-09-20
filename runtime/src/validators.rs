use primitives::H256;
use blake2::blake2s::blake2s;
use rstd::prelude::*;

use super::Address;
use consts::{CYCLE_LENGTH, MIN_COMMITTEE_SIZE, SHARD_COUNT};

#[derive(Clone, PartialEq, Eq, Encode, Decode)]
pub struct ValidatorRecord {
	pub pubkey: H256,
	pub withdrawal_shard: u16,
	pub withdrawal_address: Address,
	pub randao_commitment: H256,
	pub balance: u128,
	pub start_dynasty: u64,
	pub end_dynasty: u64,
}

#[derive(Encode, Decode)]
pub struct ShardAndCommittee {
	pub shard_id: u16,
	pub committee: Vec<u32>,
}

#[derive(Clone, Encode, Decode)]
pub struct Validators(pub Vec<ValidatorRecord>);

impl Validators {
	pub fn active(&self, dynasty: u64) -> Vec<ValidatorRecord> {
		self.0.iter()
			.filter(|v| v.start_dynasty <= dynasty && v.end_dynasty > dynasty)
			.cloned()
			.collect()
	}

	pub fn shuffle(&self, seed: H256) -> Vec<ValidatorRecord> {
		let mut ret = self.0.clone();
		assert!(ret.len() <= 16777216);
		let mut source = seed;
		let mut i = 0;

		while i < ret.len() {
			source = H256::from(blake2s(32, &[], &source).as_bytes());
			for j in 0..10 {
				let pos = j * 3;
				let m = u32::from_be(unsafe { ::rstd::mem::transmute([0u8, source[pos], source[pos+1], source[pos+2]]) });
				let remaining = ret.len() - i;
				if remaining == 0 {
					break;
				}
				let rand_max = 16777216 - 16777216 % remaining as u32;
				if m < rand_max {
					let replacement_pos = (m as usize % remaining) + i;
					ret.swap(i, replacement_pos);
					i += 1;
				}
			}
		}

		ret
	}

	pub fn split(&self, n: usize) -> Vec<Vec<ValidatorRecord>> {
		let m = self.0.len() / n;
		let mut ret = Vec::new();

		for (i, value) in self.0.clone().into_iter().enumerate() {
			if i % m == 0 {
				ret.push(Vec::new());
			}

			ret.last_mut().expect("When i is 0, one vector is always pushed; it cannot be empty; qed")
				.push(value);
		}

		ret
	}

	pub fn new_shuffling(&self, seed: H256, dynasty: u64, crosslinking_start_shard: u16) -> Vec<ShardAndCommittee> {
		let active = self.active(dynasty);
		let (committees_per_slot, slots_per_committee) = if active.len() >= CYCLE_LENGTH * MIN_COMMITTEE_SIZE {
			(active.len() / CYCLE_LENGTH / (MIN_COMMITTEE_SIZE * 2) + 1, 1)
		} else {
			let mut slots_per_committee = 1;
			while active.len() * slots_per_committee < CYCLE_LENGTH * MIN_COMMITTEE_SIZE && slots_per_committee < CYCLE_LENGTH {
				slots_per_committee *= 2;
			}
			(1, slots_per_committee)
		};

		let mut ret = Vec::new();
		for (i, slot_indices) in Validators(Validators(active).shuffle(seed)).split(CYCLE_LENGTH).into_iter().enumerate() {
			let shard_indices = Validators(slot_indices).split(committees_per_slot);
			let shard_id_start = crosslinking_start_shard + (i * committees_per_slot / slots_per_committee) as u16;
			for (j, indices) in shard_indices.into_iter().enumerate() {
				ret.push(ShardAndCommittee {
					shard_id: (shard_id_start + j as u16) % SHARD_COUNT,
					committee: indices.iter().map(|k| self.0.iter().position(|v| v == k).expect("Indices come from the original array; it always exists; qed") as u32).collect(), // TODO: get rid of this inefficient impl.
				})
			}
		}

		ret
	}
}
