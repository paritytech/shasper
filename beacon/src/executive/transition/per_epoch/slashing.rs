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

use crate::{Config, BeaconExecutive};
use core::cmp::min;

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Process slashings
	pub fn process_slashings(&mut self) {
		let current_epoch = self.current_epoch();
		let total_balance = self.total_active_balance();

		for index in 0..(self.validators.len() as u64) {
			let penalty = {
				let validator = &self.validators[index as usize];
				if validator.slashed &&
					current_epoch + C::epochs_per_slashings_vector() / 2 ==
					validator.withdrawable_epoch
				{
					let increment = C::effective_balance_increment();
					let penalty_numerator = validator.effective_balance / increment *
						min(self.slashings.iter().fold(0, |acc, x| acc + *x) * 3, total_balance);
					let penalty = penalty_numerator / total_balance * increment;

					Some(penalty)
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
