use crate::{Config, BeaconState, Error, consts, utils};
use core::cmp::min;

impl<C: Config> BeaconState<C> {
	/// Process registry updates
	pub fn process_registry_updates(&mut self) -> Result<(), Error> {
		for index in 0..self.validators.len() {
			if self.validators[index].activation_eligibility_epoch == consts::FAR_FUTURE_EPOCH &&
				self.validators[index].effective_balance == C::max_effective_balance()
			{
				self.validators[index].activation_eligibility_epoch = self.current_epoch();
			}

			if self.validators[index].is_active(self.current_epoch()) &&
				self.validators[index].effective_balance <= C::ejection_balance()
			{
				self.initiate_validator_exit(index as u64);
			}
		}

		let mut activation_queue = self.validators.iter()
			.enumerate()
			.filter(|(_, v)| {
				v.activation_eligibility_epoch != consts::FAR_FUTURE_EPOCH &&
					v.activation_epoch >=
					utils::activation_exit_epoch::<C>(self.finalized_checkpoint.epoch)
			})
			.map(|(i, _)| i as u64)
			.collect::<Vec<_>>();
		activation_queue.sort_by_key(|index| {
			self.validators[*index as usize].activation_eligibility_epoch
		});

		for index in &activation_queue[..min(activation_queue.len(),
											 self.validator_churn_limit() as usize)]
		{
			let current_epoch = self.current_epoch();
			let validator = &mut self.validators[*index as usize];
			if validator.activation_epoch == consts::FAR_FUTURE_EPOCH {
				validator.activation_epoch =
					utils::activation_exit_epoch::<C>(current_epoch);
			}
		}

		Ok(())
	}
}
