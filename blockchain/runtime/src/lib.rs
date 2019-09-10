use blockchain::Block as BlockT;
use bm_le::tree_root;
use beacon::{
	BeaconState, Config,
	primitives::H256,
	types::{BeaconBlock, BeaconBlockHeader, SigningBeaconBlockHeader}
};

#[derive(Eq, PartialEq, Clone, Debug, parity_codec::Encode, parity_codec::Decode)]
pub struct Block<C: Config>(pub BeaconBlock<C>);

impl<C: Config> ssz::Codec for Block<C> {
	type Size = <BeaconBlock<C> as ssz::Codec>::Size;
}

impl<C: Config> ssz::Encode for Block<C> {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		ssz::Encode::using_encoded(&self.0, f)
	}
}

impl<C: Config> ssz::Decode for Block<C> {
	fn decode(value: &[u8]) -> Result<Self, ssz::Error> {
		Ok(Block(ssz::Decode::decode(value)?))
	}
}

impl<C: Config> BlockT for Block<C> {
	type Identifier = H256;

	fn id(&self) -> H256 {
		let header = BeaconBlockHeader {
			slot: self.0.slot,
			parent_root: self.0.parent_root,
			state_root: self.0.state_root,
			body_root: tree_root::<C::Digest, _>(&self.0.body),
			..Default::default()
		};

		tree_root::<C::Digest, _>(&SigningBeaconBlockHeader::from(header.clone()))
	}

	fn parent_id(&self) -> Option<H256> {
		if self.0.parent_root == H256::default() {
			None
		} else {
			Some(self.0.parent_root)
		}
	}
}

pub trait StateExternalities {
	type Config: Config;

	fn state(&mut self) -> &mut BeaconState<Self::Config>;
}
