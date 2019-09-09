use core::marker::PhantomData;
use std::time::{Instant, Duration};
use smallvec::SmallVec;
use fnv::FnvHashMap;
use libp2p::swarm::protocols_handler::{
	SubstreamProtocol, ProtocolsHandlerUpgrErr, KeepAlive,
};
use tokio_io::{AsyncRead, AsyncWrite};
use crate::{RPCEvent, RPCError, RequestId};
use crate::protocol::{InboundFramed, OutboundFramed, RPCProtocol};

/// Implementation of `ProtocolsHandler` for the RPC protocol.
pub struct RPCHandler<P: RPCProtocol, TSubstream> where
	TSubstream: AsyncRead + AsyncWrite
{
	/// The upgrade for inbound substreams.
	listen_protocol: SubstreamProtocol<P>,

	/// If `Some`, something bad happened and we should shut down the handler with an error.
	pending_error: Option<ProtocolsHandlerUpgrErr<RPCError>>,

    /// Queue of events to produce in `poll()`.
    events_out: SmallVec<[RPCEvent<P::Request, P::Response>; 4]>,

    /// Queue of outbound substreams to open.
    dial_queue: SmallVec<[RPCEvent<P::Request, P::Response>; 4]>,

    /// Current number of concurrent outbound substreams being opened.
    dial_negotiated: u32,

    /// Map of current substreams awaiting a response to an RPC request.
    waiting_substreams: FnvHashMap<RequestId, WaitingResponse<P, TSubstream>>,

    /// List of outbound substreams that need to be driven to completion.
    substreams: Vec<SubstreamState<P, TSubstream>>,

    /// Sequential Id for waiting substreams.
    current_substream_id: RequestId,

    /// Maximum number of concurrent outbound substreams being opened. Value is never modified.
    max_dial_negotiated: u32,

    /// Value to return from `connection_keep_alive`.
    keep_alive: KeepAlive,

    /// After the given duration has elapsed, an inactive connection will shutdown.
    inactive_timeout: Duration,

	_marker: PhantomData<TSubstream>,
}

/// An outbound substream is waiting a response from the user.
struct WaitingResponse<P: RPCProtocol, TSubstream> {
    /// The framed negotiated substream.
    substream: InboundFramed<P, TSubstream>,
    /// The time when the substream is closed.
    timeout: Instant,
}

/// State of an outbound substream. Either waiting for a response, or in the process of sending.
pub enum SubstreamState<P: RPCProtocol, TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    /// A response has been sent, pending writing and flush.
    ResponsePendingSend {
        substream: futures::sink::Send<InboundFramed<P, TSubstream>>,
    },
    /// A request has been sent, and we are awaiting a response. This future is driven in the
    /// handler because GOODBYE requests can be handled and responses dropped instantly.
    RequestPendingResponse {
        /// The framed negotiated substream.
        substream: OutboundFramed<P, TSubstream>,
        /// Keeps track of the request id and the request to permit forming advanced responses which require
        /// data from the request.
        rpc_event: RPCEvent<P::Request, P::Response>,
        /// The time  when the substream is closed.
        timeout: Instant,
    },
}

impl<P, TSubstream> RPCHandler<P, TSubstream> where
	P: RPCProtocol,
	TSubstream: AsyncRead + AsyncWrite,
{
	pub fn new(
        listen_protocol: SubstreamProtocol<P>,
        inactive_timeout: Duration,
    ) -> Self {
        RPCHandler {
            listen_protocol,
            pending_error: None,
            events_out: SmallVec::new(),
            dial_queue: SmallVec::new(),
            dial_negotiated: 0,
            waiting_substreams: FnvHashMap::default(),
            substreams: Vec::new(),
            current_substream_id: 1,
            max_dial_negotiated: 8,
            keep_alive: KeepAlive::Yes,
            inactive_timeout,
            _marker: PhantomData,
        }
    }

    /// Returns the number of pending requests.
    pub fn pending_requests(&self) -> u32 {
        self.dial_negotiated + self.dial_queue.len() as u32
    }

    /// Returns a reference to the listen protocol configuration.
    ///
    /// > **Note**: If you modify the protocol, modifications will only applies to future inbound
    /// >           substreams, not the ones already being negotiated.
    pub fn listen_protocol_ref(&self) -> &SubstreamProtocol<P> {
        &self.listen_protocol
    }

    /// Returns a mutable reference to the listen protocol configuration.
    ///
    /// > **Note**: If you modify the protocol, modifications will only applies to future inbound
    /// >           substreams, not the ones already being negotiated.
    pub fn listen_protocol_mut(&mut self) -> &mut SubstreamProtocol<P> {
        &mut self.listen_protocol
    }

    /// Opens an outbound substream with a request.
    pub fn send_request(&mut self, rpc_event: RPCEvent<P::Request, P::Response>) {
        self.keep_alive = KeepAlive::Yes;

        self.dial_queue.push(rpc_event);
    }
}

impl<P, TSubstream> Default for RPCHandler<P, TSubstream> where
	P: RPCProtocol + Default,
	TSubstream: AsyncRead + AsyncWrite,
{
	fn default() -> Self {
		RPCHandler::new(SubstreamProtocol::new(P::default()), Duration::from_secs(30))
	}
}

// impl<P, TSubstream> ProtocolsHandler for RPCHandler<P, TSubstream> where
// 	P: RPCProtocol,
// 	TSubstream: AsyncRead + AsyncWrite,
// {
// 	type InEvent = RPCEvent<P::Request, P::Response>;
// 	type OutEvent = RPCEvent<P::Request, P::Response>;
// 	type Error = ProtocolsHandlerUpgrErr<RPCError>;
// 	type Substream = TSubstream;
// 	type InboundProtocol = RPCProtocol;
// }
