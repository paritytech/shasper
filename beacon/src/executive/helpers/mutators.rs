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

//! Routines for updating validator status.

use core::cmp::max;
use crate::primitives::{ValidatorIndex, Gwei};
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub(crate) fn increase_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
		self.state.balances[index as usize] += delta;
	}

	pub(crate) fn decrease_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
		self.state.balances[index as usize] =
			self.state.balances[index as usize].saturating_sub(delta);
	}

	pub(crate) fn initiate_validator_exit(&mut self, index: ValidatorIndex) {
		if self.state.validator_registry[index as usize].exit_epoch !=
			self.config.far_future_epoch()
		{
			return
		}

		let exit_epochs = self.state.validator_registry.iter()
			.map(|v| v.exit_epoch)
			.filter(|epoch| *epoch != self.config.far_future_epoch())
			.collect::<Vec<_>>();
		let mut exit_queue_epoch = max(
			exit_epochs.iter().fold(0, |a, b| max(a, *b)),
			self.config.delayed_activation_exit_epoch(self.current_epoch())
		);
		let exit_queue_churn = self.state.validator_registry.iter()
			.filter(|v| v.exit_epoch == exit_queue_epoch)
			.count() as u64;

		if exit_queue_churn >= self.churn_limit() {
			exit_queue_epoch += 1;
		}

		let validator = &mut self.state.validator_registry[index as usize];
		validator.exit_epoch = exit_queue_epoch;
		validator.withdrawable_epoch = validator.exit_epoch +
			self.config.min_validator_withdrawability_delay();
	}

	pub(crate) fn slash_validator(
		&mut self,
		slashed_index: ValidatorIndex,
		whistleblower_index: Option<ValidatorIndex>
	) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		self.initiate_validator_exit(slashed_index);

		self.state.validator_registry[slashed_index as usize].slashed = true;
		self.state.validator_registry[slashed_index as usize].withdrawable_epoch =
			current_epoch + self.config.latest_slashed_exit_length();
		let slashed_balance =
			self.state.validator_registry[slashed_index as usize].effective_balance;
		self.state.latest_slashed_balances[
			(current_epoch % self.config.latest_slashed_exit_length()) as usize
		] += slashed_balance;

		let proposer_index = self.beacon_proposer_index()?;
		let whistleblower_index = whistleblower_index.unwrap_or(proposer_index);
		let whistleblowing_reward =
			slashed_balance / self.config.whistleblowing_reward_quotient();
		let proposer_reward =
			whistleblowing_reward / self.config.proposer_reward_quotient();

		self.decrease_balance(slashed_index, whistleblowing_reward);
		self.increase_balance(proposer_index, proposer_reward);
		self.increase_balance(
			whistleblower_index, whistleblowing_reward - proposer_reward
		);

		Ok(())
	}
}
