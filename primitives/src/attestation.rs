// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use ssz_derive::{SszEncode, SszDecode};
use ssz_hash_derive::SszHash;
use codec_derive::{Encode, Decode};
use rstd::prelude::*;

#[cfg(feature = "std")]
use serde_derive::{Serialize, Deserialize};

use super::{H256, ValidatorId, ShardId, BitField, Signature};
use hash_db::Hasher;
use keccak_hasher::KeccakHasher;

#[derive(Clone, PartialEq, Eq, Decode, Encode, SszEncode, SszDecode, SszHash)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[ssz_codec(sorted)]
pub struct AttestationRecord {
	pub slot: u64,
	pub shard_id: ShardId,
	pub oblique_parent_hashes: Vec<H256>,
	pub shard_block_hash: H256,
	pub attester_bitfield: BitField,
	pub justified_slot: u64,
	pub justified_block_hash: H256,
	pub aggregate_sig: Signature,
}

impl Default for AttestationRecord {
	fn default() -> Self {
		Self {
			slot: 0,
			shard_id: 0,
			oblique_parent_hashes: Vec::new(),
			shard_block_hash: H256::default(),
			attester_bitfield: BitField::new(0),
			justified_slot: 0,
			justified_block_hash: H256::default(),
			aggregate_sig: Signature::default(),
		}
	}
}

impl AttestationRecord {
	pub fn message(&self, parent_hashes: &[H256]) -> H256 {
		use byteorder::{ByteOrder, BigEndian};

		let mut attestation_slot_bytes = [0u8; 8];
		BigEndian::write_u64(&mut attestation_slot_bytes, self.slot);

		let mut shard_id_bytes = [0u8; 2];
		BigEndian::write_u16(&mut shard_id_bytes, self.shard_id);

		let mut justified_slot_bytes = [0u8; 8];
		BigEndian::write_u64(&mut justified_slot_bytes, self.justified_slot);

		let mut v = Vec::new();
		for b in &attestation_slot_bytes {
			v.push(*b);
		}
		for hash in parent_hashes {
			for b in hash.as_ref() {
				v.push(*b);
			}
		}
		for b in &shard_id_bytes {
			v.push(*b);
		}
		for b in self.shard_block_hash.as_ref() {
			v.push(*b);
		}
		for b in &justified_slot_bytes {
			v.push(*b);
		}

		KeccakHasher::hash(&v)
	}

	pub fn verify_signatures(&self, parent_hashes: &[H256], pubkeys: &[ValidatorId]) {
		let sig = self.aggregate_sig.into_aggregate_signature().expect("Signature decoding failed, attestation is invalid");
		let message = self.message(parent_hashes);

		let mut inputs = Vec::new();
		for pubkey in pubkeys {
			let pubkey = pubkey.into_public().expect("Public key provided is invalid");
			inputs.push((pubkey, message.as_ref().to_vec()));
		}

		assert!(sig.verify(&inputs.iter().map(|(p, m)| (p, &m[..])).collect::<Vec<_>>()));
	}
}
