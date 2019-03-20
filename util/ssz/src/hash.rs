use primitive_types::{U256, H256, H160};
use hash_db::Hasher;
use crate::Encode;

pub trait Hashable {
	fn hash<H: Hasher>(&self) -> H::Out;
	fn truncated_hash<H: Hasher>(&self) -> H::Out {
		self.hash::<H>()
	}
}

pub trait Composite: Hashable { }

macro_rules! impl_basic_array {
	( $t:ty, $( $n:expr )* ) => { $(
		impl Hashable for [$t; $n] {
			fn hash<H: Hasher>(&self) -> H::Out {
				merkleize::<H>(pack(self.as_ref()))
			}
		}
	)* }
}

macro_rules! impl_basic {
	( $( $t:ty ),* ) => { $(
		impl Hashable for $t {
			fn hash<H: Hasher>(&self) -> H::Out {
				merkleize::<H>(pack(&[*self]))
			}
		}

		impl_basic_array!($t, 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
						  17 18 19 20 21 22 23 24 25 26 27 28 29
						  30 31 32 40 48 56 64 72 96 128 160 192
						  224 256 1024 8192);

		impl Hashable for Vec<$t> {
			fn hash<H: Hasher>(&self) -> H::Out {
				mix_in_length::<H>(merkleize::<H>(pack(self.as_ref())), self.len() as u32)
			}
		}

		impl Hashable for [$t] {
			fn hash<H: Hasher>(&self) -> H::Out {
				mix_in_length::<H>(merkleize::<H>(pack(self.as_ref())), self.len() as u32)
			}
		}
	)* }
}

impl_basic!(u16, u32, u64, u128, usize, i16, i32, i64, i128, isize, bool, U256);

macro_rules! impl_composite_array {
	( $( $n:expr )* ) => { $(
		impl<T: Composite> Composite for [T; $n] { }

		impl<T: Composite> Hashable for [T; $n] {
			fn hash<H: Hasher>(&self) -> H::Out {
				let hashes = self.iter()
					.map(|v| hash_to_array(v.hash::<H>()))
					.collect::<Vec<_>>();
				merkleize::<H>(hashes)
			}
		}
	)* }
}

impl_composite_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
					  17 18 19 20 21 22 23 24 25 26 27 28 29
					  30 31 32 40 48 56 64 72 96 128 160 192
					  224 256 1024 8192);

macro_rules! impl_tuple {
	($one:ident,) => {
		impl<$one: Hashable> Hashable for ($one,) {
			fn hash<H: Hasher>(&self) -> H::Out {
				let mut hashes = Vec::new();
				hashes.push(hash_to_array(self.0.hash::<H>()));
				merkleize::<H>(hashes)
			}
		}

		impl<$one: Hashable> Composite for ($one,) { }
	};
	($first:ident, $($rest:ident,)+) => {
		impl<$first: Hashable, $($rest: Hashable),+>
			Composite for
			($first, $($rest),+) { }

		impl<$first: Hashable, $($rest: Hashable),+>
			Hashable for
			($first, $($rest),+)
		{
			fn hash<Ha: Hasher>(&self) -> Ha::Out {
				let mut hashes = Vec::new();
				let (
					ref $first,
					$(ref $rest),+
				) = *self;
				hashes.push(hash_to_array($first.hash::<Ha>()));
				$(
					hashes.push(hash_to_array($rest.hash::<Ha>()));
				)+
				merkleize::<Ha>(hashes)
			}
		}

		impl_tuple!($($rest,)+);
	};
}

#[allow(non_snake_case)]
mod inner_tuple_impl {
	use super::*;
	impl_tuple!(A, B, C, D, E, F, G, H, I, J, K,);
}

impl<T: Composite> Composite for Vec<T> { }

impl<T: Composite> Hashable for Vec<T> {
	fn hash<H: Hasher>(&self) -> H::Out {
		let hashes = self.iter()
			.map(|v| hash_to_array(v.hash::<H>()))
			.collect::<Vec<_>>();
		mix_in_length::<H>(merkleize::<H>(hashes), self.len() as u32)
	}
}

impl<T: Composite> Composite for [T] { }

impl<T: Composite> Hashable for [T] {
	fn hash<H: Hasher>(&self) -> H::Out {
		let hashes = self.iter()
			.map(|v| hash_to_array(v.hash::<H>()))
			.collect::<Vec<_>>();
		mix_in_length::<H>(merkleize::<H>(hashes), self.len() as u32)
	}
}

impl Composite for Vec<u8> { }

impl Hashable for Vec<u8> {
	fn hash<H: Hasher>(&self) -> H::Out {
		mix_in_length::<H>(merkleize::<H>(chunkify(self)), self.len() as u32)
	}
}

impl Composite for [u8] { }

impl Hashable for [u8] {
	fn hash<H: Hasher>(&self) -> H::Out {
		mix_in_length::<H>(merkleize::<H>(chunkify(self)), self.len() as u32)
	}
}

macro_rules! impl_fixed_bytes {
	( $( $n:expr )* ) => { $(
		impl Hashable for [u8; $n] {
			fn hash<H: Hasher>(&self) -> H::Out {
				mix_in_length::<H>(merkleize::<H>(chunkify(self)), self.len() as u32)
			}
		}
	)* }
}

impl_fixed_bytes!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
				  17 18 19 20 21 22 23 24 25 26 27 28 29
				  30 31 32 40 48 56 64 72 96 128 160 192
				  224 256 1024 8192);

macro_rules! impl_fixed_hash {
	( $( $t:ty ),* ) => { $(
		impl Composite for $t { }

		impl Hashable for $t {
			fn hash<H: Hasher>(&self) -> H::Out {
				merkleize::<H>(chunkify(self.as_ref()))
			}
		}
	)* }
}

impl_fixed_hash!(H256, H160);

pub fn hash_to_array<T: AsRef<[u8]>>(value: T) -> [u8; 32] {
	assert_eq!(value.as_ref().len(), 32);
	let mut arr = [0u8; 32];
	(&mut arr).copy_from_slice(value.as_ref());
	arr
}

pub fn chunkify(bytes: &[u8]) -> Vec<[u8; 32]> {
	let mut ret = Vec::new();

	for i in 0..bytes.len() {
		if i % 32 == 0 {
			ret.push([0u8; 32]);
		}
		ret.last_mut().expect("Value is pushed when i is 0; cannot be empty; qed")
			[i % 32] = bytes[i];
	}

	ret
}

pub fn pack<T: Encode>(values: &[T]) -> Vec<[u8; 32]> {
	let mut bytes = Vec::new();

	for value in values {
		bytes.append(&mut value.encode());
	}

	chunkify(&bytes)
}

pub fn is_power_of_two(value: usize) -> bool {
	return (value > 0) && (value & (value - 1) == 0)
}

pub fn merkleize<H: Hasher>(mut packed: Vec<[u8; 32]>) -> H::Out {
	while !is_power_of_two(packed.len()) {
		packed.push([0u8; 32]);
	}

	let len = packed.len();
	let mut tree = Vec::new();
	for _ in 0..len {
		tree.push([0u8; 32]);
	}
	tree.append(&mut packed);

	for i in (1..(tree.len() / 2)).rev() {
		let mut hashing = [0u8; 64];
		(&mut hashing[0..32]).copy_from_slice(&tree[i * 2]);
		(&mut hashing[32..64]).copy_from_slice(&tree[i * 2 + 1]);
		let hashed = H::hash(&hashing);
		assert_eq!(hashed.as_ref().len(), 32);
		(&mut tree[i]).copy_from_slice(hashed.as_ref());
	}

	let mut out = H::Out::default();
	out.as_mut().copy_from_slice(&tree[1]);
	out
}

pub fn mix_in_length<H: Hasher>(root: H::Out, length: u32) -> H::Out {
	let mut bytes = [0u8; 64];
	(&mut bytes[0..32]).copy_from_slice(root.as_ref());
	(&mut bytes[32..36]).copy_from_slice(&length.encode());
	H::hash(&bytes)
}

#[cfg(test)]
mod tests {
	use crate::hash::*;

	use hash_db::Hasher;
	use primitive_types::H256;
	use sha2::{Digest, Sha256};
	use plain_hasher::PlainHasher;

	pub struct Sha256Hasher;
	impl Hasher for Sha256Hasher {
		type Out = H256;
		type StdHasher = PlainHasher;
		const LENGTH: usize = 32;

		fn hash(x: &[u8]) -> Self::Out {
			let mut out = [0; 32];
			(&mut out).copy_from_slice(Sha256::digest(x).as_slice());
			out.into()
		}
	}

	#[test]
	fn test_chunkify() {
		let chunkified = chunkify(b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf");
		assert_eq!(chunkified.len(), 2);
		assert_eq!(&chunkified[0][..], b"hello, worldasdfalsgfawieuyfawue");
		assert_eq!(&chunkified[1][..], b"ygkdhbvldzadfasdf\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
	}

	#[test]
	fn test_pack() {
		let packed = pack(&[
			b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf".to_vec(),
			b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf".to_vec()
		]);
		assert_eq!(packed.len(), 4);
		assert_eq!(&packed[0][..], b"1\x00\x00\x00hello, worldasdfalsgfawieuyf");
		assert_eq!(&packed[1][..], b"awueygkdhbvldzadfasdf1\x00\x00\x00hello, ");
		assert_eq!(&packed[2][..], b"worldasdfalsgfawieuyfawueygkdhbv");
		assert_eq!(&packed[3][..], b"ldzadfasdf\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
	}

	#[test]
	fn test_merkleize() {
		let packed = pack(&[true, false]);
		let m = merkleize::<Sha256Hasher>(packed);
		assert_eq!(m, H256::from_slice(b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"));

		let packed = pack(&[
			b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf".to_vec(),
			b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf".to_vec()
		]);
		let m = merkleize::<Sha256Hasher>(packed);
		assert_eq!(m, H256::from_slice(b"\x06\xec\x0c\xefK\x08l\x03\xe8\x07AnC\xe7O\xb6+\\\xfd\x88i\x7f\x19\x9d\xcb\x0e\xfdx}\x1c)'"));
	}

	#[test]
	fn test_basic() {
		assert_eq!(true.hash::<Sha256Hasher>(), H256::from_slice(b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"));
		assert_eq!(U256::from(452384756).hash::<Sha256Hasher>(), H256::from_slice(b"\xf4\xd7\xf6\x1a\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"));
	}

	#[test]
	fn test_basic_fixed_array() {
		assert_eq!([true, false].hash::<Sha256Hasher>(), H256::from_slice(b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"));
	}

	#[test]
	fn test_bytes() {
		assert_eq!(b"hello, world!".to_vec().hash::<Sha256Hasher>(), H256::from_slice(b"\xaf<\xe8\xbc\xd2\xf0f\xf0.\x07D\xdfI\x93\xef\x97\x9f\xe9.\x14y\x0f\xce\xf0x\xa6\xfa_\x00\x83\xa8\xcb"));
		assert_eq!(H256::from_slice(b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").hash::<Sha256Hasher>(), H256::from_slice(b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));
	}

	#[test]
	fn test_composite() {
		assert_eq!(Hashable::hash::<Sha256Hasher>(&(b"hello".to_vec(), b"world".to_vec(), true)), H256::from_slice(b"x\xb19 \x9f\xb2\xec\x07\xff\x1e\x82\x0b\xa4\x83\xa3\x95\xc9%\x86\xd4\x8f\x85\xfao\xe2\xe8\x0eH!\xaa\xd7\t"));
	}
}
