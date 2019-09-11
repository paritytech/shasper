use core::marker::PhantomData;
use libp2p::core::upgrade::UpgradeInfo;
use libp2p_rpc::RPCProtocol as RPCProtocolT;
use network_messages::{InboundCodec, OutboundCodec};
use beacon::Config;

pub use network_messages::{RPCType, RPCRequest, RPCResponse};
pub type RPC<C, TSubstream> = libp2p_rpc::RPC<RPCProtocol<C>, TSubstream>;
pub type RPCMessage<C> = libp2p_rpc::RPCMessage<RPCRequest, RPCResponse<C>>;
pub type RPCEvent<C> = libp2p_rpc::RPCEvent<RPCRequest, RPCResponse<C>>;

#[derive(Default, Clone)]
pub struct RPCProtocol<C: Config> {
	_marker: PhantomData<C>,
}

impl<C: Config> UpgradeInfo for RPCProtocol<C> {
	type Info = RPCType;
	type InfoIter = Vec<RPCType>;

	fn protocol_info(&self) -> Self::InfoIter {
		vec![
			RPCType::Hello, RPCType::Goodbye,
			RPCType::BeaconBlocks, RPCType::RecentBeaconBlocks
		]
	}
}

impl<C: Config> RPCProtocolT for RPCProtocol<C> {
	type Request = RPCRequest;
	type Response = RPCResponse<C>;

	type InboundCodec = InboundCodec<C>;
	fn inbound_codec(&self, protocol: RPCType) -> Self::InboundCodec {
		InboundCodec::new(protocol)
	}

	type OutboundCodec = OutboundCodec<C>;
	fn outbound_codec(&self, protocol: RPCType) -> Self::OutboundCodec {
		OutboundCodec::new(protocol)
	}
}
