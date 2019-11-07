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
use beacon::{genesis_beacon_state, Config, MinimalConfig, Inherent, Transaction};
use beacon::primitives::*;
use beacon::types::*;
use core::cmp::min;
use blockchain::{AsExternalities, Auxiliary, Block as BlockT};
use blockchain::backend::{SharedMemoryBackend, SharedCommittable, ChainQuery, Store, ImportLock, Operation};
use blockchain::import::{SharedBlockImporter, MutexImporter};
use blockchain_rocksdb::RocksBackend;
use shasper_blockchain::{Block, Executor, MemoryState, RocksState, Error, StateExternalities, AttestationPool};
use shasper_blockchain::backend::ShasperBackend;
use shasper_network::NetworkConfig;
use lmd_ghost::archive::{ArchiveGhostImporter, AncestorQuery};
use clap::{App, Arg};
use libp2p::Multiaddr;
use std::thread;
use std::str::FromStr;
use std::fs::File;
use std::io::{BufReader, Read};
use std::collections::HashMap;
use ssz::Decode;
use core::time::Duration;
use core::convert::TryInto;
use serde::{Serialize, Deserialize};
use log::*;
use bm_le::tree_root;
use crypto::bls::{self, BLSVerification};

fn deposit_tree<C: Config>(deposits: &[DepositData]) -> Vec<Vec<H256>> {
	let mut zerohashes = vec![H256::default()];
	for layer in 1..32 {
		zerohashes.push(C::hash(&[
			zerohashes[layer - 1].as_ref(),
			zerohashes[layer - 1].as_ref(),
		]));
	}

	let mut values = deposits.iter().map(|d| {
		tree_root::<C::Digest, _>(d)
	}).collect::<Vec<_>>();
	let values_len = values.len();
	let mut tree = vec![values.clone()];

	for h in 0..(beacon::consts::DEPOSIT_CONTRACT_TREE_DEPTH as usize) {
		if values.len() % 2 == 1 {
			values.push(zerohashes[h]);
		}
		let mut new_values = Vec::new();
		for i in 0..(values.len() / 2) {
			new_values.push(C::hash(&[
				values[i * 2].as_ref(),
				values[i * 2 + 1].as_ref()
			]));
		}
		values = new_values;
		tree.push(values.clone());
	}
	assert!(values.len() == 1);
	values.push({
		let mut ret = values_len.to_le_bytes().to_vec();
		while ret.len() < 32 {
			ret.push(0);
		}
		H256::from_slice(&ret[..])
	});
	tree[32].push(values[1]);
	tree.push(vec![C::hash(&[
		values[0].as_ref(),
		values[1].as_ref(),
	])]);
	assert!(tree.len() == 34);

	tree
}

fn deposit_root(tree: &Vec<Vec<H256>>) -> H256 {
	tree.last().expect("Merkle tree cannot be empty; qed")[0]
}

fn deposit_proof<C: Config>(tree: &Vec<Vec<H256>>, item_index: usize) -> Vec<H256> {
	let mut zerohashes = vec![H256::default()];
	for layer in 1..32 {
		zerohashes.push(C::hash(&[
			zerohashes[layer - 1].as_ref(),
			zerohashes[layer - 1].as_ref(),
		]));
	}

	let mut proof = Vec::new();
	for i in 0..(beacon::consts::DEPOSIT_CONTRACT_TREE_DEPTH as usize) {
		let subindex = (item_index / 2usize.pow(i as u32)) ^ 1;
		if subindex < tree[i].len() {
			proof.push(tree[i][subindex]);
		} else {
			proof.push(zerohashes[i]);
		}
	}
	proof.push(tree[32][1]);
	proof
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct ValidatorKey {
	pub privkey: String,
	pub pubkey: String,
}

fn string_to_bytes(string: &str) -> Result<Vec<u8>, String> {
    let string = if string.starts_with("0x") {
        &string[2..]
    } else {
        string
    };

    hex::decode(string).map_err(|e| format!("Unable to decode public or private key: {}", e))
}

fn main() {
	pretty_env_logger::init();

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
		.arg(Arg::with_name("libp2p-nodes")
			 .long("libp2p-nodes")
			 .takes_value(true)
			 .help("Comma-separated libp2p nodes to initially connect to"))
		.arg(Arg::with_name("author")
			 .long("author")
			 .help("Whether to author blocks"))
		.arg(Arg::with_name("genesis-state")
			 .long("genesis-state")
			 .takes_value(true)
			 .help("ssz raw genesis state file"))
		.arg(Arg::with_name("validator-keys")
			 .long("validator-keys")
			 .takes_value(true)
			 .help("yaml validator keys"))
		.get_matches();

	let mut keys: HashMap<ValidatorId, bls::Secret> = HashMap::new();

	if let Some(validator_keys) = matches.value_of("validator-keys") {
		const PRIVATE_KEY_BYTES: usize = 48;
		const PUBLIC_KEY_BYTES: usize = 48;

		let file = File::open(validator_keys).unwrap();
		let coll = serde_yaml::from_reader::<_, Vec<ValidatorKey>>(BufReader::new(file)).unwrap();

		for key in coll {
			let privkey = string_to_bytes(&key.privkey).unwrap();

			let sk = {
				let mut bytes = vec![0; PRIVATE_KEY_BYTES - privkey.len()];
				bytes.extend_from_slice(&privkey);
				bls::Secret::from_bytes(&bytes)
					.map_err(|e| format!("Failed to decode bytes into secret key: {:?}", e))
					.unwrap()
			};

			let pubkey = ValidatorId::from_slice(&bls::Public::from_secret_key(&sk).as_bytes()[..]);

			keys.insert(pubkey, sk);
		}
	}

	let genesis_state = if let Some(genesis_file) = matches.value_of("genesis-state") {
		let mut file = File::open(genesis_file).unwrap();
		let mut data = Vec::new();
		file.read_to_end(&mut data).unwrap();

		Decode::decode(&mut &data[..]).unwrap()
	} else {
		let mut deposit_datas = Vec::new();
		for i in 0..10 {
			let seckey = bls::Secret::random(&mut rand::thread_rng());
			let pubkey = ValidatorId::from_slice(&bls::Public::from_secret_key(&seckey).as_bytes()[..]);
			let mut data = DepositData {
				pubkey: pubkey.clone(),
				withdrawal_credentials: H256::from_low_u64_le(i as u64),
				amount: 32000000000,
				signature: Default::default(),
			};
			let signature = Signature::from_slice(&bls::Signature::new(
				&tree_root::<sha2::Sha256, _>(&SigningDepositData::from(data.clone()))[..],
				beacon::genesis_domain(MinimalConfig::domain_deposit()),
				&seckey
			).as_bytes()[..]);
			data.signature = signature;
			deposit_datas.push(data);
			keys.insert(pubkey, seckey);
		}

		let deposit_tree = deposit_tree::<MinimalConfig>(&deposit_datas);
		let deposits = deposit_datas.clone().into_iter()
			.enumerate()
			.map(|(i, deposit_data)| {
				Deposit {
					proof: deposit_proof::<MinimalConfig>(&deposit_tree, i).try_into().ok().unwrap(),
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
		let genesis_state =
			genesis_beacon_state::<MinimalConfig, BLSVerification>(
				&deposits, 0, eth1_data.clone()
			).unwrap();

		genesis_state
	};
	let genesis_block = Block(BeaconBlock {
		state_root: tree_root::<<MinimalConfig as Config>::Digest, _>(&genesis_state),
		..Default::default()
	});
	let eth1_data = genesis_state.eth1_data.clone();

	let mut network_config = NetworkConfig::default();
	network_config.libp2p_port = u16::from_str(matches.value_of("port").unwrap()).unwrap();
	network_config.discovery_port = u16::from_str(matches.value_of("port").unwrap()).unwrap();
	network_config.libp2p_nodes = if let Some(nodes) = matches.value_of("libp2p-nodes") {
		nodes.rsplit(',')
			.map(|v| FromStr::from_str(v).unwrap())
			.collect::<Vec<Multiaddr>>()
	} else {
		Vec::new()
	};

	if let Some(path) = matches.value_of("data") {
		info!("Using RocksDB backend");
		let backend = ShasperBackend::new(
			RocksBackend::<_, (), RocksState<MinimalConfig>>::open_or_create(path, |_| {
				Ok((genesis_block.clone(), genesis_state.into()))
			}).unwrap()
		);
		let lock = ImportLock::new();

		run(network_config,
			matches.is_present("author"),
			backend,
			lock,
			eth1_data,
			keys);
	} else {
		info!("Using in-memory backend");
		let backend = ShasperBackend::new(
			SharedMemoryBackend::<_, (), MemoryState<MinimalConfig>>::new_with_genesis(
				genesis_block.clone(),
				genesis_state.into(),
			)
		);
		let lock = ImportLock::new();

		run(network_config,
			matches.is_present("author"),
			backend,
			lock,
			eth1_data,
			keys);
	}
}

fn run<B, C: Config>(
	config: NetworkConfig,
	author: bool,
	backend: B,
	import_lock: ImportLock,
	eth1_data: Eth1Data,
	keys: HashMap<ValidatorId, bls::Secret>,
) where
	Block<C>: ssz::Encode + ssz::Decode + Unpin + Send + Sync,
	B: ChainQuery + AncestorQuery + Store<Block=Block<C>>,
	B::State: StateExternalities + AsExternalities<dyn StateExternalities<Config=C>>,
	B::Auxiliary: Auxiliary<Block<C>> + Unpin,
	B: SharedCommittable<Operation=Operation<<B as Store>::Block, <B as Store>::State, <B as Store>::Auxiliary>>,
	B: Send + Sync + 'static,
	C: Unpin + Clone + Send + Sync + 'static,
{
	let executor = Executor::<C, BLSVerification>::new();
	let importer = MutexImporter::new(
		ArchiveGhostImporter::new(executor, backend.clone(), import_lock.clone())
	);

	if author {
		let backend_build = backend.clone();
		let importer_build = importer.clone();
		thread::spawn(move || {
			builder_thread(backend_build, importer_build, eth1_data, keys);
		});
	}

	shasper_network::start_network_simple_sync(backend, import_lock, importer, config)
		.expect("Starting networking thread failed");
}

fn builder_thread<B, I, C: Config + Clone>(
	backend: B,
	importer: I,
	eth1_data: Eth1Data,
	keys: HashMap<ValidatorId, bls::Secret>,
) where
	B: ChainQuery + Store<Block=Block<C>>,
	B::State: StateExternalities + AsExternalities<dyn StateExternalities<Config=C>>,
	B::Auxiliary: Auxiliary<Block<C>>,
	I: SharedBlockImporter<Block=Block<C>>
{
	let executor = Executor::<C, BLSVerification>::new();
	let mut attestations = AttestationPool::<C, BLSVerification>::new();

	loop {
		thread::sleep(Duration::new(1, 0));

		let head = backend.head();
		info!("Building on top of {}", head);

		let block = {
			let head_block = backend.block_at(&head).unwrap();
			let head_state = backend.state_at(&head).unwrap();
			trace!("Justified epoch {}, finalized epoch {}",
				   { head_state.state().current_justified_checkpoint.epoch },
				   { head_state.state().finalized_checkpoint.epoch });

			let mut state = backend.state_at(&head).unwrap();
			let externalities = state.as_externalities();
			let current_slot = head_block.0.slot + 1;
			executor.initialize_block(externalities, current_slot).unwrap();
			let current_epoch = externalities.state().current_epoch();

			let randao_domain = externalities.state()
				.domain(C::domain_randao(), None);
			let proposer_domain = externalities.state()
				.domain(C::domain_beacon_proposer(), None);
			let attestation_domain = externalities.state()
				.domain(C::domain_beacon_attester(), None);

			for (validator_id, validator_seckey) in &keys {
				let validator_index = externalities.state().validator_index(validator_id);

				if let Some(validator_index) = validator_index {
					let committee_assignment = externalities.state()
						.committee_assignment(current_epoch, validator_index).unwrap();
					if let Some(committee_assignment) = committee_assignment {
						if committee_assignment.slot == current_slot {
							trace!(
								"Found validator {} attesting slot {} with index {}",
								validator_id, current_slot, committee_assignment.index);
							let committee = committee_assignment.validators;

							let target_epoch = current_epoch;
							let target_slot = beacon::utils::start_slot_of_epoch::<C>(target_epoch);
							let target_root = if target_slot == current_slot {
								head
							} else {
								externalities.state()
									.block_root(target_epoch).unwrap()
							};
							let source_epoch = externalities.state().current_justified_checkpoint.epoch;
							let source_root = externalities.state().current_justified_checkpoint.root;
							trace!(
								"Casper source {} ({}) to target {} ({})",
								source_epoch, source_root, target_epoch, target_root,
							);

							let data = AttestationData {
								beacon_block_root: head_block.id(),
								source: Checkpoint {
									epoch: source_epoch,
									root: source_root,
								},
								target: Checkpoint {
									epoch: target_epoch,
									root: target_root,
								},
								slot: committee_assignment.slot,
								index: committee_assignment.index,
							};
							let signature = Signature::from_slice(&bls::Signature::new(
								&tree_root::<C::Digest, _>(&AttestationDataAndCustodyBit {
									data: data.clone(),
									custody_bit: false,
								})[..],
								attestation_domain,
								&validator_seckey,
							).as_bytes()[..]);

							let index_into_committee = committee.iter()
								.position(|v| *v == validator_index).unwrap();
							let mut aggregation_bitfield = Vec::new();
							aggregation_bitfield.resize(committee.len(), false);
							aggregation_bitfield[index_into_committee] = true;
							let mut custody_bitfield = Vec::new();
							custody_bitfield.resize(committee.len(), false);

							let attestation = Attestation {
								aggregation_bits: aggregation_bitfield.into(),
								data,
								custody_bits: custody_bitfield.into(),
								signature
							};

							attestations.push(attestation);
						}
					}
				}
			}

			let proposer_index = externalities.state().beacon_proposer_index().unwrap();
			let proposer_pubkey = externalities.state().validator_pubkey(proposer_index).unwrap();
			trace!("Current proposer {} ({}) on epoch {}", proposer_index, proposer_pubkey, current_epoch);

			let seckey = match keys.get(&proposer_pubkey) {
				Some(value) => value.clone(),
				None => {
					warn!("No secret key, skip building block.");
					continue;
				},
			};
			let randao_reveal = Signature::from_slice(&bls::Signature::new(
				&tree_root::<C::Digest, _>(&current_epoch)[..],
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
						warn!("Error when submitting an attestation: {}", err);
					},
				}
			}
			info!("Pushed {} attestations", collected_attestations.len());
			for hash in collected_attestations {
				attestations.pop(&hash);
			}

			executor.finalize_block(
				&mut unsealed_block, state.as_externalities()
			).unwrap();

			let mut block = unsealed_block.fake_seal();
			let signature = Signature::from_slice(&bls::Signature::new(
				&tree_root::<C::Digest, _>(&UnsealedBeaconBlock::<C>::from(&block))[..],
				proposer_domain,
				&seckey
			).as_bytes()[..]);
			block.signature = signature;
			Block(block)
		};

		importer.import_block(block).unwrap();
	}
}
