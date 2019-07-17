use std::fs::File;
use std::io::{BufReader, Write};

use clap::{App, Arg};
use beacon::{Config, BLSNoVerification, MinimalConfig, MainnetConfig};
use beacon::primitives::*;
use beacon::types::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use ssz::{Encode, Decode};
use bm_le::{IntoTree, FromTree, InMemoryBackend, End};
use sha2::Sha256;

#[derive(Deserialize, Debug)]
#[serde(bound = "C: Config + Serialize + Clone + DeserializeOwned + 'static")]
pub struct Collection<C: Config> {
	pub title: String,
	pub test_cases: Vec<Test<C>>,
}

#[derive(Deserialize, Debug)]
#[serde(bound = "C: Config + Serialize + Clone + DeserializeOwned + 'static")]
pub enum Test<C: Config> {
	Attestation(TestItem<Attestation<C>>),
	AttestationData(TestItem<AttestationData>),
	AttestationDataAndCustodyBit(TestItem<AttestationDataAndCustodyBit>),
	AttesterSlashing(TestItem<AttesterSlashing<C>>),
	BeaconBlock(TestItem<BeaconBlock<C>>),
	BeaconBlockBody(TestItem<BeaconBlockBody<C>>),
	BeaconBlockHeader(TestItem<BeaconBlockHeader>),
	BeaconState(TestItem<BeaconState<C>>),
	Checkpoint(TestItem<Checkpoint>),
	CompactCommittee(TestItem<CompactCommittee<C>>),
	Crosslink(TestItem<Crosslink>),
	Deposit(TestItem<Deposit>),
	DepositData(TestItem<DepositData>),
	Eth1Data(TestItem<Eth1Data>),
	Fork(TestItem<Fork>),
	HistoricalBatch(TestItem<HistoricalBatch<C>>),
	IndexedAttestation(TestItem<IndexedAttestation<C>>),
	PendingAttestation(TestItem<PendingAttestation<C>>),
	ProposerSlashing(TestItem<ProposerSlashing>),
	Transfer(TestItem<Transfer>),
	Validator(TestItem<Validator>),
	VoluntaryExit(TestItem<VoluntaryExit>),
}

impl<C: Config + core::fmt::Debug + PartialEq> Test<C> {
	pub fn test(&self) {
		match self {
			Test::Attestation(test) => test.test(),
			Test::AttestationData(test) => test.test(),
			Test::AttestationDataAndCustodyBit(test) => test.test(),
			Test::AttesterSlashing(test) => test.test(),
			Test::BeaconBlock(test) => test.test(),
			Test::BeaconBlockBody(test) => test.test(),
			Test::BeaconBlockHeader(test) => test.test(),
			Test::BeaconState(test) => test.test(),
			Test::Checkpoint(test) => test.test(),
			Test::CompactCommittee(test) => test.test(),
			Test::Crosslink(test) => test.test(),
			Test::Deposit(test) => test.test(),
			Test::DepositData(test) => test.test(),
			Test::Eth1Data(test) => test.test(),
			Test::Fork(test) => test.test(),
			Test::HistoricalBatch(test) => test.test(),
			Test::IndexedAttestation(test) => test.test(),
			Test::PendingAttestation(test) => test.test(),
			Test::ProposerSlashing(test) => test.test(),
			Test::Transfer(test) => test.test(),
			Test::Validator(test) => test.test(),
			Test::VoluntaryExit(test) => test.test(),
		}
	}
}

#[derive(Deserialize, Debug)]
pub struct TestItem<T> {
	value: T,
	serialized: String,
	root: H256,
	signing_root: Option<H256>,
}

impl<T: core::fmt::Debug + PartialEq + Encode + Decode + IntoTree<InMemoryBackend<Sha256, End>> + FromTree<InMemoryBackend<Sha256, End>>> TestItem<T> {
	pub fn test(&self) {
		print!("Testing {} ...", self.serialized);
		std::io::stdout().flush().ok().expect("Could not flush stdout");
		assert!(self.serialized.starts_with("0x"));
		let expected = hex::decode(&self.serialized[2..]).unwrap();
		let encoded = Encode::encode(&self.value);
		assert_eq!(encoded, expected);
		let decoded = T::decode(&encoded).unwrap();
		assert_eq!(decoded, self.value);
		let mut db = InMemoryBackend::new_with_inherited_empty();
		let encoded_root = self.value.into_tree(&mut db).unwrap();
		assert_eq!(H256::from_slice(encoded_root.as_ref()), self.root);
		let decoded_root = T::from_tree(&encoded_root, &db).unwrap();
		assert_eq!(decoded_root, self.value);

		println!(" passed");
	}
}

fn main() {
	let matches = App::new("ssztests")
        .version("0.1.0")
        .author("Parity Technologies <admin@parity.io>")
        .about("Serenity Ssz YAML test utilities")
        .arg(Arg::with_name("FILE")
             .help("Target yaml file to import")
             .required(true))
		.arg(Arg::with_name("CONFIG")
			 .help("Run tests with the given config")
			 .long("config")
			 .takes_value(true))
        .get_matches();

	let file = File::open(matches.value_of("FILE").expect("FILE parameter not found")).expect("Open file failed");
	let reader = BufReader::new(file);

	match matches.value_of("CONFIG") {
		Some("small") | None => {
			let coll = serde_yaml::from_reader::<_, Collection<MinimalConfig<BLSNoVerification>>>(reader)
				.expect("Parse test cases failed");
			for test in coll.test_cases {
				test.test()
			}
		},
		Some("full") => {
			let coll = serde_yaml::from_reader::<_, Collection<MainnetConfig<BLSNoVerification>>>(reader)
				.expect("Parse test cases failed");
			for test in coll.test_cases {
				test.test()
			}
		},
		_ => panic!("Unknown config"),
	}
}