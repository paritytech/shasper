use codec::{Encode, Decode};
use sp_runtime::{RuntimeDebug, traits::Member};
use sp_staking::SessionIndex;
use support::{decl_module, decl_storage, decl_event, dispatch::Result, Parameter};
use system::ensure_none;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Checkpoint<T: Trait> {
	pub session_index: SessionIndex,
	pub hash: T::Hash,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Attestation<T: Trait> {
	pub validator_ids: Vec<T::ValidatorId>,
	pub source: Checkpoint<T>,
	pub target: Checkpoint<T>,
}

pub trait Trait: session::Trait + core::fmt::Debug {
	/// The overarching event type.
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	/// Validator weighter;
	type WeighValidators: WeighValidators<Self>;
}

pub trait WeighValidators<T: Trait> {
	fn weigh_validators(validators: Vec<T::ValidatorId>) -> Vec<(T::ValidatorId, u128)>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Casper {
		pub SessionBlockHash get(fn session_block_hash)
			build(|_| vec![]): map SessionIndex => T::Hash;

		CurrentSessionAttestations get(current_session_attestations)
			build(|_| Vec::new()): Vec<Attestation<T>>;
		PreviousSessionAttestations get(previous_session_attestations)
			build(|_| Vec::new()): Vec<Attestation<T>>;
	}
}

decl_event! {
	pub enum Event {

	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn more_attestations(
			origin,
			attestations: Vec<Attestation<T>>,
		) -> Result {
			ensure_none(origin)?;

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn push_attestation(attestation: Attestation<T>) -> Result {
		let current_session_index = session::Module::<T>::current_index();

		if current_session_index == attestation.target.session_index {
			CurrentSessionAttestations::<T>::mutate(|attestations| {
				attestations.push(attestation)
			});
		} else {
			PreviousSessionAttestations::<T>::mutate(|attestations| {
				attestations.push(attestation)
			});
		}

		Ok(())
	}
}
