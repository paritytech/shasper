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

mod proposer_slashing;
mod attester_slashing;
mod attestation;
mod deposit;
mod voluntary_exit;
mod transfer;

use core::cmp::min;
use crate::{Error, Executive, Config};
use crate::types::BeaconBlockBody;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Process block operations.
	pub fn process_operations(
		&mut self,
		body: &BeaconBlockBody
	) -> Result<(), Error> {
		// Verify that outstanding deposits are processed up to the maximum
		// number of deposits
		if body.deposits.len() as u64 != min(
			self.config.max_deposits(),
			self.state.latest_eth1_data.deposit_count -
				self.state.deposit_index
		) {
			return Err(Error::TooManyDeposits)
		}

		// Verify that there are no duplicate transfers
		if (1..body.transfers.len())
			.any(|i| body.transfers[i..].contains(&body.transfers[i - 1]))
		{
			return Err(Error::DuplicateTransfer)
		}

		if body.proposer_slashings.len() > self.config.max_proposer_slashings() as usize {
			return Err(Error::TooManyProposerSlashings)
		}
		for slashing in &body.proposer_slashings {
			self.process_proposer_slashing(slashing.clone())?;
		}

		if body.attester_slashings.len() > self.config.max_attester_slashings() as usize {
			return Err(Error::TooManyAttesterSlashings)
		}
		for slashing in &body.attester_slashings {
			self.process_attester_slashing(slashing.clone())?;
		}

		if body.attestations.len() > self.config.max_attestations() as usize {
			return Err(Error::TooManyAttestations)
		}
		for attestation in &body.attestations {
			self.process_attestation(attestation.clone())?;
		}

		if body.deposits.len() > self.config.max_deposits() as usize {
			return Err(Error::TooManyDeposits)
		}
		for deposit in &body.deposits {
			self.process_deposit(deposit.clone())?;
		}

		if body.voluntary_exits.len() > self.config.max_voluntary_exits() as usize {
			return Err(Error::TooManyVoluntaryExits)
		}
		for voluntary_exit in &body.voluntary_exits {
			self.process_voluntary_exit(voluntary_exit.clone())?;
		}

		if body.transfers.len() > self.config.max_transfers() as usize{
			return Err(Error::TooManyTransfers)
		}
		for transfer in &body.transfers {
			self.process_transfer(transfer.clone())?;
		}

		Ok(())
	}
}
