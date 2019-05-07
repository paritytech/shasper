use serde_derive::{Serialize, Deserialize};
use beacon::types::{BeaconState, Deposit};
use beacon::{Executive, Config};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Collection<T> {
	pub title: String,
	pub summary: String,
	pub forks_timeline: String,
	pub forks: Vec<String>,
	pub config: String,
	pub runner: String,
	pub handler: String,
	pub test_cases: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DepositTest {
	pub description: String,
	pub pre: BeaconState,
	pub deposit: Deposit,
	pub post: Option<BeaconState>,
}

pub trait Test {
	fn run<C: Config>(&self, config: &C);
}

impl Test for DepositTest {
	fn run<C: Config>(&self, config: &C) {
		print!("Running test: {} ...", self.description);

		let mut state = self.pre.clone();
		let mut executive = Executive {
			state: &mut state,
			config,
		};

		match executive.process_deposit(self.deposit.clone()) {
			Ok(()) => {
				print!(" accepted");

				let post = self.post.clone().unwrap();
				assert_eq!(state, post);
				print!(" passed");
			}
			Err(e) => {
				print!(" rejected({:?})", e);

				assert!(self.post.is_none());
				print!(" passed");
			}
		}

		println!("");
	}
}

pub fn run_collection<T: Test, C: Config>(coll: Collection<T>, config: &C) {
	for test in coll.test_cases {
		test.run(config);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use beacon::NoVerificationConfig;

	#[test]
	fn deposit_small() {
		let config = NoVerificationConfig::small();
		let coll = serde_yaml::from_str(&include_str!("../res/spectests/tests/operations/deposits/deposit_minimal.yaml")).unwrap();
		run_collection::<DepositTest, _>(coll, &config);
	}
}
