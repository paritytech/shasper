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

#[test]
fn ed_bool_tests() {
	assert_ed(true, b"\x01");
	assert_ed(false, b"\x00");
}

#[test]
fn ed_bytes_tests() {
	assert_ed(b"hello, world!".to_vec(), b"\r\x00\x00\x00hello, world!");
	assert_eq!(&(b"hello, world!"[..]).encode()[..], b"\r\x00\x00\x00hello, world!");
}

#[test]
fn ed_var_bytes_list_tests() {
	assert_ed(vec![b"hello, world!".to_vec()], b"\x11\x00\x00\x00\r\x00\x00\x00hello, world!");
	assert_ed(vec![b"hello".to_vec(), b"world".to_vec()], b"\x12\x00\x00\x00\x05\x00\x00\x00hello\x05\x00\x00\x00world");
}

#[test]
fn ed_var_bool_list_tests() {
	assert_ed(vec![true], b"\x01\x00\x00\x00\x01");
	assert_ed(vec![true, false], b"\x02\x00\x00\x00\x01\x00");
}

#[test]
fn ed_fixed_bytes_list_tests() {
	assert_ed([b"hello, world!".to_vec()], b"\r\x00\x00\x00hello, world!");
	assert_ed([b"hello".to_vec(), b"world".to_vec()], b"\x05\x00\x00\x00hello\x05\x00\x00\x00world");
}

#[test]
fn ed_fixed_bool_list_tests() {
	assert_ed([true], b"\x01");
	assert_ed([true, false], b"\x01\x00");
}

#[test]
fn ed_tuple_test() {
	assert_ed((true, false), b"\x01\x00");
	assert_ed((b"hello".to_vec(), false), b"\n\x00\x00\x00\x05\x00\x00\x00hello\x00");
}
