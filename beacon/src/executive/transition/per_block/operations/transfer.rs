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
use crate::{Config, BeaconState, Error, BLSConfig, consts};
use bm_le::tree_root;
use core::cmp::max;

impl<C: Config> BeaconState<C> {
	/// Push a new `Transfer` to the state.
	pub fn process_transfer<BLS: BLSConfig>(&mut self, transfer: Transfer) -> Result<(), Error> {
		if !(self.validators.len() > transfer.sender as usize &&
			 self.validators.len() > transfer.recipient as usize)
		{
			return Err(Error::TransferInvalidPublicKey)
		}

		// Verify the amount and fee are not individually too big
		// (for anti-overflow purposes)
		if self.balances[transfer.sender as usize] < max(transfer.amount + transfer.fee, max(transfer.amount, transfer.fee)) {
			return Err(Error::TransferNoFund)
		}

		// A transfer is valid in only one slot
		if self.slot != transfer.slot {
			return Err(Error::TransferNotValidSlot)
		}

		// Sender must be not yet eligible for activation, withdrawn,
		// or transfer balance over MAX_EFFECTIVE_BALANCE
		if !(self.validators[transfer.sender as usize]
			 .activation_eligibility_epoch == consts::FAR_FUTURE_EPOCH ||
			 self.current_epoch() >=
			 self.validators[transfer.sender as usize].withdrawable_epoch ||
			 transfer.amount + transfer.fee + C::max_effective_balance() <=
			 self.balances[transfer.sender as usize])
		{
			return Err(Error::TransferNoFund)
		}

		// Verify that the pubkey is valid
		if !(self.validators[transfer.sender as usize]
			 .withdrawal_credentials[0] == C::bls_withdrawal_prefix_byte() &&
			 &self.validators[transfer.sender as usize]
			 .withdrawal_credentials[1..] ==
			 &C::hash(&[&transfer.pubkey[..]])[1..])
		{
			return Err(Error::TransferInvalidPublicKey)
		}

		// Verify that the signature is valid
		if !BLS::verify(
			&transfer.pubkey,
			&tree_root::<C::Digest, _>(&SigningTransfer::from(transfer.clone())),
			&transfer.signature,
			self.domain(C::domain_transfer(), None),
		) {
			return Err(Error::TransferInvalidSignature)
		}

		// Process the transfer
		self.decrease_balance(transfer.sender, transfer.amount + transfer.fee);
		self.increase_balance(transfer.recipient, transfer.amount);
		self.increase_balance(self.beacon_proposer_index()?, transfer.fee);

		// Verify balances are not dust
		if 0 < self.balances[transfer.sender as usize] &&
			self.balances[transfer.sender as usize] <
			C::min_deposit_amount()
		{
			return Err(Error::TransferNoFund)
		}

		if 0 < self.balances[transfer.recipient as usize] &&
			self.balances[transfer.recipient as usize] <
			C::min_deposit_amount()
		{
			return Err(Error::TransferNoFund)
		}

		Ok(())
	}
}
