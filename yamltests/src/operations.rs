use serde_derive::{Serialize, Deserialize};
use beacon::types::{BeaconState, Deposit, Attestation, AttesterSlashing, BeaconBlock};
use beacon::Config;
use crate::{TestWithBLS, run_test_with};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AttestationTest {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState,
	pub attestation: Attestation,
	pub post: Option<BeaconState>,
}

impl TestWithBLS for AttestationTest {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<C: Config>(&self, config: &C) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), config, |executive| {
			executive.process_attestation(self.attestation.clone())
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AttesterSlashingTest {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState,
	pub attester_slashing: AttesterSlashing,
	pub post: Option<BeaconState>,
}

impl TestWithBLS for AttesterSlashingTest {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<C: Config>(&self, config: &C) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), config, |executive| {
			executive.process_attester_slashing(self.attester_slashing.clone())
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct BlockHeaderTest {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState,
	pub block: BeaconBlock,
	pub post: Option<BeaconState>,
}

impl TestWithBLS for BlockHeaderTest {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<C: Config>(&self, config: &C) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), config, |executive| {
			executive.process_block_header(&self.block)
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DepositTest {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState,
	pub deposit: Deposit,
	pub post: Option<BeaconState>,
}

impl TestWithBLS for DepositTest {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<C: Config>(&self, config: &C) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), config, |executive| {
			executive.process_deposit(self.deposit.clone())
		});
	}
}
