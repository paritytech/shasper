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

use primitives::{Slot, UncheckedAttestation};
use runtime_primitives::traits::{Extrinsic as ExtrinsicT};

use codec_derive::{Encode, Decode};
#[cfg(feature = "std")]
use serde_derive::{Serialize, Deserialize};

#[derive(Decode, Encode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum UncheckedExtrinsic {
	Slot(Slot),
	Attestation(UncheckedAttestation)
}

impl ExtrinsicT for UncheckedExtrinsic {
	fn is_signed(&self) -> Option<bool> {
		match self {
			UncheckedExtrinsic::Slot(_) => Some(false),
			UncheckedExtrinsic::Attestation(_) => Some(true),
		}
	}
}
