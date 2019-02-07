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

/// Data needed for casper consensus.
#[derive(Default, Clone, Eq, PartialEq)]
pub struct CasperData<A: Attestation> {
	/// Bitfield holding justification information.
	pub justification_bitfield: u64,
	/// Pending attestation to be processed.
	pub pending_attestations: Vec<A>,
	/// Current epoch.
	pub epoch: A::Epoch,
	/// Current justified epoch.
	pub justified_epoch: A::Epoch,
	/// Current finalized epoch.
	pub finalized_epoch: A::Epoch,
	/// Previous justified epoch.
	pub previous_justified_epoch: A::Epoch,
}

/// Casper struct holding pending attestation, justification and finalization information.
pub struct Casper<'a, A: Attestation, S: ValidatorStore> {
	data: CasperData<A>,
	store: &'a S,
}

impl<'a, A, S> Casper<'a, A, S> where
	A: Attestation,
	S: ValidatorStore<ValidatorId=A::ValidatorId, Epoch=A::Epoch>,
{
	/// Create a new casper context.
	pub fn new(data: CasperData<A>, store: &'a S) -> Self {
		Self { data, store }
	}

	/// Get the current epoch.
	pub fn current_epoch(&self) -> A::Epoch {
		self.data.epoch
	}

	/// Get the next epoch.
	pub fn next_epoch(&self) -> A::Epoch {
		self.data.epoch + One::one()
	}

	/// Get the previous epoch.
	pub fn previous_epoch(&self) -> A::Epoch {
		if self.data.epoch == Zero::zero() {
			self.data.epoch
		} else {
			self.data.epoch - One::one()
		}
	}

	/// Get the current justified epoch.
	pub fn justified_epoch(&self) -> A::Epoch {
		self.data.justified_epoch
	}

	/// Get the current finalized epoch.
	pub fn finalized_epoch(&self) -> A::Epoch {
		self.data.finalized_epoch
	}

	fn total_balance(&self, epoch: A::Epoch) -> S::Balance {
		let validators = self.store.active_validators(epoch);
		self.store.total_balance(&validators)
	}

	fn attesting_balance(&self, target_epoch: A::Epoch) -> S::Balance {
		let mut validators = Vec::new();
		for attestation in &self.data.pending_attestations {
			if attestation.is_canon() && attestation.target_epoch() == target_epoch {
				validators.push(attestation.validator_id().clone());
			}
		}
		self.store.total_balance(&validators)
	}

	/// Get total balance of validators at current epoch.
	pub fn current_total_balance(&self) -> S::Balance {
		self.total_balance(self.current_epoch())
	}

	/// Get total balance of attesting validators at current epoch.
	pub fn current_attesting_balance(&self) -> S::Balance {
		self.attesting_balance(self.current_epoch())
	}

	/// Get total balance of validators at previous epoch.
	pub fn previous_total_balance(&self) -> S::Balance {
		self.total_balance(self.previous_epoch())
	}

	/// Get total balance of attesting validators at previous epoch.
	pub fn previous_attesting_balance(&self) -> S::Balance {
		self.attesting_balance(self.previous_epoch())
	}

	/// Push pending attestations to Casper.
	pub fn push_pending_attestations(&mut self, mut attestations: Vec<A>) {
		self.data.pending_attestations.append(&mut attestations);
	}

	/// Prune pending attestation list.
	pub fn prune_pending_attestations(&mut self) {
		let current_epoch = self.current_epoch();
		self.data.pending_attestations.retain(|attestation| {
			attestation.target_epoch() >= current_epoch
		});
	}

	/// Advance the current epoch and start a new epoch.
	pub fn advance_epoch(&mut self) {
		// Set justification status
		let mut new_justified_epoch = self.data.justified_epoch;
		self.data.justification_bitfield <<= 1;
		if S::Balance::from(3u8) * self.previous_attesting_balance() >= S::Balance::from(2u8) * self.previous_total_balance() {
			self.data.justification_bitfield |= 2;
			new_justified_epoch = self.previous_epoch();
		}
		if S::Balance::from(3u8) * self.current_attesting_balance() >= S::Balance::from(2u8) * self.current_total_balance() {
			self.data.justification_bitfield |= 1;
			new_justified_epoch = self.current_epoch();
		}

		// Set finalization status
		if (self.data.justification_bitfield >> 1) % 8 == 0b111 && self.previous_epoch() > One::one() && self.data.previous_justified_epoch == self.previous_epoch() - One::one() - One::one() {
			self.data.finalized_epoch = self.data.previous_justified_epoch;
		}
		if (self.data.justification_bitfield >> 1) % 4 == 0b11 && self.previous_epoch() >= One::one() && self.data.previous_justified_epoch == self.previous_epoch() - One::one() {
			self.data.finalized_epoch = self.data.previous_justified_epoch;
		}
		if (self.data.justification_bitfield >> 0) % 8 == 0b111 && self.previous_epoch() >= One::one() && self.data.justified_epoch == self.previous_epoch() - One::one() {
			self.data.finalized_epoch = self.data.justified_epoch;
		}
		if (self.data.justification_bitfield >> 0) % 4 == 0b11 && self.data.justified_epoch == self.previous_epoch() {
			self.data.finalized_epoch = self.data.justified_epoch;
		}

		self.data.previous_justified_epoch = self.data.justified_epoch;
		self.data.justified_epoch = new_justified_epoch;
		self.data.epoch += One::one();

		self.prune_pending_attestations();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;

	#[derive(PartialEq, Eq, Default)]
	pub struct DummyAttestation {
		pub validator_id: usize,
		pub source_epoch: usize,
		pub target_epoch: usize,
	}

	impl Attestation for DummyAttestation {
		type ValidatorId = usize;
		type Epoch = usize;

		fn validator_id(&self) -> &usize {
			&self.validator_id
		}

		fn is_canon(&self) -> bool {
			true
		}

		fn source_epoch(&self) -> usize {
			self.source_epoch
		}

		fn target_epoch(&self) -> usize {
			self.target_epoch
		}
	}

	// Value in the order ((valid_from, valid_to), balance).
	#[derive(Default)]
	pub struct DummyStore(HashMap<usize, ((usize, usize), usize)>);

	impl ValidatorStore for DummyStore {
		type ValidatorId = usize;
		type Balance = usize;
		type Epoch = usize;

		fn total_balance(&self, validators: &[usize]) -> usize {
			let mut total = 0;
			for validator_id in validators {
				total += self.0.get(validator_id).map(|v| v.1).unwrap_or(0);
			}
			total
		}

		fn active_validators(&self, epoch: usize) -> Vec<usize> {
			let mut validators = Vec::new();
			for (validator_id, ((valid_from, valid_to), _)) in &self.0 {
				if valid_from <= &epoch && &epoch <= valid_to {
					validators.push(*validator_id);
				}
			}
			validators
		}
	}

	impl DummyStore {
		pub fn push_validator(&mut self, validator_id: usize, valid_from: usize, valid_to: usize, balance: usize) {
			self.0.insert(validator_id, ((valid_from, valid_to), balance));
		}
	}

	#[test]
	fn four_epoch_with_four_validators() {
		let mut store = DummyStore::default();
		store.push_validator(0, 0, usize::max_value(), 1);
		store.push_validator(1, 0, usize::max_value(), 1);
		store.push_validator(2, 0, usize::max_value(), 1);
		store.push_validator(3, 0, usize::max_value(), 1);

		let data = CasperData::<DummyAttestation>::default();
		let mut casper = Casper::new(data, &store);

		// Attesting on the zero round doesn't do anything, because it's already justified and finalized.
		casper.advance_epoch();

		// First round, four validators attest.
		casper.push_pending_attestations(vec![
			DummyAttestation {
				validator_id: 0,
				source_epoch: 0,
				target_epoch: 1,
			},
			DummyAttestation {
				validator_id: 1,
				source_epoch: 0,
				target_epoch: 1,
			},
			DummyAttestation {
				validator_id: 2,
				source_epoch: 0,
				target_epoch: 1,
			},
			DummyAttestation {
				validator_id: 3,
				source_epoch: 0,
				target_epoch: 1,
			},
		]);
		casper.advance_epoch();
		assert_eq!(casper.current_epoch(), 2);
		assert_eq!(casper.justified_epoch(), 1);
		assert_eq!(casper.finalized_epoch(), 0);

		// Second round, three validators attest.
		casper.push_pending_attestations(vec![
			DummyAttestation {
				validator_id: 0,
				source_epoch: 1,
				target_epoch: 2,
			},
			DummyAttestation {
				validator_id: 1,
				source_epoch: 1,
				target_epoch: 2,
			},
			DummyAttestation {
				validator_id: 2,
				source_epoch: 1,
				target_epoch: 2,
			},
		]);
		casper.advance_epoch();
		assert_eq!(casper.current_epoch(), 3);
		assert_eq!(casper.justified_epoch(), 2);
		assert_eq!(casper.finalized_epoch(), 1);

		// Third round, all four validators attest, but the one missing from previous round skipped an epoch.
		casper.push_pending_attestations(vec![
			DummyAttestation {
				validator_id: 0,
				source_epoch: 2,
				target_epoch: 3,
			},
			DummyAttestation {
				validator_id: 1,
				source_epoch: 2,
				target_epoch: 3,
			},
			DummyAttestation {
				validator_id: 2,
				source_epoch: 2,
				target_epoch: 3,
			},
			DummyAttestation {
				validator_id: 3,
				source_epoch: 1,
				target_epoch: 3,
			},
		]);
		casper.advance_epoch();
		assert_eq!(casper.current_epoch(), 4);
		assert_eq!(casper.justified_epoch(), 3);
		assert_eq!(casper.finalized_epoch(), 2);

		// Fourth round, only two validators attest.
		casper.push_pending_attestations(vec![
			DummyAttestation {
				validator_id: 0,
				source_epoch: 3,
				target_epoch: 4,
			},
			DummyAttestation {
				validator_id: 1,
				source_epoch: 3,
				target_epoch: 4,
			},
		]);
		casper.advance_epoch();
		assert_eq!(casper.current_epoch(), 5);
		assert_eq!(casper.justified_epoch(), 3);
		assert_eq!(casper.finalized_epoch(), 2);
	}
}
