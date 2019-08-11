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
		PreviousEpochNumber get(previous_epoch_number) build(|_| 0.into()): T::BlockNumber;
		PreviousJustifiedEpochNumber get(previous_justified_epoch_number)
			build(|_| 0.into()): T::BlockNumber;
		PreviousFinalizedEpochNumber get(previous_finalized_epoch_number)
			build(|_| 0.into()): T::BlockNumber;
	}
}

decl_event!(
	/// Casper events.
	pub enum Event<T> where H = <T as system::Trait>::Hash {
		/// On Casper justification happens.
		OnJustified(H),
		/// On Casper finalization happens.
		OnFinalized(H),
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
		if n == (<PreviousEpochNumber<T>>::get() + One::one()) {
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
				let source_number = <PreviousJustifiedEpochNumber<T>>::get();
				let target_number = n;

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
}

impl<T: Trait> session::OneSessionHandler<T::AccountId> for Module<T> {
	type Key = ValidatorId;

	fn on_new_session<'a, I: 'a>(changed: bool, validators: I, _queued_validators: I) where
		I: Iterator<Item=(&'a T::AccountId, ValidatorId)>
	{
		if changed {
			<Validators>::put(validators.map(|(_, k)| (k, 1u64)).collect::<Vec<_>>());
		}
		<PreviousEpochNumber<T>>::put(<system::Module<T>>::block_number());
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
