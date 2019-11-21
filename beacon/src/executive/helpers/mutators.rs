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

use crate::primitives::{ValidatorIndex, Gwei};
use crate::{BeaconExecutive, Config, Error, utils, consts};
use core::cmp::max;

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Increase validator balance.
	pub fn increase_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
		self.state.balances[index as usize] += delta;
	}

	/// Decrease validator balance.
	pub fn decrease_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
		self.state.balances[index as usize] =
			self.balances[index as usize].saturating_sub(delta);
	}

	/// Initiate validator exit.
	pub fn initiate_validator_exit(&mut self, index: ValidatorIndex) {
		if self.validators[index as usize].exit_epoch !=
			consts::FAR_FUTURE_EPOCH
		{
			return
		}

		let exit_epochs = self.validators.iter()
			.map(|v| v.exit_epoch)
			.filter(|epoch| *epoch != consts::FAR_FUTURE_EPOCH)
			.collect::<Vec<_>>();
		let mut exit_queue_epoch = max(
			exit_epochs.iter().fold(0, |a, b| max(a, *b)),
			utils::activation_exit_epoch::<C>(self.current_epoch())
		);
		let exit_queue_churn = self.validators.iter()
			.filter(|v| v.exit_epoch == exit_queue_epoch)
			.count() as u64;

		if exit_queue_churn >= self.validator_churn_limit() {
			exit_queue_epoch += 1;
		}

		let validator = &mut self.state.validators[index as usize];
		validator.exit_epoch = exit_queue_epoch;
		validator.withdrawable_epoch = validator.exit_epoch +
			C::min_validator_withdrawability_delay();
	}

	/// Slash validator.
	pub fn slash_validator(
		&mut self,
		slashed_index: ValidatorIndex,
		whistleblower_index: Option<ValidatorIndex>
	) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		self.initiate_validator_exit(slashed_index);

		self.state.validators[slashed_index as usize].slashed = true;
		self.state.validators[slashed_index as usize].withdrawable_epoch = max(
			self.validators[slashed_index as usize].withdrawable_epoch,
			current_epoch + C::epochs_per_slashings_vector()
		);
		let slashed_balance =
			self.validators[slashed_index as usize].effective_balance;
		self.state.slashings[
			(current_epoch % C::epochs_per_slashings_vector()) as usize
		] += slashed_balance;
		self.decrease_balance(slashed_index, slashed_balance / C::min_slashing_penalty_quotient());

		let proposer_index = self.beacon_proposer_index()?;
		let whistleblower_index = whistleblower_index.unwrap_or(proposer_index);
		let whistleblowing_reward =
			slashed_balance / C::whistleblower_reward_quotient();
		let proposer_reward =
			whistleblowing_reward / C::proposer_reward_quotient();

		self.increase_balance(proposer_index, proposer_reward);
		self.increase_balance(
			whistleblower_index, whistleblowing_reward - proposer_reward
		);

		Ok(())
	}
}
