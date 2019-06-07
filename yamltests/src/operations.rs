use serde_derive::{Serialize, Deserialize};
use beacon::types::{BeaconState, Deposit};
use beacon::{Config, ParameteredConfig, FromConfig};
use crypto::bls;
use crate::{Test, run_test_with};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DepositTest {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState,
	pub deposit: Deposit,
	pub post: Option<BeaconState>,
}

impl Test for DepositTest {
	fn run<C: Config>(&self, config: &C) {
		let bls_setting = self.bls_setting.unwrap_or(0);
		match bls_setting {
			0 | 2 => {
				run_test_with(&self.description, &self.pre, self.post.as_ref(), config, |executive| {
					executive.process_deposit(self.deposit.clone())
				});
			},
			1 => {
				let config = ParameteredConfig::<bls::Verification>::from_config(config);
				run_test_with(&self.description, &self.pre, self.post.as_ref(), &config, |executive| {
					executive.process_deposit(self.deposit.clone())
				});
			},
			_ => panic!("Invalid test format"),
		}
	}
}
