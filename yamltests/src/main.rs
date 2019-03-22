use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use clap::{App, Arg};
use serde_derive::{Serialize, Deserialize};
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
	pub latest_randao_mixes: Option<Vec<H256>>,
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
	pub latest_crosslinks: Option<Vec<Crosslink>>,
	pub latest_block_roots: Option<Vec<H256>>,
	pub latest_state_roots: Option<Vec<H256>>,
	pub latest_active_index_roots: Option<Vec<H256>>,
	pub latest_slashed_balances: Option<Vec<u64>>,
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
	let coll = serde_yaml::from_reader::<_, Collection>(BufReader::new(file));
	println!("{:?}", coll);
}
