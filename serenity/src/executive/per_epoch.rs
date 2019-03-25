use ssz::Hashable;

use super::Executive;
use crate::{
	Config, Error, HistoricalBatch, Crosslink,
};
use crate::util::is_power_of_two;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub fn update_justification_and_finalization(&mut self) -> Result<(), Error> {
		let mut new_justified_epoch = self.state.current_justified_epoch;
		let mut new_finalized_epoch = self.state.finalized_epoch;
		self.state.justification_bitfield <<= 1;

		let previous_boundary_attesting_balance = self.state.attesting_balance(&self.state.previous_epoch_boundary_attestations()?)?;
		if previous_boundary_attesting_balance * 3 >= self.state.previous_total_balance() * 2 {
			new_justified_epoch = self.state.current_epoch() - 1;
			self.state.justification_bitfield |= 2;
		}

		let current_boundary_attesting_balance = self.state.attesting_balance(&self.state.current_epoch_boundary_attestations()?)?;
		if current_boundary_attesting_balance * 3 >= self.state.current_total_balance() * 2 {
			new_justified_epoch = self.state.current_epoch();
			self.state.justification_bitfield |= 1;
		}

		let bitfield = self.state.justification_bitfield;
		let current_epoch = self.state.current_epoch();
		if (bitfield >> 1) % 8 == 0b111 && self.state.previous_justified_epoch == current_epoch - 3 {
			new_finalized_epoch = self.state.previous_justified_epoch;
		}
		if (bitfield >> 1) % 4 == 0b011 && self.state.previous_justified_epoch == current_epoch - 2 {
			new_finalized_epoch = self.state.previous_justified_epoch;
		}
		if (bitfield >> 0) % 8 == 0b111 && self.state.current_justified_epoch == current_epoch - 2 {
			new_finalized_epoch = self.state.current_justified_epoch;
		}
		if (bitfield >> 0) % 4 == 0b011 && self.state.current_justified_epoch == current_epoch - 1 {
			new_finalized_epoch = self.state.current_justified_epoch;
		}

		self.state.previous_justified_epoch = self.state.current_justified_epoch;
		self.state.previous_justified_root = self.state.current_justified_root;
		if new_justified_epoch != self.state.current_justified_epoch {
			self.state.current_justified_epoch = new_justified_epoch;
			self.state.current_justified_root = self.state.block_root(self.config.epoch_start_slot(new_justified_epoch))?;
		}
		if new_finalized_epoch != self.state.finalized_epoch {
			self.state.finalized_epoch = new_finalized_epoch;
			self.state.finalized_root = self.state.block_root(self.config.epoch_start_slot(new_finalized_epoch))?;
		}

		Ok(())
	}

	pub fn update_crosslinks(&mut self) -> Result<(), Error> {
		let current_epoch = self.state.current_epoch();
		let previous_epoch = current_epoch.saturating_sub(1);
		let next_epoch = current_epoch + 1;

		for slot in self.config.epoch_start_slot(previous_epoch)..self.config.epoch_start_slot(next_epoch) {
			for (crosslink_committee, shard) in self.state.crosslink_committees_at_slot(slot, false)? {
				let (winning_root, participants) = self.state.winning_root_and_participants(shard)?;
				let participating_balance = self.state.total_balance(&participants);
				let total_balance = self.state.total_balance(&crosslink_committee);
				if 3 * participating_balance >= 2 * total_balance {
					self.state.latest_crosslinks[shard as usize] = Crosslink {
						epoch: self.config.slot_to_epoch(slot),
						crosslink_data_root: winning_root,
					};
				}
			}
		}

		Ok(())
	}

	pub fn update_eth1_period(&mut self) {
		if (self.state.current_epoch() + 1) % self.config.epochs_per_eth1_voting_period() == 0 {
			for eth1_data_vote in &self.state.eth1_data_votes {
				if eth1_data_vote.vote_count * 2 > self.config.epochs_per_eth1_voting_period() * self.config.slots_per_epoch() {
					self.state.latest_eth1_data = eth1_data_vote.eth1_data.clone();
				}
			}
			self.state.eth1_data_votes = Vec::new();
		}
	}

	pub fn update_rewards(&mut self) -> Result<(), Error> {
		let delta1 = self.state.justification_and_finalization_deltas()?;
		let delta2 = self.state.crosslink_deltas()?;
		for i in 0..self.state.validator_registry.len() {
			self.state.validator_balances[i] = (self.state.validator_balances[i] + delta1.0[i] + delta2.0[i]).saturating_sub(delta1.1[i] + delta2.1[i]);
		}

		Ok(())
	}

	pub fn update_ejections(&mut self) {
		for index in self.state.active_validator_indices(self.state.current_epoch()) {
			if self.state.validator_balances[index as usize] < self.config.ejection_balance() {
				self.state.exit_validator(index);
			}
		}
	}

	pub fn update_registry_and_shuffling_data(&mut self) -> Result<(), Error> {
		self.state.previous_shuffling_epoch = self.state.current_shuffling_epoch;
		self.state.previous_shuffling_start_shard = self.state.current_shuffling_start_shard;
		self.state.previous_shuffling_seed = self.state.current_shuffling_seed;

		let current_epoch = self.state.current_epoch();
		let next_epoch = current_epoch + 1;

		if self.state.should_update_validator_registry() {
			self.state.update_validator_registry();

			self.state.current_shuffling_epoch = next_epoch;
			self.state.current_shuffling_start_shard = self.state.current_shuffling_start_shard + (self.state.current_epoch_committee_count() % self.config.shard_count()) as u64;
			self.state.current_shuffling_seed = self.state.seed(self.state.current_shuffling_epoch)?;
		} else {
			let epochs_since_last_registry_update = current_epoch - self.state.validator_registry_update_epoch;
			if epochs_since_last_registry_update > 1 && is_power_of_two(epochs_since_last_registry_update) {
				self.state.current_shuffling_epoch = next_epoch;
				self.state.current_shuffling_seed = self.state.seed(self.state.current_shuffling_epoch)?;
			}
		}

		Ok(())
	}

	pub fn update_slashings(&mut self) {
		let current_epoch = self.state.current_epoch();
		let active_validator_indices = self.state.active_validator_indices(current_epoch);
		let total_balance = self.state.total_balance(&active_validator_indices);

		let total_at_start = self.state.latest_slashed_balances[((current_epoch + 1) % self.config.latest_slashed_exit_length() as u64) as usize];
		let total_at_end = self.state.latest_slashed_balances[(current_epoch % self.config.latest_slashed_exit_length() as u64) as usize];
		let total_penalties = total_at_end - total_at_start;

		for (i, validator) in self.state.validator_registry.clone().into_iter().enumerate() {
			let index = i as u64;
			if validator.slashed && current_epoch == validator.withdrawable_epoch - self.config.latest_slashed_exit_length() as u64 / 2 {
				let penalty = core::cmp::max(
					self.state.effective_balance(index) * core::cmp::min(total_penalties * 3, total_balance) / total_balance,
					self.state.effective_balance(index) / self.config.min_penalty_quotient()
				);
				self.state.validator_balances[i] -= penalty;
			}
		}
	}

	pub fn update_exit_queue(&mut self) {
		let mut eligible_indices = (0..(self.state.validator_registry.len() as u64)).filter(|index| {
			if self.state.validator_registry[*index as usize].withdrawable_epoch != self.config.far_future_epoch() {
				false
			} else {
				self.state.current_epoch() >= self.state.validator_registry[*index as usize].exit_epoch.saturating_add(self.config.min_validator_withdrawability_delay())
			}
		}).collect::<Vec<_>>();
		eligible_indices.sort_by_key(|index| {
			self.state.validator_registry[*index as usize].exit_epoch
		});

		for (dequeues, index) in eligible_indices.into_iter().enumerate() {
			if dequeues >= self.config.max_exit_dequeues_per_epoch() {
				break
			}
			self.state.prepare_validator_for_withdrawal(index);
		}
	}

	pub fn update_finalize(&mut self) -> Result<(), Error> {
		let current_epoch = self.state.current_epoch();
		let next_epoch = current_epoch + 1;

		let index_root_position = (next_epoch + self.config.activation_exit_delay()) % self.config.latest_active_index_roots_length() as u64;
		self.state.latest_active_index_roots[index_root_position as usize] = self.state.active_validator_indices(next_epoch + self.config.activation_exit_delay()).hash::<C::Hasher>();
		self.state.latest_slashed_balances[(next_epoch % self.config.latest_slashed_exit_length() as u64) as usize] = self.state.latest_slashed_balances[(current_epoch % self.config.latest_slashed_exit_length() as u64) as usize];
		self.state.latest_randao_mixes[(next_epoch % self.config.latest_randao_mixes_length() as u64) as usize] = self.state.randao_mix(current_epoch)?;

		if next_epoch % (self.config.slots_per_historical_root() as u64 / self.config.slots_per_epoch()) == 0 {
			self.state.historical_roots.push(HistoricalBatch {
				block_roots: self.state.latest_block_roots.clone(),
				state_roots: self.state.latest_state_roots.clone(),
			}.hash::<C::Hasher>());
		}
		self.state.previous_epoch_attestations = self.state.current_epoch_attestations.clone();
		self.state.current_epoch_attestations = Vec::new();

		Ok(())
	}
}
