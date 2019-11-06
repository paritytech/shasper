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

use crate::discovery::Discovery;
use crate::{Libp2pEvent, Error, NetworkConfig};
use crate::rpc::{RPC, RPCMessage, RPCEvent};
use futures01::prelude::*;
use libp2p::{
    core::identity::Keypair,
    discv5::Discv5Event,
    gossipsub::{Gossipsub, GossipsubEvent, Topic},
    identify::{Identify, IdentifyEvent},
    ping::{Ping, PingConfig, PingEvent},
    swarm::{NetworkBehaviourAction, NetworkBehaviourEventProcess},
    tokio_io::{AsyncRead, AsyncWrite},
    NetworkBehaviour, PeerId,
};
use network_messages::{PubsubType, PubsubMessage};
use beacon::Config;
use log::*;
use std::num::NonZeroU32;
use std::time::Duration;

const MAX_IDENTIFY_ADDRESSES: usize = 20;

/// Builds the network behaviour that manages the core protocols of eth2.
/// This core behaviour is managed by `Behaviour` which adds peer management to all core
/// behaviours.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Libp2pEvent<C>", poll_method = "poll")]
pub struct Behaviour<C: Config, TSubstream: AsyncRead + AsyncWrite> {
    /// The routing pub-sub mechanism for eth2.
    gossipsub: Gossipsub<TSubstream>,
    /// The Eth2 RPC specified in the wire-0 protocol.
    rpc: RPC<C, TSubstream>,
    /// Keep regular connection to peers and disconnect if absent.
    // TODO: Remove Libp2p ping in favour of discv5 ping.
    ping: Ping<TSubstream>,
    // TODO: Using id for initial interop. This will be removed by mainnet.
    /// Provides IP addresses and peer information.
    identify: Identify<TSubstream>,
    /// Discovery behaviour.
    discovery: Discovery<TSubstream>,
    #[behaviour(ignore)]
    /// The events generated by this behaviour to be consumed in the swarm poll.
    events: Vec<Libp2pEvent<C>>,
}

impl<C: Config, TSubstream: AsyncRead + AsyncWrite> Behaviour<C, TSubstream> {
    pub fn new(
        local_key: &Keypair,
        net_conf: &NetworkConfig,
    ) -> Result<Self, Error> {
        let local_peer_id = local_key.public().clone().into_peer_id();

        let ping_config = PingConfig::new()
            .with_timeout(Duration::from_secs(30))
            .with_interval(Duration::from_secs(20))
            .with_max_failures(NonZeroU32::new(2).expect("2 != 0"))
            .with_keep_alive(false);

        let identify = Identify::new(
            "shasper/libp2p".into(),
			crate::VERSION.into(),
            local_key.public(),
        );

        Ok(Behaviour {
            rpc: RPC::new(),
            gossipsub: Gossipsub::new(local_peer_id.clone(), net_conf.gs_config.clone()),
            discovery: Discovery::new(local_key, net_conf)?,
            ping: Ping::new(ping_config),
            identify,
            events: Vec::new(),
        })
    }

    pub fn discovery(&self) -> &Discovery<TSubstream> {
        &self.discovery
    }
}

// Implement the NetworkBehaviourEventProcess trait so that we can derive NetworkBehaviour for Behaviour
impl<C: Config, TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<GossipsubEvent>
    for Behaviour<C, TSubstream>
{
    fn inject_event(&mut self, event: GossipsubEvent) {
        match event {
            GossipsubEvent::Message(_, gs_msg) => {
                trace!("Received GossipEvent");

				let typ = match gs_msg.topics.iter()
					.map(|v| PubsubType::from_gossipsub_topic_hash(v))
					.filter(|v| v.is_some())
					.next()
				{
					Some(Some(typ)) => typ,
					_ => {
						warn!("Unknown gossipsub type");
						return
					},
				};
				let msg = match PubsubMessage::from_ssz_data(typ, &gs_msg.data) {
					Ok(msg) => msg,
					Err(_) => {
						warn!("Uninterpretable gossipsub message");
						return
					},
				};

                self.events.push(Libp2pEvent::Pubsub(gs_msg.source, msg));
            }
            GossipsubEvent::Subscribed { .. } => {}
            GossipsubEvent::Unsubscribed { .. } => {}
        }
    }
}

impl<C: Config, TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<RPCMessage<C>>
    for Behaviour<C, TSubstream>
{
    fn inject_event(&mut self, event: RPCMessage<C>) {
        match event {
            RPCMessage::PeerDialed(peer_id) => {
                self.events.push(Libp2pEvent::PeerDialed(peer_id))
            }
            RPCMessage::PeerDisconnected(peer_id) => {
                self.events.push(Libp2pEvent::PeerDisconnected(peer_id))
            }
            RPCMessage::Event(peer_id, rpc_event) => {
                self.events.push(Libp2pEvent::RPC(peer_id, rpc_event))
            }
        }
    }
}

impl<C: Config, TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<PingEvent>
    for Behaviour<C, TSubstream>
{
    fn inject_event(&mut self, _event: PingEvent) {
        // not interested in ping responses at the moment.
    }
}

impl<C: Config, TSubstream: AsyncRead + AsyncWrite> Behaviour<C, TSubstream> {
    /// Consumes the events list when polled.
    fn poll<TBehaviourIn>(
        &mut self,
    ) -> Async<NetworkBehaviourAction<TBehaviourIn, Libp2pEvent<C>>> {
        if !self.events.is_empty() {
            return Async::Ready(NetworkBehaviourAction::GenerateEvent(self.events.remove(0)));
        }

        Async::NotReady
    }
}

impl<C: Config, TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<IdentifyEvent>
    for Behaviour<C, TSubstream>
{
    fn inject_event(&mut self, event: IdentifyEvent) {
        match event {
			IdentifyEvent::Received {
                peer_id, mut info, ..
            } => {
                if info.listen_addrs.len() > MAX_IDENTIFY_ADDRESSES {
                    debug!(
                        "More than 20 addresses have been identified, truncating"
                    );
                    info.listen_addrs.truncate(MAX_IDENTIFY_ADDRESSES);
                }
				debug!(
					"Identified peer ({}, {}, {}, {:?}, {:?})",
					peer_id,
					info.protocol_version,
					info.agent_version,
					info.listen_addrs,
					info.protocols,
                );
				self.events.push(Libp2pEvent::PeerDialed(peer_id));
            },
			IdentifyEvent::Sent { .. } => (),
            IdentifyEvent::Error { .. } => (),
        }
    }
}

impl<C: Config, TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<Discv5Event>
    for Behaviour<C, TSubstream>
{
    fn inject_event(&mut self, _event: Discv5Event) {
        // discv5 has no events to inject
    }
}

/// Implements the combined behaviour for the libp2p service.
impl<C: Config, TSubstream: AsyncRead + AsyncWrite> Behaviour<C, TSubstream> {
    /// Subscribes to a gossipsub topic.
    pub fn subscribe(&mut self, topic: Topic) -> bool {
        self.gossipsub.subscribe(topic)
    }

    /// Publishes a message on the pubsub (gossipsub) behaviour.
    pub fn publish(&mut self, message: PubsubMessage<C>) {
        let data = message.ssz_data();
		let typ = PubsubType::from(&message);
		self.gossipsub.publish(&typ.gossipsub_topic(), data);
    }

    /// Sends an RPC Request/Response via the RPC protocol.
    pub fn send_rpc(&mut self, peer_id: PeerId, rpc_event: RPCEvent<C>) {
        self.rpc.send_rpc(peer_id, rpc_event);
    }

    /// Connected peers.
    pub fn connected_peers(&self) -> usize {
        self.discovery.connected_peers()
    }
}
