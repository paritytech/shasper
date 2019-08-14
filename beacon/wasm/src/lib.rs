use wasm_bindgen::prelude::*;
use beacon::{BLSNoVerification, BLSConfig, Config, MinimalConfig, MainnetConfig, BeaconState};
use beacon::types::BeaconBlock;
use js_sys::JSON;

#[wasm_bindgen]
pub fn execute_minimal(block: &JsValue, state: &JsValue) -> Result<JsValue, JsValue> {
	execute::<MinimalConfig, BLSNoVerification>(block, state)
}

#[wasm_bindgen]
pub fn execute_mainnet(block: &JsValue, state: &JsValue) -> Result<JsValue, JsValue> {
	execute::<MainnetConfig, BLSNoVerification>(block, state)
}

fn execute<C: Config, BLS: BLSConfig>(block: &JsValue, state: &JsValue) -> Result<JsValue, JsValue> where
	C: serde::Serialize + serde::de::DeserializeOwned,
{
	let block_string: String = JSON::stringify(block)?.into();
	let block: BeaconBlock<C> = serde_json::from_str(&block_string)
		.map_err(|e| JsValue::from(format!("{:?}", e)))?;
	let state_string: String = JSON::stringify(state)?.into();
	let mut state: BeaconState<C> = serde_json::from_str(&state_string)
		.map_err(|e| JsValue::from(format!("{:?}", e)))?;

	beacon::execute_block::<C, BLS>(
		&block, &mut state
	).map_err(|e| JsValue::from(format!("{:?}", e)))?;

	Ok(JSON::parse(&serde_json::to_string(&state).map_err(|e| JsValue::from(format!("{:?}", e)))?)?)
}
