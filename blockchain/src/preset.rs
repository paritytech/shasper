use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Preset {
	pub bootnodes: Vec<String>,
	pub genesis_state: Vec<u8>,
}

pub fn presets() -> HashMap<&'static str, Preset> {
	let mut presets = HashMap::new();

	presets.insert("sapphire", Preset {
		bootnodes: vec!["/dns4/prylabs.net/tcp/30001/p2p/16Uiu2HAm7Qwe19vz9WzD2Mxn7fXd1vgHHp4iccuyq7TxwRXoAGfc".to_string()],
		genesis_state: include_bytes!("../res/eth2-testnets/prysm/Sapphire(v0.9.0)/genesis.ssz").to_vec(),
	});

	presets
}
