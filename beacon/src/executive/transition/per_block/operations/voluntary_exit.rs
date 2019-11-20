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

use crate::types::*;
use crate::{Config, BeaconExecutive, Error, BLSConfig, consts};
use bm_le::tree_root;

impl<'a, C: Config> BeaconExecutive<'a, C> {
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
