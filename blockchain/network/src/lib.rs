pub mod behaviour;
mod config;
mod discovery;
mod error;
pub mod rpc;
mod service;

pub use behaviour::{Behaviour, PubsubMessage};
pub use config::{
    Config as NetworkConfig, BEACON_ATTESTATION_TOPIC, BEACON_BLOCK_TOPIC, SHARD_TOPIC_PREFIX,
    TOPIC_ENCODING_POSTFIX, TOPIC_PREFIX,
};
pub use libp2p::enr::Enr;
pub use libp2p::multiaddr;
pub use libp2p::Multiaddr;
pub use libp2p::{
    gossipsub::{GossipsubConfig, GossipsubConfigBuilder},
    PeerId, Swarm,
};
pub use error::Error;
pub use rpc::RPCEvent;
pub use service::Libp2pEvent;
pub use service::Service;

use core::fmt::Debug;
use core::time::Duration;
use core::ops::DerefMut;
use parity_codec::{Encode, Decode};
use libp2p::{identity, NetworkBehaviour};
use libp2p::mdns::Mdns;
use libp2p::floodsub::{self, Floodsub};
use libp2p::kad::{Kademlia, record::store::MemoryStore};
use libp2p::swarm::{NetworkBehaviourEventProcess, NetworkBehaviourAction};
use futures::{Async, stream::Stream};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_timer::Interval;
use blockchain::backend::{SharedCommittable, ChainQuery, ImportLock};
use blockchain::import::BlockImporter;
use blockchain_network::{NetworkEnvironment, NetworkHandle, NetworkEvent};
use blockchain_network::sync::{NetworkSyncMessage, NetworkSync, StatusProducer};

pub const VERSION: &str = "v0.1";

pub fn start_network_simple_sync<Ba, I, St>(
	port: &str,
	backend: Ba,
	import_lock: ImportLock,
	importer: I,
	status: St,
) -> Result<(), Error> where
	Ba: SharedCommittable + ChainQuery + Send + Sync + 'static,
	Ba::Block: Debug + Encode + Decode + Send + Sync,
	I: BlockImporter<Block=Ba::Block> + Send + Sync + 'static,
	St: StatusProducer + Send + Sync + 'static,
	St::Status: Debug + Clone + Send + Sync,
{
    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
	println!("Local peer id: {:?}", local_peer_id);

	let config = NetworkConfig::default();

	let transport = libp2p::build_tcp_ws_secio_mplex_yamux(local_key);

	let mut sync = NetworkSync::<PeerId, _, _, _>::new(backend, import_lock, importer, status);

	let service = Service::new(config)?;

	unimplemented!()

	// let mut interval = Interval::new_interval(Duration::new(5, 0));
	// let mut listening = false;
    // tokio::run(futures::future::poll_fn(move || -> Result<_, ()> {
    //     loop {
    //         match interval.poll().expect("Error while polling interval") {
    //             Async::Ready(Some(_)) => {
	// 				sync.on_tick(swarm.deref_mut());
	// 			},
    //             Async::Ready(None) => panic!("Interval closed"),
    //             Async::NotReady => break,
    //         };
    //     }

    //     loop {
    //         match swarm.poll().expect("Error while polling swarm") {
    //             Async::Ready(Some((peer_id, message))) => {
	// 				println!("Received: {:?} from {:?}", message, peer_id);
	// 				sync.on_message(swarm.deref_mut(), &peer_id, message);
	// 			},
    //             Async::Ready(None) | Async::NotReady => {
    //                 if !listening {
    //                     if let Some(a) = libp2p::Swarm::listeners(&swarm).next() {
    //                         println!("Listening on {:?}", a);
    //                         listening = true;
    //                     }
    //                 }
    //                 break
    //             }
    //         }
    //     }

    //     Ok(Async::NotReady)
	// }));
}
