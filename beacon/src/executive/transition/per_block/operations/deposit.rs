use crate::primitives::*;
use crate::types::*;
use crate::{Config, BeaconState, Error, BLSConfig, utils, consts};
use bm_le::{tree_root, MaxVec};
use core::cmp::min;

impl<C: Config> BeaconState<C> {
	/// Push a new `Deposit` to the state.
	pub fn process_deposit<BLS: BLSConfig>(&mut self, deposit: Deposit) -> Result<(), Error> {
		if !utils::is_valid_merkle_branch::<C>(
			tree_root::<C::Digest, _>(&deposit.data),
			&deposit.proof,
			consts::DEPOSIT_CONTRACT_TREE_DEPTH + 1,
			self.eth1_deposit_index,
			self.eth1_data.deposit_root,
		) {
			return Err(Error::DepositMerkleInvalid)
		}

		self.eth1_deposit_index += 1;

		let pubkey = deposit.data.pubkey.clone();
		let amount = deposit.data.amount.clone();
		let validator_pubkeys = self.validators.iter()
			.map(|v| v.pubkey.clone()).collect::<Vec<_>>();

		if !validator_pubkeys.contains(&pubkey) {
			// Verify the deposit signature (proof of possession). Invalid
			// signatures are allowed by the deposit contract, and hence
			// included on-chain, but must not be processed.
			if !BLS::verify(
				&pubkey,
				&tree_root::<C::Digest, _>(&SigningDepositData::from(deposit.data.clone())),
				&deposit.data.signature,
				self.domain(C::domain_deposit(), None)
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
					amount - amount % C::effective_balance_increment(),
					C::max_effective_balance(),
				),
				slashed: false,
			};
			self.validators.push(validator);
			self.balances.push(amount);
		} else {
			let index = validator_pubkeys.iter().position(|v| v == &pubkey)
				.expect("Registry contains the public key");
			self.increase_balance(index as u64, amount);
		}

		Ok(())
	}
}
