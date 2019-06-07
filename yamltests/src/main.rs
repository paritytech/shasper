use std::fs::File;
use std::io::BufReader;

use clap::{App, Arg};
use beacon::{Config, NoVerificationConfig};
use serde::de::DeserializeOwned;
use yamltests::{Test, Collection, DepositTest, CrosslinksTest, run_collection};

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
	let config = match matches.value_of("CONFIG") {
		Some("small") | None => NoVerificationConfig::small(),
		Some("full") => NoVerificationConfig::full(),
		_ => panic!("Unknown config"),
	};
	match matches.value_of("RUNNER").expect("RUN parameter not found") {
		"deposit" => run::<DepositTest, _>(file, &config),
		"crosslinks" => run::<CrosslinksTest, _>(file, &config),
		_ => panic!("Unsupported runner"),
	}
}

fn run<T: Test + DeserializeOwned, C: Config>(file: File, config: &C) {
	let reader = BufReader::new(file);
	let coll = serde_yaml::from_reader::<_, Collection<T>>(reader).expect("Parse test cases failed");

	run_collection(coll, config);
}
