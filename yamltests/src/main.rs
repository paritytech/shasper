use clap::{App, Arg};

fn main() {
	let matches = App::new("yamltests")
        .version("0.1.0")
        .author("Parity Technologies <admin@parity.io>")
        .about("Serenity YAML test utilities")
        .arg(Arg::with_name("DIR")
             .help("Target yaml files to import")
             .required(true))
        .get_matches();

	let dir = matches.value_of("DIR").unwrap();
	let descs = yamltests::description::read_descriptions(dir).unwrap();

	for desc in descs {
		yamltests::test(desc);
	}
}
