use primitives::H256;
use blake2::blake2s::blake2s;

use super::Address;

#[derive(Clone)]
pub struct ValidatorRecord {
	pub pubkey: H256,
	pub withdrawal_shard: u16,
	pub withdrawal_address: Address,
	pub randao_commitment: H256,
	pub balance: u128,
	pub start_dynasty: u64,
	pub end_dynasty: u64,
}

#[derive(Clone)]
pub struct Validators(Vec<ValidatorRecord>);

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
				let m = u32::from_be(unsafe { ::std::mem::transmute([0u8, source[pos], source[pos+1], source[pos+2]]) });
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
}
