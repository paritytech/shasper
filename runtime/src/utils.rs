//! Useful utilities for runtime.

use primitives::{Slot, Epoch};
use crate::consts;

/// Convert a slot to epoch.
pub fn slot_to_epoch(slot: Slot) -> Epoch {
	slot / consts::CYCLE_LENGTH
}

/// Convert an epoch to slot.
pub fn epoch_to_slot(epoch: Epoch) -> Slot {
	epoch * consts::CYCLE_LENGTH
}
