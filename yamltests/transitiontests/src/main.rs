use std::fs::File;
use std::io::BufReader;

use clap::{App, Arg};
use beacon::{Config, MainnetConfig, MinimalConfig};
use serde::de::DeserializeOwned;
use transitiontests::*;

fn main() {
	let matches = App::new("yamltests")
        .version("0.1.0")
        .author("Parity Technologies <admin@parity.io>")
        .about("Serenity YAML test utilities")
        .arg(Arg::with_name("FILE")
             .help("Target yaml file to import")
             .required(true))
		.arg(Arg::with_name("RUNNER")
			 .help("Runner of the test")
			 .long("runner")
			 .short("r")
			 .takes_value(true)
			 .required(true))
		.arg(Arg::with_name("CONFIG")
			 .help("Run tests with the given config")
			 .long("config")
			 .takes_value(true))
        .get_matches();

	let file = File::open(matches.value_of("FILE").expect("FILE parameter not found")).expect("Open file failed");
	let runner = matches.value_of("RUNNER").expect("RUN parameter not found");

	match matches.value_of("CONFIG") {
		Some("small") | None => run_all::<MinimalConfig>(&runner, file),
		Some("full") => run_all::<MainnetConfig>(&runner, file),
		_ => panic!("Unknown config"),
	}
}

fn run_all<C: Config + DeserializeOwned>(runner: &str, file: File) {
	match runner {
		// "attestation" => run::<AttestationTest, _>(file, &config),
		// "attester_slashing" => run::<AttesterSlashingTest, _>(file, &config),
		// "block_header" => run::<BlockHeaderTest, _>(file, &config),
		// "deposit" => run::<DepositTest, _>(file, &config),
		"proposer_slashing" => run::<ProposerSlashingTest<C>>(file),
		// "transfer" => run::<TransferTest, _>(file, &config),
		// "voluntary_exit" => run::<VoluntaryExitTest, _>(file, &config),
		// "crosslinks" => run::<CrosslinksTest, _>(file, &config),
		// "registry_updates" => run::<RegistryUpdatesTest, _>(file, &config),
		// "blocks" => run::<BlocksTest, _>(file, &config),
		// "slots" => run::<SlotsTest, _>(file, &config),
		_ => panic!("Unsupported runner"),
	}
}

fn run<T: Test + DeserializeOwned>(file: File) {
	let reader = BufReader::new(file);
	let coll = serde_yaml::from_reader::<_, Collection<T>>(reader).expect("Parse test cases failed");

	run_collection(coll);
}
