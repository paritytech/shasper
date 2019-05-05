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

use core::cmp::{min, max};
use crate::{Config, Executive};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Process slashings
	pub fn process_slashings(&mut self) {
		let current_epoch = self.current_epoch();
		let active_validator_indices = self.active_validator_indices(current_epoch);
		let total_balance = self.total_balance(&active_validator_indices);

		// Compute `total_penalties`
		let total_at_start = self.state.latest_slashed_balances[
			((current_epoch + 1) % self.config.latest_slashed_exit_length()) as usize
		];
		let total_at_end = self.state.latest_slashed_balances[
			(current_epoch % self.config.latest_slashed_exit_length()) as usize
		];
		let total_penalties = total_at_end - total_at_start;

		for index in 0..(self.state.validator_registry.len() as u64) {
			let penalty = {
				let validator = &self.state.validator_registry[index as usize];
				if validator.slashed &&
					current_epoch == validator.withdrawable_epoch -
					self.config.latest_slashed_exit_length() / 2
				{
					Some(max(
						validator.effective_balance * min(
							total_penalties * 3, total_balance
						) / total_balance,
						validator.effective_balance /
							self.config.min_slashing_penalty_quotient()
					))
				} else {
					None
				}
			};
			if let Some(penalty) = penalty {
				self.decrease_balance(index, penalty);
			}
		}
	}
}
