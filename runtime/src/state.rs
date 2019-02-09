use rstd::prelude::*;
use primitives::{Epoch, Balance, ValidatorId};
use runtime_support::storage::StorageValue;
use runtime_support::storage::unhashed::StorageVec;
use codec_derive::{Encode, Decode};
use casper::store::{ValidatorStore, PendingAttestationsStore, BlockStore};
use crate::attestation::CheckedAttestation;
use crate::{storage, utils};

#[derive(Encode, Decode, PartialEq, Eq, Clone)]
pub struct ValidatorRecord {
	pub valid_from: Epoch,
	pub valid_to: Epoch,
	pub balance: Balance,
	pub validator_id: ValidatorId,
}

pub struct Store;

impl ValidatorStore for Store {
	type ValidatorId = ValidatorId;
	type ValidatorIdIterator = Vec<ValidatorId>;
	type Balance = Balance;
	type Epoch = Epoch;

	fn total_balance(&self, validators: &[ValidatorId]) -> Balance {
		let mut total_balance = 0;

		for validator_id in validators {
			if let Some(Some(validator)) = storage::Validators::items().iter().find(|v| v.as_ref().map(|v| &v.validator_id == validator_id).unwrap_or(false)) {
				total_balance += validator.balance;
			}
		}

		total_balance
	}

	fn active_validators(&self, epoch: Epoch) -> Vec<ValidatorId> {
		let mut ret = Vec::new();

		for validator in storage::Validators::items() {
			if let Some(validator) = validator {
				if validator.valid_from <= epoch && epoch <= validator.valid_to {
					ret.push(validator.validator_id);
				}
			}
		}

		ret
	}
}

impl BlockStore for Store {
	type Epoch = Epoch;

	fn epoch(&self) -> Epoch {
		let current_slot = storage::Number::get();
		utils::slot_to_epoch(current_slot)
	}
}

impl PendingAttestationsStore for Store {
	type Attestation = CheckedAttestation;
	type AttestationIterator = Vec<CheckedAttestation>;

	fn attestations(&self) -> Vec<CheckedAttestation> {
		let mut attestations = Vec::new();

		for attestation in storage::PendingAttestations::items() {
			if let Some(attestation) = attestation {
				attestations.push(attestation);
			}
		}

		attestations
	}

	fn retain<F: FnMut(&CheckedAttestation) -> bool>(&mut self, f: F) {
		let mut attestations = self.attestations();
		attestations.retain(f);

		storage::PendingAttestations::set_items(attestations.into_iter().map(|v| Some(v)).collect::<Vec<_>>());
	}
}
