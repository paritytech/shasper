#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate core;

extern crate arrayvec;
extern crate parity_codec;
extern crate substrate_primitives as primitives;

#[cfg(feature = "std")]
pub mod alloc {
	pub use std::boxed;
	pub use std::vec;
}

mod codec;

pub use self::codec::{Input, Output, Encode, Decode, Codec};
