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

use core::cmp::{min, max};
use ssz::Digestible;
use crate::primitives::{Uint, Epoch, Slot, ValidatorIndex, Gwei, Shard, H256, BitField};
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit, Block, BeaconBlockHeader, ProposerSlashing, AttesterSlashing, Transfer};
use crate::utils::to_bytes;
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Push a new `Transfer` to the state.
	pub fn process_transfer(&mut self, transfer: Transfer) -> Result<(), Error> {
		// Verify the amount and fee are not individually too big
		// (for anti-overflow purposes)
		if self.state.balances[transfer.sender as usize] < core::cmp::max(transfer.amount, transfer.fee) {
			return Err(Error::TransferNoFund)
		}

		// A transfer is valid in only one slot
		if self.state.slot != transfer.slot {
			return Err(Error::TransferNotValidSlot)
		}

		// Sender must be not yet eligible for activation, withdrawn,
		// or transfer balance over MAX_EFFECTIVE_BALANCE
		if !(self.state.validator_registry[transfer.sender as usize]
			 .activation_eligibility_epoch == self.config.far_future_epoch() ||
			 self.current_epoch() >=
			 self.state.validator_registry[transfer.sender as usize].withdrawable_epoch ||
			 transfer.amount + transfer.fee + self.config.max_effective_balance() <
			 self.state.balances[transfer.sender as usize])
		{
			return Err(Error::TransferNoFund)
		}

		// Verify that the pubkey is valid
		if !(self.state.validator_registry[transfer.sender as usize]
			 .withdrawal_credentials[0] == self.config.bls_withdrawal_prefix_byte() &&
			 &self.state.validator_registry[transfer.sender as usize]
			 .withdrawal_credentials[1..] ==
			 &self.config.hash(&[&transfer.pubkey[..]])[1..])
		{
			return Err(Error::TransferInvalidPublicKey)
		}

		// Verify that the signature is valid
		if !self.config.bls_verify(
			&transfer.pubkey,
			&H256::from_slice(
				Digestible::<C::Digest>::truncated_hash(&transfer).as_slice()
			),
			&transfer.signature,
			self.domain(self.config.domain_transfer(), None),
		) {
			return Err(Error::TransferInvalidSignature)
		}

		// Process the transfer
		self.decrease_balance(transfer.sender, transfer.amount + transfer.fee);
		self.increase_balance(transfer.recipient, transfer.amount);
		self.increase_balance(self.beacon_proposer_index()?, transfer.fee);

		// Verify balances are not dust
		if 0 < self.state.balances[transfer.sender as usize] &&
			self.state.balances[transfer.sender as usize] <
			self.config.min_deposit_amount()
		{
			return Err(Error::TransferNoFund)
		}

		if 0 < self.state.balances[transfer.recipient as usize] &&
			self.state.balances[transfer.recipient as usize] <
			self.config.min_deposit_amount()
		{
			return Err(Error::TransferNoFund)
		}

		Ok(())
	}
}
