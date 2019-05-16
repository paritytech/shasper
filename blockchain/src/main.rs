use beacon::{genesis, Config, NoVerificationConfig};
use beacon::primitives::H256;
use beacon::types::{Eth1Data, Deposit, DepositData};
use ssz::Digestible;
use blockchain::backend::{SharedBackend, MemoryBackend, MemoryLikeBackend};
use blockchain_network_simple::BestDepthStatusProducer;
use shasper_blockchain::{Block, Executor, State};
use lmd_ghost::archive::{NoCacheAncestorBackend, ArchiveGhostImporter};
use clap::{App, Arg};

fn deposit_tree<C: Config>(deposits: &[DepositData], config: &C) -> Vec<Vec<H256>> {
	let mut zerohashes = vec![H256::default()];
	for layer in 1..32 {
		zerohashes.push(config.hash(&[
			zerohashes[layer - 1].as_ref(),
			zerohashes[layer - 1].as_ref(),
		]));
	}

	let mut values = deposits.iter().map(|d| {
		H256::from_slice(
			Digestible::<C::Digest>::hash(d).as_slice()
		)
	}).collect::<Vec<_>>();
	let mut tree = vec![values.clone()];

	for h in 0..(config.deposit_contract_tree_depth() as usize) {
		if values.len() % 2 == 1 {
			values.push(zerohashes[h]);
		}
		let mut new_values = Vec::new();
		for i in 0..(values.len() / 2) {
			new_values.push(config.hash(&[
				values[i].as_ref(),
				values[i + 1].as_ref()
			]));
		}
		values = new_values;
		tree.push(values.clone());
	}

	tree
}

fn deposit_root(tree: &Vec<Vec<H256>>) -> H256 {
	tree.last().expect("Merkle tree cannot be empty; qed")[0]
}

fn deposit_proof<C: Config>(tree: &Vec<Vec<H256>>, item_index: usize, config: &C) -> Vec<H256> {
	let mut zerohashes = vec![H256::default()];
	for layer in 1..32 {
		zerohashes.push(config.hash(&[
			zerohashes[layer - 1].as_ref(),
			zerohashes[layer - 1].as_ref(),
		]));
	}

	let mut proof = Vec::new();
	for i in 0..(config.deposit_contract_tree_depth() as usize) {
		let subindex = (item_index / (0b1 << i)) ^ 1;
		if subindex < tree[i].len() {
			proof.push(tree[i][subindex]);
		} else {
			proof.push(zerohashes[i]);
		}
	}
	proof
}

fn main() {
	let matches = App::new("Shasper blockchain client")
		.arg(Arg::with_name("port")
			 .short("p")
			 .long("port")
			 .takes_value(true)
			 .help("Port to listen on"))
		.get_matches();

	let config = NoVerificationConfig::small();
	let deposit_datas = vec![DepositData {
		pubkey: Default::default(),
		withdrawal_credentials: Default::default(),
		amount: 100000000000000,
		signature: Default::default(),
	}];
	let deposit_tree = deposit_tree(&deposit_datas, &config);
	let deposits = deposit_datas.clone().into_iter()
		.enumerate()
		.map(|(i, deposit_data)| {
			Deposit {
				proof: deposit_proof(&deposit_tree, i, &config),
				index: i as u64,
				data: deposit_data,
			}
		})
		.collect::<Vec<_>>();
	let deposit_root = deposit_root(&deposit_tree);
	let (genesis_beacon_block, genesis_state) = genesis(
		&deposits, 0, Eth1Data {
			deposit_root,
			deposit_count: deposits.len() as u64,
			block_hash: Default::default(),
		}, &config
	).unwrap();
	let genesis_block = Block(genesis_beacon_block);
	let backend = SharedBackend::new(
		NoCacheAncestorBackend::<MemoryBackend<Block, (), State>>::new_with_genesis(
			genesis_block.clone(),
			genesis_state.into(),
		)
	);
	let executor = Executor::new(config);
	let importer = ArchiveGhostImporter::new(executor, backend.clone());
	let status = BestDepthStatusProducer::new(backend.clone());
	let port = matches.value_of("port").unwrap_or("37365");

	blockchain_network_simple::libp2p::start_network_simple_sync(port, backend, importer, status);
}
