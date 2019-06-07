use serde_derive::{Serialize, Deserialize};
use beacon::types::BeaconState;
use beacon::Config;
use crate::{Test, run_test_with};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CrosslinksTest {
	pub description: String,
	pub pre: BeaconState,
	pub post: Option<BeaconState>,
}

impl Test for CrosslinksTest {
	fn run<C: Config>(&self, config: &C) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), config, |executive| {
			executive.process_crosslinks()
		});
	}
}
