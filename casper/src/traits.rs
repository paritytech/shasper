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
use rstd::ops::{Add, AddAssign, Sub, SubAssign, Mul, Div};

/// Casper attestation. The source should always be canon.
pub trait Attestation: PartialEq + Eq {
	/// Type of validator Id.
	type ValidatorId: PartialEq + Eq + Clone + Copy;
	/// Type of validator Id collection.
	type ValidatorIdIterator: IntoIterator<Item=Self::ValidatorId>;
	/// Type of epoch.
	type Epoch: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Add<Output=Self::Epoch> + AddAssign + Sub<Output=Self::Epoch> + SubAssign + One + Zero;

	/// Get validator Ids of this attestation.
	fn validator_ids(&self) -> Self::ValidatorIdIterator;
	/// Whether this attestation's source is on canon chain.
	fn is_source_canon(&self) -> bool;
	/// Whether this attestation's target is on canon chain.
	fn is_target_canon(&self) -> bool;
	/// Get the source epoch of this attestation.
	fn source_epoch(&self) -> Self::Epoch;
	/// Get the target epoch of this attestation.
	fn target_epoch(&self) -> Self::Epoch;

	/// Whether this attestation's source and target is on canon chain.
	fn is_casper_canon(&self) -> bool {
		self.is_source_canon() && self.is_target_canon()
	}
}

/// Casper attestation with specific slot.
pub trait SlotAttestation: Attestation {
	/// Attestation slot.
	type Slot: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Add<Output=Self::Slot> + AddAssign + Sub<Output=Self::Slot> + SubAssign + One + Zero;

	/// Get slot of this attestation.
	fn slot(&self) -> Self::Slot;
	/// Whether this attestation's slot is on canon chain.
	fn is_slot_canon(&self) -> bool;
	/// This attestation's inclusion distance.
	fn inclusion_distance(&self) -> Self::Slot;
}

/// Basic epoch context for Casper.
pub trait BaseContext {
	/// Attestation of this context.
	type Attestation: Attestation;
}

/// Context with balance, suitable for reward calculation.
pub trait BalanceContext: BaseContext {
	/// Balance of this context.
	type Balance: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Add<Output=Self::Balance> + AddAssign + Sub<Output=Self::Balance> + SubAssign + Mul<Output=Self::Balance> + Div<Output=Self::Balance> + From<u8>;
}

/// Context with slot, suitable for collecting attestations across multiple blocks.
pub trait SlotContext: BaseContext where
	AttestationOf<Self>: SlotAttestation { }

/// Epoch of a context.
pub type EpochOf<C> = <AttestationOf<C> as Attestation>::Epoch;
/// Attestation of a context.
pub type AttestationOf<C> = <C as BaseContext>::Attestation;
/// Slot of a context.
pub type SlotOf<C> = <AttestationOf<C> as SlotAttestation>::Slot;
