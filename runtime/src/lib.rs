extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate blake2_rfc as blake2;
extern crate parity_codec as codec;
#[macro_use]
extern crate parity_codec_derive;
extern crate hashdb;
extern crate plain_hasher;
extern crate tiny_keccak;
extern crate substrate_primitives as primitives;
extern crate substrate_client as client;
extern crate sr_primitives as runtime_primitives;
#[macro_use]
extern crate sr_io as runtime_io;
#[macro_use]
extern crate srml_support as runtime_support;

mod attestation;
mod hasher;
mod header;
mod utils;
mod state;
mod process;
mod validators;

pub use attestation::AttestationRecord;
pub use header::Header;
pub use state::{CrosslinkRecord, ShardAndCommittee, ActiveState, CrystallizedState};
pub use validators::{Validators, ValidatorRecord};

use primitives::{H256, H160};

pub type Hash = H256;
pub type BlockNumber = u64;
pub type Address = H160;
pub type Extrinsic = Vec<AttestationRecord>;
pub type Block = runtime_primitives::generic::Block<Header, Extrinsic>;

pub mod api {
	use process;
	impl_stubs!(
		initialise_block => |header| process::initialise_block(header)
	);
}
