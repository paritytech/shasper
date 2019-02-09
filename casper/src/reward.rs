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
use rstd::ops::{Add, AddAssign, Sub, SubAssign, Div};
use crate::casper::CasperContext;
use crate::store::{
	Attestation, ValidatorStore, PendingAttestationsStore, BlockStore,
	PendingAttestationsStoreValidatorId, PendingAttestationsStoreEpoch,
	ValidatorStoreBalance, ValidatorStoreValidatorId, ValidatorStoreEpoch,
};

/// Rewards for Casper.
#[derive(Eq, PartialEq, Clone)]
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
#[derive(Eq, PartialEq, Clone)]
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

fn push_rewards<A, T>(rewards: &mut Vec<(A::ValidatorId, T)>, attestation: &A, reward: T) where
	A: Attestation,
	T: Clone,
{
	for	validator_id in attestation.validator_ids() {
		rewards.push((validator_id.clone(), reward.clone()));
	}
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
	let mut no_expected_head_validators = store.active_validators(store.previous_epoch());

	let mut rewards = Vec::new();
	for attestation in store.attestations() {
		if attestation.target_epoch() == store.previous_epoch() {
			push_rewards(&mut rewards, attestation, BeaconRewardType::InclusionDistance(attestation.inclusion_distance()));

			if attestation.is_slot_canon() {
				push_rewards(&mut rewards, attestation, BeaconRewardType::ExpectedHead);
				no_expected_head_validators.retain(|validator_id| {
					!attestation.validator_ids().into_iter().any(|v| v == *validator_id)
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
pub fn casper_rewards<A, S>(context: &CasperContext<A::Epoch>, store: &S) -> Vec<(A::ValidatorId, CasperRewardType)> where
	A: Attestation,
	S: PendingAttestationsStore<Attestation=A>,
	S: BlockStore<Epoch=PendingAttestationsStoreEpoch<S>>,
	S: ValidatorStore<
		ValidatorId=PendingAttestationsStoreValidatorId<S>,
		Epoch=PendingAttestationsStoreEpoch<S>
	>,
{
	let mut no_expected_source_validators = store.active_validators(context.previous_epoch());
	let mut no_expected_target_validators = no_expected_source_validators.clone();

	let mut rewards = Vec::new();
	for attestation in store.attestations() {
		if attestation.target_epoch() == store.previous_epoch() {
			push_rewards(&mut rewards, attestation, CasperRewardType::ExpectedSource);
			no_expected_source_validators.retain(|validator_id| {
				!attestation.validator_ids().into_iter().any(|v| v == *validator_id)
			});

			if attestation.is_target_canon() {
				push_rewards(&mut rewards, attestation, CasperRewardType::ExpectedTarget);
				no_expected_target_validators.retain(|validator_id| {
					!attestation.validator_ids().into_iter().any(|v| v == *validator_id)
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

/// Config for default reward scheme.
pub struct DefaultSchemeConfig<Balance> {
	/// Base reward quotient.
	pub base_reward_quotient: Balance,
	/// Inactivity penalty quotient.
	pub inactivity_penalty_quotient: Balance,
	/// Includer reward quotient.
	pub includer_reward_quotient: Balance,
	/// Min attestation inclusion delay.
	pub min_attestation_inclusion_delay: Balance,
	/// Whistleblower reward quotient.
	pub whistleblower_reward_quotient: Balance,
}

/// Reward action.
pub enum RewardAction<Balance> {
	/// Add balance to reward.
	Add(Balance),
	/// Sub balance to reward. Should wrap at zero.
	Sub(Balance),
	/// Sub balance and exit the validator.
	Penalize(Balance),
}

fn integer_sqrt<Balance>(n: Balance) -> Balance where
	Balance: Add<Output=Balance> + Div<Output=Balance> + Ord + PartialOrd + Clone + From<u8>,
{
	let mut x = n.clone();
	let mut y = (x.clone() + From::from(1u8)) / From::from(2u8);
	while y < x {
		x = y.clone();
		y = (x.clone() + n.clone() / x.clone()) / From::from(2u8);
	}
	x
}

fn combined_validators<ValidatorId, T>(
	rewards: &[(ValidatorId, T)],
	a: &T,
	b: &T,
) -> Vec<ValidatorId> where
	ValidatorId: Clone,
	T: Eq + PartialEq,
{
	let mut ret = Vec::new();
	for (validator_id, reward_type) in rewards {
		if reward_type == a || reward_type == b {
			ret.push(validator_id.clone());
		}
	}
	ret
}

/// Use default scheme for reward calculation. This only contains justification and finalization rewards.
pub fn default_scheme_rewards<S, Slot>(
	store: &S,
	beacon_rewards: &[(ValidatorStoreValidatorId<S>, BeaconRewardType<Slot>)],
	casper_rewards: &[(ValidatorStoreValidatorId<S>, CasperRewardType)],
	epochs_since_finality: ValidatorStoreEpoch<S>,
	config: &DefaultSchemeConfig<ValidatorStoreBalance<S>>,
) -> Vec<(ValidatorStoreValidatorId<S>, RewardAction<ValidatorStoreBalance<S>>)> where
	S: ValidatorStore,
	S: BlockStore<Epoch=ValidatorStoreEpoch<S>>,
	Slot: Eq + PartialEq + Clone,
	ValidatorStoreBalance<S>: From<ValidatorStoreEpoch<S>> + From<Slot>,
	ValidatorStoreEpoch<S>: From<u8>,
{
	let previous_epoch = store.previous_epoch();
	let previous_active_validators = store.active_validators(previous_epoch);
	let previous_total_balance = store.total_balance(&previous_active_validators);

	let base_reward = |validator_id: ValidatorStoreValidatorId<S>| {
		store.total_balance(&[validator_id]) / (integer_sqrt(previous_total_balance) / config.base_reward_quotient) / From::from(5u8)
	};

	let mut rewards = Vec::new();

	if epochs_since_finality <= From::from(4u8) {
		let beacon_total_head = combined_validators(
			beacon_rewards,
			&BeaconRewardType::ExpectedHead,
			&BeaconRewardType::NoExpectedHead,
		);
		let beacon_total_balance = store.total_balance(&beacon_total_head);

		let casper_source_total_head = combined_validators(
			casper_rewards,
			&CasperRewardType::ExpectedSource,
			&CasperRewardType::NoExpectedSource,
		);
		let casper_source_total_balance = store.total_balance(&casper_source_total_head);

		let casper_target_total_head = combined_validators(
			casper_rewards,
			&CasperRewardType::ExpectedTarget,
			&CasperRewardType::NoExpectedTarget,
		);
		let casper_target_total_balance = store.total_balance(&casper_target_total_head);

		for (validator_id, reward_type) in beacon_rewards {
			if reward_type == &BeaconRewardType::ExpectedHead {
				rewards.push((validator_id.clone(), RewardAction::Add(base_reward(validator_id.clone()) * beacon_total_balance / previous_total_balance)));
			}

			if reward_type == &BeaconRewardType::NoExpectedHead {
				rewards.push((validator_id.clone(), RewardAction::Sub(base_reward(validator_id.clone()))));
			}

			if let BeaconRewardType::InclusionDistance(ref distance) = reward_type {
				rewards.push((validator_id.clone(), RewardAction::Add(base_reward(validator_id.clone()) / config.min_attestation_inclusion_delay / From::from(distance.clone()))));
			}
		}

		for (validator_id, reward_type) in casper_rewards {
			if reward_type == &CasperRewardType::ExpectedSource {
				rewards.push((validator_id.clone(), RewardAction::Add(base_reward(validator_id.clone()) * casper_source_total_balance / previous_total_balance)));
			}

			if reward_type == &CasperRewardType::NoExpectedSource {
				rewards.push((validator_id.clone(), RewardAction::Sub(base_reward(validator_id.clone()))));
			}

			if reward_type == &CasperRewardType::ExpectedTarget {
				rewards.push((validator_id.clone(), RewardAction::Add(base_reward(validator_id.clone()) * casper_target_total_balance / previous_total_balance)));
			}

			if reward_type == &CasperRewardType::NoExpectedTarget {
				rewards.push((validator_id.clone(), RewardAction::Sub(base_reward(validator_id.clone()))));
			}
		}
	} else {
		let inactivity_penalty = |validator_id: ValidatorStoreValidatorId<S>| {
			base_reward(validator_id.clone()) + store.total_balance(&[validator_id]) * From::from(epochs_since_finality) / config.inactivity_penalty_quotient / From::from(2u8)
		};

		for (validator_id, reward_type) in beacon_rewards {
			if reward_type == &BeaconRewardType::NoExpectedHead {
				rewards.push((validator_id.clone(), RewardAction::Sub(inactivity_penalty(validator_id.clone()))));
			}

			if let BeaconRewardType::InclusionDistance(ref distance) = reward_type {
				rewards.push((validator_id.clone(), RewardAction::Sub(base_reward(validator_id.clone()) - base_reward(validator_id.clone()) * config.min_attestation_inclusion_delay / From::from(distance.clone()))));
			}
		}

		for (validator_id, reward_type) in casper_rewards {
			if reward_type == &CasperRewardType::NoExpectedSource {
				rewards.push((validator_id.clone(), RewardAction::Sub(base_reward(validator_id.clone()))));
			}

			if reward_type == &CasperRewardType::NoExpectedSource {
				rewards.push((validator_id.clone(), RewardAction::Sub(inactivity_penalty(validator_id.clone()))));
			}
		}
	}

	rewards
}

/// Use default scheme for penalization.
pub fn default_scheme_penalties<S>(
	store: &S,
	whistleblower: &ValidatorStoreValidatorId<S>,
	slashings: &[ValidatorStoreValidatorId<S>],
	epochs_since_finality: ValidatorStoreEpoch<S>,
	config: &DefaultSchemeConfig<ValidatorStoreBalance<S>>,
) -> Vec<(ValidatorStoreValidatorId<S>, RewardAction<ValidatorStoreBalance<S>>)> where
	S: ValidatorStore,
	S: BlockStore<Epoch=ValidatorStoreEpoch<S>>,
	ValidatorStoreBalance<S>: From<ValidatorStoreEpoch<S>>,
	ValidatorStoreEpoch<S>: From<u8>,
{
	let mut rewards = Vec::new();

	for validator_id in slashings {
		let whistleblower_reward = store.total_balance(&[validator_id.clone()]) / config.whistleblower_reward_quotient;

		rewards.push((whistleblower.clone(), RewardAction::Add(whistleblower_reward)));
		rewards.push((validator_id.clone(), RewardAction::Penalize(whistleblower_reward)));
	}

	if epochs_since_finality > From::from(4u8) {
		let previous_epoch = store.previous_epoch();
		let previous_active_validators = store.active_validators(previous_epoch);
		let previous_total_balance = store.total_balance(&previous_active_validators);

		let base_reward = |validator_id: ValidatorStoreValidatorId<S>| {
			store.total_balance(&[validator_id]) / (integer_sqrt(previous_total_balance) / config.base_reward_quotient) / From::from(5u8)
		};

		let inactivity_penalty = |validator_id: ValidatorStoreValidatorId<S>| {
			base_reward(validator_id.clone()) + store.total_balance(&[validator_id]) * From::from(epochs_since_finality) / config.inactivity_penalty_quotient / From::from(2u8)
		};

		for validator_id in previous_active_validators {
			if !slashings.contains(&validator_id) {
				rewards.push((validator_id.clone(), RewardAction::Sub(inactivity_penalty(validator_id.clone()) * From::from(2u8) + base_reward(validator_id.clone()))));
			}
		}
	}

	rewards
}
