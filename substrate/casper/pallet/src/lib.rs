#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Checkpoint<T: Trait> {
	pub epoch: Epoch,
	pub hash: T::Hash,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Attestation<T: Trait> {
	pub validator_id: ValidatorId,
	pub source: Checkpoint<T>,
	pub target: Checkpoint<T>,
}

pub trait Trait {

}

decl_storage! {
	trait Store for Module<T: Trait> as Casper {


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
