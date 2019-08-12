// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.










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
