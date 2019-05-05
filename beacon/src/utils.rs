// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use crate::primitives::H256;

pub fn fixed_vec<T: Default>(len: u64) -> Vec<T> {
	let mut ret = Vec::new();
	ret.resize_with(len as usize, Default::default);
	ret
}

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
