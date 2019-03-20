use rstd::prelude::*;
use primitives::{Slot, Epoch, Balance, ValidatorId, UncheckedAttestation, CheckedAttestation, AttestationContext, KeccakHasher};
use crypto::bls;
use runtime_support::storage::{StorageValue, StorageMap};
use runtime_support::storage::unhashed::StorageVec;
use codec::{Encode, Decode};
use casper::store::{ValidatorStore, PendingAttestationsStore, BlockStore};
use casper::randao::RandaoCommitment;
use crate::{storage, utils};

#[derive(Encode, Decode, PartialEq, Eq, Clone)]
pub struct ValidatorRecord {
	pub valid_from: Epoch,
	pub valid_to: Epoch,
	pub balance: Balance,
	pub validator_id: ValidatorId,
	pub randao_commitment: RandaoCommitment<KeccakHasher>,
	pub randao_last_reveal_slot: Slot,
}

pub struct Store;

impl ValidatorStore<AttestationContext> for Store {
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

impl BlockStore<AttestationContext> for Store {
	fn epoch(&self) -> Epoch {
		let current_slot = storage::LastSlot::get();
		utils::slot_to_epoch(current_slot)
	}
}

impl PendingAttestationsStore<AttestationContext> for Store {
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

pub fn check_attestation(unchecked: UncheckedAttestation, check_slot: bool) -> Option<CheckedAttestation> {
	let mut signature = bls::AggregateSignature::new();
	signature.add(&unchecked.signature.into_signature()?);
	let validator_ids = {
		let mut ret = Vec::new();
		for validator_index in &unchecked.data.validator_indexes {
			ret.push(storage::Validators::item(*validator_index)?.validator_id);
		}
		ret
	};
	let publics = {
		let mut ret = Vec::new();
		for validator_id in &validator_ids {
			ret.push(validator_id.into_public()?);
		}
		ret
	};
	let current_slot = storage::Slot::get();
	let aggregated_public = {
		let mut ret = bls::AggregatePublic::new();
		for public in publics {
			ret.add(&public);
		}
		ret
	};

	if !signature.verify(&unchecked.data.encode()[..], 0, &aggregated_public) {
		return None;
	}

	if check_slot && unchecked.data.slot >= current_slot {
		return None;
	}

	let is_slot_canon = storage::LatestBlockHashes::get(unchecked.data.slot) == Some(unchecked.data.slot_block_hash);
	let is_source_canon = storage::LatestBlockHashes::get(utils::epoch_to_slot(unchecked.data.source_epoch)) == Some(unchecked.data.source_epoch_block_hash);
	let is_target_canon = storage::LatestBlockHashes::get(utils::epoch_to_slot(unchecked.data.target_epoch)) == Some(unchecked.data.target_epoch_block_hash);
	let inclusion_distance = if current_slot >= unchecked.data.slot { current_slot - unchecked.data.slot } else { 0 };

	Some(CheckedAttestation {
		data: unchecked.data,
		is_slot_canon,
		is_source_canon,
		is_target_canon,
		validator_ids,
		inclusion_distance,
	})
}
