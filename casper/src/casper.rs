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
use codec_derive::{Encode, Decode};
use rstd::prelude::*;
use rstd::ops::{Add, AddAssign, Sub, SubAssign};

use crate::store::{
	self, Attestation, ValidatorStore, PendingAttestationsStore, BlockStore,
	PendingAttestationsStoreEpoch, PendingAttestationsStoreValidatorId,
};

/// Return whether given two attestations satisfy Casper slashing conditions.
pub fn slashable<C: Attestation>(a: &C, b: &C) -> Vec<C::ValidatorId> {
	let slashable = {
		// Two attestations must be different.
		if a == b {
			false
		}

		// If two attestations have the same target, then it is a double vote.
		else if a.target_epoch() == b.target_epoch() {
			true
		}

		// If one attestation surrounds the other, then it is a surround vote.
		else if a.source_epoch() < b.source_epoch() && b.target_epoch() < a.target_epoch() {
			true
		}
		else if b.source_epoch() < a.source_epoch() && a.target_epoch() < b.target_epoch() {
			true
		}

		else {
			false
		}
	};

	if slashable {
		let mut ret = Vec::new();
		for validator_id in a.validator_ids() {
			if b.validator_ids().into_iter().any(|v| v == validator_id) {
				ret.push(validator_id);
			}
		}
		ret
	} else {
		Vec::new()
	}
}

/// Data needed for casper consensus.
#[derive(Default, Clone, Eq, PartialEq, Encode, Decode)]
pub struct CasperContext<Epoch> {
	/// Bitfield holding justification information.
	pub justification_bitfield: u64,
	/// Current epoch.
	pub epoch: Epoch,
	/// Current justified epoch.
	pub justified_epoch: Epoch,
	/// Current finalized epoch.
	pub finalized_epoch: Epoch,
	/// Previous justified epoch.
	pub previous_justified_epoch: Epoch,
}

impl<Epoch> CasperContext<Epoch> where
	Epoch: Ord + Copy + Clone + Zero + One + Add<Output=Epoch> + AddAssign + Sub<Output=Epoch> + SubAssign
{
	/// Create a new Casper context.
	pub fn new(genesis_epoch: Epoch) -> Self {
		Self {
			justification_bitfield: 0,
			epoch: genesis_epoch,
			justified_epoch: genesis_epoch,
			finalized_epoch: genesis_epoch,
			previous_justified_epoch: genesis_epoch,
		}
	}

	/// Get the current epoch.
	pub fn epoch(&self) -> Epoch {
		self.epoch
	}

	/// Get the next epoch.
	pub fn next_epoch(&self) -> Epoch {
		self.epoch() + One::one()
	}

	/// Get the previous epoch.
	pub fn previous_epoch(&self) -> Epoch {
		if self.epoch() == Zero::zero() {
			Zero::zero()
		} else {
			self.epoch() - One::one()
		}
	}

	/// Validate an attestation to be included in pending attestations.
	pub fn validate_attestation<A>(&self, attestation: &A) -> bool where
		A: Attestation<Epoch=Epoch>
	{
		attestation.is_source_canon() &&
			if attestation.target_epoch() == self.epoch {
				attestation.source_epoch() == self.justified_epoch
			} else {
				attestation.source_epoch() == self.previous_justified_epoch
			}
	}

	/// Prune pending attestation list.
	fn prune_pending_attestations<A, S>(&self, store: &mut S) where
		A: Attestation<Epoch=Epoch>,
		S: PendingAttestationsStore<Attestation=A>,
	{
		let current_epoch = self.epoch();
		PendingAttestationsStore::retain(store, |attestation| {
			attestation.target_epoch() >= current_epoch
		});
	}

	/// Advance the current epoch and start a new epoch.
	pub fn advance_epoch<A, S>(&mut self, store: &mut S) where
		A: Attestation<Epoch=Epoch>,
		S: PendingAttestationsStore<Attestation=A>,
		S: BlockStore<Epoch=PendingAttestationsStoreEpoch<S>>,
		S: ValidatorStore<
			ValidatorId=PendingAttestationsStoreValidatorId<S>,
			Epoch=PendingAttestationsStoreEpoch<S>
		>,
	{
		debug_assert!({
			store.attestations().into_iter().all(|attestation| {
				self.validate_attestation(&attestation)
			})
		});

		// Set justification status
		let mut new_justified_epoch = self.justified_epoch;
		self.justification_bitfield <<= 1;
		if S::Balance::from(3u8) * store::canon_target_attesting_balance(store, self.previous_epoch()) >= S::Balance::from(2u8) * store::active_total_balance(store, self.previous_epoch()) {
			self.justification_bitfield |= 2;
			new_justified_epoch = self.previous_epoch();
		}
		if S::Balance::from(3u8) * store::canon_target_attesting_balance(store, self.epoch()) >= S::Balance::from(2u8) * store::active_total_balance(store, self.epoch()) {
			self.justification_bitfield |= 1;
			new_justified_epoch = self.epoch();
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

		self.prune_pending_attestations(store);

		self.previous_justified_epoch = self.justified_epoch;
		self.justified_epoch = new_justified_epoch;
		self.epoch += One::one();

		assert!(self.epoch() == store.epoch(), "Store block epoch must equal to casper context.");
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;

	#[derive(PartialEq, Eq, Default, Clone)]
	pub struct DummyAttestation {
		pub validator_id: usize,
		pub source_epoch: usize,
		pub target_epoch: usize,
	}

	impl Attestation for DummyAttestation {
		type ValidatorId = usize;
		type ValidatorIdIterator = Vec<usize>;
		type Epoch = usize;

		fn validator_ids(&self) -> Vec<usize> {
			vec![self.validator_id]
		}

		fn is_source_canon(&self) -> bool {
			true
		}

		fn is_target_canon(&self) -> bool {
			true
		}

		fn source_epoch(&self) -> usize {
			self.source_epoch
		}

		fn target_epoch(&self) -> usize {
			self.target_epoch
		}
	}

	#[derive(Default)]
	pub struct DummyStore {
		pub epoch: usize,
		pub pending_attestations: Vec<DummyAttestation>,
		pub validators: HashMap<usize, ((usize, usize), usize)>,
	}

	impl ValidatorStore for DummyStore {
		type ValidatorId = usize;
		type ValidatorIdIterator = Vec<usize>;
		type Balance = usize;
		type Epoch = usize;

		fn total_balance(&self, validators: &[usize]) -> usize {
			let mut total = 0;
			for validator_id in validators {
				total += self.validators.get(validator_id).map(|v| v.1).unwrap_or(0);
			}
			total
		}

		fn active_validators(&self, epoch: usize) -> Vec<usize> {
			let mut validators = Vec::new();
			for (validator_id, ((valid_from, valid_to), _)) in &self.validators {
				if valid_from <= &epoch && &epoch <= valid_to {
					validators.push(*validator_id);
				}
			}
			validators
		}
	}

	impl PendingAttestationsStore for DummyStore {
		type Attestation = DummyAttestation;
		type AttestationIterator = Vec<DummyAttestation>;

		fn attestations(&self) -> Vec<DummyAttestation> {
			self.pending_attestations.clone()
		}

		fn retain<F: FnMut(&Self::Attestation) -> bool>(&mut self, f: F) {
			self.pending_attestations.retain(f)
		}
	}

	impl BlockStore for DummyStore {
		type Epoch = usize;

		fn epoch(&self) -> usize {
			self.epoch
		}
	}

	impl DummyStore {
		pub fn push_validator(&mut self, validator_id: usize, valid_from: usize, valid_to: usize, balance: usize) {
			self.validators.insert(validator_id, ((valid_from, valid_to), balance));
		}
	}

	#[test]
	fn four_epoch_with_four_validators() {
		let mut store = DummyStore::default();
		store.push_validator(0, 0, usize::max_value(), 1);
		store.push_validator(1, 0, usize::max_value(), 1);
		store.push_validator(2, 0, usize::max_value(), 1);
		store.push_validator(3, 0, usize::max_value(), 1);

		let mut casper = CasperContext::<usize>::default();

		// Attesting on the zero round doesn't do anything, because it's already justified and finalized.
		store.epoch += 1;
		casper.advance_epoch(&mut store);

		// First round, four validators attest.
		store.pending_attestations.append(&mut vec![
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
		store.epoch += 1;
		casper.advance_epoch(&mut store);
		assert_eq!(casper.epoch, 2);
		assert_eq!(casper.justified_epoch, 1);
		assert_eq!(casper.finalized_epoch, 0);

		// Second round, three validators attest.
		store.pending_attestations.append(&mut vec![
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
		store.epoch += 1;
		casper.advance_epoch(&mut store);
		assert_eq!(casper.epoch, 3);
		assert_eq!(casper.justified_epoch, 2);
		assert_eq!(casper.finalized_epoch, 1);

		// Third round, three validators attest.
		store.pending_attestations.append(&mut vec![
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
		]);
		store.epoch += 1;
		casper.advance_epoch(&mut store);
		assert_eq!(casper.epoch, 4);
		assert_eq!(casper.justified_epoch, 3);
		assert_eq!(casper.finalized_epoch, 2);

		// Fourth round, only two validators attest.
		store.pending_attestations.append(&mut vec![
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
		store.epoch += 1;
		casper.advance_epoch(&mut store);
		assert_eq!(casper.epoch, 5);
		assert_eq!(casper.justified_epoch, 3);
		assert_eq!(casper.finalized_epoch, 2);
	}
}
