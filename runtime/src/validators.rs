use primitives::H256;

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

pub struct Validators(Vec<ValidatorRecord>);

impl Validators {
	pub fn active(&self, dynasty: u64) -> Vec<ValidatorRecord> {
		self.0.iter()
			.filter(|v| v.start_dynasty <= dynasty && v.end_dynasty > dynasty)
			.cloned()
			.collect()
	}
}
