#[macro_use]
mod macros;

mod validator_id {
	impl_beacon_fixed_hash!(H384, 48, typenum::U48);
	pub type ValidatorId = H384;
}

mod signature {
	impl_beacon_fixed_hash!(H768, 96, typenum::U96);
	pub type Signature = H768;
}

mod version {
	impl_beacon_fixed_hash!(H32, 4, typenum::U4);
	pub type Version = H32;
}

pub use self::validator_id::{ValidatorId, H384};
pub use self::signature::{Signature, H768};
pub use self::version::{Version, H32};

pub type Uint = u64;
pub use primitive_types::H256;

pub type Epoch = Uint;
pub type Slot = Uint;
pub type ValidatorIndex = Uint;
