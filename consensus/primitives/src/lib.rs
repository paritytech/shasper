// Copyright 2017-2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

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

//! Primitives for Aura.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate parity_codec as codec;
extern crate parity_codec_derive as codec_derive;
extern crate substrate_client as client;

use codec_derive::{Encode, Decode};
use inherents::InherentIdentifier;

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"shasper0";

/// Consensus inherent data
#[derive(Encode, Decode)]
pub struct InherentData {
	pub timestamp: u64,
	pub slot: primitives::Slot,
}

/// The ApiIds for Aura authorship API.
pub mod id {
	use client::runtime_api::ApiId;

	/// ApiId for the AuraApi trait.
	pub const AURA_API: ApiId = *b"aura_api";
}

/// Runtime-APIs
pub mod api {
	use rstd::prelude::*;
	use primitives::{AttestationRecord, ValidatorId, Slot};
	use client::decl_runtime_apis;

	decl_runtime_apis! {
		/// API necessary for block authorship with aura.
		pub trait AuraApi {
			/// Return the slot duration in seconds for Aura.
			/// Currently, only the value provided by this type at genesis
			/// will be used.
			///
			/// Dynamic slot duration may be supported in the future.
			fn slot_duration() -> u64;

			/// Return validator attestation map.
			fn validator_ids_from_attestation(attestation: AttestationRecord) -> Vec<ValidatorId>;

			/// Return the last finalized slot.
			fn last_finalized_slot() -> Slot;

			/// Return the last justified slot.
			fn last_justified_slot() -> Slot;

			/// Return the current slot;
			fn slot() -> Slot;
		}
	}
}
