// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

mod helpers;
mod justification;
mod crosslink;
mod reward;
mod registry;
mod slashing;
mod finalize;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub fn process_epoch(&mut self) -> Result<()> {
		self.process_justification_and_finalization()?;
		self.process_crosslinks()?;
		self.process_rewards_and_penalties()?;
		self.process_registry_updates()?;
		self.process_slashings()?;
		self.process_final_updates()?;
	}
}
