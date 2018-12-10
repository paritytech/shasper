extern crate substrate_primitives;
#[macro_use]
extern crate fixed_hash;

construct_fixed_hash! {
	/// Fixed 384-bit hash.
	pub struct H384(48);
}

pub use substrate_primitives::H256;

pub type Count = u64;
pub type Slot = u64;
