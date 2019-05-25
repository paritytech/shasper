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
use primitives::storage::well_known_keys;
use codec::Encode;
use crypto::bls;
use beacon::ParameteredConfig;
use beacon::types::{Deposit, Eth1Data};
use crate::AuthorityId;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
/// Shasper genesis config.
pub struct GenesisConfig {
	/// Authority.
	pub authority: AuthorityId,
	/// Code.
	pub code: Vec<u8>,

	/// Beacon validator deposits.
	pub validator_deposits: Vec<Deposit>,
	/// Beacon timestamp.
	pub time: u64,
	/// Beacon eth1 data.
	pub eth1_data: Eth1Data,
}

impl BuildStorage for GenesisConfig {
	fn assimilate_storage(self, storage: &mut StorageOverlay, _children_storage: &mut ChildrenStorageOverlay) -> Result<(), String> {
		storage.insert(well_known_keys::CODE.to_vec(), self.code.clone());

		storage.insert(
			twox_128(b"sys:authority").to_vec(),
			self.authority.encode()
		);

		let config = ParameteredConfig::<bls::Verification>::small();
		let (_genesis_beacon_block, genesis_state) = beacon::genesis(
			&self.validator_deposits, 0, self.eth1_data.clone(), &config
		).unwrap();

		storage.insert(
			twox_128(b"sys:state").to_vec(),
			genesis_state.encode(),
		);

		storage.insert(
			twox_128(b"sys:config").to_vec(),
			config.encode(),
		);

		Ok(())
	}
}
