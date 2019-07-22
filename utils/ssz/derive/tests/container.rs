use ssz_derive::{Codec, Encode, Decode};
use generic_array::{GenericArray, ArrayLength};
use typenum::*;

#[derive(Codec, Encode, Decode)]
pub struct A {
	a: u64,
	b: u64,
	#[bm(compact)]
	c: GenericArray<u8, U3>,
}

pub trait Config {
	type Size: ArrayLength<u8>;
}

#[derive(Codec, Encode, Decode)]
pub struct B<C: Config> {
	a: u16,
	b: u32,
	c: GenericArray<u8, C::Size>,
}
