use alloc::collections::BTreeMap;
use super::{Registry, Epoch, Checkpoint, Validator, Attestation, ValidatorIndex, Balance};
use crate::{Config, utils, consts};

pub fn base<R: Registry, C: Config>(
	registry: &R,
	index: ValidatorIndex,
) -> Result<u64, R::Error> {
	let total_balance = registry.total_active_balance();
	let effective_balance = registry.effective_balance(index)?;

	Ok(effective_balance * C::base_reward_factor() /
	   utils::integer_squareroot(total_balance) /
	   consts::BASE_REWARDS_PER_EPOCH)
}

pub fn process<R: Registry, C: Config>(
	registry: &mut R,
	previous_checkpoint: R::Checkpoint,
	finalized_checkpoint: R::Checkpoint,
) -> Result<(), R::Error> {
	let total_balance = registry.total_active_balance();
	let mut rewards: BTreeMap<ValidatorIndex, Balance> = BTreeMap::new();
	let mut penalties: BTreeMap<ValidatorIndex, Balance> = BTreeMap::new();

	// Micro-incentives for matching FFG source, FFG target, and head
	macro_rules! micro_incentives {
		( $balance:tt, $validators:tt ) => {
			let attesting_balance = registry.$balance(&previous_checkpoint)?;
			for validator in registry.validators()? {
				let index = validator.index();

				if validator.is_eligible(&previous_checkpoint) {
					if registry.$validators(&previous_checkpoint)?
						.find(|v| v.index() == index).is_some()
					{
						*rewards.entry(index).or_default() += base::<_, C>(registry, index)? *
							attesting_balance / total_balance;
					} else {
						*penalties.entry(index).or_default() += base::<_, C>(registry, index)?;
					}
				}
			}
		}
	}

	micro_incentives!(unslashed_attesting_balance, unslashed_attesting_validators);
	micro_incentives!(unslashed_attesting_target_balance, unslashed_attesting_target_validators);
	micro_incentives!(unslashed_attesting_matching_head_balance,
					  unslashed_attesting_matching_head_validators);

	// Proposer and inclusion delay micro-rewards
	for validator in registry.unslashed_attesting_validators(&previous_checkpoint)? {
		let index = validator.index();
		let attestation = registry.min_inclusion_delay_attestation(
			&previous_checkpoint, index
		)?;

		if let Some(attestation) = attestation {
			let proposer_reward = base::<_, C>(registry, index)? / C::proposer_reward_quotient();
			*rewards.entry(attestation.proposer_index()).or_default() += proposer_reward;
			let max_attester_reward = base::<_, C>(registry, index)? - proposer_reward;
			*rewards.entry(index).or_default() += max_attester_reward /
				attestation.inclusion_delay();
		}
	}

	// Inactivity penalty
	let finality_delay = previous_checkpoint.epoch() - finalized_checkpoint.epoch();
	if finality_delay > C::min_epochs_to_inactivity_penalty() {
		for validator in registry.validators()? {
			let index = validator.index();

			if validator.is_eligible(&previous_checkpoint) {
				*penalties.entry(index).or_default() += base::<_, C>(registry, index)? *
					consts::BASE_REWARDS_PER_EPOCH;

				if !registry.unslashed_attesting_target_validators(&previous_checkpoint)?
					.find(|v| v.index() == index).is_some()
				{
					*penalties.entry(index).or_default() +=
						registry.effective_balance(index)? * finality_delay /
						C::inactivity_penalty_quotient();
				}
			}
		}
	}

	for (index, delta) in rewards {
		registry.increase_balance(index, delta);
	}

	for (index, delta) in penalties {
		registry.decrease_balance(index, delta);
	}

	Ok(())
}
