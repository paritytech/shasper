pub struct AttestationRecord {
	pub target_slot: Slot,
	pub target_slot_storage_root: Hash,
	pub source_epoch: Epoch,
	pub source_epoch_storage_root: Hash,
	pub target_epoch: Epoch,
	pub target_epoch_storage_root: Hash,
	pub validator_id: ValidatorId,
}
