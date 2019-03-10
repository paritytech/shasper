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

use hash_db::Hasher;

/// Hash bytes with a hasher.
pub fn hash<H: Hasher>(seed: &[u8]) -> H::Out {
	H::hash(seed)
}

/// Hash two bytes with a hasher.
pub fn hash2<H: Hasher>(seed: &[u8], a: &[u8]) -> H::Out {
	let mut v = seed.to_vec();
	let mut a = a.to_vec();
	v.append(&mut a);
	H::hash(&v)
}

/// Hash three bytes with a hasher.
pub fn hash3<H: Hasher>(seed: &[u8], a: &[u8], b: &[u8]) -> H::Out {
	let mut v = seed.to_vec();
	let mut a = a.to_vec();
	let mut b = b.to_vec();
	v.append(&mut a);
	v.append(&mut b);
	H::hash(&v)
}

pub fn to_usize(v: &[u8]) -> usize {
	let mut ret = 0usize.to_le_bytes();
	(&mut ret[..]).copy_from_slice(&v[..v.len()]);
	usize::from_le_bytes(ret)
}
