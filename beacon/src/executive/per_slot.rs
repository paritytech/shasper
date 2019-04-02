use super::Executive;
use crate::Config;

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub fn advance_slot(&mut self) {
		self.state.slot += 1;
	}
}
