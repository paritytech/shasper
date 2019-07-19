use crate::types::*;
use crate::{Config, BeaconState, Error};
use bm_le::{MaxVec, Compact, tree_root};
use core::cmp::min;

impl<C: Config> BeaconState<C> {
	/// Process final updates
	pub fn process_final_updates(&mut self) -> Result<(), Error> {
		let current_epoch = self.current_epoch();
		let next_epoch = current_epoch + 1;

		// Reset eth1 data votes
		if (self.slot + 1) % C::slots_per_eth1_voting_period() == 0 {
			self.eth1_data_votes = Default::default();
		}

		// Update effective balances with hysteresis
		for index in 0..(self.validators.len() as u64) {
			let validator = &mut self.validators[index as usize];
			let balance = self.balances[index as usize];
			let half_increment = C::effective_balance_increment() / 2;
			if balance < validator.effective_balance ||
				validator.effective_balance + 3 * half_increment < balance
			{
				validator.effective_balance = min(
					balance - balance % C::effective_balance_increment(),
					C::max_effective_balance()
				);
			}
		}

		// Update start shard
		self.start_shard =
			(self.start_shard + self.shard_delta(current_epoch)) %
			C::shard_count();

		// Set active index root
		let index_epoch = next_epoch + C::activation_exit_delay();
		let index_root_position = index_epoch % C::epochs_per_historical_vector();
		self.active_index_roots[index_root_position as usize] =
			tree_root::<C::Digest, _>(
				&Compact(MaxVec::<_, C::ValidatorRegistryLimit>::from(self.active_validator_indices(
					next_epoch + C::activation_exit_delay()
				)))
			);

		// Set committees root
		let committee_root_position = next_epoch % C::epochs_per_historical_vector();
		self.compact_committees_roots[committee_root_position as usize] =
			self.compact_committees_root(next_epoch)?;

		// Set total slashed balances
		self.slashings[
			(next_epoch % C::epochs_per_slashings_vector()) as usize
		] = 0;

		// Set randao mix
		self.randao_mixes[
			(next_epoch % C::epochs_per_historical_vector()) as usize
		] = self.randao_mix(current_epoch);

		// Set historical root accumulator
		if next_epoch %
			(C::slots_per_historical_root() / C::slots_per_epoch())
			== 0
		{
			self.historical_roots.push(tree_root::<C::Digest, _>(&HistoricalBatch::<C> {
				block_roots: self.block_roots.clone(),
				state_roots: self.state_roots.clone(),
			}));
		}

		// Rotate current/previous epoch attestations
		self.previous_epoch_attestations =
			self.current_epoch_attestations.clone();
		self.current_epoch_attestations = Default::default();

		Ok(())
	}
}
