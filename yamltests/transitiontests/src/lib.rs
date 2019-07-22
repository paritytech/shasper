mod epoch_processing;
mod operations;
mod sanity;

pub use epoch_processing::*;
pub use operations::*;
pub use sanity::*;

use serde_derive::{Serialize, Deserialize};
use beacon::{BeaconState, Config, BLSConfig, BLSNoVerification, Error};
use crypto::bls::BLSVerification;

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

pub trait TestWithBLS {
	fn bls_setting(&self) -> Option<usize>;
	fn run<BLS: BLSConfig>(&self);
}

impl<T: TestWithBLS> Test for T {
	fn run(&self) {
		match self.bls_setting() {
			None | Some(0) | Some(2) => {
				TestWithBLS::run::<BLSNoVerification>(self);
			},
			Some(1) => {
				TestWithBLS::run::<BLSVerification>(self);
			},
			_ => panic!("Invalid test format"),
		}
	}
}

pub trait Test {
	fn run(&self);
}

pub fn run_test_with<C: Config, F: FnOnce(&mut BeaconState<C>) -> Result<(), Error>>(
	description: &str, pre: &BeaconState<C>, post: Option<&BeaconState<C>>, f: F
) {
	print!("Running test: {} ...", description);

	let mut state = pre.clone();

	match f(&mut state) {
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

pub fn run_collection<T: Test>(coll: Collection<T>) {
	for test in coll.test_cases {
		test.run();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use beacon::MinimalConfig;

	#[test]
	fn deposit_small() {
		let coll = serde_yaml::from_str(&include_str!("../../spectests/tests/operations/deposit/deposit_minimal.yaml")).unwrap();
		run_collection::<DepositTest<MinimalConfig>>(coll);
	}
}
