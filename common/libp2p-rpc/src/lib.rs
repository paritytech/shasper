mod handler;
mod protocol;

pub use protocol::RPCProtocol;

use futures::prelude::*;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::codec::Encoder;
use libp2p::{Multiaddr, PeerId};
use libp2p::core::{ConnectedPoint, ProtocolName};
use libp2p::swarm::{
	protocols_handler::ProtocolsHandler, NetworkBehaviour, NetworkBehaviourAction,
	PollParameters,
};
use core::marker::PhantomData;

pub type RequestId = usize;

#[derive(Debug, Clone)]
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

pub trait RPCRequest<T: RPCType> {
	fn is_goodbye(&self) -> bool;
	fn expect_response(&self) -> bool;
	fn typ(&self) -> T;
}

pub trait RPCType: ProtocolName + Sized {
	fn all() -> Vec<Self>;
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum RPCMessage<Req, Res> {
	Event(PeerId, RPCEvent<Req, Res>),
	PeerDialed(PeerId),
	PeerDisconnected(PeerId),
}

pub struct RPC<P: RPCProtocol, TSubstream> {
	events: Vec<NetworkBehaviourAction<RPCEvent<P::Request, P::Response>,
									   RPCMessage<P::Request, P::Response>>>,
	_marker: PhantomData<TSubstream>,
}

impl<P: RPCProtocol, TSubstream> RPC<P, TSubstream> {
	pub fn new() -> Self {
        RPC {
            events: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Submits an RPC request.
    ///
    /// The peer must be connected for this to succeed.
    pub fn send_rpc(&mut self, peer_id: PeerId, rpc_event: RPCEvent<P::Request, P::Response>) {
        self.events.push(NetworkBehaviourAction::SendEvent {
            peer_id,
            event: rpc_event,
        });
    }
}

impl<P, TSubstream> NetworkBehaviour for RPC<P, TSubstream> where
	P: RPCProtocol + Default + Clone,
	TSubstream: AsyncRead + AsyncWrite,
	<P::OutboundCodec as Encoder>::Error: core::fmt::Debug,
{
	type ProtocolsHandler = crate::handler::RPCHandler<P, TSubstream>;
    type OutEvent = RPCMessage<P::Request, P::Response>;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        Default::default()
    }

    // handled by discovery
    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        Vec::new()
    }

    fn inject_connected(&mut self, peer_id: PeerId, connected_point: ConnectedPoint) {
        self.events.push(NetworkBehaviourAction::GenerateEvent(
            RPCMessage::PeerDialed(peer_id),
        ));
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _: ConnectedPoint) {
        // inform the rpc handler that the peer has disconnected
        self.events.push(NetworkBehaviourAction::GenerateEvent(
            RPCMessage::PeerDisconnected(peer_id.clone()),
        ));
    }

    fn inject_node_event(
        &mut self,
        source: PeerId,
        event: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent,
    ) {
        // send the event to the user
        self.events
            .push(NetworkBehaviourAction::GenerateEvent(RPCMessage::Event(
                source, event,
            )));
    }

    fn poll(
        &mut self,
        _: &mut impl PollParameters,
    ) -> Async<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
            Self::OutEvent,
        >,
    > {
        if !self.events.is_empty() {
            return Async::Ready(self.events.remove(0));
        }
        Async::NotReady
    }
}
