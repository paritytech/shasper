#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "serde")]
pub use self::serde::*;

use primitive_types::H256;

pub fn to_bytes(v: u64) -> H256 {
	let bytes = v.to_le_bytes();
	let mut ret = H256::default();
	(&mut ret[0..bytes.len()]).copy_from_slice(&bytes);
	ret
}

pub fn to_uint(v: &[u8]) -> u64 {
	let mut ret = 0u64.to_le_bytes();
	(&mut ret[..]).copy_from_slice(&v[..v.len()]);
	u64::from_le_bytes(ret)
}

pub fn integer_squareroot(n: u64) -> u64 {
	let mut x = n;
	let mut y = (x + 1) / 2;
	while y < x {
		x = y;
		y = (x + n / x) / 2
	}
	x
}
