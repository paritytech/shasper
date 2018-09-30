use primitives::H256;
use runtime_primitives;

use header::Header;
use extrinsic::Extrinsic;
use spec::SpecHeader;

pub type Block = runtime_primitives::generic::Block<Header, Extrinsic>;
pub type BlockId = runtime_primitives::generic::BlockId<Block>;

pub trait BlockExt {
	fn spec_hash(&self, active_state_root: H256, crystallized_state_root: H256) -> H256;
}

impl BlockExt for Block {
	fn spec_hash(&self, active_state_root: H256, crystallized_state_root: H256) -> H256 {
		let extrinsic = &self.extrinsics[0];
		let header = &self.header;

		let spec_header = SpecHeader {
			parent_hash: header.parent_hash,
			slot_number: extrinsic.slot_number,
			randao_reveal: extrinsic.randao_reveal,
			attestations: extrinsic.attestations.clone(),
			pow_chain_ref: extrinsic.pow_chain_ref,
			active_state_root: active_state_root,
			crystallized_state_root: crystallized_state_root,
		};

		spec_header.spec_hash()
	}
}
