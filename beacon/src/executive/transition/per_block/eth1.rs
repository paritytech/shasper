use crate::types::*;
use crate::{Config, BeaconState};

impl<C: Config> BeaconState<C> {
	/// Process eth1 data vote given in a block.
	pub fn process_eth1_data(&mut self, body: &BeaconBlockBody<C>) {
		self.eth1_data_votes.push(body.eth1_data.clone());
		if self.eth1_data_votes.iter()
			.filter(|d| d == &&body.eth1_data)
			.count() * 2 >
			C::slots_per_eth1_voting_period() as usize
		{
			self.eth1_data = body.eth1_data.clone();
		}
	}
}
