use primitives::H256;
use runtime_primitives::traits::{Header as HeaderT, Digest as DigestT};

use super::BlockNumber;
use hasher::{Keccak256, KECCAK_NULL_RLP};
use utils::Never;

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct NeverDigest;

impl DigestT for NeverDigest {
	type Hash = H256;
	type Item = Never;

	fn logs(&self) -> &[Self::Item] { &[] }
	fn push(&mut self, _item: Self::Item) { panic!("Never can never be initialized; this function is impossible to be called; qed"); }
}

#[derive(Clone, PartialEq, Eq, Debug, Decode, Encode, Default, Serialize, Deserialize)]
pub struct Header {
	pub number: BlockNumber,
	pub parent_hash: H256,
	pub slot_number: u64,
	pub active_state_root: H256,
	pub crystallized_state_root: H256,
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
		_digest: Self::Digest
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
