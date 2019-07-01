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

use core::cmp::min;
use ssz::Digestible;
use crate::consts;
use crate::primitives::H256;
use crate::types::{Deposit, Validator};
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Push a new `Deposit` to the state.
	pub fn process_deposit(&mut self, deposit: Deposit) -> Result<(), Error> {
		if !self.is_valid_merkle_branch(
			H256::from_slice(
				Digestible::<C::Digest>::hash(&deposit.data).as_slice()
			),
			&deposit.proof,
			consts::DEPOSIT_CONTRACT_TREE_DEPTH,
			self.state.eth1_deposit_index,
			self.state.eth1_data.deposit_root,
		) {
			return Err(Error::DepositMerkleInvalid)
		}

		self.state.eth1_deposit_index += 1;

		let pubkey = deposit.data.pubkey.clone();
		let amount = deposit.data.amount.clone();
		let validator_pubkeys = self.state.validators.iter()
			.map(|v| v.pubkey.clone()).collect::<Vec<_>>();

		if !validator_pubkeys.contains(&pubkey) {
			// Verify the deposit signature (proof of possession). Invalid
			// signatures are allowed by the deposit contract, and hence
			// included on-chain, but must not be processed.
			if !self.config.bls_verify(
				&pubkey,
				&H256::from_slice(
					Digestible::<C::Digest>::truncated_hash(
						&deposit.data
					).as_slice()
				),
				&deposit.data.signature,
				self.domain(self.config.domain_deposit(), None)
			) {
				return Ok(())
			}

			let validator = Validator {
				pubkey,
				withdrawal_credentials: deposit.data.withdrawal_credentials,
				activation_eligibility_epoch: consts::FAR_FUTURE_EPOCH,
				activation_epoch: consts::FAR_FUTURE_EPOCH,
				exit_epoch: consts::FAR_FUTURE_EPOCH,
				withdrawable_epoch: consts::FAR_FUTURE_EPOCH,
				effective_balance: min(
					amount - amount % self.config.effective_balance_increment(),
					self.config.max_effective_balance(),
				),
				slashed: false,
			};
			self.state.validators.push(validator);
			self.state.balances.push(amount);
		} else {
			let index = validator_pubkeys.iter().position(|v| v == &pubkey)
				.expect("Registry contains the public key");
			self.increase_balance(index as u64, amount);
		}

		Ok(())
	}

}
