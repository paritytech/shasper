use serde_derive::{Serialize, Deserialize};
use beacon::{BeaconState, Config};
use crate::{Test, run_test_with};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct JustificationAndFinalizationTest<C: Config> {
	pub description: String,
	pub pre: BeaconState<C>,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> Test for JustificationAndFinalizationTest<C> {
	fn run(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_justification_and_finalization()
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CrosslinksTest<C: Config> {
	pub description: String,
	pub pre: BeaconState<C>,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> Test for CrosslinksTest<C> {
	fn run(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_crosslinks()
		});
	}
}

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(deny_unknown_fields)]
// pub struct RegistryUpdatesTest {
// 	pub description: String,
// 	pub pre: BeaconState,
// 	pub post: Option<BeaconState>,
// }

// impl Test for RegistryUpdatesTest {
// 	fn run<C: Config>(&self, config: &C) {
// 		run_test_with(&self.description, &self.pre, self.post.as_ref(), config, |executive| {
// 			executive.process_registry_updates()
// 		});
// 	}
// }
