use primitives::H256;
use rstd::prelude::*;

use attestation::AttestationRecord;
use spec::SpecHeader;

#[derive(Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct Extrinsic {
	pub slot_number: u64,
	pub randao_reveal: H256,
	pub pow_chain_ref: H256,
	pub attestations: Vec<AttestationRecord>,
}

impl Extrinsic {
	pub fn spec_hash(&self, parent_hash: H256, active_state_root: H256, crystallized_state_root: H256) -> H256 {
		let spec_header = SpecHeader {
			parent_hash: parent_hash,
			slot_number: self.slot_number,
			randao_reveal: self.randao_reveal,
			attestations: self.attestations.clone(),
			pow_chain_ref: self.pow_chain_ref,
			active_state_root: active_state_root,
			crystallized_state_root: crystallized_state_root,
		};

		spec_header.spec_hash()
	}
}
