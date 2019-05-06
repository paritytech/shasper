use std::fs::File;
use std::io::BufReader;

use clap::{App, Arg};
use beacon::NoVerificationConfig;
use yamltests::{Collection, DepositTest, run_collection};

fn main() {
	let matches = App::new("yamltests")
        .version("0.1.0")
        .author("Parity Technologies <admin@parity.io>")
        .about("Serenity YAML test utilities")
        .arg(Arg::with_name("FILE")
             .help("Target yaml file to import")
             .required(true))
		.arg(Arg::with_name("config")
			 .help("Run tests with the given config")
			 .long("config")
			 .takes_value(true))
        .get_matches();

	let file = File::open(matches.value_of("FILE").expect("FILE parameter not found")).expect("Open file failed");
	let coll = serde_yaml::from_reader::<_, Collection<DepositTest>>(BufReader::new(file)).expect("Parse test cases failed");
	let config = match matches.value_of("config") {
		Some("small") | None => NoVerificationConfig::small(),
		Some("full") => NoVerificationConfig::full(),
		_ => panic!("Unknown config"),
	};

	run_collection(coll, &config);
}
