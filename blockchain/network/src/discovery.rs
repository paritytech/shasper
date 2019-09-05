// Copyright 2019 Parity Technologies (UK) Ltd.
// Copyright 2019 Sigma Prime.
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

//! This manages the discovery and management of peers.
//!
//! Currently using discv5 for peer discovery.
//!

use crate::{Error, NetworkConfig};
use futures::prelude::*;
use libp2p::core::{identity::Keypair, ConnectedPoint, Multiaddr, PeerId};
use libp2p::discv5::{Discv5, Discv5Event};
use libp2p::enr::{Enr, EnrBuilder, NodeId};
use libp2p::multiaddr::Protocol;
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters, ProtocolsHandler};
use log::*;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_timer::Delay;

/// Maximum seconds before searching for extra peers.
const MAX_TIME_BETWEEN_PEER_SEARCHES: u64 = 60;
/// Initial delay between peer searches.
const INITIAL_SEARCH_DELAY: u64 = 5;

/// Lighthouse discovery behaviour. This provides peer management and discovery using the Discv5
/// libp2p protocol.
pub struct Discovery<TSubstream> {
    /// The peers currently connected to libp2p streams.
    connected_peers: HashSet<PeerId>,

    /// The target number of connected peers on the libp2p interface.
    max_peers: usize,

    /// The delay between peer discovery searches.
    peer_discovery_delay: Delay,

    /// Tracks the last discovery delay. The delay is doubled each round until the max
    /// time is reached.
    past_discovery_delay: u64,

    /// The TCP port for libp2p. Used to convert an updated IP address to a multiaddr. Note: This
    /// assumes that the external TCP port is the same as the internal TCP port if behind a NAT.
    //TODO: Improve NAT handling limit the above restriction
    tcp_port: u16,

    /// The discovery behaviour used to discover new peers.
    discovery: Discv5<TSubstream>,
}

impl<TSubstream> Discovery<TSubstream> {
    pub fn new(
        local_key: &Keypair,
        config: &NetworkConfig,
    ) -> Result<Self, Error> {
        // checks if current ENR matches that found on disk
        let local_enr = load_enr(local_key, config)?;

        info!("ENR Initialised {}, {}", local_enr.to_base64(), local_enr.seq());
        debug!("Discv5 Node ID Initialised {}", local_enr.node_id());

        let mut discovery = Discv5::new(local_enr, local_key.clone(), config.listen_address)
            .map_err(|e| format!("Discv5 service failed. Error: {:?}", e))?;

        // Add bootnodes to routing table
        for bootnode_enr in config.boot_nodes.clone() {
            debug!(
                "Adding node to routing table {}",
                bootnode_enr.node_id()
            );
            discovery.add_enr(bootnode_enr);
        }

        Ok(Self {
            connected_peers: HashSet::new(),
            max_peers: config.max_peers,
            peer_discovery_delay: Delay::new(Instant::now()),
            past_discovery_delay: INITIAL_SEARCH_DELAY,
            tcp_port: config.libp2p_port,
            discovery,
        })
    }

    pub fn local_enr(&self) -> &Enr {
        self.discovery.local_enr()
    }

    /// Manually search for peers. This restarts the discovery round, sparking multiple rapid
    /// queries.
    pub fn discover_peers(&mut self) {
        self.past_discovery_delay = INITIAL_SEARCH_DELAY;
        self.find_peers();
    }

    /// Add an Enr to the routing table of the discovery mechanism.
    pub fn add_enr(&mut self, enr: Enr) {
        self.discovery.add_enr(enr);
    }

    /// The current number of connected libp2p peers.
    pub fn connected_peers(&self) -> usize {
        self.connected_peers.len()
    }

    /// The current number of connected libp2p peers.
    pub fn connected_peer_set(&self) -> &HashSet<PeerId> {
        &self.connected_peers
    }

    /// Search for new peers using the underlying discovery mechanism.
    fn find_peers(&mut self) {
        // pick a random NodeId
        let random_node = NodeId::random();
        debug!("Searching for peers");
        self.discovery.find_node(random_node);

        // update the time until next discovery
        let delay = {
            if self.past_discovery_delay < MAX_TIME_BETWEEN_PEER_SEARCHES {
                self.past_discovery_delay *= 2;
                self.past_discovery_delay
            } else {
                MAX_TIME_BETWEEN_PEER_SEARCHES
            }
        };
        self.peer_discovery_delay
            .reset(Instant::now() + Duration::from_secs(delay));
    }
}

// Redirect all behaviour events to underlying discovery behaviour.
impl<TSubstream> NetworkBehaviour for Discovery<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    type ProtocolsHandler = <Discv5<TSubstream> as NetworkBehaviour>::ProtocolsHandler;
    type OutEvent = <Discv5<TSubstream> as NetworkBehaviour>::OutEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        NetworkBehaviour::new_handler(&mut self.discovery)
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        // Let discovery track possible known peers.
        self.discovery.addresses_of_peer(peer_id)
    }

    fn inject_connected(&mut self, peer_id: PeerId, _endpoint: ConnectedPoint) {
        self.connected_peers.insert(peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _endpoint: ConnectedPoint) {
        self.connected_peers.remove(peer_id);
    }

    fn inject_replaced(
        &mut self,
        _peer_id: PeerId,
        _closed: ConnectedPoint,
        _opened: ConnectedPoint,
    ) {
        // discv5 doesn't implement
    }

    fn inject_node_event(
        &mut self,
        _peer_id: PeerId,
        _event: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent,
    ) {
        // discv5 doesn't implement
    }

    fn poll(
        &mut self,
        params: &mut impl PollParameters,
    ) -> Async<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
            Self::OutEvent,
        >,
    > {
        // search for peers if it is time
        loop {
            match self.peer_discovery_delay.poll() {
                Ok(Async::Ready(_)) => {
                    if self.connected_peers.len() < self.max_peers {
                        self.find_peers();
                    }
                }
                Ok(Async::NotReady) => break,
                Err(e) => {
                    warn!("Discovery peer search failed {:?}", e);
                }
            }
        }

        // Poll discovery
        loop {
            match self.discovery.poll(params) {
                Async::Ready(NetworkBehaviourAction::GenerateEvent(event)) => {
                    match event {
                        Discv5Event::Discovered(_enr) => {
                            // not concerned about FINDNODE results, rather the result of an entire
                            // query.
                        }
                        Discv5Event::SocketUpdated(socket) => {
                            info!("Address updated (IP: {})", socket.ip());
                            let mut address = Multiaddr::from(socket.ip());
                            address.push(Protocol::Tcp(self.tcp_port));

                            return Async::Ready(NetworkBehaviourAction::ReportObservedAddr {
                                address,
                            });
                        }
                        Discv5Event::FindNodeResult { closer_peers, .. } => {
                            debug!("Discovery query completed {}", closer_peers.len());
                            if closer_peers.is_empty() {
                                debug!("Discovery random query found no peers");
                            }
                            for peer_id in closer_peers {
                                // if we need more peers, attempt a connection
                                if self.connected_peers.len() < self.max_peers
                                    && self.connected_peers.get(&peer_id).is_none()
                                {
                                    debug!("Peer discovered {:?}", peer_id);
                                    return Async::Ready(NetworkBehaviourAction::DialPeer {
                                        peer_id,
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
                // discv5 does not output any other NetworkBehaviourAction
                Async::Ready(_) => {}
                Async::NotReady => break,
            }
        }
        Async::NotReady
    }
}

/// Loads an ENR from file if it exists and matches the current NodeId and sequence number. If none
/// exists, generates a new one.
///
/// If an ENR exists, with the same NodeId and IP address, we use the disk-generated one as its
/// ENR sequence will be equal or higher than a newly generated one.
fn load_enr(
    local_key: &Keypair,
    config: &NetworkConfig,
) -> Result<Enr, String> {
    // Build the local ENR.
    // Note: Discovery should update the ENR record's IP to the external IP as seen by the
    // majority of our peers.
    let local_enr = EnrBuilder::new()
        .ip(config.discovery_address)
        .tcp(config.libp2p_port)
        .udp(config.discovery_port)
        .build(&local_key)
        .map_err(|e| format!("Could not build Local ENR: {:?}", e))?;

    Ok(local_enr)
}
