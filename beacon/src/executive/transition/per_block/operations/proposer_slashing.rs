use crate::primitives::*;
use crate::types::*;
use crate::{Config, BeaconState, Error, utils};
use bm_le::tree_root;

impl<C: Config> BeaconState<C> {
	/// Push a new `ProposerSlashing` to the state.
	pub fn process_proposer_slashing(
		&mut self,
		proposer_slashing: ProposerSlashing
	) -> Result<(), Error> {
		if utils::epoch_of_slot::<C>(proposer_slashing.header_1.slot) !=
			utils::epoch_of_slot::<C>(proposer_slashing.header_2.slot)
		{
			return Err(Error::ProposerSlashingInvalidSlot)
		}

		if proposer_slashing.header_1 == proposer_slashing.header_2 {
			return Err(Error::ProposerSlashingSameHeader)
		}

		{
			if proposer_slashing.proposer_index as usize >= self.validators.len() {
				return Err(Error::ProposerSlashingInvalidProposerIndex)
			}

			let proposer = &self.validators[
				proposer_slashing.proposer_index as usize
			];

			if !proposer.is_slashable(self.current_epoch()) {
				return Err(Error::ProposerSlashingAlreadySlashed)
			}

			for header in &[&proposer_slashing.header_1, &proposer_slashing.header_2] {
				let domain = self.domain(
					C::domain_beacon_proposer(),
					Some(utils::epoch_of_slot::<C>(header.slot))
				);

				if !C::bls_verify(
					&proposer.pubkey,
					&tree_root::<C::Digest, _>(&SigningBeaconBlockHeader::from((*header).clone())),
					&header.signature,
					domain,
				) {
					return Err(Error::ProposerSlashingInvalidSignature)
				}
			}
		}

		self.slash_validator(proposer_slashing.proposer_index, None)
	}
}
