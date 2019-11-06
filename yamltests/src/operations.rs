use std::path::PathBuf;
use std::fmt::Debug;
use serde::{Serialize, de::DeserializeOwned};
use ssz::{Encode, Decode};
use bm_le::{FromTree, IntoTree};
use beacon::{BeaconState, Config, Error, MainnetConfig, MinimalConfig};
use beacon::types::*;
use crypto::bls::BLSVerification;
use crate::{test_name, read_raw_unwrap, read_value_unwrap, test_state_with};
use crate::description::{OperationsType, TestNetwork, TestDescription, TestPhase};

pub fn test(typ: OperationsType, desc: TestDescription) {
	match desc.network {
		TestNetwork::Mainnet => test_with_config::<MainnetConfig>(typ, desc),
		TestNetwork::Minimal => test_with_config::<MinimalConfig>(typ, desc),
		TestNetwork::General => unimplemented!("Not supported"),
	}
}

pub fn test_with_config<C: Config>(typ: OperationsType, desc: TestDescription) where
	C: Serialize + DeserializeOwned,
{
	assert_eq!(desc.phase, TestPhase::Phase0);
	let path = desc.path.clone().unwrap();

	match typ {
		OperationsType::Attestation => test_operation::<C, Attestation<C>, _>(path, "attestation", |s, a| {
			s.process_attestation::<BLSVerification>(a)
		}),
		OperationsType::AttesterSlashing => test_operation::<C, AttesterSlashing<C>, _>(path, "attester_slashing", |s, a| {
			s.process_attester_slashing::<BLSVerification>(a)
		}),
		OperationsType::BlockHeader => test_operation::<C, BeaconBlock<C>, _>(path, "block", |s, a| {
			s.process_block_header::<_, BLSVerification>(&a)
		}),
		OperationsType::Deposit => test_operation::<C, Deposit, _>(path, "deposit", |s, a| {
			s.process_deposit::<BLSVerification>(a)
		}),
		OperationsType::ProposerSlashing => test_operation::<C, ProposerSlashing, _>(path, "proposer_slashing", |s, a| {
			s.process_proposer_slashing::<BLSVerification>(a)
		}),
		OperationsType::Transfer => test_operation::<C, Transfer, _>(path, "transfer", |s, a| {
			s.process_transfer::<BLSVerification>(a)
		}),
		OperationsType::VoluntaryExit => test_operation::<C, VoluntaryExit, _>(path, "voluntary_exit", |s, a| {
			s.process_voluntary_exit::<BLSVerification>(a)
		}),
	}
}

pub fn test_operation<C: Config, T, F>(path: PathBuf, operation_id: &str, f: F) where
	C: DeserializeOwned,
	T: FromTree + IntoTree + Debug + Encode + Decode + Eq + DeserializeOwned,
	F: FnOnce(&mut BeaconState<C>, T) -> Result<(), Error>,
{
	let pre = {
		let mut path = path.clone();
		path.push("pre.yaml");

		read_value_unwrap::<_, BeaconState<C>>(path)
	};

	let pre_ssz = {
		let mut path = path.clone();
		path.push("pre.ssz");

		read_raw_unwrap(path)
	};

	assert_eq!(Encode::encode(&pre), pre_ssz);

	let post = {
		let mut path = path.clone();
		path.push("post.yaml");

		if path.exists() {
			Some(read_value_unwrap::<_, BeaconState<C>>(path))
		} else {
			None
		}
	};

	if let Some(post) = post.as_ref() {
		let post_ssz = {
			let mut path = path.clone();
			path.push("post.ssz");

			read_raw_unwrap(path)
		};

		assert_eq!(Encode::encode(post), post_ssz);
	}

	let value = {
		let mut path = path.clone();
		path.push(&format!("{}.yaml", operation_id));

		read_value_unwrap::<_, T>(path)
	};

	let value_ssz = {
		let mut path = path.clone();
		path.push(&format!("{}.ssz", operation_id));

		read_raw_unwrap(path)
	};

	assert_eq!(Encode::encode(&value), value_ssz);

	test_state_with(&test_name(path).unwrap(), &pre, post.as_ref(), |state| {
		f(state, value)
	});
}
