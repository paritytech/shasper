#![cfg_attr(not(feature = "std"), no_std)]

use primitives::crypto::KeyTypeId;

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
