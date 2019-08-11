// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

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

//! # Casper Consensus Module

#![cfg_attr(not(feature = "std"), no_std)]

use srml_support::{StorageValue, dispatch::Result, decl_module, decl_storage, decl_event, print};
use system::ensure_none;
use sr_primitives::{traits::{One, MaybeDebug, Extrinsic as ExtrinsicT, ValidateUnsigned}, weights::SimpleDispatchInfo};
use sr_primitives::transaction_validity::{TransactionValidity, TransactionLongevity, ValidTransaction};
use casper_primitives::{ValidatorId, ValidatorSignature, ValidatorWeight, ValidatorIndex};
use codec::{Encode, Decode};
use app_crypto::RuntimeAppPublic;
use rstd::prelude::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Checkpoint<T: Trait> {
	pub number: T::BlockNumber,
	pub hash: T::Hash,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Attestation<T: Trait> {
	pub validator_index: ValidatorIndex,
	pub source: Checkpoint<T>,
	pub target: Checkpoint<T>,
}

/// Casper module's configuration trait.
pub trait Trait: session::Trait + MaybeDebug {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// The function call.
	type Call: From<Call<Self>>;
	/// A extrinsic right from the external world. This is unchecked and so
	/// can contain a signature.
	type UncheckedExtrinsic: ExtrinsicT<Call=<Self as Trait>::Call> + Encode + Decode + MaybeDebug;
}

decl_storage! {
	trait Store for Module<T: Trait> as Casper {
		Validators get(validators) config(): Vec<(ValidatorId, ValidatorWeight)>;
		CurrentEpochNumber get(current_epoch_number) build(|_| 0.into()): T::BlockNumber;
		PreviousEpochNumber get(previous_epoch_number) build(|_| 0.into()): T::BlockNumber;
		PreviousJustifiedEpochNumber get(previous_justified_epoch_number)
			build(|_| 0.into()): T::BlockNumber;
		CurrentJustifiedEpochNumber get(current_justified_epoch_number)
			build(|_| 0.into()): T::BlockNumber;
		FinalizedEpochNumber get(finalized_epoch_number)
			build(|_| 0.into()): T::BlockNumber;

		CurrentEpochAttestations get(current_epoch_attestations)
			build(|_| Vec::new()): Vec<Attestation<T>>;
		CurrentEpochAttestationsCount get(current_epoch_attestations_count)
			build(|_| 0u32): u32;
		PreviousEpochAttestations get(previous_epoch_attestations)
			build(|_| Vec::new()): Vec<Attestation<T>>;
		PreviousEpochAttestationsCount get(previous_epoch_attestations_count)
			build(|_| 0u32): u32;

		JustificationBits get(justification_bits)
			build(|_| [false, false, false, false]): [bool; 4]
	}
}

decl_event!(
	/// Casper events.
	pub enum Event<T> where
		H = <T as system::Trait>::Hash,
		A = Attestation<T>,
	{
		/// On Casper justification happens.
		OnJustified(H),
		/// On Casper finalization happens.
		OnFinalized(H),
		/// On new previous attestation happens.
		OnNewPreviousEpochAttestation(A),
		/// On new current attestation happens.
		OnNewCurrentEpochAttestation(A),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		#[weight = SimpleDispatchInfo::FixedNormal(5_000_000)]
		fn attest(origin, attestation: Attestation<T>, signature: ValidatorSignature) -> Result {
			ensure_none(origin)?;

			let validators = Validators::get();
			let validator_id = validators[attestation.validator_index as usize].0.clone();
			if !validator_id.verify(&attestation.encode(), &signature) {
				return Err("invalid attestation signature")
			}

			let previous_epoch_number = <PreviousEpochNumber<T>>::get();
			let previous_epoch_hash = <system::Module<T>>::block_hash(previous_epoch_number);
			let current_epoch_number = <CurrentEpochNumber<T>>::get();
			let current_epoch_hash = <system::Module<T>>::block_hash(current_epoch_number);
			let previous_justified_epoch_number = <PreviousJustifiedEpochNumber<T>>::get();
			let previous_justified_epoch_hash =
				<system::Module<T>>::block_hash(previous_justified_epoch_number);
			let current_justified_epoch_number = <CurrentJustifiedEpochNumber<T>>::get();
			let current_justified_epoch_hash =
				<system::Module<T>>::block_hash(current_justified_epoch_number);

			if attestation.source.number == previous_justified_epoch_number &&
				attestation.source.hash == previous_justified_epoch_hash &&
				attestation.target.number == previous_epoch_number &&
				attestation.target.hash == previous_epoch_hash
			{
				let mut previous_epoch_attestations = <PreviousEpochAttestations<T>>::get();
				previous_epoch_attestations.push(attestation.clone());
				<PreviousEpochAttestationsCount>::put(previous_epoch_attestations.len() as u32);
				<PreviousEpochAttestations<T>>::put(previous_epoch_attestations);
				Self::deposit_event(RawEvent::OnNewPreviousEpochAttestation(attestation));
			} else if attestation.source.number == current_justified_epoch_number &&
				attestation.source.hash == current_justified_epoch_hash &&
				attestation.target.number == current_epoch_number &&
				attestation.target.hash == current_epoch_hash
			{
				let mut current_epoch_attestations = <CurrentEpochAttestations<T>>::get();
				current_epoch_attestations.push(attestation.clone());
				<CurrentEpochAttestationsCount>::put(current_epoch_attestations.len() as u32);
				<CurrentEpochAttestations<T>>::put(current_epoch_attestations);
				Self::deposit_event(RawEvent::OnNewCurrentEpochAttestation(attestation));
			} else {
				return Err("invalid attestation source or target")
			}

			Ok(())
		}

		fn on_initialize(_n: T::BlockNumber) { }

		fn on_finalize(_n: T::BlockNumber) { }

		fn offchain_worker(n: T::BlockNumber) {
			if sr_io::is_validator() {
				match Self::offchain(n) {
					Ok(()) => (),
					Err(e) => { print(e); },
				}
			}
		}
	}
}

impl<T: Trait> Module<T> {
	fn offchain(n: T::BlockNumber) -> Result {
		if n == (<CurrentEpochNumber<T>>::get() + One::one()) {
			let validators = Validators::get();
			let mut local_keys = ValidatorId::all();
			local_keys.sort();

			for (validator_index, key) in validators.into_iter()
				.enumerate()
				.filter_map(|(index, validator)| {
					local_keys.binary_search(&validator.0)
						.ok()
						.map(|location| (index as u32, &local_keys[location]))
				})
			{
				let source_number = <CurrentJustifiedEpochNumber<T>>::get();
				let target_number = <CurrentEpochNumber<T>>::get();

				let source = Checkpoint::<T> {
					number: source_number,
					hash: <system::Module<T>>::block_hash(source_number),
				};
				let target = Checkpoint::<T> {
					number: target_number,
					hash: <system::Module<T>>::block_hash(target_number),
				};
				let attestation = Attestation::<T> {
					validator_index: validator_index as u64,
					source, target
				};
				let signature = key.sign(&attestation.encode())
					.ok_or("attestation signing failed")?;

				let call = Call::attest(attestation, signature);
				let ex = T::UncheckedExtrinsic::new_unsigned(call.into())
					.ok_or("create unsigned attestation failed")?;

				sr_io::submit_transaction(&ex).map_err(|_| "submit attestation failed")?;
			}
		}

		Ok(())
	}

	pub fn current_justified_block() -> T::Hash {
		<system::Module<T>>::block_hash(<CurrentJustifiedEpochNumber<T>>::get())
	}

	pub fn previous_justified_block() -> T::Hash {
		<system::Module<T>>::block_hash(<PreviousJustifiedEpochNumber<T>>::get())
	}

	pub fn finalized_block() -> T::Hash {
		<system::Module<T>>::block_hash(<FinalizedEpochNumber<T>>::get())
	}
}

impl<T: Trait> session::OneSessionHandler<T::AccountId> for Module<T> {
	type Key = ValidatorId;

	fn on_new_session<'a, I: 'a>(changed: bool, new_validators: I, _queued_validators: I) where
		I: Iterator<Item=(&'a T::AccountId, ValidatorId)>
	{
		let validators = <Validators>::get();
		let total_balance = validators.iter().fold(0, |acc, v| acc + v.1);
		let previous_matching_target_balance = <PreviousEpochAttestations<T>>::get().iter()
			.fold(0, |acc, attestation| acc + validators[attestation.validator_index as usize].1);
		let current_matching_target_balance = <CurrentEpochAttestations<T>>::get().iter()
			.fold(0, |acc, attestation| acc + validators[attestation.validator_index as usize].1);

		let mut justification_bits = <JustificationBits>::get();
		let old_justification_bits = justification_bits.clone();
		justification_bits[1..].copy_from_slice(
			&old_justification_bits[0..3]
		);
		let old_previous_justified_epoch_number = <PreviousJustifiedEpochNumber<T>>::get();
		let old_current_justified_epoch_number = <CurrentJustifiedEpochNumber<T>>::get();
		<PreviousJustifiedEpochNumber<T>>::put(<CurrentJustifiedEpochNumber<T>>::get());

		if previous_matching_target_balance * 3 >= total_balance * 2 {
			<CurrentJustifiedEpochNumber<T>>::put(<PreviousEpochNumber<T>>::get());
			justification_bits[1] = true;
		}
		if current_matching_target_balance * 3 >= total_balance * 2 {
			<CurrentJustifiedEpochNumber<T>>::put(<CurrentEpochNumber<T>>::get());
			justification_bits[0] = true;
		}

		if justification_bits[1..3].iter().all(|v| *v) {
			<FinalizedEpochNumber<T>>::put(old_previous_justified_epoch_number);
		}

		if justification_bits[0..2].iter().all(|v| *v) {
			<FinalizedEpochNumber<T>>::put(old_current_justified_epoch_number);
		}

		<JustificationBits>::put(justification_bits);
		<PreviousEpochAttestations<T>>::put(Vec::new());
		<CurrentEpochAttestations<T>>::put(Vec::new());

		if changed {
			<Validators>::put(new_validators.map(|(_, k)| (k, 1u64)).collect::<Vec<_>>());
		}
		<PreviousEpochNumber<T>>::put(<CurrentEpochNumber<T>>::get());
		<CurrentEpochNumber<T>>::put(<system::Module<T>>::block_number());
	}

	fn on_disabled(i: usize) {
		let mut validators = <Validators>::get();
		validators[i].1 = 0;
		<Validators>::put(validators);
	}
}

impl<T: Trait> ValidateUnsigned for Module<T> {
	type Call = crate::Call<T>;

	fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
		match call {
			Call::attest(attestation, _) => TransactionValidity::Valid(ValidTransaction {
				priority: 0,
				requires: vec![],
				provides: vec![attestation.encode()],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			}),
			_ => TransactionValidity::Invalid(0),
		}
	}
}
