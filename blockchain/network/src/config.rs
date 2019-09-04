use enr::Enr;
use libp2p::gossipsub::{GossipsubConfig, GossipsubConfigBuilder};
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The gossipsub topic names.
// These constants form a topic name of the form /TOPIC_PREFIX/TOPIC/ENCODING_POSTFIX
// For example /eth2/beacon_block/ssz
pub const TOPIC_PREFIX: &str = "eth2";
pub const TOPIC_ENCODING_POSTFIX: &str = "ssz";
pub const BEACON_BLOCK_TOPIC: &str = "beacon_block";
pub const BEACON_ATTESTATION_TOPIC: &str = "beacon_attestation";
pub const VOLUNTARY_EXIT_TOPIC: &str = "voluntary_exit";
pub const PROPOSER_SLASHING_TOPIC: &str = "proposer_slashing";
pub const ATTESTER_SLASHING_TOPIC: &str = "attester_slashing";
pub const SHARD_TOPIC_PREFIX: &str = "shard";

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
