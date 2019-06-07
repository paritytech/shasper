mod epoch_processing;
mod operations;

pub use epoch_processing::{CrosslinksTest, RegistryUpdatesTest};
pub use operations::{DepositTest};

use serde_derive::{Serialize, Deserialize};
use beacon::types::BeaconState;
use beacon::{Executive, Config, Error};

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

pub trait Test {
	fn run<C: Config>(&self, config: &C);
}

pub fn run_test_with<C: Config, F: FnOnce(&mut Executive<C>) -> Result<(), Error>>(
	description: &str, pre: &BeaconState, post: Option<&BeaconState>, config: &C, f: F
) {
	print!("Running test: {} ...", description);

	let mut state = pre.clone();
	let mut executive = Executive {
		state: &mut state,
		config,
	};

	match f(&mut executive) {
		Ok(()) => {
			print!(" accepted");

			let post = post.unwrap().clone();
			assert_eq!(state, post);
			print!(" passed");
		}
		Err(e) => {
			print!(" rejected({:?})", e);

			assert!(post.is_none());
			print!(" passed");
		}
	}

	println!("");
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
