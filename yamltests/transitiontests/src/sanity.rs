use serde::{Serialize, Deserialize};
use beacon::types::*;
use beacon::{BeaconState, Config, BLSConfig};
use crate::{TestWithBLS, run_test_with};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "C: Config + serde::Serialize + Clone + serde::de::DeserializeOwned + 'static")]
#[serde(deny_unknown_fields)]
pub struct BlocksTest<C: Config> {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState<C>,
	pub blocks: Vec<BeaconBlock<C>>,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> TestWithBLS for BlocksTest<C> where
	C: serde::Serialize + serde::de::DeserializeOwned
{
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<BLS: BLSConfig>(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			for block in self.blocks.clone() {
				state.state_transition::<_, BLS>(&block)?
			}

			Ok(())
		});
	}
}

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
