#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "serde")]
pub use self::serde::*;

pub fn integer_squareroot(n: u64) -> u64 {
	let mut x = n;
	let mut y = (x + 1) / 2;
	while y < x {
		x = y;
		y = (x + n / x) / 2
	}
	x
}
