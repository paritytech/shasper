// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use primitives::crypto::KeyTypeId;
use sr_api_macros::decl_runtime_apis;
use sr_primitives::traits::Block as BlockT;

pub const KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"casp");

mod app {
	use app_crypto::{app_crypto, ed25519};
	app_crypto!(ed25519, super::KEY_TYPE_ID);
}

#[cfg(feature = "std")]
pub type ValidatorPair = app::Pair;

/// Identity of a Casper validator.
pub type ValidatorId = app::Public;

/// Signature for a Casper validator.
pub type ValidatorSignature = app::Signature;

/// The weight of a validator.
pub type ValidatorWeight = u64;

/// The index of a validator.
pub type ValidatorIndex = u64;

/// Type of epoch.
pub type Epoch = u64;

decl_runtime_apis! {
	/// API necessary for block authorship with aura.
	pub trait CasperApi {
		/// Current justified block hash.
		fn current_justified_block() -> <Block as BlockT>::Hash;

		/// Previous justified block hash.
		fn previous_justified_block() -> <Block as BlockT>::Hash;

		/// Finalized block hash.
		fn finalized_block() -> <Block as BlockT>::Hash;
	}
}
