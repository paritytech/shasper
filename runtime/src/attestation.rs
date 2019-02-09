use rstd::prelude::*;
use primitives::{Slot, Hash, Epoch, ValidatorId};
use casper::Attestation;

#[derive(Eq, PartialEq, Clone)]
pub struct AttestationRecord {
	pub target_slot: Slot,
	pub target_slot_storage_root: Hash,
	pub source_epoch: Epoch,
	pub source_epoch_storage_root: Hash,
	pub target_epoch: Epoch,
	pub target_epoch_storage_root: Hash,
	pub validator_id: ValidatorId,
}

impl Attestation for AttestationRecord {
	type ValidatorId = ValidatorId;
	type ValidatorIdIterator = Vec<ValidatorId>;
	type Epoch = Epoch;

	fn validator_ids(&self) -> Vec<ValidatorId> {
		vec![self.validator_id]
	}

	fn is_source_canon(&self) -> bool {
		unimplemented!()
	}

	fn is_target_canon(&self) -> bool {
		unimplemented!()
	}

	fn source_epoch(&self) -> Epoch {
		self.source_epoch
	}

	fn target_epoch(&self) -> Epoch {
		self.target_epoch
	}
}
