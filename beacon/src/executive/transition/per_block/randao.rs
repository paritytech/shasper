use crate::types::*;
use crate::{Config, BeaconState, Error, BLSConfig};
use bm_le::tree_root;

impl<C: Config> BeaconState<C> {
	/// Process randao information given in a block.
	pub fn process_randao<BLS: BLSConfig>(&mut self, body: &BeaconBlockBody<C>) -> Result<(), Error> {
		let proposer = &self.validators[
			self.beacon_proposer_index()? as usize
		];

		if !BLS::verify(
			&proposer.pubkey,
			&tree_root::<C::Digest, _>(&self.current_epoch()),
			&body.randao_reveal,
			self.domain(C::domain_randao(), None)
		) {
			return Err(Error::RandaoSignatureInvalid)
		}

		let current_epoch = self.current_epoch();
		self.randao_mixes[
			(current_epoch % C::epochs_per_historical_vector()) as usize
		] = self.randao_mix(current_epoch) ^
			C::hash(&[&body.randao_reveal[..]]);

		Ok(())
	}
}
