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

//! Casper FFG generic consensus algorithm on justification and finalization.

use num_traits::{One, Zero};
use rstd::ops::{Add, AddAssign, Sub, SubAssign, Mul};

/// Store that holds validator active and balance information.
pub trait ValidatorStore {
	/// Type of validator Id.
	type ValidatorId: PartialEq + Eq;
	/// Type of balance.
	type Balance: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Mul<Output=Self::Balance> + From<u8>;
	/// Type of epoch.
	type Epoch: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Add<Output=Self::Epoch> + AddAssign + Sub<Output=Self::Epoch> + SubAssign + One + Zero;

	/// Get total balance of given validator Ids.
	fn total_balance(&self, validators: &[Self::ValidatorId]) -> Self::Balance;
	/// Get all active validators at given epoch.
	fn active_validators(&self, epoch: Self::Epoch) -> Vec<Self::ValidatorId>;
}

/// Casper attestation.
pub trait Attestation: PartialEq + Eq {
	/// Type of validator Id.
	type ValidatorId: PartialEq + Eq + Clone;
	/// Type of epoch.
	type Epoch: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Add<Output=Self::Epoch> + AddAssign + Sub<Output=Self::Epoch> + SubAssign + One + Zero;

	/// Get validator Id of this attestation.
	fn validator_id(&self) -> &Self::ValidatorId;
	/// Whether this attestation's source and target is on canon chain.
	fn is_canon(&self) -> bool;
	/// Get the source epoch of this attestation.
	fn source_epoch(&self) -> Self::Epoch;
	/// Get the target epoch of this attestation.
	fn target_epoch(&self) -> Self::Epoch;
}

/// Return whether given two attestations satisfy Casper slashing conditions.
pub fn slashable<C: Attestation>(a: &C, b: &C) -> bool {
	// Two attestations must be different, and must be from the same validator.
	if a == b || a.validator_id() != b.validator_id() {
		return false;
	}

	// If two attestations have the same target, then it is a double vote.
	if a.target_epoch() == b.target_epoch() {
		return true;
	}

	// If one attestation surrounds the other, then it is a surround vote.
	if a.source_epoch() < b.source_epoch() && b.target_epoch() < a.target_epoch() {
		return true;
	}
	if b.source_epoch() < a.source_epoch() && a.target_epoch() < b.target_epoch() {
		return true;
	}

	false
}

/// Casper struct holding pending attestation, justification and finalization information.
pub struct Casper<A: Attestation> {
	justification_bitfield: u64,
	pending_attestations: Vec<A>,
	epoch: A::Epoch,
	justified_epoch: A::Epoch,
	finalized_epoch: A::Epoch,
	previous_justified_epoch: A::Epoch,
}

impl<A: Attestation> Casper<A> {
	/// Get the current epoch.
	pub fn current_epoch(&self) -> A::Epoch {
		self.epoch
	}

	/// Get the next epoch.
	pub fn next_epoch(&self) -> A::Epoch {
		self.epoch + One::one()
	}

	/// Get the previous epoch.
	pub fn previous_epoch(&self) -> A::Epoch {
		if self.epoch == Zero::zero() {
			self.epoch
		} else {
			self.epoch - One::one()
		}
	}

	fn total_balance<S: ValidatorStore<ValidatorId=A::ValidatorId, Epoch=A::Epoch>>(&self, store: &S, epoch: A::Epoch) -> S::Balance {
		let validators = store.active_validators(epoch);
		store.total_balance(&validators)
	}

	fn attesting_balance<S: ValidatorStore<ValidatorId=A::ValidatorId, Epoch=A::Epoch>>(&self, store: &S, target_epoch: A::Epoch) -> S::Balance {
		let mut validators = Vec::new();
		for attestation in &self.pending_attestations {
			if attestation.is_canon() && attestation.target_epoch() == target_epoch {
				validators.push(attestation.validator_id().clone());
			}
		}
		store.total_balance(&validators)
	}

	/// Get total balance of validators at current epoch.
	pub fn current_total_balance<S: ValidatorStore<ValidatorId=A::ValidatorId, Epoch=A::Epoch>>(&self, store: &S) -> S::Balance {
		self.total_balance(store, self.current_epoch())
	}

	/// Get total balance of attesting validators at current epoch.
	pub fn current_attesting_balance<S: ValidatorStore<ValidatorId=A::ValidatorId, Epoch=A::Epoch>>(&self, store: &S) -> S::Balance {
		self.attesting_balance(store, self.current_epoch())
	}

	/// Get total balance of validators at previous epoch.
	pub fn previous_total_balance<S: ValidatorStore<ValidatorId=A::ValidatorId, Epoch=A::Epoch>>(&self, store: &S) -> S::Balance {
		self.total_balance(store, self.previous_epoch())
	}

	/// Get total balance of attesting validators at previous epoch.
	pub fn previous_attesting_balance<S: ValidatorStore<ValidatorId=A::ValidatorId, Epoch=A::Epoch>>(&self, store: &S) -> S::Balance {
		self.attesting_balance(store, self.previous_epoch())
	}

	/// Push pending attestations to Casper.
	pub fn push_pending_attestations(&mut self, mut attestations: Vec<A>) {
		self.pending_attestations.append(&mut attestations);
	}

	/// Prune pending attestation list.
	pub fn prune_pending_attestations(&mut self) {
		let current_epoch = self.current_epoch();
		self.pending_attestations.retain(|attestation| {
			attestation.target_epoch() >= current_epoch
		});
	}

	/// Advance the current epoch and start a new epoch.
	pub fn advance_epoch<T: ValidatorStore<ValidatorId=A::ValidatorId, Epoch=A::Epoch>>(&mut self, store: &T) {
		// Set justification status
		let mut new_justified_epoch = self.justified_epoch;
		self.justification_bitfield <<= 1;
		if T::Balance::from(3u8) * self.previous_attesting_balance(store) >= T::Balance::from(2u8) * self.previous_total_balance(store) {
			self.justification_bitfield |= 2;
			new_justified_epoch = self.previous_epoch();
		}
		if T::Balance::from(3u8) * self.current_attesting_balance(store) >= T::Balance::from(2u8) * self.current_total_balance(store) {
			self.justification_bitfield |= 1;
			new_justified_epoch = self.current_epoch();
		}

		// Set finalization status
		if (self.justification_bitfield >> 1) % 8 == 0b111 && self.previous_epoch() > One::one() && self.previous_justified_epoch == self.previous_epoch() - One::one() - One::one() {
			self.finalized_epoch = self.previous_justified_epoch;
		}
		if (self.justification_bitfield >> 1) % 4 == 0b11 && self.previous_epoch() >= One::one() && self.previous_justified_epoch == self.previous_epoch() - One::one() {
			self.finalized_epoch = self.previous_justified_epoch;
		}
		if (self.justification_bitfield >> 0) % 8 == 0b111 && self.previous_epoch() >= One::one() && self.justified_epoch == self.previous_epoch() - One::one() {
			self.finalized_epoch = self.justified_epoch;
		}
		if (self.justification_bitfield >> 0) % 4 == 0b11 && self.justified_epoch == self.previous_epoch() {
			self.finalized_epoch = self.justified_epoch;
		}

		self.previous_justified_epoch = self.justified_epoch;
		self.justified_epoch = new_justified_epoch;
		self.epoch += One::one();

		self.prune_pending_attestations();
	}
}
