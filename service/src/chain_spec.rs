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

use primitives::storage::well_known_keys;
use runtime_primitives::StorageMap;
use codec::Joiner;
use service::ChainSpec;

fn development_genesis() -> StorageMap {
	let wasm_runtime = include_bytes!("../../runtime/wasm/target/wasm32-unknown-unknown/release/shasper_runtime.compact.wasm").to_vec();
	let mut map = StorageMap::new();
	map.insert(well_known_keys::CODE.into(), wasm_runtime);
	map.insert(well_known_keys::HEAP_PAGES.into(), vec![].and(&(16 as u64)));
	map.insert(well_known_keys::AUTHORITY_COUNT.into(), vec![].and(&(1 as u32)));
	map
}

pub fn development_config() -> ChainSpec<StorageMap> {
	ChainSpec::from_genesis("Shasper Development", "shasper_dev", development_genesis, vec![], None, None)
}
