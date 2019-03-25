use std::fs::File;
use std::io::BufReader;

use clap::{App, Arg};
use yamltests::{Collection, run_collection};

fn main() {
	let matches = App::new("yamltests")
        .version("0.1.0")
        .author("Parity Technologies <admin@parity.io>")
        .about("Serenity YAML test utilities")
        .arg(Arg::with_name("FILE")
             .help("Target yaml file to import")
             .required(true))
		.arg(Arg::with_name("only")
			 .help("Only run the particular test")
			 .long("only")
			 .takes_value(true))
        .get_matches();

	let file = File::open(matches.value_of("FILE").expect("FILE parameter not found")).expect("Open file failed");
	let only = matches.value_of("only");
	let coll = serde_yaml::from_reader::<_, Collection>(BufReader::new(file)).expect("Parse test cases failed");

	run_collection(coll, only);
}
