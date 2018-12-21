use primitives::H256;
use rstd::prelude::Vec;
use keccak_hasher::KeccakHasher;
use super::{AttestationRecord, Block};
use consts;

#[derive(Clone, PartialEq, Eq, Default, SszEncode, SszDecode, SszHash)]
#[cfg_attr(feature = "std", derive(Debug))]
#[ssz_codec(sorted)]
pub struct SpecHeader {
	pub parent_hash: H256,
	pub slot_number: u64,
	pub randao_reveal: H256,
	pub attestations: Vec<AttestationRecord>,
	pub pow_chain_ref: H256,
	pub active_state_root: H256,
	pub crystallized_state_root: H256,
}

pub trait SpecBlockExt {
	fn header_spec_hash(&self, active_state_root: H256, crystallized_state_root: H256) -> H256;
}

impl SpecBlockExt for Block {
	fn header_spec_hash(&self, active_state_root: H256, crystallized_state_root: H256) -> H256 {
		let slot_number = self.extrinsics[consts::SLOT_POSITION as usize].clone().slot().expect("Invalid slot extrinsic");
		let randao_reveal = self.extrinsics[consts::RANDAO_REVEAL_POSITION as usize].clone().randao_reveal().expect("Invalid randao reveal extrinsic");
		let pow_chain_ref = self.extrinsics[consts::POW_CHAIN_REF_POSITION as usize].clone().pow_chain_ref().expect("Invalid pow chain ref extrinsic");
		let attestations = (&self.extrinsics[consts::ATTESTATION_START_POSITION as usize..])
			.iter()
			.cloned()
			.map(|extrinsic| extrinsic.attestation().expect("Invalid attestation extrinsic"))
			.collect();

		let header = &self.header;

		let spec_header = SpecHeader {
			parent_hash: header.parent_hash,
			slot_number: slot_number,
			randao_reveal: randao_reveal,
			attestations: attestations,
			pow_chain_ref: pow_chain_ref,
			active_state_root: active_state_root,
			crystallized_state_root: crystallized_state_root,
		};

		ssz_hash::SpecHash::spec_hash::<KeccakHasher>(&spec_header)
	}
}
