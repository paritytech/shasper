use primitives::{H256, BlockNumber, Hash, ValidatorId};
use primitives::storage::well_known_keys;
use srml_support::storage::unhashed;
use state::{ActiveState, CrystallizedState, BlockVoteInfo};
use attestation::AttestationRecord;
use super::UncheckedExtrinsic;
use super::Digest as DigestT;

storage_items! {
	pub Number: b"sys:num" => default BlockNumber;
	pub ParentHash: b"sys:parenthash" => default Hash;
	pub ExtrinsicsRoot: b"sys:extrinsicsroot" => default Hash;
	pub Digest: b"sys:digest" => default DigestT;
	pub Timestamp: b"sys:timestamp" => default u64;
	pub Slot: b"sys:slot" => default u64;
	pub ParentSlot: b"sys:parentslot" => default u64;
	pub LastHeaderHash: b"sys:lasthash" => default H256;
	pub RandaoReveal: b"sys:randaoreveal" => default H256;
	pub PowChainRef: b"sys:powchainref" => default H256;

	pub BlockHashesBySlot: b"sys:blockhashesbyslot" => map [ u64 => H256 ];
	pub Active: b"sys:active" => default ActiveState;
	pub ActiveRoot: b"sys:activeroot" => default H256;
	pub Crystallized: b"sys:crystallized" => default CrystallizedState;
	pub CrystallizedRoot: b"sys:crystallizedroot" => default H256;
	pub BlockVoteCache: b"sys:blockvotecache" => default map [ H256 => BlockVoteInfo ];
}

pub struct UncheckedExtrinsics;
impl unhashed::StorageVec for UncheckedExtrinsics {
	type Item = UncheckedExtrinsic;
	const PREFIX: &'static [u8] = b"sys:extrinsics";
}

pub struct Authorities;
impl unhashed::StorageVec for Authorities {
	type Item = ValidatorId;
	const PREFIX: &'static [u8] = well_known_keys::AUTHORITY_PREFIX;
}

pub struct Attestations;
impl unhashed::StorageVec for Attestations {
	type Item = AttestationRecord;
	const PREFIX: &'static [u8] = b"sys:attestations";
}
