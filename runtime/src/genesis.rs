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

use runtime_primitives::{BuildStorage, StorageOverlay, ChildrenStorageOverlay};
use runtime_io::twox_128;
use primitives::{KeccakHasher, ValidatorId, Epoch, Slot, Timestamp, Balance, AttestationContext, storage::well_known_keys};
use codec::{Encode, KeyedVec};
use casper::CasperProcess;
use casper::randao::RandaoCommitment;
use crate::{storage, consts, utils};
use crate::state::ValidatorRecord;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
/// Shasper genesis config.
pub struct GenesisConfig {
	/// Initial validator set.
	pub authorities: Vec<(ValidatorId, Balance, RandaoCommitment<KeccakHasher>)>,
	/// Code being set as `:code`.
	pub code: Vec<u8>,
	/// Genesis timestamp.
	pub timestamp: Timestamp,
}

impl BuildStorage for GenesisConfig {
	fn build_storage(self) -> Result<(StorageOverlay, ChildrenStorageOverlay), String> {
		let mut storage = StorageOverlay::default();

		storage.insert(well_known_keys::CODE.to_vec(), self.code.clone());

		let auth_count = self.authorities.len() as u32;
		self.authorities.iter().enumerate().for_each(|(i, (v, b, r))| {
			let record = ValidatorRecord {
				valid_from: 0,
				valid_to: Epoch::max_value(),
				balance: *b,
				validator_id: *v,
				randao_commitment: r.clone(),
			};

			storage.insert((i as u32).to_keyed_vec(storage::VALIDATORS_PREFIX), Some(record).encode());
		});
		storage.insert(b"len".to_keyed_vec(storage::VALIDATORS_PREFIX), auth_count.encode());

		let slot = {
			let ret = self.timestamp / consts::SLOT_DURATION as Slot;
			(ret / consts::CYCLE_LENGTH) * consts::CYCLE_LENGTH
		};
		storage.insert(
			twox_128(b"sys:genesisslot").to_vec(),
			slot.encode()
		);
		storage.insert(
			twox_128(b"sys:lastslot").to_vec(),
			slot.encode()
		);
		storage.insert(
			twox_128(b"sys:slot").to_vec(),
			slot.encode()
		);
		storage.insert(
			twox_128(b"sys:caspercontext").to_vec(),
			CasperProcess::<AttestationContext>::new(utils::slot_to_epoch(slot)).encode()
		);


		Ok((storage, Default::default()))
	}
}
