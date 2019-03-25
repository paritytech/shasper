use crate::{BeaconState, Config};

mod cache;
mod per_block;
mod per_epoch;
mod per_slot;

pub struct Executive<'state, 'config, C: Config> {
	state: &'state mut BeaconState,
	config: &'config C,
}

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub fn new(state: &'state mut BeaconState, config: &'config C) -> Self {
		Self { state, config }
	}
}
