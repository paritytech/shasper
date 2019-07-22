mod helpers;
mod justification;
mod crosslink;
mod reward;
mod registry;
mod slashing;
mod finalize;

use crate::{Config, BeaconState, Error};

impl<C: Config> BeaconState<C> {
	/// Process an epoch.
	pub fn process_epoch(&mut self) -> Result<(), Error> {
		self.process_justification_and_finalization()?;
		self.process_crosslinks()?;
		self.process_rewards_and_penalties()?;
		self.process_registry_updates()?;
		self.process_slashings();
		self.process_final_updates()?;

		Ok(())
	}
}
