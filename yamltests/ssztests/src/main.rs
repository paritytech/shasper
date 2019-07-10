use std::fs::File;
use std::io::{BufReader, Write};

use clap::{App, Arg};
use beacon::{Config, NoVerificationConfig};
use beacon::primitives::*;
use beacon::types::*;
use serde::Deserialize;
use ssz::{Encode, Decode};
use bm_le::{IntoTree, NoopBackend, Intermediate, End};
use sha2::Sha256;

#[derive(Deserialize, Debug)]
pub struct Collection {
	pub title: String,
	pub test_cases: Vec<Test>,
}

#[derive(Deserialize, Debug)]
pub enum Test {
	Attestation { },
	AttestationData { },
	AttestationDataAndCustodyBit { },
	AttesterSlashing { },
	BeaconBlock { },
	BeaconBlockBody { },
	BeaconBlockHeader { },
	BeaconState { },
	Checkpoint { },
	CompactCommittee { },
	Crosslink { },
	Deposit { },
	DepositData { },
	Eth1Data { },
	Fork(TestItem<Fork>),
	HistoricalBatch { },
	IndexedAttestation { },
	PendingAttestation { },
	ProposerSlashing { },
	Transfer { },
	Validator { },
	VoluntaryExit { },
}

#[derive(Deserialize, Debug)]
pub struct TestItem<T> {
	value: T,
	serialized: String,
	root: H256,
	signing_root: Option<H256>,
}

impl<T: Encode + Decode + IntoTree<NoopBackend<Sha256, End>>> TestItem<T> {
	pub fn test(&self) {
		print!("Testing {} ...", self.serialized);
		std::io::stdout().flush().ok().expect("Could not flush stdout");
		assert!(self.serialized.starts_with("0x"));
		let expected = hex::decode(&self.serialized[2..]).unwrap();
		let encoded = Encode::encode(&self.value);
		assert_eq!(encoded, expected);
		let encoded_root = bm_le::tree_root(&self.value);
		assert_eq!(encoded_root, self.root);
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
	let config = match matches.value_of("CONFIG") {
		Some("small") | None => NoVerificationConfig::small(),
		Some("full") => NoVerificationConfig::full(),
		_ => panic!("Unknown config"),
	};

	let reader = BufReader::new(file);
	let coll = serde_yaml::from_reader::<_, Collection>(reader).expect("Parse test cases failed");

	for test in coll.test_cases {
		match test {
			Test::Attestation { } => (),
			Test::AttestationData { } => (),
			Test::AttestationDataAndCustodyBit { } => (),
			Test::AttesterSlashing { } => (),
			Test::BeaconBlock { } => (),
			Test::BeaconBlockBody { } => (),
			Test::BeaconBlockHeader { } => (),
			Test::BeaconState { } => (),
			Test::Checkpoint { } => (),
			Test::CompactCommittee { } => (),
			Test::Crosslink { } => (),
			Test::Deposit { } => (),
			Test::DepositData { } => (),
			Test::Eth1Data { } => (),
			Test::Fork(test) => test.test(),
			Test::HistoricalBatch { } => (),
			Test::IndexedAttestation { } => (),
			Test::PendingAttestation { } => (),
			Test::ProposerSlashing { } => (),
			Test::Transfer { } => (),
			Test::Validator { } => (),
			Test::VoluntaryExit { } => (),
		}
	}
}
