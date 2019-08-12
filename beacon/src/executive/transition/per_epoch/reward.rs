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

use crate::primitives::*;
use crate::{Config, BeaconState, Error, utils, consts};

impl<C: Config> BeaconState<C> {
	fn base_reward(&self, index: ValidatorIndex) -> Gwei {
		let total_balance = self.total_active_balance();

		let effective_balance =
			self.validators[index as usize].effective_balance;

		effective_balance * C::base_reward_factor() /
			utils::integer_squareroot(total_balance) /
			consts::BASE_REWARDS_PER_EPOCH
	}

	fn attestation_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let previous_epoch = self.previous_epoch();
		let total_balance = self.total_active_balance();
		let mut rewards = (0..self.validators.len())
			.map(|_| 0).collect::<Vec<_>>();
		let mut penalties = (0..self.validators.len())
			.map(|_| 0).collect::<Vec<_>>();
		let eligible_validator_indices = self.validators.iter()
			.enumerate()
			.filter(|(_, v)| {
				v.is_active(previous_epoch) ||
					(v.slashed && previous_epoch + 1 < v.withdrawable_epoch)
			})
			.map(|(i, _)| i as u64)
			.collect::<Vec<_>>();

		// Micro-incentives for matching FFG source, FFG target, and head
		let matching_source_attestations =
			self.matching_source_attestations(previous_epoch)?;
		let matching_target_attestations =
			self.matching_target_attestations(previous_epoch)?;
		let matching_head_attestations =
			self.matching_head_attestations(previous_epoch)?;
		for attestations in &[&matching_source_attestations,
							  &matching_target_attestations,
							  &matching_head_attestations]
		{
			let unslashed_attesting_indices =
				self.unslashed_attesting_indices(attestations)?;
			let attesting_balance = self.total_balance(&unslashed_attesting_indices);
			for index in &eligible_validator_indices {
				if unslashed_attesting_indices.contains(index) {
					rewards[*index as usize] += self.base_reward(*index) *
						attesting_balance / total_balance;
				} else {
					penalties[*index as usize] += self.base_reward(*index);
				}
			}
		}

		// Proposer and inclusion delay micro-rewards
		for index in self.unslashed_attesting_indices(&matching_source_attestations)? {
			let attestation = matching_source_attestations.iter()
				.map(|a| Ok((
					a,
					self.attesting_indices(&a.data, &a.aggregation_bits)?
						.contains(&index)
				)))
				.collect::<Result<Vec<_>, _>>()?
				.into_iter()
				.filter(|(_, c)| *c)
				.map(|(a, _)| a)
				.fold(matching_source_attestations[0].clone(), |a, b| {
					if a.inclusion_delay < b.inclusion_delay { a } else { b.clone() }
				});

			let proposer_reward = self.base_reward(index) / C::proposer_reward_quotient();
			rewards[attestation.proposer_index as usize] += proposer_reward;
			let max_attester_reward = self.base_reward(index) - proposer_reward;
			rewards[index as usize] += max_attester_reward *
				(C::slots_per_epoch() + C::min_attestation_inclusion_delay() -
				 attestation.inclusion_delay) /
				C::slots_per_epoch();
		}

		// Inactivity penalty
		let finality_delay = previous_epoch - self.finalized_checkpoint.epoch;
		if finality_delay > C::min_epochs_to_inactivity_penalty() {
			let matching_target_attesting_indices =
				self.unslashed_attesting_indices(&matching_target_attestations)?;
			for index in &eligible_validator_indices {
				penalties[*index as usize] += consts::BASE_REWARDS_PER_EPOCH *
					self.base_reward(*index);
				if !matching_target_attesting_indices.contains(index) {
					penalties[*index as usize] +=
						self.validators[*index as usize].effective_balance *
						finality_delay / C::inactivity_penalty_quotient();
				}
			}
		}

		Ok((rewards, penalties))
	}

	fn crosslink_deltas(&self) -> Result<(Vec<Gwei>, Vec<Gwei>), Error> {
		let mut rewards = (0..self.validators.len())
			.map(|_| 0).collect::<Vec<_>>();
		let mut penalties = (0..self.validators.len())
			.map(|_| 0).collect::<Vec<_>>();
		let epoch = self.previous_epoch();

		for offset in 0..self.committee_count(epoch) {
			let shard = (self.start_shard(epoch)? + offset) % C::shard_count();
			let crosslink_committee = self.crosslink_committee(epoch, shard)?;
			let (_winning_crosslink, attesting_indices) =
				self.winning_crosslink_and_attesting_indices(epoch, shard)?;
			let attesting_balance = self.total_balance(&attesting_indices);
			let committee_balance = self.total_balance(&crosslink_committee);
			for index in crosslink_committee {
				let base_reward = self.base_reward(index);
				if attesting_indices.contains(&index) {
					rewards[index as usize] += base_reward * attesting_balance /
						committee_balance;
				} else {
					penalties[index as usize] += base_reward;
				}
			}
		}

		Ok((rewards, penalties))
	}

	/// Process rewards and penalties
	pub fn process_rewards_and_penalties(&mut self) -> Result<(), Error> {
		if self.current_epoch() == C::genesis_epoch() {
			return Ok(())
		}

		let (rewards1, penalties1) = self.attestation_deltas()?;
		let (rewards2, penalties2) = self.crosslink_deltas()?;
		for i in 0..self.validators.len() {
			self.increase_balance(i as u64, rewards1[i] + rewards2[i]);
			self.decrease_balance(i as u64, penalties1[i] + penalties2[i]);
		}

		Ok(())
	}
}
