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

use ssz::Hashable;
use crate::state::{BeaconState, HistoricalBatch};
use crate::attestation::Crosslink;
use crate::error::Error;
use crate::util::{Hasher, epoch_start_slot, slot_to_epoch, is_power_of_two};
use crate::consts::{
	EPOCHS_PER_ETH1_VOTING_PERIOD, SLOTS_PER_EPOCH, EJECTION_BALANCE,
	SHARD_COUNT, LATEST_SLASHED_EXIT_LENGTH, MIN_PENALTY_QUOTIENT,
	FAR_FUTURE_EPOCH, MIN_VALIDATOR_WITHDRAWABILITY_DELAY,
	MAX_EXIT_DEQUEUES_PER_EPOCH, ACTIVATION_EXIT_DELAY,
	LATEST_ACTIVE_INDEX_ROOTS_LENGTH, LATEST_RANDAO_MIXES_LENGTH,
	SLOTS_PER_HISTORICAL_ROOT,
};

impl BeaconState {
	pub fn update_justification_and_finalization(&mut self) -> Result<(), Error> {
		let mut new_justified_epoch = self.justified_epoch;
		self.justification_bitfield <<= 1;

		let previous_boundary_attesting_balance = self.attesting_balance(&self.previous_epoch_boundary_attestations()?)?;
		if previous_boundary_attesting_balance * 3 >= self.previous_total_balance() * 2 {
			new_justified_epoch = self.current_epoch() - 1;
			self.justification_bitfield |= 2;
		}

		let current_boundary_attesting_balance = self.attesting_balance(&self.current_epoch_boundary_attestations()?)?;
		if current_boundary_attesting_balance * 3 >= self.current_total_balance() * 2 {
			new_justified_epoch = self.current_epoch();
			self.justification_bitfield |= 1;
		}

		let bitfield = self.justification_bitfield;
		let current_epoch = self.current_epoch();
		if (bitfield >> 1) % 8 == 0b111 && self.previous_justified_epoch == current_epoch - 3 {
			self.finalized_epoch = self.previous_justified_epoch;
		}
		if (bitfield >> 1) % 4 == 0b011 && self.previous_justified_epoch == current_epoch - 2 {
			self.finalized_epoch = self.previous_justified_epoch;
		}
		if (bitfield >> 0) % 8 == 0b111 && self.justified_epoch == current_epoch - 2 {
			self.finalized_epoch = self.justified_epoch;
		}
		if (bitfield >> 0) % 4 == 0b011 && self.justified_epoch == current_epoch - 1 {
			self.finalized_epoch = self.justified_epoch;
		}

		self.previous_justified_epoch = self.justified_epoch;
		self.justified_epoch = new_justified_epoch;

		Ok(())
	}

	pub fn update_crosslinks(&mut self) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		let previous_epoch = current_epoch.saturating_sub(1);
		let next_epoch = current_epoch + 1;

		for slot in epoch_start_slot(previous_epoch)..epoch_start_slot(next_epoch) {
			for (crosslink_committee, shard) in self.crosslink_committees_at_slot(slot, false)? {
				let (winning_root, participants) = self.winning_root_and_participants(shard)?;
				let participating_balance = self.total_balance(&participants);
				let total_balance = self.total_balance(&crosslink_committee);
				if 3 * participating_balance >= 2 * total_balance {
					self.latest_crosslinks[shard as usize] = Crosslink {
						epoch: slot_to_epoch(slot),
						crosslink_data_root: winning_root,
					};
				}
			}
		}

		Ok(())
	}

	pub fn update_eth1_period(&mut self) {
		if (self.current_epoch() + 1) % EPOCHS_PER_ETH1_VOTING_PERIOD == 0 {
			for eth1_data_vote in &self.eth1_data_votes {
				if eth1_data_vote.vote_count * 2 > EPOCHS_PER_ETH1_VOTING_PERIOD * SLOTS_PER_EPOCH {
					self.latest_eth1_data = eth1_data_vote.eth1_data.clone();
				}
			}
			self.eth1_data_votes = Vec::new();
		}
	}

	pub fn update_rewards(&mut self) -> Result<(), Error> {
		let delta1 = self.justification_and_finalization_deltas()?;
		let delta2 = self.crosslink_deltas()?;
		for i in 0..self.validator_registry.len() {
			self.validator_balances[i] = (self.validator_balances[i] + delta1.0[i] + delta2.0[i]).saturating_sub(delta1.1[i] + delta2.1[i]);
		}

		Ok(())
	}

	pub fn update_ejections(&mut self) {
		for index in self.active_validator_indices(self.current_epoch()) {
			if self.validator_balances[index as usize] < EJECTION_BALANCE {
				self.exit_validator(index);
			}
		}
	}

	pub fn update_registry_and_shuffling_data(&mut self) -> Result<(), Error> {
		self.previous_shuffling_epoch = self.current_shuffling_epoch;
		self.previous_shuffling_start_shard = self.current_shuffling_start_shard;
		self.previous_shuffling_seed = self.current_shuffling_seed;

		let current_epoch = self.current_epoch();
		let next_epoch = current_epoch + 1;

		if self.should_update_validator_registry() {
			self.update_validator_registry();

			self.current_shuffling_epoch = next_epoch;
			self.current_shuffling_start_shard = self.current_shuffling_start_shard + (self.current_epoch_committee_count() % SHARD_COUNT) as u64;
			self.current_shuffling_seed = self.seed(self.current_shuffling_epoch)?;
		} else {
			let epochs_since_last_registry_update = current_epoch - self.validator_registry_update_epoch;
			if epochs_since_last_registry_update > 1 && is_power_of_two(epochs_since_last_registry_update) {
				self.current_shuffling_epoch = next_epoch;
				self.current_shuffling_seed = self.seed(self.current_shuffling_epoch)?;
			}
		}

		Ok(())
	}

	pub fn update_slashings(&mut self) {
		let current_epoch = self.current_epoch();
		let active_validator_indices = self.active_validator_indices(current_epoch);
		let total_balance = self.total_balance(&active_validator_indices);

		let total_at_start = self.latest_slashed_balances[((current_epoch + 1) % LATEST_SLASHED_EXIT_LENGTH as u64) as usize];
		let total_at_end = self.latest_slashed_balances[(current_epoch % LATEST_SLASHED_EXIT_LENGTH as u64) as usize];
		let total_penalties = total_at_end - total_at_start;

		for (i, validator) in self.validator_registry.clone().into_iter().enumerate() {
			let index = i as u64;
			if validator.slashed && current_epoch == validator.withdrawable_epoch - LATEST_SLASHED_EXIT_LENGTH as u64 / 2 {
				let penalty = core::cmp::max(
					self.effective_balance(index) * core::cmp::min(total_penalties * 3, total_balance) / total_balance,
					self.effective_balance(index) / MIN_PENALTY_QUOTIENT
				);
				self.validator_balances[i] -= penalty;
			}
		}
	}

	pub fn update_exit_queue(&mut self) {
		let mut eligible_indices = (0..(self.validator_registry.len() as u64)).filter(|index| {
			if self.validator_registry[*index as usize].withdrawable_epoch != FAR_FUTURE_EPOCH {
				false
			} else {
				self.current_epoch() >= self.validator_registry[*index as usize].exit_epoch + MIN_VALIDATOR_WITHDRAWABILITY_DELAY
			}
		}).collect::<Vec<_>>();
		eligible_indices.sort_by_key(|index| {
			self.validator_registry[*index as usize].exit_epoch
		});

		for (dequeues, index) in eligible_indices.into_iter().enumerate() {
			if dequeues >= MAX_EXIT_DEQUEUES_PER_EPOCH {
				break
			}
			self.prepare_validator_for_withdrawal(index);
		}
	}

	pub fn update_finalize(&mut self) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		let next_epoch = current_epoch + 1;

		let index_root_position = (next_epoch + ACTIVATION_EXIT_DELAY) % LATEST_ACTIVE_INDEX_ROOTS_LENGTH as u64;
		self.latest_active_index_roots[index_root_position as usize] = self.active_validator_indices(next_epoch + ACTIVATION_EXIT_DELAY).hash::<Hasher>();
		self.latest_slashed_balances[(next_epoch % LATEST_SLASHED_EXIT_LENGTH as u64) as usize] = self.latest_slashed_balances[(current_epoch % LATEST_SLASHED_EXIT_LENGTH as u64) as usize];
		self.latest_randao_mixes[(next_epoch % LATEST_RANDAO_MIXES_LENGTH as u64) as usize] = self.randao_mix(current_epoch)?;

		if next_epoch % (SLOTS_PER_HISTORICAL_ROOT as u64 / SLOTS_PER_EPOCH) == 0 {
			self.historical_roots.push(HistoricalBatch {
				block_roots: self.latest_block_roots.clone(),
				state_roots: self.latest_state_roots.clone(),
			}.hash::<Hasher>());
		}
		self.previous_epoch_attestations = self.current_epoch_attestations.clone();
		self.current_epoch_attestations = Vec::new();

		Ok(())
	}
}
