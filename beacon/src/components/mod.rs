mod reward;
mod justification;

pub use self::justification::Justifier;

use vecarray::VecArray;
use crate::consts;

type Epoch = u64;
type Balance = u64;
type ValidatorIndex = u64;

pub trait Validator {
	type Checkpoint;

	fn index(&self) -> ValidatorIndex;
	fn is_eligible(&self, checkpoint: &Self::Checkpoint) -> bool;
}

pub trait Attestation {
	fn proposer_index(&self) -> ValidatorIndex;
	fn inclusion_delay(&self) -> u64;
}

pub trait Checkpoint: Clone {
	fn epoch(&self) -> Epoch;
}

pub trait Registry {
	type Checkpoint: Checkpoint;
	type Validator: Validator<Checkpoint=Self::Checkpoint>;
	type Attestation: Attestation;
	type Error;

	fn total_active_balance(&self) -> Balance;
	fn attesting_target_balance(
		&self,
		source_checkpoint: &Self::Checkpoint
	) -> Result<Balance, Self::Error>;
	fn min_inclusion_delay_attestation(
		&self,
		source_checkpoint: &Self::Checkpoint,
		index: ValidatorIndex,
	) -> Result<Option<&Self::Attestation>, Self::Error>;

	fn unslashed_attesting_balance(
		&self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<Balance, Self::Error>;
	fn unslashed_attesting_validators(
		&self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<Box<dyn Iterator<Item=&Self::Validator>>, Self::Error>;
	fn unslashed_attesting_target_balance(
		&self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<Balance, Self::Error>;
	fn unslashed_attesting_target_validators(
		&self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<Box<dyn Iterator<Item=&Self::Validator>>, Self::Error>;
	fn unslashed_attesting_matching_head_balance(
		&self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<Balance, Self::Error>;
	fn unslashed_attesting_matching_head_validators(
		&self,
		source_checkpoint: &Self::Checkpoint,
	) -> Result<Box<dyn Iterator<Item=&Self::Validator>>, Self::Error>;

	fn balance(
		&self,
		index: ValidatorIndex
	) -> Result<Balance, Self::Error>;
	fn effective_balance(
		&self,
		index: ValidatorIndex,
	) -> Result<Balance, Self::Error>;
	fn increase_balance(
		&mut self,
		index: ValidatorIndex,
		value: Balance,
	);
	fn decrease_balance(
		&mut self,
		index: ValidatorIndex,
		value: Balance,
	);
	fn validators(
		&self,
	) -> Result<Box<dyn Iterator<Item=&Self::Validator>>, Self::Error>;
}
