use crate::primitives::H256;

pub fn to_bytes(v: u64) -> H256 {
	let bytes = v.to_le_bytes();
	let mut ret = H256::default();
	(&mut ret[0..bytes.len()]).copy_from_slice(&bytes);
	ret
}

pub fn to_usize(v: &[u8]) -> usize {
	let mut ret = 0usize.to_le_bytes();
	(&mut ret[..]).copy_from_slice(&v[..v.len()]);
	usize::from_le_bytes(ret)
}

pub fn split_offset(len: usize, chunks: usize, index: usize) -> usize {
	(len * index) / chunks
}

pub fn compare_hash(a: &H256, b: &H256) -> core::cmp::Ordering {
	for i in 0..32 {
		if a[i] > b[i] {
			return core::cmp::Ordering::Greater
		} else if a[i] < b[i] {
			return core::cmp::Ordering::Less
		}
	}
	core::cmp::Ordering::Equal
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

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;
	use crate::{Config, NoVerificationConfig};

	#[test]
	fn test_permuted_index() {
		let config = NoVerificationConfig::small();
		let seed = H256::from_str("c0c7f226fbd574a8c63dc26864c27833ea931e7c70b34409ba765f3d2031633d").unwrap();

		assert_eq!(config.permuted_index(0, &seed, 128), 25);
		assert_eq!(config.permuted_index(1, &seed, 128), 62);
		assert_eq!(config.permuted_index(2, &seed, 128), 82);
		assert_eq!(config.permuted_index(3, &seed, 128), 1);
		assert_eq!(config.permuted_index(4, &seed, 128), 60);
		assert_eq!(config.permuted_index(5, &seed, 128), 81);
		assert_eq!(config.permuted_index(6, &seed, 128), 61);
		assert_eq!(config.permuted_index(7, &seed, 128), 123);
		assert_eq!(config.permuted_index(8, &seed, 128), 73);
		assert_eq!(config.permuted_index(9, &seed, 128), 103);
		assert_eq!(config.permuted_index(10, &seed, 128), 49);
		assert_eq!(config.permuted_index(11, &seed, 128), 23);
		assert_eq!(config.permuted_index(12, &seed, 128), 64);
		assert_eq!(config.permuted_index(13, &seed, 128), 0);
		assert_eq!(config.permuted_index(14, &seed, 128), 65);
		assert_eq!(config.permuted_index(15, &seed, 128), 21);
		assert_eq!(config.permuted_index(16, &seed, 128), 74);
		assert_eq!(config.permuted_index(17, &seed, 128), 8);
		assert_eq!(config.permuted_index(18, &seed, 128), 100);
		assert_eq!(config.permuted_index(19, &seed, 128), 119);
		assert_eq!(config.permuted_index(20, &seed, 128), 34);
		assert_eq!(config.permuted_index(21, &seed, 128), 101);
		assert_eq!(config.permuted_index(22, &seed, 128), 86);
		assert_eq!(config.permuted_index(23, &seed, 128), 110);
		assert_eq!(config.permuted_index(24, &seed, 128), 50);
		assert_eq!(config.permuted_index(25, &seed, 128), 71);
		assert_eq!(config.permuted_index(26, &seed, 128), 85);
		assert_eq!(config.permuted_index(27, &seed, 128), 51);
		assert_eq!(config.permuted_index(28, &seed, 128), 22);
		assert_eq!(config.permuted_index(29, &seed, 128), 80);
		assert_eq!(config.permuted_index(30, &seed, 128), 112);
		assert_eq!(config.permuted_index(31, &seed, 128), 17);
		assert_eq!(config.permuted_index(32, &seed, 128), 52);
		assert_eq!(config.permuted_index(33, &seed, 128), 92);
		assert_eq!(config.permuted_index(34, &seed, 128), 105);
		assert_eq!(config.permuted_index(35, &seed, 128), 99);
		assert_eq!(config.permuted_index(36, &seed, 128), 38);
		assert_eq!(config.permuted_index(37, &seed, 128), 16);
		assert_eq!(config.permuted_index(38, &seed, 128), 6);
		assert_eq!(config.permuted_index(39, &seed, 128), 127);
		assert_eq!(config.permuted_index(40, &seed, 128), 69);
		assert_eq!(config.permuted_index(41, &seed, 128), 67);
		assert_eq!(config.permuted_index(42, &seed, 128), 2);
		assert_eq!(config.permuted_index(43, &seed, 128), 118);
		assert_eq!(config.permuted_index(44, &seed, 128), 30);
		assert_eq!(config.permuted_index(45, &seed, 128), 37);
		assert_eq!(config.permuted_index(46, &seed, 128), 108);
		assert_eq!(config.permuted_index(47, &seed, 128), 15);
		assert_eq!(config.permuted_index(48, &seed, 128), 57);
		assert_eq!(config.permuted_index(49, &seed, 128), 75);
		assert_eq!(config.permuted_index(50, &seed, 128), 3);
		assert_eq!(config.permuted_index(51, &seed, 128), 121);
		assert_eq!(config.permuted_index(52, &seed, 128), 12);
		assert_eq!(config.permuted_index(53, &seed, 128), 42);
		assert_eq!(config.permuted_index(54, &seed, 128), 111);
		assert_eq!(config.permuted_index(55, &seed, 128), 47);
		assert_eq!(config.permuted_index(56, &seed, 128), 78);
		assert_eq!(config.permuted_index(57, &seed, 128), 45);
		assert_eq!(config.permuted_index(58, &seed, 128), 59);
		assert_eq!(config.permuted_index(59, &seed, 128), 56);
		assert_eq!(config.permuted_index(60, &seed, 128), 19);
		assert_eq!(config.permuted_index(61, &seed, 128), 89);
		assert_eq!(config.permuted_index(62, &seed, 128), 18);
		assert_eq!(config.permuted_index(63, &seed, 128), 36);
		assert_eq!(config.permuted_index(64, &seed, 128), 104);
		assert_eq!(config.permuted_index(65, &seed, 128), 102);
		assert_eq!(config.permuted_index(66, &seed, 128), 87);
		assert_eq!(config.permuted_index(67, &seed, 128), 97);
		assert_eq!(config.permuted_index(68, &seed, 128), 31);
		assert_eq!(config.permuted_index(69, &seed, 128), 66);
		assert_eq!(config.permuted_index(70, &seed, 128), 95);
		assert_eq!(config.permuted_index(71, &seed, 128), 120);
		assert_eq!(config.permuted_index(72, &seed, 128), 5);
		assert_eq!(config.permuted_index(73, &seed, 128), 54);
		assert_eq!(config.permuted_index(74, &seed, 128), 76);
		assert_eq!(config.permuted_index(75, &seed, 128), 27);
		assert_eq!(config.permuted_index(76, &seed, 128), 48);
		assert_eq!(config.permuted_index(77, &seed, 128), 126);
		assert_eq!(config.permuted_index(78, &seed, 128), 26);
		assert_eq!(config.permuted_index(79, &seed, 128), 58);
		assert_eq!(config.permuted_index(80, &seed, 128), 44);
		assert_eq!(config.permuted_index(81, &seed, 128), 32);
		assert_eq!(config.permuted_index(82, &seed, 128), 40);
		assert_eq!(config.permuted_index(83, &seed, 128), 90);
		assert_eq!(config.permuted_index(84, &seed, 128), 20);
		assert_eq!(config.permuted_index(85, &seed, 128), 10);
		assert_eq!(config.permuted_index(86, &seed, 128), 79);
		assert_eq!(config.permuted_index(87, &seed, 128), 11);
		assert_eq!(config.permuted_index(88, &seed, 128), 24);
		assert_eq!(config.permuted_index(89, &seed, 128), 114);
		assert_eq!(config.permuted_index(90, &seed, 128), 106);
		assert_eq!(config.permuted_index(91, &seed, 128), 77);
		assert_eq!(config.permuted_index(92, &seed, 128), 98);
		assert_eq!(config.permuted_index(93, &seed, 128), 117);
		assert_eq!(config.permuted_index(94, &seed, 128), 55);
		assert_eq!(config.permuted_index(95, &seed, 128), 35);
		assert_eq!(config.permuted_index(96, &seed, 128), 14);
		assert_eq!(config.permuted_index(97, &seed, 128), 13);
		assert_eq!(config.permuted_index(98, &seed, 128), 70);
		assert_eq!(config.permuted_index(99, &seed, 128), 94);
		assert_eq!(config.permuted_index(100, &seed, 128), 46);
		assert_eq!(config.permuted_index(101, &seed, 128), 29);
		assert_eq!(config.permuted_index(102, &seed, 128), 84);
		assert_eq!(config.permuted_index(103, &seed, 128), 96);
		assert_eq!(config.permuted_index(104, &seed, 128), 53);
		assert_eq!(config.permuted_index(105, &seed, 128), 33);
		assert_eq!(config.permuted_index(106, &seed, 128), 113);
		assert_eq!(config.permuted_index(107, &seed, 128), 68);
		assert_eq!(config.permuted_index(108, &seed, 128), 88);
		assert_eq!(config.permuted_index(109, &seed, 128), 41);
		assert_eq!(config.permuted_index(110, &seed, 128), 109);
		assert_eq!(config.permuted_index(111, &seed, 128), 7);
		assert_eq!(config.permuted_index(112, &seed, 128), 63);
		assert_eq!(config.permuted_index(113, &seed, 128), 9);
		assert_eq!(config.permuted_index(114, &seed, 128), 115);
		assert_eq!(config.permuted_index(115, &seed, 128), 124);
		assert_eq!(config.permuted_index(116, &seed, 128), 43);
		assert_eq!(config.permuted_index(117, &seed, 128), 28);
		assert_eq!(config.permuted_index(118, &seed, 128), 91);
		assert_eq!(config.permuted_index(119, &seed, 128), 125);
		assert_eq!(config.permuted_index(120, &seed, 128), 107);
		assert_eq!(config.permuted_index(121, &seed, 128), 83);
		assert_eq!(config.permuted_index(122, &seed, 128), 39);
		assert_eq!(config.permuted_index(123, &seed, 128), 122);
		assert_eq!(config.permuted_index(124, &seed, 128), 116);
		assert_eq!(config.permuted_index(125, &seed, 128), 72);
		assert_eq!(config.permuted_index(126, &seed, 128), 93);
		assert_eq!(config.permuted_index(127, &seed, 128), 4);
	}
}
