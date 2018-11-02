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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "std")]
pub mod alloc {
	pub use std::boxed;
	pub use std::vec;
}

extern crate hash_db;
extern crate substrate_primitives as primitives;

use primitives::U256;
use hash_db::Hasher;

pub fn merkle_root<H: Hasher, A>(input: &[A]) -> H::Out where
	A: AsRef<[u8]>
{
	let min_pow_of_2 = {
		let mut o = 1;
		while o <= input.len() {
			o *= 2;
		}
		o
	};

	let mut hashes: Vec<Vec<u8>> = Vec::new();

	let mut len_bytes = Vec::new();
	len_bytes.resize(32, 0);
	U256::from(input.len()).to_big_endian(&mut len_bytes);
	hashes.push(len_bytes);

	for v in input {
		hashes.push(v.as_ref().to_vec());
	}

	for _ in 0..(min_pow_of_2 - input.len()) {
		let mut bytes = Vec::new();
		bytes.resize(32, 0);
		hashes.push(bytes);
	}

	let mut outs: Vec<Option<H::Out>> = Vec::new();
	for _ in 0..min_pow_of_2 {
		outs.push(None);
	}

	for i in (1..min_pow_of_2).rev() {
		let x = i * 2;
		let y = i * 2 + 1;

		let mut bytes = if x >= min_pow_of_2 {
			hashes[x - min_pow_of_2].clone()
		} else {
			outs[x].as_ref().expect("outs at x always exists because we iterate from higher to lower.").as_ref().to_vec()
		};

		bytes.append(&mut if y >= min_pow_of_2 {
			hashes[y - min_pow_of_2].clone()
		} else {
			outs[y].as_ref().expect("outs at x always exists because we iterate from higher to lower.").as_ref().to_vec()
		});

		outs[i] = Some(H::hash(&bytes));
	}

	outs[1].expect("outs at 1 always exists because we iterate to 1.")
}
