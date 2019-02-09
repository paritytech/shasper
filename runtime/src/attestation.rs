use rstd::prelude::*;
use runtime_support::storage::StorageValue;
use runtime_support::storage::unhashed::StorageVec;
use codec::Encode;
use codec_derive::{Encode, Decode};
use primitives::{Slot, Hash, Epoch, ValidatorId, Signature};
use casper::{Attestation, BeaconAttestation};
use crate::{storage, utils};

#[derive(Eq, PartialEq, Clone, Encode, Decode)]
pub struct UnsignedAttestation {
	pub slot: Slot,
	pub slot_storage_root: Hash,
	pub source_epoch: Epoch,
	pub source_epoch_storage_root: Hash,
	pub target_epoch: Epoch,
	pub target_epoch_storage_root: Hash,
	pub validator_index: u32,
}

#[derive(Eq, PartialEq, Clone, Encode, Decode)]
pub struct UncheckedAttestation {
	pub data: UnsignedAttestation,
	pub signature: Signature,
}

pub fn check_attestation(unchecked: UncheckedAttestation) -> Option<CheckedAttestation> {
	let signature = unchecked.signature.into_signature()?;
	let validator_id = storage::Validators::item(unchecked.data.validator_index)?.validator_id;
	let public = validator_id.into_public()?;
	let current_slot = storage::Number::get();

	if !public.verify(&unchecked.data.encode()[..], &signature) {
		return None;
	}

	if current_slot >= unchecked.data.slot {
		return None;
	}

	let is_slot_canon = storage::LatestStorageRoots::item(unchecked.data.slot as u32) == Some(unchecked.data.slot_storage_root);
	let is_source_canon = storage::LatestStorageRoots::item(utils::epoch_to_slot(unchecked.data.source_epoch) as u32) == Some(unchecked.data.source_epoch_storage_root);
	let is_target_canon = storage::LatestStorageRoots::item(utils::epoch_to_slot(unchecked.data.target_epoch) as u32) == Some(unchecked.data.target_epoch_storage_root);
	let inclusion_distance = current_slot - unchecked.data.slot;

	Some(CheckedAttestation {
		slot: unchecked.data.slot,
		is_slot_canon,
		source_epoch: unchecked.data.source_epoch,
		is_source_canon,
		target_epoch: unchecked.data.target_epoch,
		is_target_canon,
		validator_id,
		inclusion_distance,
	})
}

#[derive(Eq, PartialEq, Clone, Encode, Decode)]
pub struct CheckedAttestation {
	pub slot: Slot,
	pub is_slot_canon: bool,
	pub source_epoch: Epoch,
	pub is_source_canon: bool,
	pub target_epoch: Epoch,
	pub is_target_canon: bool,
	pub validator_id: ValidatorId,
	pub inclusion_distance: Slot,
}

impl Attestation for CheckedAttestation {
	type ValidatorId = ValidatorId;
	type ValidatorIdIterator = Vec<ValidatorId>;
	type Epoch = Epoch;

	fn validator_ids(&self) -> Vec<ValidatorId> {
		vec![self.validator_id]
	}

	fn is_source_canon(&self) -> bool {
		self.is_source_canon
	}

	fn is_target_canon(&self) -> bool {
		self.is_target_canon
	}

	fn source_epoch(&self) -> Epoch {
		self.source_epoch
	}

	fn target_epoch(&self) -> Epoch {
		self.target_epoch
	}
}

impl BeaconAttestation for CheckedAttestation {
	type Slot = Slot;

	fn slot(&self) -> Slot {
		self.slot
	}

	fn is_slot_canon(&self) -> bool {
		self.is_slot_canon
	}

	fn inclusion_distance(&self) -> Slot {
		self.inclusion_distance
	}
}
