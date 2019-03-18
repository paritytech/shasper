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

#![allow(unused_imports)]

extern crate ssz;

#[macro_use]
extern crate ssz_derive;

use core::fmt::Debug;
use ssz::{Encode, Decode, Prefixable};

fn assert_ed<T: Encode + Decode + Debug + PartialEq>(t: T, mut a: &[u8]) {
	assert_eq!(&t.encode()[..], a);
	assert_eq!(T::decode(&mut a).unwrap(), t);
}

#[derive(Debug, PartialEq, Ssz)]
struct Unit;

#[test]
fn unit_ed() {
	assert_eq!(Unit::prefixed(), false);
	assert_ed(Unit, &[]);
}

#[derive(Debug, PartialEq, Ssz)]
struct IndexedFixed(bool, bool);

#[test]
fn indexed_fixed_ed() {
	assert_eq!(IndexedFixed::prefixed(), false);
	assert_ed(IndexedFixed(true, false), b"\x01\x00");
}

#[derive(Debug, PartialEq, Ssz)]
struct IndexedVar(Vec<u8>, bool);

#[test]
fn indexed_var_ed() {
	assert_eq!(IndexedVar::prefixed(), true);
	assert_ed(IndexedVar(b"hello".to_vec(), false), b"\n\x00\x00\x00\x05\x00\x00\x00hello\x00");
}

#[derive(Debug, PartialEq, Ssz)]
struct NamedFixed {
	b: bool,
	a: bool,
}

#[test]
fn named_fixed_ed() {
	assert_eq!(NamedFixed::prefixed(), false);
	assert_ed(NamedFixed {
		b: true,
		a: false
	}, b"\x01\x00");
}

#[derive(Debug, PartialEq, Ssz)]
struct NamedVar {
	b: Vec<u8>,
	a: bool
}

#[test]
fn named_var_ed() {
	assert_eq!(NamedVar::prefixed(), true);
	assert_ed(NamedVar {
		b: b"hello".to_vec(),
		a: false,
	}, b"\n\x00\x00\x00\x05\x00\x00\x00hello\x00");
}

#[derive(Debug, PartialEq, Ssz)]
struct IndexedGeneric<A, B>(A, B);

#[derive(Debug, PartialEq, Ssz)]
struct NamedGeneric<A, B> {
	a: A,
	b: B,
}

#[test]
fn generic() {
	assert_eq!(IndexedGeneric::<bool, bool>::prefixed(), false);
	assert_eq!(IndexedGeneric::<Vec<u8>, bool>::prefixed(), true);
	assert_eq!(NamedGeneric::<bool, bool>::prefixed(), false);
	assert_eq!(NamedGeneric::<Vec<u8>, bool>::prefixed(), true);
}
