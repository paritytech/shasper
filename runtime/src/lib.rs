extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate parity_codec as codec;
#[macro_use]
extern crate parity_codec_derive;
extern crate hashdb;
extern crate plain_hasher;
extern crate tiny_keccak;
extern crate substrate_primitives as primitives;
extern crate substrate_client as client;
extern crate sr_primitives as runtime_primitives;
extern crate sr_io as runtime_io;

mod attestation;
mod hasher;
mod header;
mod utils;
mod state;

pub use attestation::AttestationRecord;
pub use header::Header;
pub use state::{ValidatorRecord, CrosslinkRecord, ShardAndCommittee, ActiveState, CrystallizedState};

use primitives::H160;

pub type BlockNumber = u64;
pub type Address = H160;
pub type Extrinsic = Vec<AttestationRecord>;
pub type Block = runtime_primitives::generic::Block<Header, Extrinsic>;
