use crate::{Config, BeaconState};
use core::cmp::min;

impl<C: Config> BeaconState<C> {
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
