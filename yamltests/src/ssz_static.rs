use std::fmt::Debug;
use std::io::Write;
use std::path::PathBuf;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use beacon::{Config, MinimalConfig, MainnetConfig, BeaconState};
use beacon::primitives::*;
use beacon::types::*;
use bm_le::{FromTree, IntoTree, DigestConstruct, InMemoryBackend};
use ssz::{Encode, Decode};
use sha2::Sha256;
use crate::{test_name, read_raw_unwrap, read_value_unwrap};
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
	let path = desc.path.clone().unwrap();

	match typ {
		SszStaticType::Attestation => test_ssz::<C, Attestation<C>>(path),
		SszStaticType::AttestationData => test_ssz::<C, AttestationData>(path),
		SszStaticType::AttestationDataAndCustodyBit => test_ssz::<C, AttestationDataAndCustodyBit>(path),
		SszStaticType::AttesterSlashing => test_ssz::<C, AttesterSlashing<C>>(path),
		SszStaticType::BeaconBlock => test_ssz::<C, BeaconBlock<C>>(path),
		SszStaticType::BeaconBlockBody => test_ssz::<C, BeaconBlockBody<C>>(path),
		SszStaticType::BeaconBlockHeader => test_ssz::<C, BeaconBlockHeader>(path),
		SszStaticType::BeaconState => test_ssz::<C, BeaconState<C>>(path),
		SszStaticType::Checkpoint => test_ssz::<C, Checkpoint>(path),
		SszStaticType::CompactCommittee => test_ssz::<C, CompactCommittee<C>>(path),
		SszStaticType::Crosslink => test_ssz::<C, Crosslink>(path),
		SszStaticType::Deposit => test_ssz::<C, Deposit>(path),
		SszStaticType::DepositData => test_ssz::<C, DepositData>(path),
		SszStaticType::Eth1Data => test_ssz::<C, Eth1Data>(path),
		SszStaticType::Fork => test_ssz::<C, Fork>(path),
		SszStaticType::HistoricalBatch => test_ssz::<C, HistoricalBatch<C>>(path),
		SszStaticType::IndexedAttestation => test_ssz::<C, IndexedAttestation<C>>(path),
		SszStaticType::PendingAttestation => test_ssz::<C, PendingAttestation<C>>(path),
		SszStaticType::ProposerSlashing => test_ssz::<C, ProposerSlashing>(path),
		SszStaticType::Transfer => test_ssz::<C, Transfer>(path),
		SszStaticType::Validator => test_ssz::<C, Validator>(path),
		SszStaticType::VoluntaryExit => test_ssz::<C, VoluntaryExit>(path),
	}
}

pub fn test_ssz<C: Config, T>(path: PathBuf) where
	T: FromTree + IntoTree + Debug + Encode + Decode + Eq + DeserializeOwned,
{
	print!("Testing {} ...", test_name(&path).unwrap());
	std::io::stdout().flush().ok().expect("Could not flush stdout");

	let path = PathBuf::from(path);

	let roots = {
		let mut path = path.clone();
		path.push("roots.yaml");

		read_value_unwrap::<_, Roots>(path)
	};

	let serialized = {
		let mut path = path.clone();
		path.push("serialized.ssz");

		read_raw_unwrap(path)
	};

	let value = {
		let mut path = path.clone();
		path.push("value.yaml");

		read_value_unwrap::<_, T>(path)
	};

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
