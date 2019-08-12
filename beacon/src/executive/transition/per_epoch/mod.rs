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










mod helpers;
mod justification;
mod crosslink;
mod reward;
mod registry;
mod slashing;
mod finalize;

use crate::{Config, BeaconState, Error};

impl<C: Config> BeaconState<C> {
	/// Process an epoch.
	pub fn process_epoch(&mut self) -> Result<(), Error> {
		self.process_justification_and_finalization()?;
		self.process_crosslinks()?;
		self.process_rewards_and_penalties()?;
		self.process_registry_updates()?;
		self.process_slashings();
		self.process_final_updates()?;

		Ok(())
	}
}
