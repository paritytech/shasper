use ssz::Hashable;

use super::Executive;
use crate::Config;
use crate::primitives::H256;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Update state cache.
	pub fn update_cache(&mut self) {
		let previous_slot_state_root = Hashable::<C::Hasher>::hash(self.state);

		self.state.latest_state_roots[(self.state.slot % self.config.slots_per_historical_root() as u64) as usize] = previous_slot_state_root;

		if self.state.latest_block_header.state_root == H256::default() {
			self.state.latest_block_header.state_root = previous_slot_state_root;
		}

		self.state.latest_block_roots[(self.state.slot % self.config.slots_per_historical_root() as u64) as usize] = Hashable::<C::Hasher>::truncated_hash(&self.state.latest_block_header);
	}
}
