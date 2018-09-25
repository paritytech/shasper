#![cfg_attr(not(feature = "std"), no_std)]

extern crate blake2_rfc as blake2;
extern crate parity_codec as codec;
#[macro_use]
extern crate parity_codec_derive;
extern crate hashdb;
extern crate plain_hasher;
extern crate tiny_keccak;

#[cfg(feature = "std")]
extern crate serde;

#[cfg(feature = "std")]
#[macro_use]
extern crate serde_derive;

extern crate substrate_primitives as primitives;
extern crate sr_std as rstd;
extern crate sr_primitives as runtime_primitives;
#[macro_use]
extern crate sr_io as runtime_io;
#[macro_use]
extern crate sr_version as runtime_version;
#[macro_use]
extern crate srml_support as runtime_support;

mod attestation;
mod extrinsic;
mod hasher;
mod header;
mod utils;
mod state;
mod system;
mod validators;
mod consts;

pub use attestation::AttestationRecord;
pub use header::Header;
pub use extrinsic::Extrinsic;
pub use state::{CrosslinkRecord, ActiveState, CrystallizedState};
pub use validators::{Validators, ValidatorRecord, ShardAndCommittee};
pub use hasher::KeccakHasher;

use primitives::{H256, H160};

use rstd::prelude::*;
use runtime_version::RuntimeVersion;

/// Shasper runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: ver_str!("shasper"),
	impl_name: ver_str!("parity-shasper"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: apis_vec!([]),
};

fn version() -> RuntimeVersion {
	VERSION
}

pub type Hash = H256;
pub type BlockNumber = u64;
pub type Address = H160;
pub type Block = runtime_primitives::generic::Block<Header, Extrinsic>;

pub mod api {
	use system;
	impl_stubs!(
		initialise_block => |header| system::initialise_block(header),
		execute_block => |block| system::execute_block(block),
		version => |()| ::version()
	);
}
