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

use primitives::{Slot, H256};
use codec_derive::{Encode, Decode};
use inherents::InherentIdentifier;
#[cfg(feature = "std")]
use primitives::KeccakHasher;
#[cfg(feature = "std")]
use casper::randao::RandaoOnion;
#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(feature = "std")]
use inherents::{RuntimeString, ProvideInherentData};

pub const RANDAO_INHERENT_IDENTIFIER: InherentIdentifier = *b"shasperr";
pub const TIMESTAMP_INHERENT_IDENTIFIER: InherentIdentifier = *b"shaspert";

/// Consensus inherent data
#[derive(Encode, Decode)]
pub struct RandaoInherentData {
	pub randao_reveal: H256,
}

/// Importer inherent data.
#[derive(Encode, Decode)]
pub struct TimestampInherentData {
	pub timestamp: u64,
	pub slot: Slot,
}

/// Consensus inherent data provider.
#[cfg(feature = "std")]
pub struct RandaoInherentDataProvider {
	slot_duration: u64,
	start_slot: Slot,
	randao_onion: Arc<RandaoOnion<KeccakHasher>>,
}

#[cfg(feature = "std")]
impl RandaoInherentDataProvider {
	pub fn new(slot_duration: u64, start_slot: Slot, randao_onion: Arc<RandaoOnion<KeccakHasher>>) -> Self {
		Self {
			slot_duration, start_slot, randao_onion
		}
	}
}

#[cfg(feature = "std")]
impl ProvideInherentData for RandaoInherentDataProvider {
	fn inherent_identifier(&self) -> &'static inherents::InherentIdentifier {
		&RANDAO_INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut inherents::InherentData,
	) -> Result<(), RuntimeString> {
		let (_, slot) = match utils::timestamp_and_slot_now(self.slot_duration) {
			Some(data) => data,
			None => return Err("Timestamp generation failed".into()),
		};
		let randao_reveal = self.randao_onion.at((slot - self.start_slot) as usize);
		inherent_data.put_data(RANDAO_INHERENT_IDENTIFIER, &RandaoInherentData {
			randao_reveal,
		})
	}

	fn error_to_string(&self, _error: &[u8]) -> Option<String> {
		None
	}
}

/// Consensus inherent data provider.
#[cfg(feature = "std")]
pub struct TimestampInherentDataProvider {
	slot_duration: u64,
}

#[cfg(feature = "std")]
impl TimestampInherentDataProvider {
	pub fn new(slot_duration: u64) -> Self {
		Self {
			slot_duration,
		}
	}
}

#[cfg(feature = "std")]
impl ProvideInherentData for TimestampInherentDataProvider {
	fn inherent_identifier(&self) -> &'static inherents::InherentIdentifier {
		&TIMESTAMP_INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut inherents::InherentData,
	) -> Result<(), RuntimeString> {
		let (timestamp, slot) = match utils::timestamp_and_slot_now(self.slot_duration) {
			Some(data) => data,
			None => return Err("Timestamp generation failed".into()),
		};
		inherent_data.put_data(TIMESTAMP_INHERENT_IDENTIFIER, &TimestampInherentData {
			timestamp, slot,
		})
	}

	fn error_to_string(&self, _error: &[u8]) -> Option<String> {
		None
	}
}

#[cfg(feature = "std")]
pub mod utils {
	use std::time::Duration;

	pub fn timestamp_now() -> Option<Duration> {
		use std::time::SystemTime;

		let now = SystemTime::now();
		now.duration_since(SystemTime::UNIX_EPOCH).ok()
	}

	pub fn timestamp_and_slot_now(slot_duration: u64) -> Option<(u64, u64)> {
		timestamp_now().map(|s| {
			let s = s.as_secs();
			(s, s / slot_duration)
		})
	}

	pub fn slot_now(slot_duration: u64) -> Option<u64> {
		timestamp_and_slot_now(slot_duration).map(|(_, slot)| slot)
	}

	pub fn time_until_next(now: Duration, slot_duration: u64) -> Duration {
		let remaining_full_secs = slot_duration - (now.as_secs() % slot_duration) - 1;
		let remaining_nanos = 1_000_000_000 - now.subsec_nanos();
		Duration::new(remaining_full_secs, remaining_nanos)
	}
}


/// The ApiIds for Aura authorship API.
pub mod id {
	use client::runtime_api::ApiId;

	/// ApiId for the AuraApi trait.
	pub const SHASPER_API: ApiId = *b"shaspera";
}

/// Runtime-APIs
pub mod api {
	use primitives::{Epoch, UncheckedAttestation, CheckedAttestation, Slot, ValidatorId, ValidatorIndex};
	use client::decl_runtime_apis;

	decl_runtime_apis! {
		/// API necessary for block authorship with Shasper.
		pub trait ShasperApi {
			/// Return the last finalized epoch.
			fn finalized_epoch() -> Epoch;

			/// Return the last justified epoch.
			fn justified_epoch() -> Epoch;

			/// Return the last finalized slot.
			fn finalized_slot() -> Slot;

			/// Return the last justified slot.
			fn justified_slot() -> Slot;

			/// Return the current slot.
			fn slot() -> Slot;

			/// Return the genesis slot.
			fn genesis_slot() -> Slot;

			/// Get the proposer id at slot.
			fn proposer(slot: Slot) -> ValidatorId;

			/// Check an attestation.
			fn check_attestation(unchecked: UncheckedAttestation) -> Option<CheckedAttestation>;

			/// Given an attestation, return the validator index.
			fn validator_index(validator_id: ValidatorId) -> Option<ValidatorIndex>;
		}
	}
}
