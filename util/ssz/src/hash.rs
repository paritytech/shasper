use primitive_types::{U256, H256, H160};
use hash_db::Hasher;
use digest::Digest;
use generic_array::GenericArray;
use crate::Fixed;

pub trait Digestible<D: Digest> {
	fn hash(&self) -> GenericArray<u8, D::OutputSize>;
	fn truncated_hash(&self) -> GenericArray<u8, D::OutputSize> {
		self.hash()
	}
}

pub trait Hashable<H: Hasher> {
	fn hash(&self) -> H::Out;
	fn truncated_hash(&self) -> H::Out {
		self.hash()
	}
}

pub trait Composite { }

macro_rules! impl_hashable_and_digestible_with_chunkify {
	( $t:ty, $self:ident, $merkleize:ident, $chunkify:ident, $mix_in_length:ident, $e:expr ) => {
		impl<H: Hasher> Hashable<H> for $t {
			fn hash(&$self) -> H::Out {
				#[allow(unused_variables)]
				let $merkleize = self::hash_db_hasher::merkleize::<H>;
				#[allow(unused_variables)]
				let $chunkify = self::hash_db_hasher::chunkify;
				#[allow(unused_variables)]
				let $mix_in_length = self::hash_db_hasher::mix_in_length::<H>;

				$e
			}
		}

		impl<D: Digest> Digestible<D> for $t {
			fn hash(&$self) -> GenericArray<u8, D::OutputSize> {
				#[allow(unused_variables)]
				let $merkleize = self::digest_hasher::merkleize::<D>;
				#[allow(unused_variables)]
				let $chunkify = self::digest_hasher::chunkify::<D::OutputSize>;
				#[allow(unused_variables)]
				let $mix_in_length = self::digest_hasher::mix_in_length::<D>;

				$e
			}
		}
	}
}

macro_rules! impl_hashable_and_digestible_with_pack {
	( $t:ty, $self:ident, $merkleize:ident, $pack:ident, $mix_in_length:ident, $e:expr ) => {
		impl<H: Hasher> Hashable<H> for $t {
			fn hash(&$self) -> H::Out {
				#[allow(unused_variables)]
				let $merkleize = self::hash_db_hasher::merkleize::<H>;
				#[allow(unused_variables)]
				let $pack = self::hash_db_hasher::pack::<_>;
				#[allow(unused_variables)]
				let $mix_in_length = self::hash_db_hasher::mix_in_length::<H>;

				$e
			}
		}

		impl<D: Digest> Digestible<D> for $t {
			fn hash(&$self) -> GenericArray<u8, D::OutputSize> {
				#[allow(unused_variables)]
				let $merkleize = self::digest_hasher::merkleize::<D>;
				#[allow(unused_variables)]
				let $pack = self::digest_hasher::pack::<_, D::OutputSize>;
				#[allow(unused_variables)]
				let $mix_in_length = self::digest_hasher::mix_in_length::<D>;

				$e
			}
		}
	}
}

macro_rules! impl_basic_array {
	( $t:ty, $( $n:expr )* ) => { $(
		impl_hashable_and_digestible_with_pack!(
			[$t; $n], self, merkleize, pack, mix_in_length, {
				merkleize(pack(self.as_ref()))
			}
		);
	)* }
}

macro_rules! impl_basic {
	( $( $t:ty ),* ) => { $(
		impl_hashable_and_digestible_with_pack!(
			$t, self, merkleize, pack, mix_in_length, {
				merkleize(pack(&[*self]))
			}
		);

		impl_basic_array!($t, 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
						  17 18 19 20 21 22 23 24 25 26 27 28 29
						  30 31 32 40 48 56 64 72 96 128 160 192
						  224 256 1024 8192);

		impl<'a, H: Hasher> Hashable<H> for Fixed<'a, $t> {
			fn hash(&self) -> H::Out {
				self::hash_db_hasher::merkleize::<H>(
					self::hash_db_hasher::pack(self.0.as_ref())
				)
			}
		}

		impl<'a, D: Digest> Digestible<D> for Fixed<'a, $t> {
			fn hash(&self) -> GenericArray<u8, D::OutputSize> {
				self::digest_hasher::merkleize::<D>(
					self::digest_hasher::pack::<_, D::OutputSize>(self.0.as_ref())
				)
			}
		}

		impl_hashable_and_digestible_with_pack!(
			Vec<$t>, self, merkleize, pack, mix_in_length, {
				mix_in_length(merkleize(pack(self.as_ref())), self.len() as u32)
			}
		);

		impl_hashable_and_digestible_with_pack!(
			[$t], self, merkleize, pack, mix_in_length, {
				mix_in_length(merkleize(pack(self.as_ref())), self.len() as u32)
			}
		);
	)* }
}

impl_basic!(u16, u32, u64, u128, usize, i16, i32, i64, i128, isize, bool, U256);

macro_rules! impl_composite_array {
	( $( $n:expr )* ) => { $(
		impl<T: Composite> Composite for [T; $n] { }

		impl<T: Composite + Hashable<H>, H: Hasher> Hashable<H> for [T; $n] {
			fn hash(&self) -> H::Out {
				let hashes = self.iter()
					.map(|v| self::hash_db_hasher::hash_to_array(Hashable::<H>::hash(v)))
					.collect::<Vec<_>>();
				self::hash_db_hasher::merkleize::<H>(hashes)
			}
		}

		impl<T: Composite + Digestible<D>, D: Digest> Digestible<D> for [T; $n] {
			fn hash(&self) -> GenericArray<u8, D::OutputSize> {
				let hashes = self.iter()
					.map(|v| Digestible::<D>::hash(v))
					.collect::<Vec<_>>();
				self::digest_hasher::merkleize::<D>(hashes)
			}
		}
	)* }
}

impl_composite_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
					  17 18 19 20 21 22 23 24 25 26 27 28 29
					  30 31 32 40 48 56 64 72 96 128 160 192
					  224 256 1024 8192);

impl<'a, T: Composite> Composite for Fixed<'a, T> { }

impl<'a, T: Composite + Hashable<H>, H: Hasher> Hashable<H> for Fixed<'a, T> {
	fn hash(&self) -> H::Out {
		let hashes = self.0.iter()
			.map(|v| self::hash_db_hasher::hash_to_array(Hashable::<H>::hash(v)))
			.collect::<Vec<_>>();
		self::hash_db_hasher::merkleize::<H>(hashes)
	}
}

impl<'a, T: Composite + Digestible<D>, D: Digest> Digestible<D> for Fixed<'a, T> {
	fn hash(&self) -> GenericArray<u8, D::OutputSize> {
		let hashes = self.0.iter()
			.map(|v| Digestible::<D>::hash(v))
			.collect::<Vec<_>>();
		self::digest_hasher::merkleize::<D>(hashes)
	}
}

macro_rules! impl_tuple {
	($one:ident,) => {
		impl<H: Hasher, $one: Hashable<H>> Hashable<H> for ($one,) {
			fn hash(&self) -> H::Out {
				let mut hashes = Vec::new();
				hashes.push(self::hash_db_hasher::hash_to_array(Hashable::<H>::hash(&self.0)));
				self::hash_db_hasher::merkleize::<H>(hashes)
			}
		}

		impl<D: Digest, $one: Digestible<D>> Digestible<D> for ($one,) {
			fn hash(&self) -> GenericArray<u8, D::OutputSize> {
				let mut hashes = Vec::new();
				hashes.push(Digestible::<D>::hash(&self.0));
				self::digest_hasher::merkleize::<D>(hashes)
			}
		}

		impl<$one> Composite for ($one,) { }
	};
	($first:ident, $($rest:ident,)+) => {
		impl<$first, $($rest),+>
			Composite for
			($first, $($rest),+) { }

		impl<Ha: Hasher, $first: Hashable<Ha>, $($rest: Hashable<Ha>),+>
			Hashable<Ha> for
			($first, $($rest),+)
		{
			fn hash(&self) -> Ha::Out {
				let mut hashes = Vec::new();
				let (
					ref $first,
					$(ref $rest),+
				) = *self;
				hashes.push(self::hash_db_hasher::hash_to_array(Hashable::<Ha>::hash($first)));
				$(
					hashes.push(self::hash_db_hasher::hash_to_array(Hashable::<Ha>::hash($rest)));
				)+
				self::hash_db_hasher::merkleize::<Ha>(hashes)
			}
		}

		impl<Ha: Digest, $first: Digestible<Ha>, $($rest: Digestible<Ha>),+>
			Digestible<Ha> for
			($first, $($rest),+)
		{
			fn hash(&self) -> GenericArray<u8, Ha::OutputSize> {
				let mut hashes = Vec::new();
				let (
					ref $first,
					$(ref $rest),+
				) = *self;
				hashes.push(Digestible::<Ha>::hash($first));
				$(
					hashes.push(Digestible::<Ha>::hash($rest));
				)+
				self::digest_hasher::merkleize::<Ha>(hashes)
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

impl<T: Composite + Hashable<H>, H: Hasher> Hashable<H> for Vec<T> {
	fn hash(&self) -> H::Out {
		let hashes = self.iter()
			.map(|v| self::hash_db_hasher::hash_to_array(Hashable::<H>::hash(v)))
			.collect::<Vec<_>>();
		self::hash_db_hasher::mix_in_length::<H>(
			self::hash_db_hasher::merkleize::<H>(hashes),
			self.len() as u32
		)
	}
}

impl<T: Composite + Digestible<D>, D: Digest> Digestible<D> for Vec<T> {
	fn hash(&self) -> GenericArray<u8, D::OutputSize> {
		let hashes = self.iter()
			.map(|v| Digestible::<D>::hash(v))
			.collect::<Vec<_>>();
		self::digest_hasher::mix_in_length::<D>(
			self::digest_hasher::merkleize::<D>(hashes),
			self.len() as u32
		)
	}
}

impl<T: Composite> Composite for [T] { }

impl<T: Composite + Hashable<H>, H: Hasher> Hashable<H> for [T] {
	fn hash(&self) -> H::Out {
		let hashes = self.iter()
			.map(|v| self::hash_db_hasher::hash_to_array(Hashable::<H>::hash(v)))
			.collect::<Vec<_>>();
		self::hash_db_hasher::mix_in_length::<H>(
			self::hash_db_hasher::merkleize::<H>(hashes),
			self.len() as u32
		)
	}
}

impl<T: Composite + Digestible<D>, D: Digest> Digestible<D> for [T] {
	fn hash(&self) -> GenericArray<u8, D::OutputSize> {
		let hashes = self.iter()
			.map(|v| Digestible::<D>::hash(v))
			.collect::<Vec<_>>();
		self::digest_hasher::mix_in_length::<D>(
			self::digest_hasher::merkleize::<D>(hashes),
			self.len() as u32
		)
	}
}

impl Composite for Vec<u8> { }

impl_hashable_and_digestible_with_chunkify!(
	Vec<u8>, self, merkleize, chunkify, mix_in_length, {
		mix_in_length(merkleize(chunkify(self)), self.len() as u32)
	}
);

impl Composite for [u8] { }

impl_hashable_and_digestible_with_chunkify!(
	[u8], self, merkleize, chunkify, mix_in_length, {
		mix_in_length(merkleize(chunkify(self)), self.len() as u32)
	}
);

macro_rules! impl_fixed_bytes {
	( $( $n:expr )* ) => { $(
		impl_hashable_and_digestible_with_chunkify!(
			[u8; $n], self, merkleize, chunkify, mix_in_length, {
				merkleize(chunkify(self))
			}
		);
	)* }
}

impl_fixed_bytes!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
				  17 18 19 20 21 22 23 24 25 26 27 28 29
				  30 31 32 40 48 56 64 72 96 128 160 192
				  224 256 1024 8192);

macro_rules! impl_fixed_hash {
	( $( $t:ty ),* ) => { $(
		impl Composite for $t { }

		impl_hashable_and_digestible_with_chunkify!(
			$t, self, merkleize, chunkify, mix_in_length, {
				merkleize(chunkify(self.as_ref()))
			}
		);
	)* }
}

impl_fixed_hash!(H256, H160);

pub fn is_power_of_two(value: usize) -> bool {
	return (value > 0) && (value & (value - 1) == 0)
}

pub mod hash_db_hasher {
	use hash_db::Hasher;

	use crate::codec::Encode;

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

	pub fn merkleize<H: Hasher>(mut packed: Vec<[u8; 32]>) -> H::Out {
		while !super::is_power_of_two(packed.len()) {
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
}

pub mod digest_hasher {
	use generic_array::{GenericArray, ArrayLength};
	use digest::Digest;

	use crate::Encode;

	pub fn chunkify<L: ArrayLength<u8>>(bytes: &[u8]) -> Vec<GenericArray<u8, L>> {
		let mut ret: Vec<GenericArray<u8, L>> = Vec::new();

		for i in 0..bytes.len() {
			if i % L::to_usize() == 0 {
				ret.push(Default::default());
			}
			ret.last_mut().expect("Value is pushed when i is 0; cannot be empty; qed")
				[i % L::to_usize()] = bytes[i];
		}

		ret
	}

	pub fn pack<T: Encode, L: ArrayLength<u8>>(values: &[T]) -> Vec<GenericArray<u8, L>> {
		let mut bytes = Vec::new();

		for value in values {
			bytes.append(&mut value.encode());
		}

		chunkify(&bytes)
	}

	pub fn merkleize<D: Digest>(
		mut packed: Vec<GenericArray<u8, D::OutputSize>>
	) -> GenericArray<u8, D::OutputSize> {
		while !super::is_power_of_two(packed.len()) {
			packed.push(Default::default());
		}

		let len = packed.len();
		let mut tree = Vec::new();
		for _ in 0..len {
			tree.push(Default::default());
		}
		tree.append(&mut packed);

		for i in (1..(tree.len() / 2)).rev() {
			let mut hasher = D::new();
			hasher.input(&tree[i * 2]);
			hasher.input(&tree[i * 2 + 1]);
			let hashed = hasher.result();
			tree[i] = hashed;
		}

		tree[1].clone()
	}

	pub fn mix_in_length<D: Digest>(
		root: GenericArray<u8, D::OutputSize>, length: u32
	) -> GenericArray<u8, D::OutputSize> {
		let mut len_bytes: GenericArray<u8, D::OutputSize> = Default::default();
		(&mut len_bytes[0..4]).copy_from_slice(&length.encode());

		let mut hasher = D::new();
		hasher.input(root.as_ref());
		hasher.input(len_bytes.as_ref());
		hasher.result()
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
		let chunkified = hash_db_hasher::chunkify(b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf");
		assert_eq!(chunkified.len(), 2);
		assert_eq!(&chunkified[0][..], b"hello, worldasdfalsgfawieuyfawue");
		assert_eq!(&chunkified[1][..], b"ygkdhbvldzadfasdf\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
	}

	#[test]
	fn test_pack() {
		let packed = hash_db_hasher::pack(&[
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
		let packed = hash_db_hasher::pack(&[true, false]);
		let m = hash_db_hasher::merkleize::<Sha256Hasher>(packed);
		assert_eq!(m, H256::from_slice(b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"));

		let packed = hash_db_hasher::pack(&[
			b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf".to_vec(),
			b"hello, worldasdfalsgfawieuyfawueygkdhbvldzadfasdf".to_vec()
		]);
		let m = hash_db_hasher::merkleize::<Sha256Hasher>(packed);
		assert_eq!(m, H256::from_slice(b"\x06\xec\x0c\xefK\x08l\x03\xe8\x07AnC\xe7O\xb6+\\\xfd\x88i\x7f\x19\x9d\xcb\x0e\xfdx}\x1c)'"));
	}

	#[test]
	fn test_basic() {
		assert_eq!(Hashable::<Sha256Hasher>::hash(&true), H256::from_slice(b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"));
		assert_eq!(Hashable::<Sha256Hasher>::hash(&U256::from(452384756)), H256::from_slice(b"\xf4\xd7\xf6\x1a\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"));
	}

	#[test]
	fn test_basic_fixed_array() {
		assert_eq!(Hashable::<Sha256Hasher>::hash(&[true, false]), H256::from_slice(b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"));
	}

	#[test]
	fn test_bytes() {
		assert_eq!(Hashable::<Sha256Hasher>::hash(&b"hello, world!".to_vec()), H256::from_slice(b"\xaf<\xe8\xbc\xd2\xf0f\xf0.\x07D\xdfI\x93\xef\x97\x9f\xe9.\x14y\x0f\xce\xf0x\xa6\xfa_\x00\x83\xa8\xcb"));
		assert_eq!(Hashable::<Sha256Hasher>::hash(&H256::from_slice(b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")), H256::from_slice(b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));
	}

	#[test]
	fn test_composite() {
		assert_eq!(Hashable::<Sha256Hasher>::hash(&(b"hello".to_vec(), b"world".to_vec(), true)), H256::from_slice(b"x\xb19 \x9f\xb2\xec\x07\xff\x1e\x82\x0b\xa4\x83\xa3\x95\xc9%\x86\xd4\x8f\x85\xfao\xe2\xe8\x0eH!\xaa\xd7\t"));
	}
}
