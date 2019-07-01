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

use core::cmp::min;
use ssz::Digestible;
use crate::primitives::H256;
use crate::types::HistoricalBatch;
use crate::{Config, Executive};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Process final updates
	pub fn process_final_updates(&mut self) {
		let current_epoch = self.current_epoch();
		let next_epoch = current_epoch + 1;

		// Reset eth1 data votes
		if (self.state.slot + 1) % self.config.slots_per_eth1_voting_period() == 0 {
			self.state.eth1_data_votes = Vec::new();
		}

		// Update effective balances with hysteresis
		for index in 0..(self.state.validator_registry.len() as u64) {
			let validator = &mut self.state.validator_registry[index as usize];
			let balance = self.state.balances[index as usize];
			let half_increment = self.config.effective_balance_increment() / 2;
			if balance < validator.effective_balance ||
				validator.effective_balance + 3 * half_increment < balance
			{
				validator.effective_balance = min(
					balance - balance % self.config.effective_balance_increment(),
					self.config.max_effective_balance()
				);
			}
		}

		// Update start shard
		self.state.latest_start_shard =
			(self.state.latest_start_shard + self.shard_delta(current_epoch)) %
			self.config.shard_count();

		// Set active index root
		let index_root_position = (next_epoch + self.config.activation_exit_delay()) %
			self.config.latest_active_index_roots_length();
		self.state.latest_active_index_roots[index_root_position as usize] =
			H256::from_slice(
				Digestible::<C::Digest>::hash(
					&self.active_validator_indices(
						next_epoch + self.config.activation_exit_delay()
					)
				).as_slice()
			);

		// Set total slashed balances
		self.state.latest_slashed_balances[
			(next_epoch % self.config.latest_slashed_exit_length()) as usize
		] = self.state.latest_slashed_balances[
			(current_epoch % self.config.latest_slashed_exit_length()) as usize
		];

		// Set randao mix
		self.state.latest_randao_mixes[
			(next_epoch % self.config.latest_randao_mixes_length()) as usize
		] = self.randao_mix(current_epoch);

		// Set historical root accumulator
		if next_epoch %
			(self.config.slots_per_historical_root() / self.config.slots_per_epoch())
			== 0
		{
			self.state.historical_roots.push(H256::from_slice(
				Digestible::<C::Digest>::hash(&HistoricalBatch {
					block_roots: self.state.block_roots.clone(),
					state_roots: self.state.state_roots.clone(),
				}).as_slice()
			));
		}

		// Rotate current/previous epoch attestations
		self.state.previous_epoch_attestations =
			self.state.current_epoch_attestations.clone();
		self.state.current_epoch_attestations = Vec::new();
	}
}
