use beacon::{genesis, Config, ParameteredConfig, Inherent, Transaction};
use beacon::primitives::{H256, Signature, ValidatorId, BitField};
use beacon::types::{Eth1Data, Deposit, DepositData, AttestationData, AttestationDataAndCustodyBit, Attestation, Crosslink};
use ssz::Digestible;
use core::cmp::min;
use blockchain::backend::{SharedMemoryBackend, SharedCommittable, ChainQuery, Store, ImportLock, Operation};
use blockchain::import::{SharedBlockImporter, MutexImporter};
use blockchain::traits::{AsExternalities, Auxiliary, Block as BlockT};
use blockchain_network_simple::BestDepthStatusProducer;
use shasper_blockchain::{Block, Executor, State, Error, StateExternalities, AttestationPool};
use shasper_blockchain::rocksdb::RocksBackend;
use shasper_blockchain::backend::ShasperBackend;
use lmd_ghost::archive::{ArchiveGhostImporter, AncestorQuery};
use clap::{App, Arg};
use std::thread;
use std::collections::HashMap;
use core::time::Duration;
use crypto::bls;

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
		.arg(Arg::with_name("data")
			 .short("d")
			 .long("data")
			 .takes_value(true)
			 .help("Use rocksdb instead of in-memory database"))
		.arg(Arg::with_name("author")
			 .long("author")
			 .help("Whether to author blocks"))
		.get_matches();

	let config = ParameteredConfig::<bls::Verification>::small();
	let mut keys: HashMap<ValidatorId, bls::Secret> = HashMap::new();
	let mut deposit_datas = Vec::new();
	for i in 0..32 {
		let seckey = bls::Secret::random(&mut rand::thread_rng());
		let pubkey = ValidatorId::from_slice(&bls::Public::from_secret_key(&seckey).as_bytes()[..]);
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

	if let Some(path) = matches.value_of("data") {
		println!("Using RocksDB backend");
		let backend = ShasperBackend::new(
			RocksBackend::<_, (), State>::open_or_create(path, || {
				Ok((genesis_block.clone(), genesis_state.into()))
			}).unwrap()
		);
		let lock = ImportLock::new();

		run(matches.value_of("port").unwrap_or("37365"),
			matches.is_present("author"),
			backend,
			lock,
			eth1_data,
			keys,
			config);
	} else {
		println!("Using in-memory backend");
		let backend = ShasperBackend::new(
			SharedMemoryBackend::<_, (), State>::new_with_genesis(
				genesis_block.clone(),
				genesis_state.into(),
			)
		);
		let lock = ImportLock::new();

		run(matches.value_of("port").unwrap_or("37365"),
			matches.is_present("author"),
			backend,
			lock,
			eth1_data,
			keys,
			config);
	}
}

fn run<B, C: Config>(
	port: &str,
	author: bool,
	backend: B,
	import_lock: ImportLock,
	eth1_data: Eth1Data,
	keys: HashMap<ValidatorId, bls::Secret>,
	config: C,
) where
	B: ChainQuery + AncestorQuery + Store<Block=Block, State=State>,
	B::Auxiliary: Auxiliary<Block>,
	B: SharedCommittable<Operation=Operation<<B as Store>::Block, <B as Store>::State, <B as Store>::Auxiliary>>,
	B: Send + Sync + 'static,
	C: Clone + Send + Sync + 'static,
	blockchain::import::Error: From<B::Error>,
{
	let executor = Executor::new(config.clone());
	let importer = MutexImporter::new(
		ArchiveGhostImporter::new(executor, backend.clone(), import_lock.clone())
	);
	let status = BestDepthStatusProducer::new(backend.clone());

	if author {
		let backend_build = backend.clone();
		let importer_build = importer.clone();
		thread::spawn(move || {
			builder_thread(backend_build, importer_build, eth1_data, keys, config);
		});
	}

	blockchain_network_simple::libp2p::start_network_simple_sync(port, backend, import_lock, importer, status);
}

fn builder_thread<B, I, C: Config + Clone>(
	backend: B,
	importer: I,
	eth1_data: Eth1Data,
	keys: HashMap<ValidatorId, bls::Secret>,
	config: C,
) where
	B: ChainQuery + Store<Block=Block, State=State>,
	B::Auxiliary: Auxiliary<Block>,
	I: SharedBlockImporter<Block=Block>
{
	let executor = Executor::new(config.clone());
	let mut attestations = AttestationPool::new(&config);

	loop {
		thread::sleep(Duration::new(1, 0));

		let head = backend.head();
		println!("Building on top of {}", head);

		let block = {
			let head_block = backend.block_at(&head).unwrap();
			let mut head_state = backend.state_at(&head).unwrap();
			println!("Justified epoch {}, finalized epoch {}",
					 { head_state.state().current_justified_epoch },
					 { head_state.state().finalized_epoch });

			let mut state = backend.state_at(&head).unwrap();
			let externalities = state.as_externalities();
			let current_slot = head_block.0.slot + 1;
			executor.initialize_block(externalities, current_slot).unwrap();
			let current_epoch = executor.executive(externalities).current_epoch();

			let randao_domain = executor.executive(externalities)
				.domain(config.domain_randao(), None);
			let proposer_domain = executor.executive(externalities)
				.domain(config.domain_beacon_proposer(), None);
			let attestation_domain = executor.executive(externalities)
				.domain(config.domain_attestation(), None);

			for (validator_id, validator_seckey) in &keys {
				let validator_index = externalities.state().validator_index(validator_id);

				if let Some(validator_index) = validator_index {
					let committee_assignment = executor.executive(externalities)
						.committee_assignment(current_epoch, validator_index).unwrap();
					if let Some(committee_assignment) = committee_assignment {
						if committee_assignment.slot == current_slot {
							println!(
								"Found validator {} attesting slot {} with shard {}",
								validator_id, current_slot, committee_assignment.shard);
							let shard = committee_assignment.shard;
							let committee = committee_assignment.validators;

							let target_epoch = current_epoch;
							let target_slot = config.epoch_start_slot(target_epoch);
							let target_root = if target_slot == current_slot {
								head
							} else {
								executor.executive(externalities)
									.block_root(target_epoch).unwrap()
							};
							let source_epoch = externalities.state().current_justified_epoch;
							let source_root = externalities.state().current_justified_root;
							println!(
								"Casper source {} ({}) to target {} ({})",
								source_epoch, source_root, target_epoch, target_root,
							);

							let parent_crosslink = head_state.state()
								.current_crosslinks[shard as usize].clone();

							let data = AttestationData {
								beacon_block_root: head_block.id(),

								source_epoch, source_root, target_epoch, target_root,

								crosslink: Crosslink {
									shard,
									start_epoch: parent_crosslink.end_epoch,
									end_epoch: min(
										target_epoch,
										parent_crosslink.end_epoch + config.max_epochs_per_crosslink()
									),
									parent_root: H256::from_slice(
										Digestible::<C::Digest>::hash(&parent_crosslink).as_slice(),
									),
									data_root: H256::default(),
								},
							};
							let signature = Signature::from_slice(&bls::Signature::new(
								Digestible::<C::Digest>::hash(&AttestationDataAndCustodyBit {
									data: data.clone(),
									custody_bit: false,
								}).as_slice(),
								attestation_domain,
								&validator_seckey,
							).as_bytes()[..]);

							let index_into_committee = committee.iter()
								.position(|v| *v == validator_index).unwrap();
							let mut aggregation_bitfield = BitField::new(committee.len());
							aggregation_bitfield.set_bit(index_into_committee, true);
							let custody_bitfield = BitField::new(committee.len());

							let attestation = Attestation {
								aggregation_bitfield, data, custody_bitfield, signature
							};

							attestations.push(attestation);
						}
					}
				}
			}

			let proposer_index = executor.executive(externalities).beacon_proposer_index().unwrap();
			let proposer_pubkey = externalities.state().validator_pubkey(proposer_index).unwrap();
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

			let mut collected_attestations = Vec::new();
			for (hash, attestation) in attestations.iter() {
				match executor.apply_extrinsic(
					&mut unsealed_block, state.as_externalities(),
					Transaction::Attestation(attestation.clone())
				) {
					Ok(()) => {
						collected_attestations.push(*hash);
					},
					Err(Error::Beacon(ref err)) if err == &beacon::Error::AttestationSubmittedTooQuickly => {},
					Err(err) => {
						println!("Warning: error when submitting an attestation: {}", err);
					},
				}
			}
			println!("Pushed {} attestations", collected_attestations.len());
			for hash in collected_attestations {
				attestations.pop(&hash);
			}

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
