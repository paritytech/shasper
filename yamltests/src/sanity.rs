use serde_derive::{Serialize, Deserialize};
use beacon::types::{BeaconState, BeaconBlock};
use beacon::{self, Config, Executive, Strategy};
use crate::{TestWithBLS, run_state_test_with};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct BlocksTest {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState,
	pub blocks: Vec<BeaconBlock>,
	pub post: Option<BeaconState>,
}

impl TestWithBLS for BlocksTest {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<C: Config>(&self, config: &C) {
		run_state_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			for block in self.blocks.clone() {
				let mut executive = Executive { state, config };

				executive.state_transition(&block, Strategy::IgnoreRandaoAndStateRoot)?
			}

			Ok(())
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SlotsTest {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState,
	pub slots: u64,
	pub post: Option<BeaconState>,
}

impl TestWithBLS for SlotsTest {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<C: Config>(&self, config: &C) {
		run_state_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			let target_slot = state.slot + self.slots;

			beacon::initialize_block(state, target_slot, config)
		});
	}
}
