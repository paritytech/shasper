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

use crate::types::*;
use crate::{Config, BeaconState, Error};
use bm_le::{MaxVec, Compact, tree_root};
use core::cmp::min;

impl<C: Config> BeaconState<C> {
	/// Process final updates
	pub fn process_final_updates(&mut self) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		let next_epoch = current_epoch + 1;

		// Reset eth1 data votes
		if (self.slot + 1) % C::slots_per_eth1_voting_period() == 0 {
			self.eth1_data_votes = Default::default();
		}

		// Update effective balances with hysteresis
		for index in 0..(self.validators.len() as u64) {
			let validator = &mut self.validators[index as usize];
			let balance = self.balances[index as usize];
			let half_increment = C::effective_balance_increment() / 2;
			if balance < validator.effective_balance ||
				validator.effective_balance + 3 * half_increment < balance
			{
				validator.effective_balance = min(
					balance - balance % C::effective_balance_increment(),
					C::max_effective_balance()
				);
			}
		}

		// Set total slashed balances
		self.slashings[
			(next_epoch % C::epochs_per_slashings_vector()) as usize
		] = 0;

		// Set randao mix
		self.randao_mixes[
			(next_epoch % C::epochs_per_historical_vector()) as usize
		] = self.randao_mix(current_epoch);

		// Set historical root accumulator
		if next_epoch %
			(C::slots_per_historical_root() / C::slots_per_epoch())
			== 0
		{
			self.historical_roots.push(tree_root::<C::Digest, _>(&HistoricalBatch::<C> {
				block_roots: self.block_roots.clone(),
				state_roots: self.state_roots.clone(),
			}));
		}

		// Rotate current/previous epoch attestations
		self.previous_epoch_attestations =
			self.current_epoch_attestations.clone();
		self.current_epoch_attestations = Default::default();

		Ok(())
	}
}
