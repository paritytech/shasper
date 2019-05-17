use beacon::{genesis, Config, NoVerificationConfig, Inherent};
use beacon::primitives::{H256, Signature, ValidatorId};
use beacon::types::{Eth1Data, Deposit, DepositData};
use ssz::Digestible;
use blockchain::backend::{SharedBackend, MemoryBackend, MemoryLikeBackend};
use blockchain::chain::BlockBuilder;
use blockchain::traits::{ChainQuery, ImportOperation, Block as BlockT};
use blockchain_network_simple::BestDepthStatusProducer;
use shasper_blockchain::{Block, Executor, State};
use lmd_ghost::archive::{NoCacheAncestorBackend, ArchiveGhostImporter};
use clap::{App, Arg};
use std::thread;
use core::time::Duration;

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
				values[i * 2].as_ref(),
				values[i * 2 + 1].as_ref()
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
		let subindex = (item_index / 2usize.pow(i as u32)) ^ 1;
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
		.arg(Arg::with_name("author")
			 .long("author")
			 .help("Whether to author blocks"))
		.get_matches();

	let config = NoVerificationConfig::small();
	let mut deposit_datas = Vec::new();
	for i in 0..32 {
		deposit_datas.push(DepositData {
			pubkey: ValidatorId::from_low_u64_le(i as u64),
			withdrawal_credentials: H256::from_low_u64_le(i as u64),
			amount: 32000000000,
			signature: Signature::from_low_u64_le(i as u64),
		});
	}

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
	let executor = Executor::new(config.clone());
	let importer = ArchiveGhostImporter::new(executor, backend.clone());
	let status = BestDepthStatusProducer::new(backend.clone());
	let port = matches.value_of("port").unwrap_or("37365");
	let author = matches.is_present("author");

	if author {
		let backend_build = backend.clone();
		thread::spawn(move || {
			builder_thread(backend_build, config);
		});
	}

	blockchain_network_simple::libp2p::start_network_simple_sync(port, backend, importer, status);
}

fn builder_thread<C: Config>(
	backend: SharedBackend<NoCacheAncestorBackend<MemoryBackend<Block, (), State>>>,
	config: C,
) {
	let executor = Executor::new(config);

	loop {
		let head = backend.read().head();
		println!("Building on top of {}", head);

		let head_block = backend.read().block_at(&head).unwrap();
		let builder = BlockBuilder::new(&backend, &executor, &head, Inherent {
			slot: head_block.0.slot + 1,
			randao_reveal: Default::default(),
			eth1_data: head_block.0.body.eth1_data.clone(),
		}).unwrap();
		let (unsealed_block, state) = builder.finalize().unwrap();
		let block = Block(unsealed_block.fake_seal());

		// Import the built block.
		let mut build_importer = backend.begin_import(&executor);
		let new_block_hash = block.id();
		let op = ImportOperation { block, state };
		build_importer.import_raw(op);
		build_importer.set_head(new_block_hash);
		build_importer.commit().unwrap();

		thread::sleep(Duration::new(5, 0));
	}
}
