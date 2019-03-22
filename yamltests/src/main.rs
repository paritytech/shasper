use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use clap::{App, Arg};
use serde::{Serialize, Deserialize};
use serenity::{BeaconState, BeaconBlock};

#[derive(Serialize, Deserialize, Debug)]
pub struct Collection {
	pub title: String,
	pub summary: String,
	pub test_suite: String,
	pub fork: String,
	pub test_cases: Vec<Test>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Test {
	pub name: String,
	pub config: HashMap<String, String>,
	pub verify_signatures: bool,
	pub initial_state: BeaconState,
	pub blocks: Vec<BeaconBlock>,
}

fn main() {
	let matches = App::new("yamltests")
        .version("0.1.0")
        .author("Parity Technologies <admin@parity.io>")
        .about("Serenity YAML test utilities")
        .arg(Arg::with_name("FILE")
             .help("Target yaml file to import")
             .required(true))
        .get_matches();

	let file = File::open(matches.value_of("FILE").expect("FILE parameter not found")).expect("Open file failed");
	let coll = serde_yaml::from_reader::<_, Collection>(BufReader::new(file));
	println!("{:?}", coll);
}
