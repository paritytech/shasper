use primitive_types::{U256, H256, H160};
use hash_db::Hasher;
use crate::Encode;

pub trait Hashable {
	fn hash<H: Hasher>(&self) -> H::Out;
	fn truncated_hash<H: Hasher>(&self) -> H::Out {
		self.hash::<H>()
	}
}

macro_rules! impl_encoded {
	( $( $t:ty ),* ) => { $(
		impl Hashable for $t {
			fn hash<H: Hasher>(&self) -> H::Out {
				let encoded = Encode::encode(self);
				H::hash(&encoded)
			}
		}
	)* }
}

impl_encoded!(u16, u32, u64, u128, usize, i16, i32, i64, i128, isize, bool, U256, H256, H160, Vec<u8>);

macro_rules! impl_array {
	( $( $n:expr )* ) => { $(
		impl<T: Hashable> Hashable for [T; $n] {
			fn hash<H: Hasher>(&self) -> H::Out {
				let values: Vec<_> = self.iter()
					.map(|item| Hashable::hash::<H>(item).as_ref().to_vec())
					.collect();

				merkle_root::<H, _>(&values)
			}
		}
	)* }
}

impl_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
			40 48 56 64 72 96 128 160 192 224 256 1024 8192);

impl<T: Hashable> Hashable for Vec<T> {
	fn hash<H: Hasher>(&self) -> H::Out {
		let values: Vec<_> = self.iter()
			.map(|item| Hashable::hash::<H>(item).as_ref().to_vec())
			.collect();

		merkle_root::<H, _>(&values)
	}
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

pub fn merkleize<H: Hasher>(mut packed: Vec<[u8; 32]>) -> [u8; 32] {
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

	tree[1]
}

pub enum HashItem {
	List(Vec<HashItem>),
	Single(Vec<u8>),
}

pub fn hash_object<H: Hasher>(input: HashItem) -> H::Out {
	match input {
		HashItem::List(list) => {
			let values: Vec<_> = list
				.into_iter()
				.map(|item| hash_object::<H>(item).as_ref().to_vec())
				.collect();

			merkle_root::<H, _>(&values)
		},
		HashItem::Single(obj) => {
			H::hash(&obj)
		},
	}
}

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

	if outs.len() < 2 {
		let target = &mut hashes[1 - outs.len()];

		let mut out = H::Out::default();
		for i in (0..out.as_ref().len()).rev() {
			match target.pop() {
				Some(v) => out.as_mut()[i] = v,
				None => break,
			}
		}
		out
	} else {
		outs[1].expect("outs at 1 always exists because we iterate to 1.")
	}
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
		assert_eq!(&m, b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");

		let packed = pack(&[
			b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf".to_vec(),
			b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf".to_vec()
		]);
		let m = merkleize::<Sha256Hasher>(packed);
		assert_eq!(&m, b"\x06\xec\x0c\xefK\x08l\x03\xe8\x07AnC\xe7O\xb6+\\\xfd\x88i\x7f\x19\x9d\xcb\x0e\xfdx}\x1c)'");
	}
}
