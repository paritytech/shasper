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

//! Beacon reward constructs.

use num_traits::{One, Zero};
use rstd::ops::{Add, AddAssign, Sub, SubAssign};
use crate::casper::{Attestation, PendingAttestations, ValidatorStore};

/// Rewards for beacon chain.
pub enum BeaconRewardType<Slot> {
	/// The validator attested on the expected head.
	ExpectedHead,
	/// The validator is active, but does not attest on the epxected head.
	NoExpectedHead,
	/// Inclusion distance for attestations.
	InclusionDistance(Slot),
}

/// Beacon chain attestation.
pub trait BeaconAttestation: Attestation {
	/// Attestation slot.
	type Slot: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Add<Output=Self::Slot> + AddAssign + Sub<Output=Self::Slot> + SubAssign + One + Zero;

	/// Get slot of this attestation.
	fn slot(&self) -> Self::Slot;
	/// Whether this attestation's slot is on canon chain.
	fn is_beacon_canon(&self) -> bool;
	/// This attestation's inclusion distance.
	fn inclusion_distance(&self) -> Self::Slot;
}

/// Struct for handle beacon rewards.
pub struct BeaconReward<'a, A: BeaconAttestation, S: ValidatorStore> {
	epoch: A::Epoch,
	store: &'a S,
	pending_attestations: &'a PendingAttestations<A>,
}

impl<'a, A, S> BeaconReward<'a, A, S> where
	A: BeaconAttestation,
	S: ValidatorStore<ValidatorId=A::ValidatorId, Epoch=A::Epoch>,
{
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

	/// Get rewards for beacon chain.
	pub fn rewards(&self) -> Vec<(A::ValidatorId, BeaconRewardType<A::Slot>)> {
		let mut no_expected_head_validators = self.store.active_validators(self.current_epoch());

		let mut rewards = Vec::new();
		for attestation in self.pending_attestations.iter() {
			// Expected beacon chain head.
			if attestation.is_beacon_canon() && attestation.target_epoch() == self.previous_epoch() {
				rewards.push((attestation.validator_id().clone(), BeaconRewardType::ExpectedHead));
				no_expected_head_validators.retain(|validator_id| {
					validator_id != attestation.validator_id()
				});
			}

			if attestation.target_epoch() == self.previous_epoch() {
				rewards.push((attestation.validator_id().clone(), BeaconRewardType::InclusionDistance(attestation.inclusion_distance())));
			}
		}

		for validator_id in no_expected_head_validators {
			rewards.push((validator_id, BeaconRewardType::NoExpectedHead));
		}

		rewards
	}
}
