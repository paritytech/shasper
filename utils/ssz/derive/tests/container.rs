use ssz_derive::{Codec, Encode, Decode};
use generic_array::GenericArray;
use typenum::*;

#[derive(Codec, Encode, Decode)]
pub struct A {
	a: u64,
	b: u64,
	#[bm(compact)]
	c: GenericArray<u8, U3>,
}
