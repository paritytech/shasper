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

mod behaviour;
mod config;
mod discovery;
mod error;
mod rpc;
mod service;
mod handler;

pub use behaviour::Behaviour;
pub use config::Config as NetworkConfig;
pub use libp2p::enr::Enr;
pub use libp2p::multiaddr;
pub use libp2p::Multiaddr;
pub use libp2p::{
	gossipsub::{GossipsubConfig, GossipsubConfigBuilder},
	PeerId, Swarm,
};
pub use error::Error;
pub use service::Service;
pub use handler::Handler;

use log::*;
use core::time::Duration;
use libp2p::identity;
use futures01::{Async, stream::Stream};
use futures::{Poll, StreamExt as _};
use blockchain::{Auxiliary, AsExternalities};
use blockchain::backend::{Store, SharedCommittable, ChainQuery, ImportLock};
use blockchain::import::BlockImporter;
use blockchain_network::sync::{NetworkSync, SyncConfig, SyncEvent};
use beacon::Config;
use shasper_runtime::{Block, StateExternalities};
use network_messages::{HelloMessage, PubsubMessage};
use crate::rpc::{RPCEvent, RPCRequest, RPCResponse};

pub const VERSION: &str = "v0.1";

/// Events that can be obtained from polling the Libp2p Service.
#[derive(Debug)]
pub enum Libp2pEvent<C: Config> {
    /// An RPC response request has been received on the swarm.
    RPC(PeerId, RPCEvent<C>),
    /// Initiated the connection to a new peer.
    PeerDialed(PeerId),
    /// A peer has disconnected.
    PeerDisconnected(PeerId),
    /// Received pubsub message.
    Pubsub(PeerId, PubsubMessage<C>),
}

pub fn start_network_simple_sync<C, Ba, I>(
	backend: Ba,
	import_lock: ImportLock,
	importer: I,
	config: NetworkConfig,
) -> Result<(), Error> where
	C: Config,
	Ba: Store<Block=Block<C>> + SharedCommittable + ChainQuery + Send + Sync + 'static,
	Ba::Block: Unpin + Send + Sync,
	Ba::State: StateExternalities + AsExternalities<dyn StateExternalities<Config=C>>,
	Ba::Auxiliary: Auxiliary<Block<C>> + Unpin,
	I: BlockImporter<Block=Block<C>> + Unpin + Send + Sync + 'static,
{
	// Create a random PeerId
	let local_key = identity::Keypair::generate_ed25519();
	let local_peer_id = PeerId::from(local_key.public());
	info!("Local peer id: {:?}", local_peer_id);

	let sync_config = SyncConfig {
		peer_update_frequency: 2,
		update_frequency: 1,
		request_timeout: 4,
	};

	let handler = Handler::<C, Ba>::new(backend, import_lock);
	let head_status = handler.status();
	let mut sync = NetworkSync::<PeerId, HelloMessage, I>::new(
		head_status,
		importer,
		Duration::new(1, 0),
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
							trace!("Peer noted to be dialed: {:?}", peer);
							sync.note_connected(peer);
						},
						Libp2pEvent::PeerDisconnected(peer) => {
							trace!("Peer noted to disconnect: {:?}", peer);
							sync.note_disconnected(peer);
						},
						Libp2pEvent::Pubsub(peer, message) => {
							warn!("Unhandled pubsub message {:?}, {:?}", peer, message);
						},
						Libp2pEvent::RPC(peer, event) => {
							trace!("Received RPC event {:?}, {:?}", peer, event);
							match event {
								RPCEvent::Request(request_id, RPCRequest::BeaconBlocks(request)) => {
									service.swarm.send_rpc(peer, RPCEvent::Response(
										request_id, RPCResponse::BeaconBlocks(
											handler.blocks_by_slot(
												request.head_block_root,
												request.start_slot,
												1, // TODO: request.count as usize,
											)
										)
									));
								},
								RPCEvent::Request(request_id, RPCRequest::Hello(hello)) => {
									service.swarm.send_rpc(peer.clone(), RPCEvent::Response(
										request_id, RPCResponse::Hello(
											handler.status()
										)
									));
									sync.note_peer_status(peer, hello);
								},
								RPCEvent::Response(_, RPCResponse::Hello(hello)) => {
									sync.note_peer_status(peer, hello);
								},
								RPCEvent::Response(_, RPCResponse::BeaconBlocks(blocks)) => {
									sync.note_blocks(
										blocks.into_iter().map(Into::into).collect(),
										Some(peer)
									);
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
					trace!("Sync requested status query");
					sync.note_status(handler.status());
				},
				Poll::Ready(Some(SyncEvent::QueryPeerStatus(peer))) => {
					trace!("Sync requested peer status query to {:?}", peer);
					service.swarm.send_rpc(peer, RPCEvent::Request(
						0,
						RPCRequest::Hello(handler.status())
					));
				},
				Poll::Ready(Some(SyncEvent::QueryBlocks(peer))) => {
					trace!("Sync requested blocks query to {:?}", peer);
					service.swarm.send_rpc(peer, RPCEvent::Request(
						0,
						RPCRequest::BeaconBlocks(handler.head_request(50))
					));
				},
			}
		}

		Poll::Pending
	});

	tokio::run(futures::compat::Compat::new(poll));

	Err(Error::Other("Shutdown".to_string()))
}
