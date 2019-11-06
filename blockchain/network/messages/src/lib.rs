mod items;
mod codec;

pub use items::{HelloMessage, GoodbyeReason, BeaconBlocksRequest, RecentBeaconBlocksRequest};
pub use codec::{InboundCodec, OutboundCodec};

use beacon::{
	Config, types::{BeaconBlock, Attestation, VoluntaryExit, ProposerSlashing, AttesterSlashing},
};
use libp2p::gossipsub;

/// RPC type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RPCType {
	Hello = 0,
	Goodbye = 1,
	BeaconBlocks = 2,
	RecentBeaconBlocks = 3,
}

impl libp2p_rpc::RPCType for RPCType {
	fn all() -> Vec<Self> {
		vec![
			RPCType::Hello, RPCType::Goodbye,
			RPCType::BeaconBlocks, RPCType::RecentBeaconBlocks,
		]
	}
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

impl libp2p_rpc::RPCRequest<RPCType> for RPCRequest {
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

	fn typ(&self) -> RPCType {
		match self {
			Self::Hello(_) => RPCType::Hello,
			Self::Goodbye(_) => RPCType::Goodbye,
			Self::BeaconBlocks(_) => RPCType::BeaconBlocks,
			Self::RecentBeaconBlocks(_) => RPCType::RecentBeaconBlocks,
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

impl PubsubType {
	pub fn from_gossipsub_topic_hash(topic: &gossipsub::TopicHash) -> Option<Self> {
		match topic.as_str() {
			"/eth2/beacon_block/ssz/beacon_block" => Some(Self::Block),
			"/eth2/beacon_block/ssz/beacon_attestation" => Some(Self::Attestation),
			"/eth2/beacon_block/ssz/voluntary_exit" => Some(Self::VoluntaryExit),
			"/eth2/beacon_block/ssz/proposer_slashing" => Some(Self::ProposerSlashing),
			"/eth2/beacon_block/ssz/attester_slashing" => Some(Self::AttesterSlashing),
			_ => None,
		}
	}

	pub fn gossipsub_topic_hash(&self) -> gossipsub::TopicHash {
		gossipsub::TopicHash::from_raw(match self {
			Self::Block => "/eth2/beacon_block/ssz/beacon_block".to_string(),
			Self::Attestation => "/eth2/beacon_block/ssz/beacon_attestation".to_string(),
			Self::VoluntaryExit => "/eth2/beacon_block/ssz/voluntary_exit".to_string(),
			Self::ProposerSlashing => "/eth2/beacon_block/ssz/proposer_slashing".to_string(),
			Self::AttesterSlashing => "/eth2/beacon_block/ssz/attester_slashing".to_string(),
		})
	}

	pub fn gossipsub_topic(&self) -> gossipsub::Topic {
		gossipsub::Topic::new(match self {
			Self::Block => "/eth2/beacon_block/ssz/beacon_block".to_string(),
			Self::Attestation => "/eth2/beacon_block/ssz/beacon_attestation".to_string(),
			Self::VoluntaryExit => "/eth2/beacon_block/ssz/voluntary_exit".to_string(),
			Self::ProposerSlashing => "/eth2/beacon_block/ssz/proposer_slashing".to_string(),
			Self::AttesterSlashing => "/eth2/beacon_block/ssz/attester_slashing".to_string(),
		})
	}
}

impl<'a, C: Config> From<&'a PubsubMessage<C>> for PubsubType {
	fn from(message: &'a PubsubMessage<C>) -> PubsubType {
		match message {
			PubsubMessage::Block(_) => PubsubType::Block,
			PubsubMessage::Attestation(_) => PubsubType::Attestation,
			PubsubMessage::VoluntaryExit(_) => PubsubType::VoluntaryExit,
			PubsubMessage::ProposerSlashing(_) => PubsubType::ProposerSlashing,
			PubsubMessage::AttesterSlashing(_) => PubsubType::AttesterSlashing,
		}
	}
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

impl<C: Config> PubsubMessage<C> {
	pub fn ssz_data(&self) -> Vec<u8> {
		match self {
			Self::Block(item) => ssz::Encode::encode(item),
			Self::Attestation(item) => ssz::Encode::encode(item),
			Self::VoluntaryExit(item) => ssz::Encode::encode(item),
			Self::ProposerSlashing(item) => ssz::Encode::encode(item),
			Self::AttesterSlashing(item) => ssz::Encode::encode(item),
		}
	}

	pub fn from_ssz_data(typ: PubsubType, mut data: &[u8]) -> Result<Self, ssz::Error> {
		Ok(match typ {
			PubsubType::Block => Self::Block(ssz::Decode::decode(&mut data)?),
			PubsubType::Attestation => Self::Attestation(ssz::Decode::decode(&mut data)?),
			PubsubType::VoluntaryExit => Self::VoluntaryExit(ssz::Decode::decode(&mut data)?),
			PubsubType::ProposerSlashing => Self::ProposerSlashing(ssz::Decode::decode(&mut data)?),
			PubsubType::AttesterSlashing => Self::AttesterSlashing(ssz::Decode::decode(&mut data)?),
		})
	}
}
