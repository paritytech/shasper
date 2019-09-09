mod handler;
mod protocol;

use libp2p::PeerId;
use libp2p::swarm::NetworkBehaviourAction;
use core::marker::PhantomData;

pub type RequestId = usize;

#[derive(Debug)]
pub enum RPCError {
	Codec,
	StreamTimeout,
	Custom(String),
}

impl std::fmt::Display for RPCError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for RPCError { }

impl<T> From<tokio::timer::timeout::Error<T>> for RPCError {
    fn from(err: tokio::timer::timeout::Error<T>) -> Self {
        if err.is_elapsed() {
            RPCError::StreamTimeout
        } else {
            RPCError::Custom("Stream timer failed".into())
        }
    }
}

pub trait RPCRequest {
	fn is_goodbye(&self) -> bool;
	fn expect_response(&self) -> bool;
}

pub enum RPCEvent<Req, Res> {
	Request(RequestId, Req),
	Response(RequestId, Res),
	Error(RequestId, RPCError),
}

impl<Req, Res> RPCEvent<Req, Res> {
	pub fn id(&self) -> RequestId {
		match self {
			RPCEvent::Request(id, _) => *id,
			RPCEvent::Response(id, _) => *id,
			RPCEvent::Error(id, _) => *id,
		}
	}
}

pub enum RPCMessage<Req, Res> {
	Event(PeerId, RPCEvent<Req, Res>),
	PeerDailed(PeerId),
	PeerDisconnected(PeerId),
}

pub struct RPC<Req, Res, TSubstream> {
	events: Vec<NetworkBehaviourAction<RPCEvent<Req, Res>, RPCMessage<Req, Res>>>,
	_marker: PhantomData<TSubstream>,
}
