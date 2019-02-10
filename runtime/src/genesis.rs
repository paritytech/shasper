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
use primitives::{ValidatorId, Epoch, Balance, storage::well_known_keys};
use codec::{Encode, KeyedVec};
use crate::storage;
use crate::state::ValidatorRecord;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GenesisConfig {
	pub authorities: Vec<(ValidatorId, Balance)>,
	pub code: Vec<u8>,
}

impl BuildStorage for GenesisConfig {
	fn build_storage(self) -> Result<(StorageMap, ChildrenStorageMap), String> {
		let mut storage = StorageMap::default();

		storage.insert(well_known_keys::CODE.to_vec(), self.code.clone());

		let auth_count = self.authorities.len() as u32;
		self.authorities.iter().enumerate().for_each(|(i, (v, b))| {
			let record = ValidatorRecord {
				valid_from: 0,
				valid_to: Epoch::max_value(),
				balance: *b,
				validator_id: *v,
			};

			storage.insert((i as u32).to_keyed_vec(storage::VALIDATORS_PREFIX), Some(record).encode());
		});
		storage.insert(b"len".to_keyed_vec(storage::VALIDATORS_PREFIX), auth_count.encode());

		Ok((storage, Default::default()))
	}
}
