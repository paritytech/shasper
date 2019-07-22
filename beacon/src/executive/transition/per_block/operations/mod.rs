mod proposer_slashing;
mod attester_slashing;
mod attestation;
mod deposit;
mod voluntary_exit;
mod transfer;

use crate::types::*;
use crate::{Config, BLSConfig, BeaconState, Error};
use core::cmp::min;

impl<C: Config> BeaconState<C> {
	/// Process block operations.
	pub fn process_operations<BLS: BLSConfig>(
		&mut self,
		body: &BeaconBlockBody<C>
	) -> Result<(), Error> {
		// Verify that outstanding deposits are processed up to the maximum
		// number of deposits
		if body.deposits.len() as u64 != min(
			C::max_deposits(),
			self.eth1_data.deposit_count -
				self.eth1_deposit_index
		) {
			return Err(Error::TooManyDeposits)
		}

		// Verify that there are no duplicate transfers
		if (1..body.transfers.len())
			.any(|i| body.transfers[i..].contains(&body.transfers[i - 1]))
		{
			return Err(Error::DuplicateTransfer)
		}

		if body.proposer_slashings.len() > C::max_proposer_slashings() as usize {
			return Err(Error::TooManyProposerSlashings)
		}
		for slashing in body.proposer_slashings.iter() {
			self.process_proposer_slashing::<BLS>(slashing.clone())?;
		}

		if body.attester_slashings.len() > C::max_attester_slashings() as usize {
			return Err(Error::TooManyAttesterSlashings)
		}
		for slashing in body.attester_slashings.iter() {
			self.process_attester_slashing::<BLS>(slashing.clone())?;
		}

		if body.attestations.len() > C::max_attestations() as usize {
			return Err(Error::TooManyAttestations)
		}
		for attestation in body.attestations.iter() {
			self.process_attestation::<BLS>(attestation.clone())?;
		}

		if body.deposits.len() > C::max_deposits() as usize {
			return Err(Error::TooManyDeposits)
		}
		for deposit in body.deposits.iter() {
			self.process_deposit::<BLS>(deposit.clone())?;
		}

		if body.voluntary_exits.len() > C::max_voluntary_exits() as usize {
			return Err(Error::TooManyVoluntaryExits)
		}
		for voluntary_exit in body.voluntary_exits.iter() {
			self.process_voluntary_exit::<BLS>(voluntary_exit.clone())?;
		}

		if body.transfers.len() > C::max_transfers() as usize{
			return Err(Error::TooManyTransfers)
		}
		for transfer in body.transfers.iter() {
			self.process_transfer::<BLS>(transfer.clone())?;
		}

		Ok(())
	}
}
