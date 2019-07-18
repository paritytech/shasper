use crate::primitives::*;
use crate::types::*;
use crate::{Config, BeaconState, Error, BLSConfig, utils, consts};
use bm_le::{tree_root, MaxVec};
use core::cmp::min;

impl<C: Config> BeaconState<C> {
	/// Push a new `VoluntaryExit` to the state.
	pub fn process_voluntary_exit<BLS: BLSConfig>(
		&mut self,
		exit: VoluntaryExit
	) -> Result<(), Error> {
		{
			if exit.validator_index >= self.validators.len() as u64 {
				return Err(Error::VoluntaryExitInvalidSignature)
			}

			let validator = &self.validators[exit.validator_index as usize];

			if !validator.is_active(self.current_epoch()) {
				return Err(Error::VoluntaryExitAlreadyInitiated)
			}

			if validator.exit_epoch != consts::FAR_FUTURE_EPOCH {
				return Err(Error::VoluntaryExitAlreadyExited)
			}

			if self.current_epoch() < exit.epoch {
				return Err(Error::VoluntaryExitNotYetValid)
			}

			if self.current_epoch() < validator.activation_epoch + C::persistent_committee_period() {
				return Err(Error::VoluntaryExitNotLongEnough)
			}

			let domain = self.domain(
				C::domain_voluntary_exit(),
				Some(exit.epoch)
			);
			if !BLS::verify(
				&validator.pubkey,
				&tree_root::<C::Digest, _>(&SigningVoluntaryExit::from(exit.clone())),
				&exit.signature,
				domain
			) {
				return Err(Error::VoluntaryExitInvalidSignature)
			}
		}

		self.initiate_validator_exit(exit.validator_index);
		Ok(())
	}
}
