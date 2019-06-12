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

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub fn process_operations(
		&mut self,
		body: &BeaconBlockBody
	) -> Result<(), Error> {
		// Verify that outstanding deposits are processed up to the maximum
		// number of deposits
		if body.deposits.len() != min(
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

		if block.body.proposer_slashings.len() > config.max_proposer_slashings() as usize {
			return Err(Error::TooManyProposerSlashings)
		}
		for slashing in &block.body.proposer_slashings {
			executive.process_proposer_slashing(slashing.clone())?;
		}

		if block.body.attester_slashings.len() > config.max_attester_slashings() as usize {
			return Err(Error::TooManyAttesterSlashings)
		}
		for slashing in &block.body.attester_slashings {
			executive.process_attester_slashing(slashing.clone())?;
		}

		if block.body.attestations.len() > config.max_attestations() as usize {
			return Err(Error::TooManyAttestations)
		}
		for attestation in &block.body.attestations {
			executive.process_attestation(attestation.clone())?;
		}

		if block.body.deposits.len() > config.max_deposits() as usize {
			return Err(Error::TooManyDeposits)
		}
		for deposit in &block.body.deposits {
			executive.process_deposit(deposit.clone())?;
		}

		if block.body.voluntary_exits.len() > config.max_voluntary_exits() as usize {
			return Err(Error::TooManyVoluntaryExits)
		}
		for voluntary_exit in &block.body.voluntary_exits {
			executive.process_voluntary_exit(voluntary_exit.clone())?;
		}

		if block.body.transfers.len() > config.max_transfers() as usize{
			return Err(Error::TooManyTransfers)
		}
		for transfer in &block.body.transfers {
			executive.process_transfer(transfer.clone())?;
		}
	}
}
