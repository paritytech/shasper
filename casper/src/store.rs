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

//! Common store traits

use num_traits::{One, Zero};
use crate::context::{
	ValidatorContext, BalanceOf, EpochOf, AttestationOf, Attestation,
	ValidatorIdOf,
};

/// Store that holds validator active and balance information.
pub trait ValidatorStore<C: ValidatorContext> {
	/// Get total balance of given validator Ids.
	fn total_balance(&self, validators: &[ValidatorIdOf<C>]) -> BalanceOf<C>;
	/// Get all active validators at given epoch.
	fn active_validators(&self, epoch: EpochOf<C>) -> Vec<ValidatorIdOf<C>>;
}

/// Store that holds pending attestations.
pub trait PendingAttestationsStore<C: ValidatorContext> {
	/// Get the current list of attestations.
	fn attestations(&self) -> Vec<AttestationOf<C>>;
	/// Retain specific attestations and remove the rest.
	fn retain<F: FnMut(&AttestationOf<C>) -> bool>(&mut self, f: F);
}

/// Store that holds general block information.
pub trait BlockStore<C: ValidatorContext> {
	/// Get the current epoch.
	fn epoch(&self) -> EpochOf<C>;
	/// Get the next epoch.
	fn next_epoch(&self) -> EpochOf<C> {
		self.epoch() + One::one()
	}
	/// Get the previous epoch.
	fn previous_epoch(&self) -> EpochOf<C> {
		if self.epoch() == Zero::zero() {
			Zero::zero()
		} else {
			self.epoch() - One::one()
		}
	}
}

/// Attesting canon target balance at epoch.
pub fn canon_target_attesting_balance<C: ValidatorContext, S>(
	store: &S,
	epoch: EpochOf<C>
) -> BalanceOf<C> where
	S: PendingAttestationsStore<C> + ValidatorStore<C>,
{
	let mut validators = Vec::new();
	for attestation in store.attestations() {
		if attestation.is_casper_canon() && attestation.target_epoch() == epoch {
			for validator_id in attestation.validator_ids() {
				validators.push(validator_id.clone());
			}
		}
	}
	store.total_balance(&validators)
}

/// Attesting canon source balance at epoch.
pub fn canon_source_attesting_balance<C: ValidatorContext, S>(
	store: &S,
	epoch: EpochOf<C>
) -> BalanceOf<C> where
	S: PendingAttestationsStore<C> + ValidatorStore<C>,
{
	let mut validators = Vec::new();
	for attestation in store.attestations() {
		if attestation.is_casper_canon() && attestation.source_epoch() == epoch {
			for validator_id in attestation.validator_ids() {
				validators.push(validator_id.clone());
			}
		}
	}
	store.total_balance(&validators)
}

/// Total balance at epoch.
pub fn active_total_balance<C: ValidatorContext, S>(
	store: &S,
	epoch: EpochOf<C>
) -> BalanceOf<C> where
	S: ValidatorStore<C>
{
	let validators = store.active_validators(epoch).into_iter().collect::<Vec<_>>();
	store.total_balance(&validators)
}
