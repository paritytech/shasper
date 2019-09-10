mod items;
mod codec;

pub use items::{HelloMessage, GoodbyeReason, BeaconBlocksRequest, RecentBeaconBlocksRequest};
pub use codec::{InboundCodec, OutboundCodec};

use beacon::{Config, types::BeaconBlock};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RPCType {
	Hello = 0,
	Goodbye = 1,
	BeaconBlocks = 2,
	RecentBeaconBlocks = 3,
}

pub enum RPCRequest {
	Hello(HelloMessage),
	Goodbye(GoodbyeReason),
	BeaconBlocks(BeaconBlocksRequest),
	RecentBeaconBlocks(RecentBeaconBlocksRequest),
}

pub enum RPCResponse<C: Config> {
	Hello(HelloMessage),
	BeaconBlocks(Vec<BeaconBlock<C>>),
	RecentBeaconBlocks(Vec<BeaconBlock<C>>),
	Unknown(u8, Vec<u8>),
}
