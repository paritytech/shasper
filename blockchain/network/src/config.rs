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

use enr::Enr;
use libp2p::gossipsub::{GossipsubConfig, GossipsubConfigBuilder};
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
/// Network configuration for lighthouse.
pub struct Config {
    /// IP address to listen on.
    pub listen_address: std::net::IpAddr,

    /// The TCP port that libp2p listens on.
    pub libp2p_port: u16,

    /// The address to broadcast to peers about which address we are listening on.
    pub discovery_address: std::net::IpAddr,

    /// UDP port that discovery listens on.
    pub discovery_port: u16,

    /// Target number of connected peers.
    pub max_peers: usize,

    /// Gossipsub configuration parameters.
    #[serde(skip)]
    pub gs_config: GossipsubConfig,

    /// List of nodes to initially connect to.
    pub boot_nodes: Vec<Enr>,

    /// List of libp2p nodes to initially connect to.
    pub libp2p_nodes: Vec<Multiaddr>,

    /// Client version
    pub client_version: String,

    /// List of extra topics to initially subscribe to as strings.
    pub topics: Vec<String>,
}

impl Default for Config {
    /// Generate a default network configuration.
    fn default() -> Self {
        Config {
            listen_address: "127.0.0.1".parse().expect("valid ip address"),
            libp2p_port: 9000,
            discovery_address: "127.0.0.1".parse().expect("valid ip address"),
            discovery_port: 9000,
            max_peers: 10,
            // Note: The topics by default are sent as plain strings. Hashes are an optional
            // parameter.
            gs_config: GossipsubConfigBuilder::new()
                .max_transmit_size(1_048_576)
                .heartbeat_interval(Duration::from_secs(20))
                .build(),
            boot_nodes: vec![],
            libp2p_nodes: vec![],
            client_version: crate::VERSION.to_string(),
            topics: Vec::new(),
        }
    }
}

/// Generates a default Config.
impl Config {
    pub fn new() -> Self {
        Config::default()
    }
}
