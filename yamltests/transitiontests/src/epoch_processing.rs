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

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RegistryUpdatesTest<C: Config> {
	pub description: String,
	pub pre: BeaconState<C>,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> Test for RegistryUpdatesTest<C> {
	fn run(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_registry_updates()
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SlashingsTest<C: Config> {
	pub description: String,
	pub pre: BeaconState<C>,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> Test for SlashingsTest<C> {
	fn run(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			Ok(state.process_slashings())
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FinalUpdatesTest<C: Config> {
	pub description: String,
	pub pre: BeaconState<C>,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> Test for FinalUpdatesTest<C> {
	fn run(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_final_updates()
		});
	}
}
