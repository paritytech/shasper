mod header;
mod randao;
mod eth1;
mod operations;

use crate::types::*;
use crate::{Config, BLSConfig, BeaconState, Error};

impl<C: Config> BeaconState<C> {
	/// Process a block, assuming we are at given slot.
	pub fn process_block<'a, 'b, B: Block<Config=C>, BLS: BLSConfig>(
		&'a mut self,
		block: &'b B,
	) -> Result<(), Error> where
		UnsealedBeaconBlock<C>: From<&'b B>,
	{
		self.process_block_header::<_, BLS>(block)?;
		self.process_randao::<BLS>(block.body())?;
		self.process_eth1_data(block.body());
		self.process_operations::<BLS>(block.body())?;

		Ok(())
	}
}
