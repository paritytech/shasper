use runtime_primitives::{BuildStorage, StorageMap, ChildrenStorageMap};
use primitives::{ValidatorId, storage::well_known_keys};
use parity_codec::{Encode, KeyedVec};

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

		Ok((storage, Default::default()))
	}
}
