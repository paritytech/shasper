use std::fs::File;
use std::fmt::Debug;
use std::io::{Read, Write, BufReader};
use std::path::PathBuf;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use beacon::{Config, MinimalConfig, MainnetConfig, BeaconState};
use beacon::primitives::*;
use beacon::types::*;
use bm_le::{FromTree, IntoTree, DigestConstruct, InMemoryBackend};
use ssz::{Encode, Decode};
use sha2::Sha256;
use crate::description::{TestNetwork, TestPhase, TestDescription, SszStaticType};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Roots {
	pub root: H256,
	pub signing_root: Option<H256>,
}

pub fn test(typ: SszStaticType, desc: TestDescription) {
	match desc.network {
		TestNetwork::Mainnet => test_with_config::<MainnetConfig>(typ, desc),
		TestNetwork::Minimal => test_with_config::<MinimalConfig>(typ, desc),
		TestNetwork::General => unimplemented!("Not supported"),
	}
}

pub fn test_with_config<C: Config>(typ: SszStaticType, desc: TestDescription) where
	C: Serialize + DeserializeOwned,
{
	assert_eq!(desc.phase, TestPhase::Phase0);

	match typ {
		SszStaticType::Attestation => test_ssz::<C, Attestation<C>>(desc.path),
		SszStaticType::AttestationData => test_ssz::<C, AttestationData>(desc.path),
		SszStaticType::AttestationDataAndCustodyBit => test_ssz::<C, AttestationDataAndCustodyBit>(desc.path),
		SszStaticType::AttesterSlashing => test_ssz::<C, AttesterSlashing<C>>(desc.path),
		SszStaticType::BeaconBlock => test_ssz::<C, BeaconBlock<C>>(desc.path),
		SszStaticType::BeaconBlockBody => test_ssz::<C, BeaconBlockBody<C>>(desc.path),
		SszStaticType::BeaconBlockHeader => test_ssz::<C, BeaconBlockHeader>(desc.path),
		SszStaticType::BeaconState => test_ssz::<C, BeaconState<C>>(desc.path),
		SszStaticType::Checkpoint => test_ssz::<C, Checkpoint>(desc.path),
		SszStaticType::CompactCommittee => test_ssz::<C, CompactCommittee<C>>(desc.path),
		SszStaticType::Crosslink => test_ssz::<C, Crosslink>(desc.path),
		SszStaticType::Deposit => test_ssz::<C, Deposit>(desc.path),
		SszStaticType::DepositData => test_ssz::<C, DepositData>(desc.path),
		SszStaticType::Eth1Data => test_ssz::<C, Eth1Data>(desc.path),
		SszStaticType::Fork => test_ssz::<C, Fork>(desc.path),
		SszStaticType::HistoricalBatch => test_ssz::<C, HistoricalBatch<C>>(desc.path),
		SszStaticType::IndexedAttestation => test_ssz::<C, IndexedAttestation<C>>(desc.path),
		SszStaticType::PendingAttestation => test_ssz::<C, PendingAttestation<C>>(desc.path),
		SszStaticType::ProposerSlashing => test_ssz::<C, ProposerSlashing>(desc.path),
		SszStaticType::Transfer => test_ssz::<C, Transfer>(desc.path),
		SszStaticType::Validator => test_ssz::<C, Validator>(desc.path),
		SszStaticType::VoluntaryExit => test_ssz::<C, VoluntaryExit>(desc.path),
	}
}

pub fn test_ssz<C: Config, T>(path: String) where
	T: FromTree + IntoTree + Debug + Encode + Decode + Eq + DeserializeOwned,
{
	let path = PathBuf::from(path);

	let roots = {
		let mut path = path.clone();
		path.push("roots.yaml");

		let file = File::open(path).expect("Open roots failed");
		let reader = BufReader::new(file);
		serde_yaml::from_reader::<_, Roots>(reader).expect("Parse roots failed")
	};

	let serialized = {
		let mut path = path.clone();
		path.push("serialized.ssz");

		File::open(path).expect("Open serialized failed")
			.bytes()
			.map(|v| v.unwrap())
			.collect::<Vec<_>>()
	};

	let value = {
		let mut path = path.clone();
		path.push("value.yaml");

		let file = File::open(path).expect("Open roots failed");
		let reader = BufReader::new(file);
		serde_yaml::from_reader::<_, T>(reader).expect("Parse roots failed")
	};

	print!("Testing {} ...", path.to_str().unwrap());
	std::io::stdout().flush().ok().expect("Could not flush stdout");

	let encoded = Encode::encode(&value);
	assert_eq!(encoded, serialized);
	let decoded = T::decode(&encoded).unwrap();
	assert_eq!(decoded, value);
	let mut db = InMemoryBackend::<DigestConstruct<Sha256>>::default();
	let encoded_root = value.into_tree(&mut db).unwrap();
	assert_eq!(H256::from_slice(encoded_root.as_ref()), roots.root);
	let decoded_root = T::from_tree(&encoded_root, &mut db).unwrap();
	assert_eq!(decoded_root, value);

	println!(" passed");
}
