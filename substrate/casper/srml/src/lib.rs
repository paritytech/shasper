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

use srml_support::{StorageValue, dispatch::Result, decl_module, decl_storage, decl_event};
use system::{ensure_signed, ensure_root};
use sr_primitives::weights::SimpleDispatchInfo;
use casper_primitives::{ValidatorId, ValidatorWeight};

/// Casper module's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Casper {
		Validators get(validators) config(): Vec<(ValidatorId, ValidatorWeight)>;
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

		// The signature could also look like: `fn on_initialize()`
		fn on_initialize(_n: T::BlockNumber) {
			// Anything that needs to be done at the start of the block.
			// We don't do anything here.
		}

		// The signature could also look like: `fn on_finalize()`
		fn on_finalize(_n: T::BlockNumber) {
			// Anything that needs to be done at the end of the block.
			// We just kill our dummy storage item.
		}

		// A runtime code run after every block and have access to extended set of APIs.
		//
		// For instance you can generate extrinsics for the upcoming produced block.
		fn offchain_worker(_n: T::BlockNumber) {
			// We don't do anything here.
			// but we could dispatch extrinsic (transaction/unsigned/inherent) using
			// runtime_io::submit_extrinsic
		}
	}
}

// The main implementation block for the module. Functions here fall into three broad
// categories:
// - Public interface. These are functions that are `pub` and generally fall into inspector
// functions that do not write to storage and operation functions that do.
// - Private functions. These are your usual private utilities unavailable to other modules.
impl<T: Trait> Module<T> {

}

impl<T: Trait> session::OneSessionHandler<T::AccountId> for Module<T> {
	type Key = ValidatorId;

	fn on_new_session<'a, I: 'a>(changed: bool, validators: I, _queued_validators: I) where
		I: Iterator<Item=(&'a T::AccountId, ValidatorId)>
	{
		if changed {
			<Validators>::put(validators.map(|(_, k)| (k, 1u64)).collect::<Vec<_>>());
		}
	}

	fn on_disabled(i: usize) {
		let mut validators = <Validators>::get();
		validators[i].1 = 0;
		<Validators>::put(validators);
	}
}
