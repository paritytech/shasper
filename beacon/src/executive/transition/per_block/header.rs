use crate::primitives::*;
use crate::types::*;
use crate::{Config, BeaconState, Error, BLSConfig};
use bm_le::tree_root;

impl<C: Config> BeaconState<C> {
	pub fn process_block_header<'a, B: Block, BLS: BLSConfig>(
		&'a mut self,
		block: &'a B
	) -> Result<(), Error> where
		UnsealedBeaconBlock<C>: From<&'a B>,
	{
		if block.slot() != self.slot {
			return Err(Error::BlockSlotInvalid)
		}

		if block.parent_root() != &tree_root::<C::Digest, _>(
			&SigningBeaconBlockHeader::from(self.latest_block_header.clone())
		) {
			return Err(Error::BlockPreviousRootInvalid)
		}

		self.latest_block_header = BeaconBlockHeader {
			slot: block.slot(),
			parent_root: *block.parent_root(),
			body_root: tree_root::<C::Digest, _>(block.body()),
			..Default::default()
		};

		let proposer = &self.validators[
			self.beacon_proposer_index()? as usize
		];
		if proposer.slashed {
			return Err(Error::BlockProposerSlashed)
		}

		if let Some(signature) = block.signature() {
			if !BLS::verify(
				&proposer.pubkey,
				&tree_root::<C::Digest, _>(&UnsealedBeaconBlock::from(block)),
				signature,
				self.domain(C::domain_beacon_proposer(), None)
			) {
				return Err(Error::BlockSignatureInvalid)
			}
		}

		Ok(())
	}
}
