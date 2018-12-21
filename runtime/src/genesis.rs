use runtime_primitives::{BuildStorage, StorageMap, ChildrenStorageMap};
use primitives::{ValidatorId, storage::well_known_keys};
use runtime_io::twox_128;
use parity_codec::{Encode, KeyedVec};
use state::{ActiveState, CrystallizedState};
use validators::{ValidatorRecord, ShardAndCommittee};
use consts;

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
