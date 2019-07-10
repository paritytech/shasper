#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use ssz::Ssz;
use bm_le::{IntoTree, FromTree};
use crate::primitives::{Version, Uint, H256};

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Fork information.
pub struct Fork {
	/// Previous fork version
	pub previous_version: Version,
	/// Current fork version
	pub current_version: Version,
	/// Fork epoch number
	pub epoch: Uint,
}

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Checkpoint
pub struct Checkpoint {
	/// Epoch
	pub epoch: Uint,
	/// Root of the checkpoint
	pub root: H256,
}
