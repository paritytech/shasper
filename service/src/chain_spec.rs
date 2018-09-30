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
