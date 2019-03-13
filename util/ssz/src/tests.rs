use crate::*;
use primitive_types::*;
use core::fmt::Debug;

fn assert_ed<T: Encode + Decode + Debug + PartialEq>(t: T, mut a: &[u8]) {
	assert_eq!(&t.encode()[..], a);
	assert_eq!(T::decode(&mut a).unwrap(), t);
}

#[test]
fn ed_uint_tests() {
	assert_ed(U256::from(659821), b"m\x11\n\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
	assert_ed(362452341234145u128, b"\xe1\x19\x0c\x03\xa6I\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00");
	assert_ed(8474385234u64, b"R\xdb\x1c\xf9\x01\x00\x00\x00");
	assert_ed(76465737u32, b"I\xc6\x8e\x04");
	assert_ed(52437u16, b"\xd5\xcc");
}
