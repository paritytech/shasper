use primitives::H256;
use rstd::prelude::Vec;
use super::AttestationRecord;

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
