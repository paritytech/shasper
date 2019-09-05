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
pub use rpc::{RPCEvent, RPCErrorResponse, RPCRequest, RPCResponse, methods::BeaconBlocksRequest};
pub use service::Libp2pEvent;
pub use service::Service;

use log::*;
use core::fmt::Debug;
use core::time::Duration;
use ssz::{Encode, Decode};
use libp2p::identity;
use futures::{Async, stream::Stream};
use tokio_timer::Interval;
use rand::seq::SliceRandom;
use blockchain::backend::{SharedCommittable, ChainQuery, ImportLock};
use blockchain::import::BlockImporter;

pub const VERSION: &str = "v0.1";

pub fn start_network_simple_sync<Ba, I>(
	backend: Ba,
	import_lock: ImportLock,
	mut importer: I,
) -> Result<(), Error> where
	Ba: SharedCommittable + ChainQuery + Send + Sync + 'static,
	Ba::Block: Debug + Encode + Decode + Send + Sync,
	I: BlockImporter<Block=Ba::Block> + Send + Sync + 'static,
{
	// Create a random PeerId
	let local_key = identity::Keypair::generate_ed25519();
	let local_peer_id = PeerId::from(local_key.public());
	info!("Local peer id: {:?}", local_peer_id);

	let config = NetworkConfig::default();

	let mut service = Service::new(config)?;
	let mut peers = <Vec<PeerId>>::new();

	let mut interval = Interval::new_interval(Duration::new(5, 0));
	let mut listening = false;

	tokio::run(futures::future::poll_fn(move || -> Result<_, ()> {
		loop {
			match interval.poll().expect("Error while polling interval") {
				Async::Ready(Some(_)) => {
					if let Some(peer) = peers.choose(&mut rand::thread_rng()) {
						let best_depth = {
							let best_hash = backend.head();
							backend.depth_at(&best_hash)
								.expect("Best block depth hash cannot fail")
						};

						service.swarm.send_rpc(peer.clone(), RPCEvent::Request(
							0,
							RPCRequest::BeaconBlocks(
								BeaconBlocksRequest {
									head_block_root: Default::default(),
									start_slot: best_depth as u64,
									count: 10,
									step: 1
								}
							)
						));
					}
				},
				Async::Ready(None) => panic!("Interval closed"),
				Async::NotReady => break,
			};
		}

		loop {
			match service.poll().expect("Error while polling swarm") {
				Async::Ready(Some(message)) => {
					match message {
						Libp2pEvent::PeerDialed(peer) => {
							peers.push(peer);
						},
						Libp2pEvent::PeerDisconnected(peer) => {
							peers.retain(|p| p != &peer);
						},
						Libp2pEvent::PubsubMessage {
							source, topics, message,
						} => {
							warn!("Unhandled pubsub message {:?}, {:?}, {:?}",
								  source, topics, message);
						},
						Libp2pEvent::RPC(peer, event) => {
							match event {
								RPCEvent::Request(request_id, RPCRequest::BeaconBlocks(request)) => {
									let mut ret = Vec::new();
									{
										let _ = import_lock.lock();
										let start_slot = request.start_slot;
										let count = request.count;

										for d in start_slot..(start_slot + count) {
											match backend.lookup_canon_depth(d as usize) {
												Ok(Some(hash)) => {
													let block = backend.block_at(&hash)
														.expect("Found hash cannot fail");
													ret.push(block);
												},
												_ => break,
											}
										}
									}
									service.swarm.send_rpc(peer, RPCEvent::Response(
										request_id, RPCErrorResponse::Success(
											RPCResponse::BeaconBlocks(ret.encode())
										)
									));
								},
								RPCEvent::Response(_, RPCErrorResponse::Success(
									RPCResponse::BeaconBlocks(blocks)
								)) => {
									let blocks = match <Vec<Ba::Block> as Decode>::decode(
										&mut &blocks[..]
									) {
										Ok(blocks) => blocks,
										Err(e) => {
											warn!("Received RPC response error: {:?}", e);
											Vec::new()
										},
									};

									for block in blocks {
										match importer.import_block(block) {
											Ok(()) => (),
											Err(_) => {
												warn!("Error happened on block response message");
												break
											},
										}
									}
								},
								event => {
									warn!("Unhandled RPC message {:?}, {:?}", peer, event);
								},
							}
						},
					}
				},
				Async::Ready(None) | Async::NotReady => {
					if !listening {
						if let Some(a) = libp2p::Swarm::listeners(&service.swarm).next() {
							info!("Listening on {:?}", a);
							listening = true;
						}
					}
					break
				}
			}
		}

		Ok(Async::NotReady)
	}));

	Err(Error::Other("Shutdown".to_string()))
}
