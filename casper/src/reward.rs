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
use crate::casper::CasperContext;
use crate::store::{
	Attestation, ValidatorStore, PendingAttestationsStore, BlockStore,
	PendingAttestationsStoreValidatorId, PendingAttestationsStoreEpoch
};

/// Rewards for Casper.
pub enum CasperRewardType {
	/// The attestation has an expected source.
	ExpectedSource,
	/// The validator is active, but does not have an attestation with expected source.
	NoExpectedSource,
	/// The attestation has an expected target.
	ExpectedTarget,
	/// The validator is active, but does not have an attestation with expected target.
	NoExpectedTarget,
}

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
	fn is_slot_canon(&self) -> bool;
	/// This attestation's inclusion distance.
	fn inclusion_distance(&self) -> Self::Slot;
}

/// Get rewards for beacon chain.
pub fn beacon_rewards<A, S>(store: &S) -> Vec<(A::ValidatorId, BeaconRewardType<A::Slot>)> where
	A: BeaconAttestation,
	S: PendingAttestationsStore<Attestation=A>,
	S: BlockStore<Epoch=PendingAttestationsStoreEpoch<S>>,
	S: ValidatorStore<
		ValidatorId=PendingAttestationsStoreValidatorId<S>,
		Epoch=PendingAttestationsStoreEpoch<S>
	>,
{
	let mut no_expected_head_validators = store.active_validators(store.epoch());

	let mut rewards = Vec::new();
	for attestation in store.attestations() {
		if attestation.target_epoch() == store.previous_epoch() {
			rewards.push((attestation.validator_id().clone(), BeaconRewardType::InclusionDistance(attestation.inclusion_distance())));

			if attestation.is_slot_canon() {
				rewards.push((attestation.validator_id().clone(), BeaconRewardType::ExpectedHead));
				no_expected_head_validators.retain(|validator_id| {
					validator_id != attestation.validator_id()
				});
			}
		}
	}

	for validator_id in no_expected_head_validators {
		rewards.push((validator_id, BeaconRewardType::NoExpectedHead));
	}

	rewards
}

/// Get rewards for casper. Note that this usually needs to be called before `advance_epoch`, but after all pending
/// attestations have been pushed.
pub fn casper_rewards<A, S>(context: &CasperContext<A>, store: &S) -> Vec<(A::ValidatorId, CasperRewardType)> where
	A: Attestation,
	S: PendingAttestationsStore<Attestation=A>,
	S: BlockStore<Epoch=PendingAttestationsStoreEpoch<S>>,
	S: ValidatorStore<
		ValidatorId=PendingAttestationsStoreValidatorId<S>,
		Epoch=PendingAttestationsStoreEpoch<S>
	>,
{
	let previous_justified_epoch = context.previous_justified_epoch;
	let mut no_expected_source_validators = store.active_validators(context.epoch());
	let mut no_expected_target_validators = no_expected_source_validators.clone();

	let mut rewards = Vec::new();
	for attestation in store.attestations() {
		if attestation.source_epoch() == previous_justified_epoch {
			rewards.push((attestation.validator_id().clone(), CasperRewardType::ExpectedSource));
			no_expected_source_validators.retain(|validator_id| {
				validator_id != attestation.validator_id()
			});

			if attestation.is_casper_canon() {
				rewards.push((attestation.validator_id().clone(), CasperRewardType::ExpectedTarget));
				no_expected_target_validators.retain(|validator_id| {
					validator_id != attestation.validator_id()
				});
			}
		}
	}

	for validator in no_expected_source_validators {
		rewards.push((validator, CasperRewardType::NoExpectedSource));
	}

	for validator in no_expected_target_validators {
		rewards.push((validator, CasperRewardType::NoExpectedTarget));
	}

	rewards
}
