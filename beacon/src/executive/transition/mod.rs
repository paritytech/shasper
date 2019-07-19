mod per_block;
mod per_epoch;

use crate::primitives::*;
use crate::types::*;
use crate::{Error, Config, BeaconState, BLSConfig};
use bm_le::tree_root;

impl<C: Config> BeaconState<C> {
	/// Execute state transition.
	pub fn state_transition<B: Block<Config=C>, BLS: BLSConfig>(
		&mut self,
		block: &B,
	) -> Result<(), Error> {
		self.process_slots(block.slot())?;
		self.process_block::<_, BLS>(block)?;

		if !(block.state_root() == &tree_root::<C::Digest, _>(self)) {
			return Err(Error::BlockStateRootInvalid)
		}

		Ok(())
	}

	/// Process slots, process epoch if at epoch boundary.
	pub fn process_slots(&mut self, slot: Uint) -> Result<(), Error> {
		if self.slot > slot {
			return Err(Error::SlotOutOfRange)
		}

		while self.slot < slot {
			self.process_slot();
			if (self.slot + 1) % C::slots_per_epoch() == 0 {
				self.process_epoch()?;
			}
			self.slot += 1;
		}

		Ok(())
	}

	/// Advance slot
	pub fn process_slot(&mut self) {
		let previous_state_root = tree_root::<C::Digest, _>(self);
		self.state_roots[
			(self.slot % C::slots_per_historical_root()) as usize
		] = previous_state_root;

		if self.latest_block_header.state_root == H256::default() {
			self.latest_block_header.state_root = previous_state_root;
		}

		let previous_block_root = tree_root::<C::Digest, _>(
			&SigningBeaconBlockHeader::from(self.latest_block_header.clone())
		);
		self.block_roots[
			(self.slot % C::slots_per_historical_root()) as usize
		] = previous_block_root;
	}
}
