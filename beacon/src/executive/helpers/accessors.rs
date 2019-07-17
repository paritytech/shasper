use crate::types::*;
use crate::primitives::*;
use crate::{BeaconState, Config, Error, utils};

impl<C: Config> BeaconState<C> {
	pub fn current_epoch(&self) -> Epoch {
		utils::epoch_of_slot(self.slot)
	}

	pub fn previous_epoch(&self) -> Epoch {
		let current_epoch = self.current_epoch();
		if current_epoch == C::genesis_epoch() {
			C::genesis_epoch()
		} else {
			current_epoch.saturating_sub(1)
		}
	}

	pub fn domain(&self, domain_type: Uint, message_epoch: Option<Uint>) -> Uint {
		let epoch = message_epoch.unwrap_or(self.current_epoch());
		let fork_version = if epoch < self.fork.epoch {
			self.fork.previous_version
		} else {
			self.fork.current_version
		};

		utils::bls_domain(domain_type, fork_version)
	}
}
