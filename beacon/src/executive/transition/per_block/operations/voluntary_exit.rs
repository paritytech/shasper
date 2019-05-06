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

use ssz::Digestible;
use crate::primitives::H256;
use crate::types::VoluntaryExit;
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Push a new `VoluntaryExit` to the state.
	pub fn process_voluntary_exit(&mut self, exit: VoluntaryExit) -> Result<(), Error> {
		{
			let validator = &self.state.validator_registry[exit.validator_index as usize];

			if !validator.is_active(self.current_epoch()) {
				return Err(Error::VoluntaryExitAlreadyInitiated)
			}

			if validator.exit_epoch != self.config.far_future_epoch() {
				return Err(Error::VoluntaryExitAlreadyExited)
			}

			if self.current_epoch() < exit.epoch {
				return Err(Error::VoluntaryExitNotYetValid)
			}

			if self.current_epoch() - validator.activation_epoch < self.config.persistent_committee_period() {
				return Err(Error::VoluntaryExitNotLongEnough)
			}

			let domain = self.domain(
				self.config.domain_voluntary_exit(),
				Some(exit.epoch)
			);
			if !self.config.bls_verify(
				&validator.pubkey,
				&H256::from_slice(
					Digestible::<C::Digest>::truncated_hash(&exit).as_slice()
				),
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
