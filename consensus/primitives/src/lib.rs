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

#[cfg(feature = "std")]
use inherents::{RuntimeString, ProvideInherentData};

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"shasper0";

/// Consensus inherent data
#[derive(Encode, Decode)]
pub struct InherentData {
	pub timestamp: u64,
	pub slot: primitives::Slot,
}

/// Consensus inherent data provider.
#[cfg(feature = "std")]
pub struct InherentDataProvider {
	slot_duration: u64,
}

#[cfg(feature = "std")]
impl InherentDataProvider {
	pub fn new(slot_duration: u64) -> Self {
		Self {
			slot_duration
		}
	}
}

#[cfg(feature = "std")]
impl ProvideInherentData for InherentDataProvider {
	fn inherent_identifier(&self) -> &'static inherents::InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut inherents::InherentData,
	) -> Result<(), RuntimeString> {
		let (timestamp, slot) = match utils::timestamp_and_slot_now(self.slot_duration) {
			Some(data) => data,
			None => return Err("Timestamp generation failed".into()),
		};
		inherent_data.put_data(INHERENT_IDENTIFIER, &InherentData {
			timestamp, slot
		})
	}

	fn error_to_string(&self, _error: &[u8]) -> Option<String> {
		None
	}
}

#[cfg(feature = "std")]
pub mod utils {
	use std::time::Duration;
	use primitives::ValidatorId;

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

	/// Get slot author for given block along with authorities.
	pub fn slot_author(slot_num: u64, authorities: &[ValidatorId]) -> Option<ValidatorId> {
		if authorities.is_empty() { return None }

		let idx = slot_num % (authorities.len() as u64);
		assert!(idx <= usize::max_value() as u64,
				"It is impossible to have a vector with length beyond the address space; qed");

		let current_author = *authorities.get(idx as usize)
			.expect("authorities not empty; index constrained to list length;\
					 this is a valid index; qed");

		Some(current_author)
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
	use primitives::Epoch;
	use client::decl_runtime_apis;

	decl_runtime_apis! {
		/// API necessary for block authorship with Shasper.
		pub trait ShasperApi {
			/// Return the last finalized slot.
			fn finalized_epoch() -> Epoch;

			/// Return the last justified slot.
			fn justified_epoch() -> Epoch;
		}
	}
}
