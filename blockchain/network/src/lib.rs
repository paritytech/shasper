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
pub use rpc::{RPCEvent, RPCErrorResponse, RPCRequest, RPCResponse,
			  methods::{HelloMessage, BeaconBlocksRequest}};
pub use service::Libp2pEvent;
pub use service::Service;

use log::*;
use core::time::Duration;
use ssz::{Encode, Decode};
use libp2p::identity;
use futures01::{Async, stream::Stream};
use futures::{Poll, StreamExt as _};
use blockchain::Auxiliary;
use blockchain::backend::{Store, SharedCommittable, ChainQuery, ImportLock};
use blockchain::import::BlockImporter;
use blockchain_network::sync::{NetworkSync, SyncConfig, SyncEvent};
use beacon::Config;
use shasper_runtime::Block;

pub const VERSION: &str = "v0.1";

pub fn start_network_simple_sync<C, Ba, I>(
	backend: Ba,
	import_lock: ImportLock,
	importer: I,
) -> Result<(), Error> where
	C: Config,
	Ba: Store<Block=Block<C>> + SharedCommittable + ChainQuery + Send + Sync + 'static,
	Ba::Block: Unpin + Send + Sync,
	Ba::Auxiliary: Auxiliary<Block<C>> + Unpin,
	I: BlockImporter<Block=Block<C>> + Unpin + Send + Sync + 'static,
{
	// Create a random PeerId
	let local_key = identity::Keypair::generate_ed25519();
	let local_peer_id = PeerId::from(local_key.public());
	info!("Local peer id: {:?}", local_peer_id);

	let config = NetworkConfig::default();
	let sync_config = SyncConfig {
		peer_update_frequency: 2,
		update_frequency: 1,
		request_timeout: 4,
	};
	let best_depth = {
		let best_hash = backend.head();
		backend.depth_at(&best_hash)
			.expect("Best block depth hash cannot fail")
	};
	let mut sync = NetworkSync::<PeerId, _, _>::new(
		best_depth,
		importer,
		Duration::new(2, 0),
		sync_config
	);

	let mut service = Service::new(config)?;

	let mut listening = false;

	let poll = futures::future::poll_fn::<Result<(), ()>, _>(move |ctx| {
		loop {
			match service.poll().expect("Error while polling swarm") {
				Async::Ready(Some(message)) => {
					match message {
						Libp2pEvent::PeerDialed(peer) => {
							sync.note_connected(peer);
						},
						Libp2pEvent::PeerDisconnected(peer) => {
							sync.note_disconnected(peer);
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
								RPCEvent::Request(request_id, RPCRequest::Hello(hello)) => {
									let best_hash = backend.head();
									let best_depth = backend.depth_at(&best_hash)
										.expect("Best block depth hash cannot fail");
									service.swarm.send_rpc(peer.clone(), RPCEvent::Response(
										request_id, RPCErrorResponse::Success(
											RPCResponse::Hello(HelloMessage {
												fork_version: Default::default(),
												finalized_root: Default::default(),
												finalized_epoch: Default::default(),
												head_root: best_hash,
												head_slot: best_depth as u64,
											})
										)
									));
									sync.note_peer_status(peer, hello.head_slot as usize);
								},
								RPCEvent::Response(_, RPCErrorResponse::Success(
									RPCResponse::Hello(hello)
								)) => {
									sync.note_peer_status(peer, hello.head_slot as usize);
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

									sync.note_blocks(blocks, Some(peer));
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

		loop {
			match sync.poll_next_unpin(ctx) {
				Poll::Pending | Poll::Ready(None) => break,
				Poll::Ready(Some(SyncEvent::QueryStatus)) => {
					let best_hash = backend.head();
					let best_depth = backend.depth_at(&best_hash)
						.expect("Best block depth hash cannot fail");
					sync.note_status(best_depth);
				},
				Poll::Ready(Some(SyncEvent::QueryPeerStatus(peer))) => {
					let best_hash = backend.head();
					let best_depth = backend.depth_at(&best_hash)
						.expect("Best block depth hash cannot fail");
					service.swarm.send_rpc(peer, RPCEvent::Request(
						0,
						RPCRequest::Hello(HelloMessage {
							fork_version: Default::default(),
							finalized_root: Default::default(),
							finalized_epoch: Default::default(),
							head_root: best_hash,
							head_slot: best_depth as u64,
						})
					));
				},
				Poll::Ready(Some(SyncEvent::QueryBlocks(peer))) => {
					let best_hash = backend.head();
					let best_depth = backend.depth_at(&best_hash)
						.expect("Best block depth hash cannot fail");
					service.swarm.send_rpc(peer, RPCEvent::Request(
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
				},
			}
		}

		Poll::Pending
	});

	tokio::run(futures::compat::Compat::new(poll));

	Err(Error::Other("Shutdown".to_string()))
}
