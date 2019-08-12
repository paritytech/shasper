// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
mod macros;

mod validator_id {
	impl_beacon_fixed_hash!(H384, 48, typenum::U48);
	/// Validator id.
	pub type ValidatorId = H384;
}

mod signature {
	impl_beacon_fixed_hash!(H768, 96, typenum::U96);
	/// Signature.
	pub type Signature = H768;
}

mod version {
	impl_beacon_fixed_hash!(H32, 4, typenum::U4);
	/// Version.
	pub type Version = H32;
}

pub use self::validator_id::{ValidatorId, H384};
pub use self::signature::{Signature, H768};
pub use self::version::{Version, H32};

/// Integer type for beacon chain.
pub type Uint = u64;
pub use primitive_types::H256;

/// Epoch.
pub type Epoch = Uint;
/// Slot.
pub type Slot = Uint;
/// Validator index.
pub type ValidatorIndex = Uint;
/// Shard.
pub type Shard = Uint;
/// Gwei.
pub type Gwei = Uint;
