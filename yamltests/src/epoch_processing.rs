use std::path::PathBuf;
use serde::{Serialize, de::DeserializeOwned};
use beacon::{BeaconExecutive, Config, Error, MainnetConfig, MinimalConfig};
use crate::{test_name, test_state_with, read_pre_post_unwrap};
use crate::description::{EpochProcessingType, TestNetwork, TestDescription, TestPhase};

pub fn test(typ: EpochProcessingType, desc: TestDescription) {
	match desc.network {
		TestNetwork::Mainnet => test_with_config::<MainnetConfig>(typ, desc),
		TestNetwork::Minimal => test_with_config::<MinimalConfig>(typ, desc),
		TestNetwork::General => unimplemented!("Not supported"),
	}
}

pub fn test_with_config<C: Config>(typ: EpochProcessingType, desc: TestDescription) where
	C: Serialize + DeserializeOwned,
{
	assert_eq!(desc.phase, TestPhase::Phase0);
	let path = desc.path.clone().unwrap();

	match typ {
		EpochProcessingType::FinalUpdates =>
			test_epoch_processing::<C, _>(path, |state| {
				state.process_final_updates()
			}),
		EpochProcessingType::JustificationAndFinalization =>
			test_epoch_processing::<C, _>(path, |state| {
				state.process_justification_and_finalization()
			}),
		EpochProcessingType::RegistryUpdates =>
			test_epoch_processing::<C, _>(path, |state| {
				state.process_registry_updates()
			}),
		EpochProcessingType::RewardsAndPenalties =>
			test_epoch_processing::<C, _>(path, |state| {
				state.process_rewards_and_penalties()
			}),
		EpochProcessingType::Slashings =>
			test_epoch_processing::<C, _>(path, |state| {
				state.process_slashings();
				Ok(())
			}),
	}
}

pub fn test_epoch_processing<C: Config, F>(path: PathBuf, f: F) where
	C: DeserializeOwned,
	F: FnOnce(&mut BeaconExecutive<C>) -> Result<(), Error>,
{
	let (pre, post) = read_pre_post_unwrap(path.clone());

	test_state_with(&test_name(path).unwrap(), &pre, post.as_ref(), f);
}
