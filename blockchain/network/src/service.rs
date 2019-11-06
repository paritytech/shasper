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

use crate::behaviour::Behaviour;
use crate::{NetworkConfig, Error, Libp2pEvent};
use crate::multiaddr::Protocol;
use network_messages::PubsubType;
use futures01::prelude::*;
use futures01::Stream;
use libp2p::core::{
    identity::Keypair,
    multiaddr::Multiaddr,
    muxing::StreamMuxerBox,
    nodes::Substream,
    transport::boxed::Boxed,
    upgrade::{InboundUpgradeExt, OutboundUpgradeExt},
};
use libp2p::{core, secio, PeerId, Swarm, Transport};
use libp2p::gossipsub::Topic;
use beacon::Config;
use log::*;
use std::time::Duration;

type Libp2pStream = Boxed<(PeerId, StreamMuxerBox), Error>;
type Libp2pBehaviour<C> = Behaviour<C, Substream<StreamMuxerBox>>;

/// The configuration and state of the libp2p components for the beacon node.
pub struct Service<C: Config> {
    /// The libp2p Swarm handler.
    pub swarm: Swarm<Libp2pStream, Libp2pBehaviour<C>>,
    /// This node's PeerId.
    pub local_peer_id: PeerId,
}

impl<C: Config> Service<C> {
    pub fn new(config: NetworkConfig) -> Result<Self, Error> {
        trace!("Libp2p Service starting");

        // load the private key from CLI flag, disk or generate a new one
        let local_private_key = load_private_key();
        let local_peer_id = PeerId::from(local_private_key.public());
        info!("Libp2p Service {:?}", local_peer_id);

        let mut swarm = {
            // Set up the transport - tcp/ws with secio and mplex/yamux
            let transport = build_transport(local_private_key.clone());
            // Lighthouse network behaviour
            let behaviour = Behaviour::new(&local_private_key, &config)?;
            Swarm::new(transport, behaviour, local_peer_id.clone())
        };

        // listen on the specified address
        let listen_multiaddr = {
            let mut m = Multiaddr::from(config.listen_address);
            m.push(Protocol::Tcp(config.libp2p_port));
            m
        };

        match Swarm::listen_on(&mut swarm, listen_multiaddr.clone()) {
            Ok(_) => {
                info!("Listening established {}", listen_multiaddr);
            }
            Err(err) => {
                warn!(
                    "Unable to listen on libp2p address {:?} {}",
					err,
                    listen_multiaddr,
                );
                return Err("Libp2p was unable to listen on the given listen address."
						   .to_string().into());
            }
        };

        // attempt to connect to user-input libp2p nodes
        for multiaddr in config.libp2p_nodes {
            match Swarm::dial_addr(&mut swarm, multiaddr.clone()) {
                Ok(()) => debug!("Dialing libp2p peer {}", multiaddr),
                Err(err) => debug!(
                    "Could not connect to peer {}, {:?}", multiaddr, err
                ),
            };
        }

        // subscribe to default gossipsub topics
        let topics = vec![
			PubsubType::Block, PubsubType::Attestation,
			PubsubType::VoluntaryExit, PubsubType::ProposerSlashing,
			PubsubType::AttesterSlashing,
		];

		let mut topics = topics.into_iter().map(|v| v.gossipsub_topic()).collect::<Vec<_>>();

        // Add any topics specified by the user
        topics.append(
            &mut config
                .topics
                .iter()
                .cloned()
                .map(|s| Topic::new(s))
                .collect(),
        );

        let mut subscribed_topics = vec![];
        for topic in topics {
            if swarm.subscribe(topic.clone()) {
                trace!("Subscribed to topic {}", topic);
                subscribed_topics.push(topic);
            } else {
                warn!("Could not subscribe to topic {}", topic);
            }
        }
        info!("Subscribed to topics {:?}", subscribed_topics.iter().map(|t| format!("{}", t)).collect::<Vec<String>>());

        Ok(Service {
            local_peer_id,
            swarm,
        })
    }
}

impl<C: Config> Stream for Service<C> {
    type Item = Libp2pEvent<C>;
    type Error = crate::error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
		self.swarm.poll().map_err(Into::into)
    }
}

/// The implementation supports TCP/IP, WebSockets over TCP/IP, secio as the encryption layer, and
/// mplex or yamux as the multiplexing layer.
fn build_transport(local_private_key: Keypair) -> Boxed<(PeerId, StreamMuxerBox), Error> {
    // TODO: The Wire protocol currently doesn't specify encryption and this will need to be customised
    // in the future.
    let transport = libp2p::tcp::TcpConfig::new();
    let transport = libp2p::dns::DnsConfig::new(transport);
    #[cfg(feature = "libp2p-websocket")]
    let transport = {
        let trans_clone = transport.clone();
        transport.or_transport(websocket::WsConfig::new(trans_clone))
    };
    transport
		.upgrade(core::upgrade::Version::V1)
        .authenticate(secio::SecioConfig::new(local_private_key))
        .multiplex(core::upgrade::SelectUpgrade::new(
            libp2p::yamux::Config::default(),
            libp2p::mplex::MplexConfig::new(),
        ))
        .map(|(peer, muxer), _| (peer, core::muxing::StreamMuxerBox::new(muxer)))
        .timeout(Duration::from_secs(20))
        .map_err(|e| Error::Libp2p(Box::new(e)))
        .boxed()
}

/// Loads a private key from disk. If this fails, a new key is
/// generated and is then saved to disk.
///
/// Currently only secp256k1 keys are allowed, as these are the only keys supported by discv5.
fn load_private_key() -> Keypair {
    // if a key could not be loaded from disk, generate a new one and save it
    let local_private_key = Keypair::generate_secp256k1();
    local_private_key
}
