mod items;
mod codec;

pub use items::{HelloMessage, GoodbyeReason, BeaconBlocksRequest, RecentBeaconBlocksRequest};
pub use codec::{InboundCodec, OutboundCodec};

use beacon::{
	Config, types::{BeaconBlock, Attestation, VoluntaryExit, ProposerSlashing, AttesterSlashing},
};

/// RPC type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RPCType {
	Hello = 0,
	Goodbye = 1,
	BeaconBlocks = 2,
	RecentBeaconBlocks = 3,
}

impl libp2p::core::ProtocolName for RPCType {
	fn protocol_name(&self) -> &[u8] {
		match self {
			RPCType::Hello => b"/eth2/beacon_chain/req/hello/1/ssz",
			RPCType::Goodbye => b"/eth2/beacon_chain/req/goodbye/1/ssz",
			RPCType::BeaconBlocks => b"/eth2/beacon_chain/req/beacon_blocks/1/ssz",
			RPCType::RecentBeaconBlocks => b"/eth2/beacon_chain/req/recent_beacon_blocks/1/ssz",
		}
	}
}

/// Possible RPC requests.
#[derive(Debug, Clone)]
pub enum RPCRequest {
	Hello(HelloMessage),
	Goodbye(GoodbyeReason),
	BeaconBlocks(BeaconBlocksRequest),
	RecentBeaconBlocks(RecentBeaconBlocksRequest),
}

impl libp2p_rpc::RPCRequest for RPCRequest {
	fn is_goodbye(&self) -> bool {
		match self {
			RPCRequest::Goodbye(_) => true,
			_ => false,
		}
	}

	fn expect_response(&self) -> bool {
		match self {
			RPCRequest::Goodbye(_) => false,
			_ => true,
		}
	}
}

/// Corresponding RPC responses.
#[derive(Debug, Clone)]
pub enum RPCResponse<C: Config> {
	Hello(HelloMessage),
	BeaconBlocks(Vec<BeaconBlock<C>>),
	RecentBeaconBlocks(Vec<BeaconBlock<C>>),
	Unknown(u8, Vec<u8>),
}

/// Pubsub type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PubsubType {
	Block,
	Attestation,
	VoluntaryExit,
	ProposerSlashing,
	AttesterSlashing,
}

/// Messages that are passed to and from the pubsub (Gossipsub) behaviour. These are encoded and
/// decoded upstream.
#[derive(Debug, Clone)]
pub enum PubsubMessage<C: Config> {
	/// Gossipsub message providing notification of a new block.
    Block(BeaconBlock<C>),
    /// Gossipsub message providing notification of a new attestation.
    Attestation(Attestation<C>),
    /// Gossipsub message providing notification of a voluntary exit.
    VoluntaryExit(VoluntaryExit),
    /// Gossipsub message providing notification of a new proposer slashing.
    ProposerSlashing(ProposerSlashing),
    /// Gossipsub message providing notification of a new attester slashing.
    AttesterSlashing(AttesterSlashing<C>),
}
