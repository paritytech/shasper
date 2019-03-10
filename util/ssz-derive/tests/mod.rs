// Copyright 2017, 2018 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate ssz;

#[macro_use]
extern crate ssz_derive;

use ssz::{Encode, Decode};

#[derive(Debug, PartialEq, Ssz)]
struct Unit;

#[derive(Debug, PartialEq, Ssz)]
struct Indexed(u32, u64);

#[derive(Debug, PartialEq, Ssz)]
struct Struct<A, B, C> {
	pub a: A,
	pub b: B,
	#[ssz(truncate)]
	pub c: C,
}

#[derive(Debug, PartialEq, Ssz)]
#[ssz(sorted)]
struct SortedType {
	pub b: u32,
	pub c: u32,
	pub a: u32,
}

type TestType = Struct<u32, u64, Vec<u8>>;

impl <A, B, C> Struct<A, B, C> {
	fn new(a: A, b: B, c: C) -> Self {
		Self { a, b, c }
	}
}

#[test]
fn should_work_for_sorted() {
	let a = SortedType {
		c: 3, b: 2, a: 1
	};

	a.using_encoded(|ref slice| {
		assert_eq!(slice, &b"\0\0\0\x01\0\0\0\x02\0\0\0\x03");
	});

	let mut da: &[u8] = b"\0\0\0\x01\0\0\0\x02\0\0\0\x03";
	assert_eq!(SortedType::decode(&mut da), Some(a));
}

#[test]
fn should_derive_encode() {
	let v = TestType::new(15, 9, b"Hello world".to_vec());

	v.using_encoded(|ref slice| {
		assert_eq!(slice, &b"\0\0\0\x0f\0\0\0\0\0\0\0\x09\0\0\0\x0bHello world")
	});
}

#[test]
fn should_derive_decode() {
	let slice = b"\0\0\0\x0f\0\0\0\0\0\0\0\x09\0\0\0\x0bHello world".to_vec();

	let v = TestType::decode(&mut &*slice);

	assert_eq!(v, Some(TestType::new(15, 9, b"Hello world".to_vec())));
}

#[test]
fn should_work_for_unit() {
	let v = Unit;

	v.using_encoded(|ref slice| {
		assert_eq!(slice, &[]);
	});

	let mut a: &[u8] = &[];
	assert_eq!(Unit::decode(&mut a), Some(Unit));
}

#[test]
fn should_work_for_indexed() {
	let v = Indexed(1, 2);

	v.using_encoded(|ref slice| {
		assert_eq!(slice, &b"\0\0\0\x01\0\0\0\0\0\0\0\x02")
	});

	let mut v: &[u8] = b"\0\0\0\x01\0\0\0\0\0\0\0\x02";
	assert_eq!(Indexed::decode(&mut v), Some(Indexed(1, 2)));
}
