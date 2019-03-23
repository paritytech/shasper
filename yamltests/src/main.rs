use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Write};

use clap::{App, Arg};
use serde_derive::{Serialize, Deserialize};
use ssz::FixedVec;
use primitives::H256;
use serenity::{BeaconState, BeaconBlock, Slot, Fork, Timestamp, Validator, Epoch, Shard, Eth1Data, Eth1DataVote, PendingAttestation, Crosslink, BeaconBlockHeader};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExpectedBeaconState {
	// Misc
	pub slot: Option<Slot>,
	pub genesis_time: Option<Timestamp>,
	pub fork: Option<Fork>,

	// Validator registry
	pub validator_registry: Option<Vec<Validator>>,
	pub validator_balances: Option<Vec<u64>>,
	pub validator_registry_update_epoch: Option<Epoch>,

	// Randomness and committees
	pub latest_randao_mixes: Option<FixedVec<H256>>,
	pub previous_shuffling_start_shard: Option<Shard>,
	pub current_shuffling_start_shard: Option<Shard>,
	pub previous_shuffling_epoch: Option<Epoch>,
	pub current_shuffling_epoch: Option<Epoch>,
	pub previous_shuffling_seed: Option<H256>,
	pub current_shuffling_seed: Option<H256>,

	// Finality
	pub previous_epoch_attestations: Option<Vec<PendingAttestation>>,
	pub current_epoch_attestations: Option<Vec<PendingAttestation>>,
	pub previous_justified_epoch: Option<Epoch>,
	pub current_justified_epoch: Option<Epoch>,
	pub previous_justified_root: Option<H256>,
	pub current_justified_root: Option<H256>,
	pub justification_bitfield: Option<u64>,
	pub finalized_epoch: Option<Epoch>,
	pub finalized_root: Option<H256>,

	// Recent state
	pub latest_crosslinks: Option<FixedVec<Crosslink>>,
	pub latest_block_roots: Option<FixedVec<H256>>,
	pub latest_state_roots: Option<FixedVec<H256>>,
	pub latest_active_index_roots: Option<FixedVec<H256>>,
	pub latest_slashed_balances: Option<FixedVec<u64>>,
	pub latest_block_header: Option<BeaconBlockHeader>,
	pub historical_roots: Option<Vec<H256>>,

	// Ethereum 1.0 chain data
	pub latest_eth1_data: Option<Eth1Data>,
	pub eth1_data_votes: Option<Vec<Eth1DataVote>>,
	pub deposit_index: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Collection {
	pub title: String,
	pub summary: String,
	pub test_suite: String,
	pub fork: String,
	pub test_cases: Vec<Test>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Test {
	pub name: String,
	pub config: HashMap<String, String>,
	pub verify_signatures: bool,
	pub initial_state: BeaconState,
	pub blocks: Vec<BeaconBlock>,
	pub expected_state: ExpectedBeaconState,
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
	let coll = serde_yaml::from_reader::<_, Collection>(BufReader::new(file)).expect("Parse test cases failed");

	for test in coll.test_cases {
		run_test(test);
	}
}

fn run_test(test: Test) {
	print!("Running test: {} ...", test.name);
	io::stdout().flush().ok().expect("Could not flush stdout");
	let mut state = test.initial_state;

	for block in test.blocks {
		match serenity::execute_block(&block, &mut state) {
			Ok(()) => {
				println!(" done");
			},
			Err(err) => {
				println!(" failed\n");
				println!("Error: {:?}", err);
				panic!();
			}
		}
	}

	check_expected(&state, test.expected_state);
}

fn check_expected(state: &BeaconState, expected: ExpectedBeaconState) {
	macro_rules! check {
		( $($field:tt,)+ ) => {
			$(
				if let Some($field) = expected.$field {
					if $field != state.$field {
						println!("\nExpected state check failed for {}", stringify!($field));
						println!("Expected: {:?}", $field);
						println!("Actual: {:?}", state.$field);
						panic!();
					}
				}
			)+
		}
	}

	check!(
		// Misc
		slot, genesis_time, fork,
		// Validator registry
		validator_registry, validator_balances, validator_registry_update_epoch,
		// Randomness and committees
		latest_randao_mixes, previous_shuffling_start_shard,
		current_shuffling_start_shard, previous_shuffling_epoch,
		current_shuffling_epoch, previous_shuffling_seed,
		current_shuffling_seed,
		// Finality
		previous_epoch_attestations, current_epoch_attestations,
		previous_justified_epoch, current_justified_epoch,
		previous_justified_root, current_justified_root,
		justification_bitfield, finalized_epoch, finalized_root,
		// Recent state
		latest_crosslinks, latest_block_roots, latest_state_roots,
		latest_active_index_roots, latest_slashed_balances,
		latest_block_header, historical_roots,
		// Ethereum 1.0 chain data
		latest_eth1_data, eth1_data_votes, deposit_index,
	);
}
