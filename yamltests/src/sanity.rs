use std::path::PathBuf;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use ssz::Encode;
use beacon::{Config, BeaconState, MinimalConfig, MainnetConfig};
use beacon::types::*;
use crypto::bls::BLSVerification;
use crate::{test_state_with, test_name, read_value_unwrap, read_raw_unwrap, read_pre_post_unwrap};
use crate::description::{TestDescription, TestPhase, SanityType, TestNetwork};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Meta {
	pub blocks_count: usize,
}

pub fn test(typ: SanityType, desc: TestDescription) {
	match desc.network {
		TestNetwork::Mainnet => test_with_config::<MainnetConfig>(typ, desc),
		TestNetwork::Minimal => test_with_config::<MinimalConfig>(typ, desc),
		TestNetwork::General => unimplemented!("Not supported"),
	}
}

pub fn test_with_config<C: Config>(typ: SanityType, desc: TestDescription) where
	C: Serialize + DeserializeOwned,
{
	assert_eq!(desc.phase, TestPhase::Phase0);
	let path = desc.path.clone().unwrap();

	let (pre, post) = read_pre_post_unwrap::<C>(path.clone());
	match typ {
		SanityType::Blocks => test_blocks(path, pre, post),
		SanityType::Slots => test_slots(path, pre, post),
	}
}

pub fn test_blocks<C: Config>(path: PathBuf, pre: BeaconState<C>, post: Option<BeaconState<C>>) where
	C: Serialize + DeserializeOwned,
{
	let meta = {
		let mut path = path.clone();
		path.push("meta.yaml");

		read_value_unwrap::<_, Meta>(path)
	};

	let mut blocks = Vec::new();
	for i in 0..meta.blocks_count {
		let block = {
			let mut path = path.clone();
			path.push(&format!("blocks_{}.yaml", i));

			read_value_unwrap::<_, BeaconBlock<C>>(path)
		};

		let block_ssz = {
			let mut path = path.clone();
			path.push(&format!("blocks_{}.ssz", i));

			read_raw_unwrap(path)
		};

		assert_eq!(Encode::encode(&block), block_ssz);
		blocks.push(block);
	}

	test_state_with(&test_name(path).unwrap(), &pre, post.as_ref(), move |state| {
		for block in blocks {
			state.state_transition::<_, BLSVerification>(&block)?
		}

		Ok(())
	});
}

pub fn test_slots<C: Config>(path: PathBuf, pre: BeaconState<C>, post: Option<BeaconState<C>>) where
	C: DeserializeOwned,
{
	let slots = {
		let mut path = path.clone();
		path.push("slots.yaml");

		read_value_unwrap::<_, u64>(path)
	};

	test_state_with(&test_name(path).unwrap(), &pre, post.as_ref(), move |state| {
		let target_slot = state.slot + slots;

		state.process_slots(target_slot)
	});
}
