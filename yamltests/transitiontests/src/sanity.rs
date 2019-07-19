use serde::{Serialize, Deserialize};
use beacon::types::*;
use beacon::{BeaconState, Config, BLSConfig};
use crate::{TestWithBLS, run_test_with};

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(deny_unknown_fields)]
// pub struct BlocksTest {
// 	pub bls_setting: Option<usize>,
// 	pub description: String,
// 	pub pre: BeaconState,
// 	pub blocks: Vec<BeaconBlock>,
// 	pub post: Option<BeaconState>,
// }

// impl TestWithBLS for BlocksTest {
// 	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

// 	fn run<C: Config>(&self, config: &C) {
// 		run_state_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
// 			for block in self.blocks.clone() {
// 				let mut executive = Executive { state, config };

// 				executive.state_transition(&block, Strategy::Full)?
// 			}

// 			Ok(())
// 		});
// 	}
// }

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SlotsTest<C: Config> {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState<C>,
	pub slots: u64,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> TestWithBLS for SlotsTest<C> {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<BLS: BLSConfig>(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			let target_slot = state.slot + self.slots;

			state.process_slots(target_slot)
		});
	}
}
