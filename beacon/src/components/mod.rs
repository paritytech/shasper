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

}

pub trait Checkpoint: Clone {
	fn epoch(&self) -> Epoch;
}

pub trait Registry {
	type Checkpoint;
	type Validator: Validator<Checkpoint=Self::Checkpoint>;
	type Attestation;
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
	fn increase_balance(
		&mut self,
		index: ValidatorIndex,
		value: Balance,
	) -> Result<(), Self::Error>;
	fn decrease_balance(
		&mut self,
		index: ValidatorIndex,
		value: Balance,
	) -> Result<(), Self::Error>;
	fn validators(
		&self,
	) -> Result<Box<Iterator<Item=&dyn Self::Validator>>, Self::Error>;
}

pub struct JustificationProcessor<C: Checkpoint> {
	pub justification_bits: VecArray<bool, consts::JustificationBitsLength>,
	pub current_justified_checkpoint: C,
	pub previous_justified_checkpoint: C,
	pub finalized_checkpoint: C,
}

impl<C: Checkpoint> JustificationProcessor<C> {
	pub fn advance_epoch<R: Registry<Checkpoint=C>>(
		&mut self,
		previous_checkpoint: C,
		current_checkpoint: C,
		registry: &R
	) -> Result<(), R::Error> {
		let current_epoch = current_checkpoint.epoch();
		let old_previous_justified_checkpoint = self.previous_justified_checkpoint.clone();
		let old_current_justified_checkpoint = self.current_justified_checkpoint.clone();

		// Process justifications
		self.previous_justified_checkpoint = self.current_justified_checkpoint.clone();
		let old_justification_bits = self.justification_bits.clone();
		let justification_bits_len = self.justification_bits.len();
		self.justification_bits[1..].copy_from_slice(
			&old_justification_bits[0..(justification_bits_len - 1)]
		);
		self.justification_bits[0] = false;

		if registry.attesting_target_balance(&previous_checkpoint)? * 3 >=
			registry.total_active_balance() * 2
		{
			self.current_justified_checkpoint = previous_checkpoint;
			self.justification_bits[1] = true;
		}
		if registry.attesting_target_balance(&current_checkpoint)? * 3 >=
			registry.total_active_balance() * 2
		{
			self.current_justified_checkpoint = current_checkpoint;
			self.justification_bits[0] = true;
		}

		// Process finalizations
		let bits = self.justification_bits.clone();
		// The 2nd/3rd/4th most recent epochs are justified,
		// the 2nd using the 4th as source
		if bits[1..4].iter().all(|v| *v) &&
			old_previous_justified_checkpoint.epoch() + 3 == current_epoch
		{
			self.finalized_checkpoint = old_previous_justified_checkpoint.clone();
		}
		// The 2nd/3rd most recent epochs are justified,
		// the 2nd using the 3rd as source
		if bits[1..3].iter().all(|v| *v) &&
			old_previous_justified_checkpoint.epoch() + 2 == current_epoch
		{
			self.finalized_checkpoint = old_previous_justified_checkpoint.clone();
		}
		// The 1st/2nd/3rd most recent epochs are justified,
		// the 1st using the 3rd as source
		if bits[0..3].iter().all(|v| *v) &&
			old_current_justified_checkpoint.epoch() + 2 == current_epoch
		{
			self.finalized_checkpoint = old_current_justified_checkpoint.clone();
		}
		// The 1st/2nd most recent epochs are justified,
		// the 1st using the 2nd as source
		if bits[0..2].iter().all(|v| *v) &&
			old_current_justified_checkpoint.epoch() + 1 == current_epoch
		{
			self.finalized_checkpoint = old_current_justified_checkpoint.clone();
		}

		Ok(())
	}
}
