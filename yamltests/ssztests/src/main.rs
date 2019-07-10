use std::fs::File;
use std::io::BufReader;

use clap::{App, Arg};
use beacon::{Config, NoVerificationConfig};
use beacon::primitives::*;
use beacon::types::*;
use serde::Deserialize;

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
	Fork {
		value: Fork,
		serialized: String,
		root: H256,
	},
	HistoricalBatch { },
	IndexedAttestation { },
	PendingAttestation { },
	ProposerSlashing { },
	Transfer { },
	Validator { },
	VoluntaryExit { },
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

	println!("collection: {:?}", coll);
}
