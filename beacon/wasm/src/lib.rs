use wasm_bindgen::prelude::*;
use beacon::{BLSNoVerification, BLSConfig, Config, MinimalConfig, MainnetConfig, BeaconState};
use beacon::types::BeaconBlock;

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
	let block: BeaconBlock<C> = block.into_serde().map_err(|e| JsValue::from(format!("{:?}", e)))?;
	let mut state: BeaconState<C> = state.into_serde().map_err(|e| JsValue::from(format!("{:?}", e)))?;

	beacon::execute_block::<C, BLS>(
		&block, &mut state
	).map_err(|e| JsValue::from(format!("{:?}", e)))?;

	Ok(JsValue::from_serde(&state).map_err(|e| JsValue::from(format!("{:?}", e)))?)
}
