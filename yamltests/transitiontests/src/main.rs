// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.










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

fn run_all<C: Config + serde::Serialize + DeserializeOwned>(runner: &str, file: File) {
	match runner {
		"attestation" => run::<AttestationTest<C>>(file),
		"attester_slashing" => run::<AttesterSlashingTest<C>>(file),
		"block_header" => run::<BlockHeaderTest<C>>(file),
		"deposit" => run::<DepositTest<C>>(file),
		"proposer_slashing" => run::<ProposerSlashingTest<C>>(file),
		"transfer" => run::<TransferTest<C>>(file),
		"voluntary_exit" => run::<VoluntaryExitTest<C>>(file),
		"justification_and_finalization" => run::<JustificationAndFinalizationTest<C>>(file),
		"crosslinks" => run::<CrosslinksTest<C>>(file),
		"registry_updates" => run::<RegistryUpdatesTest<C>>(file),
		"slashings" => run::<SlashingsTest<C>>(file),
		"final_updates" => run::<FinalUpdatesTest<C>>(file),
		"blocks" => run::<BlocksTest<C>>(file),
		"slots" => run::<SlotsTest<C>>(file),
		_ => panic!("Unsupported runner"),
	}
}

fn run<T: Test + DeserializeOwned>(file: File) {
	let reader = BufReader::new(file);
	let coll = serde_yaml::from_reader::<_, Collection<T>>(reader).expect("Parse test cases failed");

	run_collection(coll);
}

#[cfg(test)]
mod spectests {
	use super::*;
	use std::path::PathBuf;

	macro_rules! run {
		( $name:ident, $typ:ty, $path:expr ) => {
			#[test]
			fn $name() {
				let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
				d.push("..");
				d.push($path);
				let file = File::open(d).unwrap();
				let reader = BufReader::new(file);
				let coll = serde_yaml::from_reader::<_, Collection<$typ>>(reader)
					.expect("Parse test cases failed");

				run_collection(coll)
			}
		}
	}

	run!(attestation_mainnet, AttestationTest<MainnetConfig>,
		 "spectests/tests/operations/attestation/attestation_mainnet.yaml");
	run!(attestation_minimal, AttestationTest<MinimalConfig>,
		 "spectests/tests/operations/attestation/attestation_minimal.yaml");

	run!(attester_slashing_mainnet, AttesterSlashingTest<MainnetConfig>,
		 "spectests/tests/operations/attester_slashing/attester_slashing_mainnet.yaml");
	run!(attester_slashing_minimal, AttesterSlashingTest<MinimalConfig>,
		 "spectests/tests/operations/attester_slashing/attester_slashing_minimal.yaml");

	run!(block_header_mainnet, BlockHeaderTest<MainnetConfig>,
		 "spectests/tests/operations/block_header/block_header_mainnet.yaml");
	run!(block_header_minimal, BlockHeaderTest<MinimalConfig>,
		 "spectests/tests/operations/block_header/block_header_minimal.yaml");

	run!(deposit_mainnet, DepositTest<MainnetConfig>,
		 "spectests/tests/operations/deposit/deposit_mainnet.yaml");
	run!(deposit_minimal, DepositTest<MinimalConfig>,
		 "spectests/tests/operations/deposit/deposit_minimal.yaml");

	run!(proposer_slashing_mainnet, ProposerSlashingTest<MainnetConfig>,
		 "spectests/tests/operations/proposer_slashing/proposer_slashing_mainnet.yaml");
	run!(proposer_slashing_minimal, ProposerSlashingTest<MinimalConfig>,
		 "spectests/tests/operations/proposer_slashing/proposer_slashing_minimal.yaml");

	run!(transfer_minimal, TransferTest<MinimalConfig>,
		 "spectests/tests/operations/transfer/transfer_minimal.yaml");

	run!(voluntary_exit_mainnet, VoluntaryExitTest<MainnetConfig>,
		 "spectests/tests/operations/voluntary_exit/voluntary_exit_mainnet.yaml");
	run!(voluntary_exit_minimal, VoluntaryExitTest<MinimalConfig>,
		 "spectests/tests/operations/voluntary_exit/voluntary_exit_minimal.yaml");

	run!(crosslinks_mainnet, CrosslinksTest<MainnetConfig>,
		 "spectests/tests/epoch_processing/crosslinks/crosslinks_mainnet.yaml");
	run!(crosslinks_minimal, CrosslinksTest<MinimalConfig>,
		 "spectests/tests/epoch_processing/crosslinks/crosslinks_minimal.yaml");

	run!(final_updates_mainnet, FinalUpdatesTest<MainnetConfig>,
		 "spectests/tests/epoch_processing/final_updates/final_updates_mainnet.yaml");
	run!(final_updates_minimal, FinalUpdatesTest<MinimalConfig>,
		 "spectests/tests/epoch_processing/final_updates/final_updates_minimal.yaml");

	run!(justification_and_finalization_mainnet, JustificationAndFinalizationTest<MainnetConfig>,
		 "spectests/tests/epoch_processing/justification_and_finalization/justification_and_finalization_mainnet.yaml");
	run!(justification_and_finalization_minimal, JustificationAndFinalizationTest<MinimalConfig>,
		 "spectests/tests/epoch_processing/justification_and_finalization/justification_and_finalization_minimal.yaml");

	run!(registry_updates_mainnet, RegistryUpdatesTest<MainnetConfig>,
		 "spectests/tests/epoch_processing/registry_updates/registry_updates_mainnet.yaml");
	run!(registry_updates_minimal, RegistryUpdatesTest<MinimalConfig>,
		 "spectests/tests/epoch_processing/registry_updates/registry_updates_minimal.yaml");

	run!(slashings_mainnet, SlashingsTest<MainnetConfig>,
		 "spectests/tests/epoch_processing/slashings/slashings_mainnet.yaml");
	run!(slashings_minimal, SlashingsTest<MinimalConfig>,
		 "spectests/tests/epoch_processing/slashings/slashings_minimal.yaml");

	run!(sanity_blocks_mainnet, BlocksTest<MainnetConfig>,
		 "spectests/tests/sanity/blocks/sanity_blocks_mainnet.yaml");
	run!(sanity_blocks_minimal, BlocksTest<MinimalConfig>,
		 "spectests/tests/sanity/blocks/sanity_blocks_minimal.yaml");

	run!(sanity_slots_mainnet, SlotsTest<MainnetConfig>,
		 "spectests/tests/sanity/slots/sanity_slots_mainnet.yaml");
	run!(sanity_slots_minimal, SlotsTest<MinimalConfig>,
		 "spectests/tests/sanity/slots/sanity_slots_minimal.yaml");
}
