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

use crate::{Config, ExecutiveMut, Error};
use core::cmp::min;

impl<'state, 'config, C: Config> ExecutiveMut<'state, 'config, C> {
	/// Process registry updates
	pub fn process_registry_updates(&mut self) -> Result<(), Error> {
		for index in 0..self.state.validator_registry.len() {
			if self.state.validator_registry[index].activation_eligibility_epoch ==
				self.config.far_future_epoch() &&
				self.state.validator_registry[index].effective_balance >=
				self.config.max_effective_balance()
			{
				self.state.validator_registry[index].activation_eligibility_epoch =
					self.current_epoch();
			}

			if self.state.validator_registry[index].is_active(self.current_epoch()) &&
				self.state.validator_registry[index].effective_balance <=
				self.config.ejection_balance()
			{
				self.initiate_validator_exit(index as u64);
			}
		}

		let mut activation_queue = self.state.validator_registry.iter()
			.enumerate()
			.filter(|(_, v)| {
				v.activation_eligibility_epoch != self.config.far_future_epoch() &&
					v.activation_epoch >=
					self.config.delayed_activation_exit_epoch(self.state.finalized_epoch)
			})
			.map(|(i, _)| i as u64)
			.collect::<Vec<_>>();
		activation_queue.sort_by_key(|index| {
			self.state.validator_registry[*index as usize].activation_eligibility_epoch
		});

		for index in &activation_queue[..min(activation_queue.len(), self.churn_limit() as usize)] {
			let current_epoch = self.current_epoch();
			let validator = &mut self.state.validator_registry[*index as usize];
			if validator.activation_epoch == self.config.far_future_epoch() {
				validator.activation_epoch =
					self.config.delayed_activation_exit_epoch(current_epoch);
			}
		}

		Ok(())
	}
}
