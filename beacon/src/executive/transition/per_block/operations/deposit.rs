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
use crate::types::{Attestation, AttestationData, IndexedAttestation, AttestationDataAndCustodyBit, Block, BeaconBlockHeader, ProposerSlashing, AttesterSlashing, PendingAttestation, Validator, Deposit};
use crate::utils::to_bytes;
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Push a new `Deposit` to the state.
	pub fn process_deposit(&mut self, deposit: Deposit) -> Result<(), Error> {
		if !self.config.verify_merkle_branch(
			H256::from_slice(
				Digestible::<C::Digest>::hash(&deposit.data).as_slice()
			),
			&deposit.proof,
			self.config.deposit_contract_tree_depth(),
			deposit.index,
			self.state.latest_eth1_data.deposit_root,
		) {
			return Err(Error::DepositMerkleInvalid)
		}

		if deposit.index != self.state.deposit_index {
			return Err(Error::DepositIndexMismatch)
		}
		self.state.deposit_index += 1;

		let pubkey = deposit.data.pubkey.clone();
		let amount = deposit.data.amount.clone();
		let validator_pubkeys = self.state.validator_registry.iter()
			.map(|v| v.pubkey.clone()).collect::<Vec<_>>();

		if !validator_pubkeys.contains(&pubkey) {
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
				activation_eligibility_epoch: self.config.far_future_epoch(),
				activation_epoch: self.config.far_future_epoch(),
				exit_epoch: self.config.far_future_epoch(),
				withdrawable_epoch: self.config.far_future_epoch(),
				effective_balance: min(
					amount - amount % self.config.effective_balance_increment(),
					self.config.max_effective_balance(),
				),
				slashed: false,
			};
			self.state.validator_registry.push(validator);
			self.state.balances.push(amount);
		} else {
			let index = validator_pubkeys.iter().position(|v| v == &pubkey)
				.expect("Registry contains the public key");
			self.increase_balance(index as u64, amount);
		}

		Ok(())
	}

}
