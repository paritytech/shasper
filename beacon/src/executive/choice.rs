use crate::primitives::*;
use crate::types::*;
use crate::{Config, BeaconState, Error};

impl<C: Config> BeaconState<C> {
	/// Get justified active validators from current state.
	pub fn justified_active_validators(&self) -> Vec<u64> {
		let current_justified_epoch = self.current_justified_checkpoint.epoch;
		self.active_validator_indices(current_justified_epoch)
	}

	/// Get block attestation vote targets.
	pub fn block_vote_targets(&self, block: &BeaconBlock<C>) -> Result<Vec<(u64, H256)>, Error> {
		let mut ret = Vec::new();
		for attestation in block.body.attestations.iter() {
			let indexed = self.indexed_attestation(attestation.clone())?;

			for v in indexed.custody_bit_0_indices.iter().cloned()
				.chain(indexed.custody_bit_1_indices.iter().cloned())
			{
				ret.push((v, indexed.data.target.root));
			}
		}

		Ok(ret)
	}
}
