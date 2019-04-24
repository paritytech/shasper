use ssz::Hashable;

use super::Executive;
use crate::{
	Config, Error, HistoricalBatch, Crosslink, Gwei, Epoch, ValidatorIndex, PendingAttestation,
	Slot, Shard,
};
use crate::primitives::H256;
use crate::util::{is_power_of_two, integer_squareroot, compare_hash};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	fn attesting_indices(&self, attestations: &[PendingAttestation]) -> Result<Vec<ValidatorIndex>, Error> {
		let mut ret = Vec::new();
		for attestation in attestations {
			for index in self.attestation_participants(&attestation.data, &attestation.aggregation_bitfield)? {
				if !ret.contains(&index) {
					ret.push(index);
				}
			}
		}
		Ok(ret)
	}

	fn winning_root_and_participants(&self, shard: Shard) -> Result<(H256, Vec<ValidatorIndex>), Error> {
		let all_attestations = self.state.current_epoch_attestations.clone().into_iter()
			.chain(self.state.previous_epoch_attestations.clone().into_iter());
		let valid_attestations = all_attestations.filter(|a| {
			a.data.previous_crosslink == self.state.latest_crosslinks[shard as usize]
		}).collect::<Vec<_>>();
		let all_roots = valid_attestations.iter()
			.map(|a| a.data.crosslink_data_root)
			.collect::<Vec<_>>();

		let attestations_for = |root| {
			valid_attestations.clone().into_iter()
				.filter(|a| a.data.crosslink_data_root == root)
				.collect::<Vec<_>>()
		};

		let all_roots_with_balances = {
			let mut ret = Vec::new();
			for root in all_roots {
				let balance = self.attesting_balance(&attestations_for(root))?;
				ret.push((root, balance));
			}
			ret
		};

		let winning_root = match all_roots_with_balances.into_iter()
			.max_by(|(a, a_balance), (b, b_balance)| {
				if a_balance == b_balance {
					compare_hash(a, b)
				} else {
					a_balance.cmp(b_balance)
				}
			})
		{
			Some(winning_root) => winning_root.0,
			None => return Ok((H256::default(), Vec::new()))
		};

		Ok((winning_root, self.attesting_indices(&attestations_for(winning_root))?))
	}

	fn inclusion_distance(&self, index: ValidatorIndex) -> Result<Slot, Error> {
		let attestation = self.earliest_attestation(index)?;
		Ok(attestation.inclusion_slot - attestation.data.slot)
	}

	fn attesting_balance(&self, attestations: &[PendingAttestation]) -> Result<Gwei, Error> {
		Ok(self.total_balance(&self.attesting_indices(attestations)?))
	}

	/// Update casper justification and finalization.
	pub fn update_justification_and_finalization(&mut self) -> Result<(), Error> {
		let mut new_justified_epoch = self.state.current_justified_epoch;
		let mut new_finalized_epoch = self.state.finalized_epoch;
		self.state.justification_bitfield <<= 1;

		let previous_boundary_attesting_balance = self.attesting_balance(&self.previous_epoch_boundary_attestations()?)?;
		if previous_boundary_attesting_balance * 3 >= self.previous_total_balance() * 2 {
			new_justified_epoch = self.current_epoch() - 1;
			self.state.justification_bitfield |= 2;
		}

		let current_boundary_attesting_balance = self.attesting_balance(&self.current_epoch_boundary_attestations()?)?;
		if current_boundary_attesting_balance * 3 >= self.current_total_balance() * 2 {
			new_justified_epoch = self.current_epoch();
			self.state.justification_bitfield |= 1;
		}

		let bitfield = self.state.justification_bitfield;
		let current_epoch = self.current_epoch();
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
			self.state.current_justified_root = self.block_root(self.config.epoch_start_slot(new_justified_epoch))?;
		}
		if new_finalized_epoch != self.state.finalized_epoch {
			self.state.finalized_epoch = new_finalized_epoch;
			self.state.finalized_root = self.block_root(self.config.epoch_start_slot(new_finalized_epoch))?;
		}

		Ok(())
	}

	/// Update crosslink data in state.
	pub fn update_crosslinks(&mut self) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		let previous_epoch = current_epoch.saturating_sub(1);
		let next_epoch = current_epoch + 1;

		for slot in self.config.epoch_start_slot(previous_epoch)..self.config.epoch_start_slot(next_epoch) {
			for (crosslink_committee, shard) in self.crosslink_committees_at_slot(slot, false)? {
				let (winning_root, participants) = self.winning_root_and_participants(shard)?;
				let participating_balance = self.total_balance(&participants);
				let total_balance = self.total_balance(&crosslink_committee);
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

	/// Update voting period for eth1.
	pub fn update_eth1_period(&mut self) {
		if (self.current_epoch() + 1) % self.config.epochs_per_eth1_voting_period() == 0 {
			for eth1_data_vote in &self.state.eth1_data_votes {
				if eth1_data_vote.vote_count * 2 > self.config.epochs_per_eth1_voting_period() * self.config.slots_per_epoch() {
					self.state.latest_eth1_data = eth1_data_vote.eth1_data.clone();
				}
			}
			self.state.eth1_data_votes = Vec::new();
		}
	}

	fn base_reward(&self, index: ValidatorIndex) -> Gwei {
		if self.previous_total_balance() == 0 {
			return 0
		}

		let adjusted_quotient = integer_squareroot(self.previous_total_balance()) / self.config.base_reward_quotient();
		self.effective_balance(index) / adjusted_quotient / 5
	}

	fn inactivity_penalty(&self, index: ValidatorIndex, epochs_since_finality: Epoch) -> Gwei {
		self.base_reward(index) + self.effective_balance(index) * epochs_since_finality / self.config.inactivity_penalty_quotient() / 2
	}

	fn justification_and_finalization_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let epochs_since_finality = self.current_epoch() + 1 - self.state.finalized_epoch;
		if epochs_since_finality <= 4 {
			self.normal_justification_and_finalization_deltas()
		} else {
			self.inactivity_leak_deltas()
		}
	}

	fn normal_justification_and_finalization_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let mut rewards = Vec::new();
		rewards.resize(self.state.validator_registry.len(), 0);
		let mut penalties = Vec::new();
		penalties.resize(self.state.validator_registry.len(), 0);

		let boundary_attestations = self.previous_epoch_boundary_attestations()?;
		let boundary_attesting_balance = self.attesting_balance(&boundary_attestations)?;
		let total_balance = self.previous_total_balance();
		let total_attesting_balance = self.attesting_balance(&self.state.previous_epoch_attestations)?;
		let matching_head_attestations = self.previous_epoch_matching_head_attestations()?;
		let matching_head_balance = self.attesting_balance(&matching_head_attestations)?;

		for index in self.state.active_validator_indices(self.previous_epoch()) {
			if self.attesting_indices(&self.state.previous_epoch_attestations)?.contains(&index) {
				rewards[index as usize] += self.base_reward(index) * total_attesting_balance / total_balance;
				rewards[index as usize] += self.base_reward(index) * self.config.min_attestation_inclusion_delay() / self.inclusion_distance(index)?;
			} else {
				penalties[index as usize] += self.base_reward(index);
			}

			if self.attesting_indices(&boundary_attestations)?.contains(&index) {
				rewards[index as usize] += self.base_reward(index) * boundary_attesting_balance / total_balance;
			} else {
				penalties[index as usize] += self.base_reward(index);
			}

			if self.attesting_indices(&matching_head_attestations)?.contains(&index) {
				rewards[index as usize] += self.base_reward(index) * matching_head_balance / total_balance;
			} else {
				penalties[index as usize] += self.base_reward(index);
			}

			if self.attesting_indices(&self.state.previous_epoch_attestations)?.contains(&index) {
				let proposer_index = self.beacon_proposer_index(self.inclusion_slot(index)?, false)?;
				rewards[proposer_index as usize] += self.base_reward(index) / self.config.attestation_inclusion_reward_quotient();
			}
		}

		Ok((rewards, penalties))
	}

	fn inactivity_leak_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let mut rewards = Vec::new();
		rewards.resize(self.state.validator_registry.len(), 0);
		let mut penalties = Vec::new();
		penalties.resize(self.state.validator_registry.len(), 0);

		let boundary_attestations = self.previous_epoch_boundary_attestations()?;
		let matching_head_attestations = self.previous_epoch_matching_head_attestations()?;
		let active_validator_indices = self.state.active_validator_indices(self.previous_epoch());
		let epochs_since_finality = self.current_epoch() + 1 - self.state.finalized_epoch;

		for index in &active_validator_indices {
			if !self.attesting_indices(&self.state.previous_epoch_attestations)?.contains(index) {
				penalties[*index as usize] += self.inactivity_penalty(*index, epochs_since_finality);
			} else {
				rewards[*index as usize] += self.base_reward(*index) * self.config.min_attestation_inclusion_delay() / self.inclusion_distance(*index)?;
				penalties[*index as usize] += self.base_reward(*index);
			}

			if !self.attesting_indices(&boundary_attestations)?.contains(index) {
				penalties[*index as usize] += self.inactivity_penalty(*index, epochs_since_finality);
			}

			if !self.attesting_indices(&matching_head_attestations)?.contains(index) {
				penalties[*index as usize] += self.base_reward(*index);
			}
		}

		for index in 0..(self.state.validator_registry.len() as u64) {
			let eligible = !active_validator_indices.contains(&index) &&
				self.state.validator_registry[index as usize].slashed &&
				self.current_epoch() < self.state.validator_registry[index as usize].withdrawable_epoch;

			if eligible {
				penalties[index as usize] += 2 * self.inactivity_penalty(index, epochs_since_finality) + self.base_reward(index);
			}
		}

		Ok((rewards, penalties))
	}

	fn crosslink_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let mut rewards = Vec::new();
		rewards.resize(self.state.validator_registry.len(), 0);
		let mut penalties = Vec::new();
		penalties.resize(self.state.validator_registry.len(), 0);

		let previous_epoch_start_slot = self.config.epoch_start_slot(self.previous_epoch());
		let current_epoch_start_slot = self.config.epoch_start_slot(self.current_epoch());

		for slot in previous_epoch_start_slot..current_epoch_start_slot {
			for (crosslink_committee, shard) in self.crosslink_committees_at_slot(slot, false)? {
				let (_, participants) = self.winning_root_and_participants(shard)?;
				let participating_balance = self.total_balance(&participants);
				let total_balance = self.total_balance(&crosslink_committee);
				for index in crosslink_committee {
					if participants.contains(&index) {
						rewards[index as usize] += self.base_reward(index) * participating_balance / total_balance;
					} else {
						penalties[index as usize] += self.base_reward(index);
					}
				}
			}
		}

		Ok((rewards, penalties))
	}

	/// Update validator rewards.
	pub fn update_rewards(&mut self) -> Result<(), Error> {
		let delta1 = self.justification_and_finalization_deltas()?;
		let delta2 = self.crosslink_deltas()?;
		for i in 0..self.state.validator_registry.len() {
			self.state.validator_balances[i] = (self.state.validator_balances[i] + delta1.0[i] + delta2.0[i]).saturating_sub(delta1.1[i] + delta2.1[i]);
		}

		Ok(())
	}

	/// Update validator ejections.
	pub fn update_ejections(&mut self) {
		for index in self.state.active_validator_indices(self.current_epoch()) {
			if self.state.validator_balances[index as usize] < self.config.ejection_balance() {
				self.exit_validator(index);
			}
		}
	}

	fn should_update_validator_registry(&self) -> bool {
		if self.state.finalized_epoch <= self.state.validator_registry_update_epoch {
			return false
		}

		for i in 0..self.current_epoch_committee_count() {
			let s = (self.state.current_shuffling_start_shard as usize + i) % self.config.shard_count();
			if self.state.latest_crosslinks[s].epoch <= self.state.validator_registry_update_epoch {
				return false
			}
		}

		true
	}

	fn update_validator_registry(&mut self) {
		let current_epoch = self.current_epoch();
		let active_validator_indices = self.state.active_validator_indices(current_epoch);
		let total_balance = self.total_balance(&active_validator_indices);

		let max_balance_churn = core::cmp::max(
			self.config.max_deposit_amount(),
			total_balance / (2 * self.config.max_balance_churn_quotient())
		);

		let mut balance_churn = 0;
		for (i, validator) in self.state.validator_registry.clone().into_iter().enumerate() {
			let index = i as u64;
			if validator.activation_epoch == self.config.far_future_epoch() && self.state.validator_balances[i] >= self.config.max_deposit_amount() {
				balance_churn += self.effective_balance(index);
				if balance_churn > max_balance_churn {
					break
				}

				self.activate_validator(index, false);
			}
		}

		let mut balance_churn = 0;
		for (i, validator) in self.state.validator_registry.clone().into_iter().enumerate() {
			let index = i as u64;
			if validator.exit_epoch == self.config.far_future_epoch() && validator.initiated_exit {
				balance_churn += self.effective_balance(index);
				if balance_churn > max_balance_churn {
					break
				}

				self.exit_validator(index);
			}
		}

		self.state.validator_registry_update_epoch = current_epoch;
	}

	/// Update validator registry and shuffling data.
	pub fn update_registry_and_shuffling_data(&mut self) -> Result<(), Error> {
		self.state.previous_shuffling_epoch = self.state.current_shuffling_epoch;
		self.state.previous_shuffling_start_shard = self.state.current_shuffling_start_shard;
		self.state.previous_shuffling_seed = self.state.current_shuffling_seed;

		let current_epoch = self.current_epoch();
		let next_epoch = current_epoch + 1;

		if self.should_update_validator_registry() {
			self.update_validator_registry();

			self.state.current_shuffling_epoch = next_epoch;
			self.state.current_shuffling_start_shard = self.state.current_shuffling_start_shard + (self.current_epoch_committee_count() % self.config.shard_count()) as u64;
			self.state.current_shuffling_seed = self.seed(self.state.current_shuffling_epoch)?;
		} else {
			let epochs_since_last_registry_update = current_epoch - self.state.validator_registry_update_epoch;
			if epochs_since_last_registry_update > 1 && is_power_of_two(epochs_since_last_registry_update) {
				self.state.current_shuffling_epoch = next_epoch;
				self.state.current_shuffling_seed = self.seed(self.state.current_shuffling_epoch)?;
			}
		}

		Ok(())
	}

	/// Update validator slashings.
	pub fn update_slashings(&mut self) {
		let current_epoch = self.current_epoch();
		let active_validator_indices = self.state.active_validator_indices(current_epoch);
		let total_balance = self.total_balance(&active_validator_indices);

		let total_at_start = self.state.latest_slashed_balances[((current_epoch + 1) % self.config.latest_slashed_exit_length() as u64) as usize];
		let total_at_end = self.state.latest_slashed_balances[(current_epoch % self.config.latest_slashed_exit_length() as u64) as usize];
		let total_penalties = total_at_end - total_at_start;

		for (i, validator) in self.state.validator_registry.clone().into_iter().enumerate() {
			let index = i as u64;
			if validator.slashed && current_epoch == validator.withdrawable_epoch - self.config.latest_slashed_exit_length() as u64 / 2 {
				let penalty = core::cmp::max(
					self.effective_balance(index) * core::cmp::min(total_penalties * 3, total_balance) / total_balance,
					self.effective_balance(index) / self.config.min_penalty_quotient()
				);
				self.state.validator_balances[i] -= penalty;
			}
		}
	}

	fn prepare_validator_for_withdrawal(&mut self, index: ValidatorIndex) {
		self.state.validator_registry[index as usize].withdrawable_epoch = self.current_epoch() + self.config.min_validator_withdrawability_delay();
	}

	/// Process the exit queue.
	pub fn update_exit_queue(&mut self) {
		let mut eligible_indices = (0..(self.state.validator_registry.len() as u64)).filter(|index| {
			if self.state.validator_registry[*index as usize].withdrawable_epoch != self.config.far_future_epoch() {
				false
			} else {
				self.current_epoch() >= self.state.validator_registry[*index as usize].exit_epoch.saturating_add(self.config.min_validator_withdrawability_delay())
			}
		}).collect::<Vec<_>>();
		eligible_indices.sort_by_key(|index| {
			self.state.validator_registry[*index as usize].exit_epoch
		});

		for (dequeues, index) in eligible_indices.into_iter().enumerate() {
			if dequeues >= self.config.max_exit_dequeues_per_epoch() {
				break
			}
			self.prepare_validator_for_withdrawal(index);
		}
	}

	/// Finalize per-epoch update.
	pub fn update_finalize(&mut self) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		let next_epoch = current_epoch + 1;

		let index_root_position = (next_epoch + self.config.activation_exit_delay()) % self.config.latest_active_index_roots_length() as u64;
		self.state.latest_active_index_roots[index_root_position as usize] = Hashable::<C::Hasher>::hash(&self.state.active_validator_indices(next_epoch + self.config.activation_exit_delay()));
		self.state.latest_slashed_balances[(next_epoch % self.config.latest_slashed_exit_length() as u64) as usize] = self.state.latest_slashed_balances[(current_epoch % self.config.latest_slashed_exit_length() as u64) as usize];
		self.state.latest_randao_mixes[(next_epoch % self.config.latest_randao_mixes_length() as u64) as usize] = self.randao_mix(current_epoch)?;

		if next_epoch % (self.config.slots_per_historical_root() as u64 / self.config.slots_per_epoch()) == 0 {
			self.state.historical_roots.push(Hashable::<C::Hasher>::hash(&HistoricalBatch {
				block_roots: self.state.latest_block_roots.clone(),
				state_roots: self.state.latest_state_roots.clone(),
			}));
		}
		self.state.previous_epoch_attestations = self.state.current_epoch_attestations.clone();
		self.state.current_epoch_attestations = Vec::new();

		Ok(())
	}
}
