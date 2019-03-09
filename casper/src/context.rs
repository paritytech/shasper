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

//! Casper related traits.

use num_traits::{One, Zero};
use core::ops::{Add, AddAssign, Sub, SubAssign, Mul, Div};
use codec::{Encode, Decode};

/// Block context.
pub trait BlockContext: Eq + PartialEq + Clone {
	/// Type of epoch.
	type Epoch: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Add<Output=Self::Epoch> + AddAssign + Sub<Output=Self::Epoch> + SubAssign + One + Zero + Encode + Decode;
	/// Attestation slot.
	type Slot: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Add<Output=Self::Slot> + AddAssign + Sub<Output=Self::Slot> + SubAssign + One + Zero + Encode + Decode;
}

/// Validator context.
pub trait ValidatorContext: BlockContext {
	/// Attestation of this context.
	type Attestation: Attestation<Context=Self>;
	/// Balance of this context.
	type Balance: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Add<Output=Self::Balance> + AddAssign + Sub<Output=Self::Balance> + SubAssign + Mul<Output=Self::Balance> + Div<Output=Self::Balance> + From<u8> + One + Zero;
	/// Type of validator Id.
	type ValidatorId: PartialEq + Eq + Clone + Copy;
}

/// Casper attestation. The source should always be canon.
pub trait Attestation: PartialEq + Eq {
	/// Validator context of this attestation.
	type Context: ValidatorContext;

	/// Get slot of this attestation.
	fn slot(&self) -> SlotOf<Self::Context>;
	/// Whether this attestation's slot is on canon chain.
	fn is_slot_canon(&self) -> bool;
	/// This attestation's inclusion distance.
	fn inclusion_distance(&self) -> SlotOf<Self::Context>;
	/// Get validator Ids of this attestation.
	fn validator_ids(&self) -> Vec<ValidatorIdOf<Self::Context>>;
	/// Whether this attestation's source is on canon chain.
	fn is_source_canon(&self) -> bool;
	/// Whether this attestation's target is on canon chain.
	fn is_target_canon(&self) -> bool;
	/// Get the source epoch of this attestation.
	fn source_epoch(&self) -> EpochOf<Self::Context>;
	/// Get the target epoch of this attestation.
	fn target_epoch(&self) -> EpochOf<Self::Context>;

	/// Whether this attestation's source and target is on canon chain.
	fn is_casper_canon(&self) -> bool {
		self.is_source_canon() && self.is_target_canon()
	}
}

/// Epoch of a context.
pub type EpochOf<C> = <C as BlockContext>::Epoch;
/// Attestation of a context.
pub type AttestationOf<C> = <C as ValidatorContext>::Attestation;
/// Slot of a context.
pub type SlotOf<C> = <C as BlockContext>::Slot;
/// Validator id of a context.
pub type ValidatorIdOf<C> = <C as ValidatorContext>::ValidatorId;
/// Balance of a context.
pub type BalanceOf<C> = <C as ValidatorContext>::Balance;
