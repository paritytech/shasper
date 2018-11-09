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

use rstd::prelude::*;

use primitives::H256;
use bitfield::BitField;
use super::{PublicKey, ShardId};

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
	pub aggregate_sig: Vec<u8>,
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
			aggregate_sig: {
				let mut ret = Vec::with_capacity(2 * 48 + 1);
				ret.resize(2 * 48 + 1, 0);
				ret
			},
		}
	}
}

impl AttestationRecord {
	pub fn message(&self, parent_hashes: &[H256]) -> H256 {
		use blake2::{Blake2b, crypto_mac::Mac};
		use byteorder::{ByteOrder, BigEndian};

		let mut attestation_slot_bytes = [0u8; 8];
		BigEndian::write_u64(&mut attestation_slot_bytes, self.slot);

		let mut shard_id_bytes = [0u8; 2];
		BigEndian::write_u16(&mut shard_id_bytes, self.shard_id);

		let mut justified_slot_bytes = [0u8; 8];
		BigEndian::write_u64(&mut justified_slot_bytes, self.justified_slot);

		let mut hasher = Blake2b::new_keyed(&[], 64);
		hasher.input(&attestation_slot_bytes);
		for hash in parent_hashes {
			hasher.input(hash.as_ref());
		}
		hasher.input(&shard_id_bytes);
		hasher.input(self.shard_block_hash.as_ref());
		hasher.input(&justified_slot_bytes);

		H256::from_slice(&hasher.result().code()[0..32])
	}

	pub fn verify_signatures(&self, parent_hashes: &[H256], pubkeys: &[PublicKey]){
		use bls_aggregates::{AggregateSignature, AggregatePublicKey,
							 PublicKey as BlsPublicKey};

		let message = self.message(parent_hashes);
		let aggsig = AggregateSignature::from_bytes(&self.aggregate_sig).expect("Aggregate signature decoding failed, attestation is invalid");
		let pubkeys = pubkeys
			.iter()
			.map(|bytes| BlsPublicKey::from_bytes(bytes).expect("Public key decoding failed, attestation is invalid"))
			.collect();
		let aggpub = AggregatePublicKey::from_public_keys(&pubkeys);
		assert!(aggsig.verify(message.as_ref(), &aggpub));
	}
}
