mod handler;
mod protocol;

use libp2p::PeerId;
use libp2p::swarm::NetworkBehaviourAction;
use core::marker::PhantomData;

pub type RequestId = usize;

pub enum RPCError {
	Codec,
	StreamTimeout,
	Custom(String),
}

impl<T> From<tokio::timer::timeout::Error<T>> for RPCError {
    fn from(err: tokio::timer::timeout::Error<T>) -> Self {
        if err.is_elapsed() {
            RPCError::StreamTimeout
        } else {
            RPCError::Custom("Stream timer failed".into())
        }
    }
}

pub enum RPCEvent<Req, Res> {
	Request(RequestId, Req),
	Response(RequestId, Res),
	Error(RequestId, RPCError),
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
