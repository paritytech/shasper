extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate hashdb;
extern crate plain_hasher;
extern crate tiny_keccak;
extern crate substrate_primitives as primitives;
extern crate substrate_client as client;
extern crate sr_primitives as runtime_primitives;
extern crate sr_io as runtime_io;

use hashdb::Hasher;
use tiny_keccak::Keccak;
use plain_hasher::PlainHasher;
use primitives::{H256, U256, RlpCodec};
use runtime_primitives::traits::{Block as BlockT, Header as HeaderT, Hash as HashT, Digest as DigestT, DigestItem as DigestItemT};
use runtime_io::codec::{Decode, Encode, Codec, Input, Output};

pub const KECCAK_NULL_RLP: H256 = H256(
    [86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248,
     110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180,
     33]);

pub type BlockNumber = u64;

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
		runtime_io::enumerated_trie_root::<KeccakHasher>(items).into()
	}

	fn trie_root<
		I: IntoIterator<Item = (A, B)>,
		A: AsRef<[u8]> + Ord,
		B: AsRef<[u8]>
	>(input: I) -> Self::Output {
		runtime_io::trie_root::<KeccakHasher, _, _, _>(input).into()
	}

	fn ordered_trie_root<
		I: IntoIterator<Item = A>,
		A: AsRef<[u8]>
	>(input: I) -> Self::Output {
		runtime_io::ordered_trie_root::<KeccakHasher, _, _>(input).into()
	}

	fn storage_root() -> Self::Output {
		runtime_io::storage_root().into()
	}
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct AttestationRecord {
    pub slot: u64,
    pub shard_id: u16,
    pub oplique_parent_hashes: Vec<H256>,
    pub shard_block_hash: H256,
    pub attester_bitfield: Vec<u8>,
    pub justified_slot: u64,
    pub justified_block_hash: H256,
    pub aggregate_sig: Vec<U256>,
}

impl Decode for AttestationRecord {
    fn decode<I: Input>(input: &mut I) -> Option<Self> {
        Some(AttestationRecord {
            slot: Decode::decode(input)?,
            shard_id: Decode::decode(input)?,
            oplique_parent_hashes: Decode::decode(input)?,
            shard_block_hash: Decode::decode(input)?,
            attester_bitfield: Decode::decode(input)?,
            justified_slot: Decode::decode(input)?,
            justified_block_hash: Decode::decode(input)?,
            aggregate_sig: Decode::decode(input)?,
        })
    }
}

impl Encode for AttestationRecord {
    fn encode_to<T: Output>(&self, dest: &mut T) {
        dest.push(&self.slot);
        dest.push(&self.shard_id);
        dest.push(&self.oplique_parent_hashes);
        dest.push(&self.shard_block_hash);
        dest.push(&self.attester_bitfield);
        dest.push(&self.justified_slot);
        dest.push(&self.justified_block_hash);
        dest.push(&self.aggregate_sig);
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Never { }

impl DigestItemT for Never {
    type AuthorityId = Never;
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct NeverDigest;

impl DigestT for NeverDigest {
    type Item = Never;

    fn logs(&self) -> &[Self::Item] { &[] }
    fn push(&mut self, item: Self::Item) { panic!("Never can never be initialized; this function is impossible to be called; qed"); }
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Header {
    pub number: BlockNumber, // Note: this field is not yet in the spec.
    pub parent_hash: H256,
    pub slot_number: u64,
    pub randao_reveal: H256,
    pub attestations: Vec<AttestationRecord>,
    pub pow_chain_ref: H256,
    pub active_state_root: H256,
    pub crystallized_state_root: H256,
}

impl Decode for Header {
    fn decode<I: Input>(input: &mut I) -> Option<Self> {
        Some(Header {
            number: Decode::decode(input)?,
            parent_hash: Decode::decode(input)?,
            slot_number: Decode::decode(input)?,
            randao_reveal: Decode::decode(input)?,
            attestations: Decode::decode(input)?,
            pow_chain_ref: Decode::decode(input)?,
            active_state_root: Decode::decode(input)?,
            crystallized_state_root: Decode::decode(input)?,
        })
    }
}

impl Encode for Header {
    fn encode_to<T: Output>(&self, dest: &mut T) {
        dest.push(&self.number);
        dest.push(&self.parent_hash);
        dest.push(&self.slot_number);
        dest.push(&self.randao_reveal);
        dest.push(&self.attestations);
        dest.push(&self.pow_chain_ref);
        dest.push(&self.active_state_root);
        dest.push(&self.crystallized_state_root);
    }
}

impl HeaderT for Header {
    type Number = BlockNumber;
    type Hash = H256;
    type Hashing = Keccak256;
    type Digest = NeverDigest;

	fn new(
		number: Self::Number,
		extrinsics_root: Self::Hash,
		state_root: Self::Hash,
		parent_hash: Self::Hash,
		digest: Self::Digest
	) -> Self {
        assert_eq!(extrinsics_root, KECCAK_NULL_RLP);
        assert_eq!(state_root, KECCAK_NULL_RLP);

        let mut this = Self::default();
        this.number = number;
        this.parent_hash = parent_hash;

        this
    }

	fn number(&self) -> &Self::Number {
        &self.number
    }

	fn set_number(&mut self, number: Self::Number) {
        self.number = number;
    }

	fn extrinsics_root(&self) -> &Self::Hash {
        &KECCAK_NULL_RLP
    }

	fn set_extrinsics_root(&mut self, hash: Self::Hash) {
        assert_eq!(hash, KECCAK_NULL_RLP);
    }

	fn state_root(&self) -> &Self::Hash {
        &KECCAK_NULL_RLP
    }

	fn set_state_root(&mut self, hash: Self::Hash) {
        assert_eq!(hash, KECCAK_NULL_RLP);
    }

	fn parent_hash(&self) -> &Self::Hash {
        &self.parent_hash
    }

	fn set_parent_hash(&mut self, hash: Self::Hash) {
        self.parent_hash = hash;
    }

	fn digest(&self) -> &Self::Digest {
        &NeverDigest
    }

	fn set_digest(&mut self, _: Self::Digest) {

    }
}

pub type Block = runtime_primitives::generic::Block<Header, Never>;
pub type Backend = client::in_mem::Backend<Block, KeccakHasher, RlpCodec>;

fn main() {
    println!("Hello, world!");
}
