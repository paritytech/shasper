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

use runtime_primitives::{BuildStorage, StorageMap, ChildrenStorageMap};
use primitives::{ValidatorId, storage::well_known_keys};
use runtime_io::twox_128;
use codec::{Encode, KeyedVec};
use crate::state::{ActiveState, CrystallizedState};
use crate::validators::{ValidatorRecord, ShardAndCommittee};
use crate::consts;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GenesisConfig {
	pub authorities: Vec<ValidatorId>,
	pub code: Vec<u8>,
}

impl BuildStorage for GenesisConfig {
	fn build_storage(self) -> Result<(StorageMap, ChildrenStorageMap), String> {
		let mut storage = StorageMap::default();

		storage.insert(well_known_keys::EXTRINSIC_INDEX.to_vec(), 0u32.encode());
		storage.insert(well_known_keys::CODE.to_vec(), self.code.clone());

		let auth_count = self.authorities.len() as u32;
		self.authorities.iter().enumerate().for_each(|(i, v)| {
			storage.insert((i as u32).to_keyed_vec(well_known_keys::AUTHORITY_PREFIX), v.encode());
		});
		storage.insert(well_known_keys::AUTHORITY_COUNT.to_vec(), auth_count.encode());

		let mut active = ActiveState::default();
		for _ in 0..consts::CYCLE_LENGTH {
			active.recent_block_hashes.push(Default::default());
		}
		storage.insert(twox_128(b"sys:active").to_vec(), active.encode());

		let start_time = match ::std::time::SystemTime::now().duration_since(::std::time::SystemTime::UNIX_EPOCH) {
			Ok(start_time) => start_time.as_secs(),
			Err(e) => return Err(format!("{:?}", e)),
		};
		storage.insert(twox_128(b"sys:startslot").to_vec(), (start_time / 10).encode());

		let mut crystallized = CrystallizedState::default();
		for authority in self.authorities.clone() {
			let validator = ValidatorRecord {
				pubkey: authority,
				withdrawal_shard: 0,
				withdrawal_address: Default::default(),
				randao_commitment: Default::default(),
				balance: 50000 * consts::WEI_PER_ETH,
				start_dynasty: 0,
				end_dynasty: u64::max_value() - 1,
			};
			crystallized.validators.push(validator);
		}
		let committee: Vec<u32> = self.authorities.iter().enumerate().map(|(k, _)| k as u32).collect();
		let mut shards_and_committees_for_slot = Vec::new();
		shards_and_committees_for_slot.push(ShardAndCommittee {
			shard_id: 0,
			committee: committee.clone(),
		});
		for _ in 0..(2 * consts::CYCLE_LENGTH) {
			crystallized.shards_and_committees_for_slots.push(shards_and_committees_for_slot.clone());
		}
		storage.insert(twox_128(b"sys:crystallized").to_vec(), crystallized.encode());

		Ok((storage, Default::default()))
	}
}
