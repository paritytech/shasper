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
use core::cmp;
use core::ops::{Add, Div};
use crate::casper::CasperProcess;
use crate::store::{ValidatorStore, PendingAttestationsStore, BlockStore};
use crate::context::{
	Attestation, ValidatorIdOf, EpochOf, BalanceContext, BalanceOf,
	SlotOf, SlotContext, AttestationOf, SlotAttestation,
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
pub enum BeaconRewardType<C: SlotContext> where
	AttestationOf<C>: SlotAttestation,
{
	/// The validator attested on the expected head.
	ExpectedHead,
	/// The validator is active, but does not attest on the epxected head.
	NoExpectedHead,
	/// Inclusion distance for attestations.
	InclusionDistance(SlotOf<C>),
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
pub fn beacon_rewards<C: SlotContext, S>(
	store: &S
) -> Vec<(ValidatorIdOf<C>, BeaconRewardType<C>)> where
	AttestationOf<C>: SlotAttestation,
	S: PendingAttestationsStore<C> + BlockStore<C> + ValidatorStore<C>,
{
	let mut no_expected_head_validators = store.active_validators(store.previous_epoch()).into_iter().collect::<Vec<_>>();

	let mut rewards = Vec::new();
	for attestation in store.attestations() {
		if attestation.target_epoch() == store.previous_epoch() {
			push_rewards(&mut rewards, &attestation, BeaconRewardType::InclusionDistance(attestation.inclusion_distance()));

			if attestation.is_slot_canon() {
				push_rewards(&mut rewards, &attestation, BeaconRewardType::ExpectedHead);
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
pub fn casper_rewards<C: BalanceContext, S>(
	context: &CasperProcess<C>,
	store: &S
) -> Vec<(ValidatorIdOf<C>, CasperRewardType)> where
	S: PendingAttestationsStore<C> + BlockStore<C> + ValidatorStore<C>,
{
	let mut no_expected_source_validators = store.active_validators(context.previous_epoch()).into_iter().collect::<Vec<_>>();
	let mut no_expected_target_validators = no_expected_source_validators.clone();

	let mut rewards = Vec::new();
	for attestation in store.attestations() {
		if attestation.target_epoch() == store.previous_epoch() {
			push_rewards(&mut rewards, &attestation, CasperRewardType::ExpectedSource);
			no_expected_source_validators.retain(|validator_id| {
				!attestation.validator_ids().into_iter().any(|v| v == *validator_id)
			});

			if attestation.is_target_canon() {
				push_rewards(&mut rewards, &attestation, CasperRewardType::ExpectedTarget);
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
pub struct DefaultSchemeConfig<C: BalanceContext + SlotContext> where
	AttestationOf<C>: SlotAttestation,
{
	/// Base reward quotient.
	pub base_reward_quotient: BalanceOf<C>,
	/// Inactivity penalty quotient.
	pub inactivity_penalty_quotient: BalanceOf<C>,
	/// Includer reward quotient.
	pub includer_reward_quotient: BalanceOf<C>,
	/// Min attestation inclusion delay.
	pub min_attestation_inclusion_delay: SlotOf<C>,
	/// Whistleblower reward quotient.
	pub whistleblower_reward_quotient: BalanceOf<C>,
}

/// Reward action.
pub enum RewardAction<C: BalanceContext> {
	/// Add balance to reward.
	Add(BalanceOf<C>),
	/// Sub balance to reward. Should wrap at zero.
	Sub(BalanceOf<C>),
	/// Sub balance and exit the validator.
	Penalize(BalanceOf<C>),
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
pub fn default_scheme_rewards<C: BalanceContext + SlotContext, S>(
	store: &S,
	beacon_rewards: &[(ValidatorIdOf<C>, BeaconRewardType<C>)],
	casper_rewards: &[(ValidatorIdOf<C>, CasperRewardType)],
	epochs_since_finality: EpochOf<C>,
	config: &DefaultSchemeConfig<C>,
) -> Vec<(ValidatorIdOf<C>, RewardAction<C>)> where
	AttestationOf<C>: SlotAttestation,
	EpochOf<C>: From<u8>,
	BalanceOf<C>: From<EpochOf<C>> + From<SlotOf<C>>,
	S: ValidatorStore<C> + BlockStore<C>,
{
	let previous_epoch = store.previous_epoch();
	let previous_active_validators = store.active_validators(previous_epoch).into_iter().collect::<Vec<_>>();
	let previous_total_balance = store.total_balance(&previous_active_validators);

	let base_reward = |validator_id: ValidatorIdOf<C>| {
		let quotient = cmp::max(One::one(), integer_sqrt(previous_total_balance) / config.base_reward_quotient);
		store.total_balance(&[validator_id]) / quotient / From::from(5u8)
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
				let distance = if *distance == Zero::zero() { One::one() } else { *distance };
				let min_attestation_inclusion_delay = if config.min_attestation_inclusion_delay == Zero::zero() { One::one() } else { config.min_attestation_inclusion_delay };

				rewards.push((validator_id.clone(), RewardAction::Add(base_reward(validator_id.clone()) / From::from(min_attestation_inclusion_delay) / From::from(distance))));
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
		let inactivity_penalty = |validator_id: ValidatorIdOf<C>| {
			base_reward(validator_id.clone()) + store.total_balance(&[validator_id]) * From::from(epochs_since_finality) / config.inactivity_penalty_quotient / From::from(2u8)
		};

		for (validator_id, reward_type) in beacon_rewards {
			if reward_type == &BeaconRewardType::NoExpectedHead {
				rewards.push((validator_id.clone(), RewardAction::Sub(inactivity_penalty(validator_id.clone()))));
			}

			if let BeaconRewardType::InclusionDistance(ref distance) = reward_type {
				let distance = if *distance == Zero::zero() { One::one() } else { *distance };
				let min_attestation_inclusion_delay = if config.min_attestation_inclusion_delay == Zero::zero() { One::one() } else { config.min_attestation_inclusion_delay };

				rewards.push((validator_id.clone(), RewardAction::Sub(base_reward(validator_id.clone()) - base_reward(validator_id.clone()) * From::from(min_attestation_inclusion_delay) / From::from(distance))));
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
pub fn default_scheme_penalties<C: BalanceContext + SlotContext, S>(
	store: &S,
	whistleblower: &ValidatorIdOf<C>,
	slashings: &[ValidatorIdOf<C>],
	epochs_since_finality: EpochOf<C>,
	config: &DefaultSchemeConfig<C>,
) -> Vec<(ValidatorIdOf<C>, RewardAction<C>)> where
	AttestationOf<C>: SlotAttestation,
	EpochOf<C>: From<u8>,
	BalanceOf<C>: From<EpochOf<C>> + From<SlotOf<C>>,
	S: ValidatorStore<C> + BlockStore<C>,
{
	let mut rewards = Vec::new();

	for validator_id in slashings {
		let whistleblower_reward = store.total_balance(&[validator_id.clone()]) / config.whistleblower_reward_quotient;

		rewards.push((whistleblower.clone(), RewardAction::Add(whistleblower_reward)));
		rewards.push((validator_id.clone(), RewardAction::Penalize(whistleblower_reward)));
	}

	if epochs_since_finality > From::from(4u8) {
		let previous_epoch = store.previous_epoch();
		let previous_active_validators = store.active_validators(previous_epoch).into_iter().collect::<Vec<_>>();
		let previous_total_balance = store.total_balance(&previous_active_validators);

		let base_reward = |validator_id: ValidatorIdOf<C>| {
			store.total_balance(&[validator_id]) / (integer_sqrt(previous_total_balance) / config.base_reward_quotient) / From::from(5u8)
		};

		let inactivity_penalty = |validator_id: ValidatorIdOf<C>| {
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
