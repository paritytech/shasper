use core::fmt::Debug;
use core::time::Duration;
use core::ops::DerefMut;
use parity_codec::{Encode, Decode};
use libp2p::{identity, NetworkBehaviour, PeerId};
use libp2p::mdns::Mdns;
use libp2p::floodsub::{Floodsub, Topic, TopicBuilder};
use libp2p::kad::Kademlia;
use libp2p::core::swarm::{NetworkBehaviourEventProcess, NetworkBehaviourAction};
use futures::{Async, stream::Stream};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_timer::Interval;
use blockchain::backend::{SharedCommittable, ChainQuery, ImportLock};
use blockchain::import::BlockImporter;
use blockchain_network::{NetworkEnvironment, NetworkHandle, NetworkEvent};
use blockchain_network::sync::{NetworkSyncMessage, NetworkSync, StatusProducer};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "(PeerId, NetworkSyncMessage<B, S>)", poll_method = "poll")]
struct Behaviour<TSubstream: AsyncRead + AsyncWrite, B, S> {
	floodsub: Floodsub<TSubstream>,
	kademlia: Kademlia<TSubstream>,
	mdns: Mdns<TSubstream>,

	#[behaviour(ignore)]
	topic: Topic,
	#[behaviour(ignore)]
	events: Vec<(PeerId, NetworkSyncMessage<B, S>)>,
}

impl<TSubstream: AsyncRead + AsyncWrite, B, S> Behaviour<TSubstream, B, S> {
	fn poll<TEv>(&mut self) -> Async<NetworkBehaviourAction<TEv, (PeerId, NetworkSyncMessage<B, S>)>> {
		if !self.events.is_empty() {
			return Async::Ready(NetworkBehaviourAction::GenerateEvent(self.events.remove(0)))
		}

		Async::NotReady
	}
}

impl<TSubstream: AsyncRead + AsyncWrite, B, S> NetworkEnvironment for Behaviour<TSubstream, B, S> {
	type PeerId = PeerId;
	type Message = NetworkSyncMessage<B, S>;
}

impl<TSubstream: AsyncRead + AsyncWrite, B, S> NetworkHandle for Behaviour<TSubstream, B, S>  where
	B: Encode,
	S: Encode,
{
	fn send(&mut self, _peer: &PeerId, message: NetworkSyncMessage<B, S>) {
		self.floodsub.publish(&self.topic, message.encode());
	}

	fn broadcast(&mut self, message: NetworkSyncMessage<B, S>) {
		self.floodsub.publish(&self.topic, message.encode());
	}
}

impl<TSubstream: AsyncRead + AsyncWrite, B, S> NetworkBehaviourEventProcess<libp2p::floodsub::FloodsubEvent> for Behaviour<TSubstream, B, S> where
	B: Encode + Decode + Debug,
	S: Encode + Decode + Debug,
{
	fn inject_event(&mut self, floodsub_message: libp2p::floodsub::FloodsubEvent) {
		if let libp2p::floodsub::FloodsubEvent::Message(floodsub_message) = floodsub_message {
			let message = NetworkSyncMessage::<B, S>::decode(&mut &floodsub_message.data[..]).unwrap();

			self.events.push((floodsub_message.source.clone(), message));
		}
	}
}


impl<TSubstream: AsyncRead + AsyncWrite, B, S> NetworkBehaviourEventProcess<libp2p::kad::KademliaOut> for Behaviour<TSubstream, B, S> {
	fn inject_event(&mut self, message: libp2p::kad::KademliaOut) {
		if let libp2p::kad::KademliaOut::Discovered { peer_id, .. } = message {
			println!("Discovered via Kademlia {:?}", peer_id);
			self.floodsub.add_node_to_partial_view(peer_id);
		}
	}
}

impl<TSubstream: AsyncRead + AsyncWrite, B, S> NetworkBehaviourEventProcess<libp2p::mdns::MdnsEvent> for Behaviour<TSubstream, B, S> {
    fn inject_event(&mut self, event: libp2p::mdns::MdnsEvent) {
        match event {
            libp2p::mdns::MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            },
            libp2p::mdns::MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}

pub fn start_network_simple_sync<Ba, I, St>(
	port: &str,
	backend: Ba,
	import_lock: ImportLock,
	importer: I,
	status: St,
) where
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

	let transport = libp2p::build_tcp_ws_secio_mplex_yamux(local_key);
	let topic = TopicBuilder::new("blocks").build();

	let mut sync = NetworkSync::new(backend, import_lock, importer, status);

	let mut swarm = {
		let mut behaviour = Behaviour {
			floodsub: Floodsub::new(local_peer_id.clone()),
			kademlia: Kademlia::new(local_peer_id.clone()),
			mdns: libp2p::mdns::Mdns::new().expect("Failed to create mDNS service"),

			topic: topic.clone(),
			events: Vec::new(),
		};

		assert!(behaviour.floodsub.subscribe(topic.clone()));
		libp2p::Swarm::new(transport, behaviour, local_peer_id)
	};

	// Listen on all interfaces and whatever port the OS assigns
	let addr = libp2p::Swarm::listen_on(&mut swarm, format!("/ip4/0.0.0.0/tcp/{}", port).parse().unwrap()).unwrap();
	println!("Listening on {:?}", addr);

	let mut interval = Interval::new_interval(Duration::new(5, 0));
	let mut listening = false;
    tokio::run(futures::future::poll_fn(move || -> Result<_, ()> {
        loop {
            match interval.poll().expect("Error while polling interval") {
                Async::Ready(Some(_)) => {
					sync.on_tick(swarm.deref_mut());
				},
                Async::Ready(None) => panic!("Interval closed"),
                Async::NotReady => break,
            };
        }

        loop {
            match swarm.poll().expect("Error while polling swarm") {
                Async::Ready(Some((peer_id, message))) => {
					println!("Received: {:?} from {:?}", message, peer_id);
					sync.on_message(swarm.deref_mut(), &peer_id, message);
				},
                Async::Ready(None) | Async::NotReady => {
                    if !listening {
                        if let Some(a) = libp2p::Swarm::listeners(&swarm).next() {
                            println!("Listening on {:?}", a);
                            listening = true;
                        }
                    }
                    break
                }
            }
        }

        Ok(Async::NotReady)
	}));
}
