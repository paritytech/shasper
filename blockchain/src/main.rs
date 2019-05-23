use beacon::{genesis, Config, ParameteredConfig, Inherent};
use beacon::primitives::{H256, Signature, ValidatorId};
use beacon::types::{Eth1Data, Deposit, DepositData};
use ssz::Digestible;
use blockchain::backend::{SharedBackend, MemoryBackend, MemoryLikeBackend};
use blockchain::chain::SharedImportBlock;
use blockchain::traits::{ChainQuery, AsExternalities, ImportBlock};
use blockchain_network_simple::BestDepthStatusProducer;
use shasper_blockchain::{Block, Executor, State, AMCLVerification};
use lmd_ghost::archive::{NoCacheAncestorBackend, ArchiveGhostImporter};
use clap::{App, Arg};
use std::thread;
use std::collections::HashMap;
use core::time::Duration;
use bls_aggregates as bls;

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

	let config = ParameteredConfig::<AMCLVerification>::small();
	let mut keys: HashMap<ValidatorId, bls::SecretKey> = HashMap::new();
	let mut deposit_datas = Vec::new();
	for i in 0..32 {
		let seckey = bls::SecretKey::random(&mut rand::thread_rng());
		let pubkey = ValidatorId::from_slice(&bls::PublicKey::from_secret_key(&seckey).as_bytes()[..]);
		let mut data = DepositData {
			pubkey: pubkey.clone(),
			withdrawal_credentials: H256::from_low_u64_le(i as u64),
			amount: 32000000000,
			signature: Default::default(),
		};
		let signature = Signature::from_slice(&bls::Signature::new(
			Digestible::<sha2::Sha256>::truncated_hash(&data).as_slice(),
			beacon::genesis_domain(config.domain_deposit()),
			&seckey
		).as_bytes()[..]);
		data.signature = signature;
		deposit_datas.push(data);
		keys.insert(pubkey, seckey);
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
	let eth1_data = Eth1Data {
		deposit_root,
		deposit_count: deposits.len() as u64,
		block_hash: Default::default(),
	};
	let (genesis_beacon_block, genesis_state) = genesis(
		&deposits, 0, eth1_data.clone(), &config
	).unwrap();
	let genesis_block = Block(genesis_beacon_block);
	let backend = SharedBackend::new(
		NoCacheAncestorBackend::<MemoryBackend<Block, (), State>>::new_with_genesis(
			genesis_block.clone(),
			genesis_state.into(),
		)
	);
	let executor = Executor::new(config.clone());
	let importer = SharedImportBlock::new(
		ArchiveGhostImporter::new(executor, backend.clone())
	);
	let status = BestDepthStatusProducer::new(backend.clone());
	let port = matches.value_of("port").unwrap_or("37365");
	let author = matches.is_present("author");

	if author {
		let backend_build = backend.clone();
		let importer_build = importer.clone();
		thread::spawn(move || {
			builder_thread(backend_build, importer_build, eth1_data, keys, config);
		});
	}

	blockchain_network_simple::libp2p::start_network_simple_sync(port, backend, importer, status);
}

fn builder_thread<C: Config + Clone>(
	backend: SharedBackend<NoCacheAncestorBackend<MemoryBackend<Block, (), State>>>,
	mut importer: SharedImportBlock<ArchiveGhostImporter<Executor<C>, NoCacheAncestorBackend<MemoryBackend<Block, (), State>>>>,
	eth1_data: Eth1Data,
	keys: HashMap<ValidatorId, bls::SecretKey>,
	config: C,
) {
	let executor = Executor::new(config.clone());

	loop {
		thread::sleep(Duration::new(1, 0));

		let head = backend.read().head();
		println!("Building on top of {}", head);

		let block = {
			let head_block = backend.read().block_at(&head).unwrap();
			let head_state = backend.read().state_at(&head).unwrap();

			let mut state = head_state;
			let current_slot = head_block.0.slot + 1;
			executor.initialize_block(
				state.as_externalities(), current_slot,
			).unwrap();
			let current_epoch = executor.current_epoch(state.as_externalities()).unwrap();

			let randao_domain = executor.domain(
				state.as_externalities(),
				config.domain_randao(),
				None
			).unwrap();
			let proposer_domain = executor.domain(
				state.as_externalities(),
				config.domain_beacon_proposer(),
				None
			).unwrap();

			for (validator_id, _) in &keys {
				let validator_index = executor.validator_index(
					validator_id,
					state.as_externalities()
				).unwrap();

				if let Some(validator_index) = validator_index {
					let committee_assignment = executor.committee_assignment(
						state.as_externalities(),
						current_epoch,
						validator_index,
					).unwrap();
					if let Some(committee_assignment) = committee_assignment {
						if committee_assignment.slot == current_slot {
							println!(
								"Found validator {} attesting slot {} with shard {}",
								validator_id, current_slot, committee_assignment.shard);

							let epoch_start_slot = config.epoch_start_slot(current_epoch);
							let epoch_boundary_block =

							let data = AttestationData {
								beacon_block_root: H256::from_slice(
									Digestible::<C::Digest>::truncated_hash(&head_block).as_slice(),
								),
								source_epoch: state.state().current_justified_epoch,
								source_root: state.state().current_justified_root,
								target_epoch: current_epoch,
								target_root:
							};
						}
					}
				}
			}

			let proposer_index = executor.proposer_index(state.as_externalities()).unwrap();
			let proposer_pubkey = executor
				.validator_pubkey(proposer_index, state.as_externalities())
				.unwrap();
			println!("Current proposer {} ({}) on epoch {}", proposer_index, proposer_pubkey, current_epoch);

			let seckey = match keys.get(&proposer_pubkey) {
				Some(value) => value.clone(),
				None => {
					println!("No secret key, skip building block.");
					continue;
				},
			};
			let randao_reveal = Signature::from_slice(&bls::Signature::new(
				Digestible::<C::Digest>::hash(&current_epoch).as_slice(),
				randao_domain,
				&seckey
			).as_bytes()[..]);

			let mut unsealed_block = executor.apply_inherent(
				&head_block, state.as_externalities(),
				Inherent {
					randao_reveal,
					eth1_data: eth1_data.clone(),
				}
			).unwrap();

			executor.finalize_block(
				&mut unsealed_block, state.as_externalities()
			).unwrap();

			let mut block = unsealed_block.fake_seal();
			let signature = Signature::from_slice(&bls::Signature::new(
				Digestible::<C::Digest>::truncated_hash(&block).as_slice(),
				proposer_domain,
				&seckey
			).as_bytes()[..]);
			block.signature = signature;
			Block(block)
		};

		importer.import_block(block).unwrap();
	}
}
