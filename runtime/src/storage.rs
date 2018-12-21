use primitives::{BlockNumber, Hash, ValidatorId};
use primitives::storage::well_known_keys;
use srml_support::storage::unhashed;
use super::UncheckedExtrinsic;
use super::Digest as DigestT;

storage_items! {
	pub Number: b"sys:num" => default BlockNumber;
	pub ParentHash: b"sys:parenthash" => default Hash;
	pub ExtrinsicsRoot: b"sys:extrinsicsroot" => default Hash;
	pub Digest: b"sys:digest" => default DigestT;
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
