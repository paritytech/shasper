use primitives::{Slot, Epoch};
use crate::consts;

pub fn slot_to_epoch(slot: Slot) -> Epoch {
	slot / consts::CYCLE_LENGTH
}

pub fn epoch_to_slot(epoch: Epoch) -> Slot {
	epoch * consts::CYCLE_LENGTH
}
