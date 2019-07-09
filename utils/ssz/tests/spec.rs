use ssz::{Encode, Decode};
use core::fmt::Debug;

fn t<T: Debug + Encode + Decode + PartialEq + Eq>(value: T, expected: &[u8]) {
	let encoded = value.encode();
	assert_eq!(&encoded[..], expected);
	let decoded = T::decode(&mut &encoded[..]).unwrap();
	assert_eq!(value, decoded);
}

#[test]
fn spec() {
	t(false, &[0x00]);
	t(true, &[0x01]);
}
