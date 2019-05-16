use beacon::{genesis, NoVerificationConfig};
use beacon::types::Eth1Data;
use blockchain::backend::{SharedBackend, MemoryBackend, MemoryLikeBackend};
use blockchain_network_simple::BestDepthStatusProducer;
use shasper_blockchain::{Block, Executor, State};
use lmd_ghost::archive::{NoCacheAncestorBackend, ArchiveGhostImporter};
use clap::{App, Arg};

fn main() {
	let matches = App::new("Shasper blockchain client")
		.arg(Arg::with_name("port")
			 .short("p")
			 .long("port")
			 .takes_value(true)
			 .help("Port to listen on"))
		.get_matches();

	let config = NoVerificationConfig::full();
	let (genesis_beacon_block, genesis_state) = genesis(
		&[], 0, Eth1Data::default(), &config
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
