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

extern crate hash_db;
extern crate ssz_hash;
extern crate substrate_primitives as primitives;

#[macro_use]
extern crate ssz_hash_derive;

use primitives::{H256, Blake2Hasher};
use ssz_hash::SpecHash;

#[derive(SszHash)]
struct Struct<A, B, C> {
	pub a: A,
	pub b: B,
	pub c: C,
}

#[test]
fn should_work_for_struct() {
	let s = Struct {
		a: 0u64,
		b: 0u64,
		c: 0u64,
	};

	assert_eq!(SpecHash::spec_hash::<Blake2Hasher>(&s), H256::from("0xd6bcc4731213bbe7640cd9a44610ad4ae2717e5f3551a4c96565579fe494a45a"));
}
