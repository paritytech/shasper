use rstd::prelude::*;
use codec_derive::{Encode, Decode};
#[cfg(feature = "std")]
use serde_derive::{Serialize, Deserialize};
use casper::{Attestation, BeaconAttestation};
use crate::{Slot, Hash, Epoch, ValidatorId, Signature};

#[derive(Eq, PartialEq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct UnsignedAttestation {
	pub slot: Slot,
	pub slot_block_hash: Hash,
	pub source_epoch: Epoch,
	pub source_epoch_block_hash: Hash,
	pub target_epoch: Epoch,
	pub target_epoch_block_hash: Hash,
	pub validator_index: u32,
}

#[derive(Eq, PartialEq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct UncheckedAttestation {
	pub data: UnsignedAttestation,
	pub signature: Signature,
}

#[derive(Eq, PartialEq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct CheckedAttestation {
	pub data: UnsignedAttestation,
	pub is_slot_canon: bool,
	pub is_source_canon: bool,
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
		self.data.source_epoch
	}

	fn target_epoch(&self) -> Epoch {
		self.data.target_epoch
	}
}

impl BeaconAttestation for CheckedAttestation {
	type Slot = Slot;

	fn slot(&self) -> Slot {
		self.data.slot
	}

	fn is_slot_canon(&self) -> bool {
		self.is_slot_canon
	}

	fn inclusion_distance(&self) -> Slot {
		self.inclusion_distance
	}
}
