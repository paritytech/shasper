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

pub trait Trait: system::Trait + core::fmt::Debug {
	/// The overarching event type.
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	/// A stable ID for a validator.
	type ValidatorId: Member + Parameter;
}

decl_storage! {
	trait Store for Module<T: Trait> as Casper {
		pub SessionBlockHash get(fn session_block_hash)
			build(|_| vec![]): map SessionIndex => T::Hash;

		CurrentEpochAttestations get(current_epoch_attestations)
			build(|_| Vec::new()): Vec<Attestation<T>>;
		CurrentEpochAttestationsCount get(current_epoch_attestations_count)
			build(|_| 0u32): u32;
		PreviousEpochAttestations get(previous_epoch_attestations)
			build(|_| Vec::new()): Vec<Attestation<T>>;
		PreviousEpochAttestationsCount get(previous_epoch_attestations_count)
			build(|_| 0u32): u32;
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
