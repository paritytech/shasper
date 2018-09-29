use primitives::H256;
use rstd::prelude::*;

use attestation::AttestationRecord;

#[derive(Encode, Decode)]
pub enum Extrinsic {
	SlotNumber(u64),
	RandaoReveal(H256),
	PoWChainRef(H256),
	Attestations(Vec<AttestationRecord>),
}

impl Extrinsic {
	pub fn slot_number(&self) -> Option<u64> {
		match &self {
			&Extrinsic::SlotNumber(v) => Some(*v),
			_ => None,
		}
	}

	pub fn randao_reveal(&self) -> Option<H256> {
		match &self {
			&Extrinsic::RandaoReveal(v) => Some(*v),
			_ => None,
		}
	}

	pub fn pow_chain_ref(&self) -> Option<H256> {
		match &self {
			&Extrinsic::PoWChainRef(v) => Some(*v),
			_ => None,
		}
	}

	pub fn attestations(&self) -> Option<&[AttestationRecord]> {
		match &self {
			&Extrinsic::Attestations(ref v) => Some(v),
			_ => None,
		}
	}
}
