use primitives::H256;
use runtime_primitives::traits::{Hash as HashT};

pub const KECCAK_NULL_RLP: H256 = H256(
	[86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248,
	 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180,
	 33]);

use plain_hasher::PlainHasher;
use hashdb::Hasher;
use tiny_keccak::Keccak;

// Note: We can't use keccak-hasher crate because that one uses ethereum_types::H256.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct KeccakHasher;

impl Hasher for KeccakHasher {
	type Out = H256;
	type StdHasher = PlainHasher;
	const LENGTH: usize = 32;

	fn hash(x: &[u8]) -> Self::Out {
		let mut out = [0;32];
		Keccak::keccak256(x, &mut out);
		out.into()
	}
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keccak256;

impl HashT for Keccak256 {
	type Output = H256;

	fn hash(s: &[u8]) -> Self::Output {
		KeccakHasher::hash(s)
	}

	fn enumerated_trie_root(items: &[&[u8]]) -> Self::Output {
		::runtime_io::enumerated_trie_root::<KeccakHasher>(items).into()
	}

	fn trie_root<
		I: IntoIterator<Item = (A, B)>,
		A: AsRef<[u8]> + Ord,
		B: AsRef<[u8]>
	>(input: I) -> Self::Output {
		::runtime_io::trie_root::<KeccakHasher, _, _, _>(input).into()
	}

	fn ordered_trie_root<
		I: IntoIterator<Item = A>,
		A: AsRef<[u8]>
	>(input: I) -> Self::Output {
		::runtime_io::ordered_trie_root::<KeccakHasher, _, _>(input).into()
	}

	fn storage_root() -> Self::Output {
		::runtime_io::storage_root().into()
	}

	fn storage_changes_root(block: u64) -> Option<Self::Output> {
		::runtime_io::storage_changes_root(block).into()
	}
}
