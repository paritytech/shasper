#[macro_use]
extern crate fixed_hash;

construct_fixed_hash! {
	/// Fixed 384-bit hash.
	pub struct H384(48);
}
