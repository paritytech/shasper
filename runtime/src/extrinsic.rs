// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use primitives::H256;
use runtime_primitives::traits::{Extrinsic as ExtrinsicT};
use crate::AttestationRecord;

use codec_derive::{Encode, Decode};

#[derive(Decode, Encode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum UncheckedExtrinsic {
	Timestamp(u64),
	Slot(u64),
	RandaoReveal(H256),
	PowChainRef(H256),
	Attestation(AttestationRecord)
}

impl UncheckedExtrinsic {
	pub fn timestamp(self) -> Option<u64> {
		match self {
			UncheckedExtrinsic::Timestamp(v) => Some(v),
			_ => None,
		}
	}

	pub fn slot(self) -> Option<u64> {
		match self {
			UncheckedExtrinsic::Slot(v) => Some(v),
			_ => None,
		}
	}

	pub fn randao_reveal(self) -> Option<H256> {
		match self {
			UncheckedExtrinsic::RandaoReveal(v) => Some(v),
			_ => None,
		}
	}

	pub fn pow_chain_ref(self) -> Option<H256> {
		match self {
			UncheckedExtrinsic::PowChainRef(v) => Some(v),
			_ => None,
		}
	}

	pub fn attestation(self) -> Option<AttestationRecord> {
		match self {
			UncheckedExtrinsic::Attestation(v) => Some(v),
			_ => None,
		}
	}
}

impl Default for UncheckedExtrinsic {
	fn default() -> UncheckedExtrinsic {
		UncheckedExtrinsic::Attestation(AttestationRecord::default())
	}
}

impl ExtrinsicT for UncheckedExtrinsic {
	fn is_signed(&self) -> Option<bool> {
		match self {
			UncheckedExtrinsic::Timestamp(_) => Some(false),
			UncheckedExtrinsic::Slot(_) => Some(false),
			UncheckedExtrinsic::RandaoReveal(_) => Some(false),
			UncheckedExtrinsic::PowChainRef(_) => Some(false),
			UncheckedExtrinsic::Attestation(_) => None,
		}
	}
}
