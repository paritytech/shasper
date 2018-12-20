#![cfg_attr(not(feature = "std"), no_std)]

extern crate bls as bls_crate;
extern crate pairing;

pub mod bls {
	use bls_crate;
	use pairing::bls12_381::Bls12;

	pub type Public = bls_crate::Public<Bls12>;
	pub type Secret = bls_crate::Secret<Bls12>;
	pub type Pair = bls_crate::Pair<Bls12>;
	pub type Signature = bls_crate::Signature<Bls12>;
}
