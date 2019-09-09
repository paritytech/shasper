use core::marker::PhantomData;
use std::time::{Instant, Duration};
use smallvec::SmallVec;
use fnv::FnvHashMap;
use libp2p::{InboundUpgrade, OutboundUpgrade};
use libp2p::swarm::protocols_handler::{
	SubstreamProtocol, ProtocolsHandlerUpgrErr, KeepAlive, ProtocolsHandlerEvent,
	ProtocolsHandler,
};
use futures::prelude::*;
use tokio_io::{AsyncRead, AsyncWrite};
use crate::{RPCEvent, RPCRequest, RPCError, RequestId};
use crate::protocol::{InboundFramed, OutboundFramed, RPCProtocol, RPCInbound, RPCOutbound};

/// The time (in seconds) before a substream that is awaiting a response from the user times out.
pub const RESPONSE_TIMEOUT: u64 = 10;

/// Implementation of `ProtocolsHandler` for the RPC protocol.
pub struct RPCHandler<P: RPCProtocol, TSubstream> where
	TSubstream: AsyncRead + AsyncWrite
{
	/// The upgrade for inbound substreams.
	listen_protocol: SubstreamProtocol<RPCInbound<P>>,

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

	/// The protocol handler.
	protocol: P,

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
	P: RPCProtocol + Clone,
	TSubstream: AsyncRead + AsyncWrite,
{
	pub fn new(
		protocol: P,
        inactive_timeout: Duration,
    ) -> Self {
        RPCHandler {
            listen_protocol: SubstreamProtocol::new(RPCInbound(protocol.clone())),
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
			protocol,
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
    pub fn listen_protocol_ref(&self) -> &SubstreamProtocol<RPCInbound<P>> {
        &self.listen_protocol
    }

    /// Returns a mutable reference to the listen protocol configuration.
    ///
    /// > **Note**: If you modify the protocol, modifications will only applies to future inbound
    /// >           substreams, not the ones already being negotiated.
    pub fn listen_protocol_mut(&mut self) -> &mut SubstreamProtocol<RPCInbound<P>> {
        &mut self.listen_protocol
    }

    /// Opens an outbound substream with a request.
    pub fn send_request(&mut self, rpc_event: RPCEvent<P::Request, P::Response>) {
        self.keep_alive = KeepAlive::Yes;

        self.dial_queue.push(rpc_event);
    }
}

impl<P, TSubstream> Default for RPCHandler<P, TSubstream> where
	P: RPCProtocol + Default + Clone,
	TSubstream: AsyncRead + AsyncWrite,
{
	fn default() -> Self {
		RPCHandler::new(P::default(), Duration::from_secs(30))
	}
}

impl<P, TSubstream> ProtocolsHandler for RPCHandler<P, TSubstream> where
	P: RPCProtocol + Clone,
	TSubstream: AsyncRead + AsyncWrite,
{
	type InEvent = RPCEvent<P::Request, P::Response>;
	type OutEvent = RPCEvent<P::Request, P::Response>;
	type Error = ProtocolsHandlerUpgrErr<RPCError>;
	type Substream = TSubstream;
	type InboundProtocol = RPCInbound<P>;
	type OutboundProtocol = RPCOutbound<P>;
	type OutboundOpenInfo = RPCEvent<P::Request, P::Response>;

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol> {
        self.listen_protocol.clone()
    }

    fn inject_fully_negotiated_inbound(
        &mut self,
        out: <RPCInbound<P> as InboundUpgrade<TSubstream>>::Output,
    ) {
        let (req, substream) = out;
        // drop the stream and return a 0 id for goodbye "requests"
        if req.is_goodbye() {
            self.events_out.push(RPCEvent::Request(0, req));
            return;
        }

        // New inbound request. Store the stream and tag the output.
        let awaiting_stream = WaitingResponse {
            substream,
            timeout: Instant::now() + Duration::from_secs(RESPONSE_TIMEOUT),
        };
        self.waiting_substreams
            .insert(self.current_substream_id, awaiting_stream);

        self.events_out
            .push(RPCEvent::Request(self.current_substream_id, req));
        self.current_substream_id += 1;
    }

    fn inject_fully_negotiated_outbound(
        &mut self,
        out: <RPCOutbound<P> as OutboundUpgrade<TSubstream>>::Output,
        rpc_event: Self::OutboundOpenInfo,
    ) {
        self.dial_negotiated -= 1;

        if self.dial_negotiated == 0
            && self.dial_queue.is_empty()
            && self.waiting_substreams.is_empty()
        {
            self.keep_alive = KeepAlive::Until(Instant::now() + self.inactive_timeout);
        } else {
            self.keep_alive = KeepAlive::Yes;
        }

        // add the stream to substreams if we expect a response, otherwise drop the stream.
        if let RPCEvent::Request(id, req) = rpc_event {
            if req.expect_response() {
                let awaiting_stream = SubstreamState::RequestPendingResponse {
                    substream: out,
                    rpc_event: RPCEvent::Request(id, req),
                    timeout: Instant::now() + Duration::from_secs(RESPONSE_TIMEOUT),
                };

                self.substreams.push(awaiting_stream);
            }
        }
    }

    // Note: If the substream has closed due to inactivity, or the substream is in the
    // wrong state a response will fail silently.
    fn inject_event(&mut self, rpc_event: Self::InEvent) {
        match rpc_event {
            RPCEvent::Request(_, _) => self.send_request(rpc_event),
            RPCEvent::Response(rpc_id, res) => {
                // check if the stream matching the response still exists
                if let Some(waiting_stream) = self.waiting_substreams.remove(&rpc_id) {
                    // only send one response per stream. This must be in the waiting state.
                    self.substreams.push(SubstreamState::ResponsePendingSend {
                        substream: waiting_stream.substream.send(res),
                    });
                }
            }
            RPCEvent::Error(_, _) => {}
        }
    }

    fn inject_dial_upgrade_error(
        &mut self,
        _: Self::OutboundOpenInfo,
        error: ProtocolsHandlerUpgrErr<
            <Self::OutboundProtocol as OutboundUpgrade<Self::Substream>>::Error,
        >,
    ) {
        if self.pending_error.is_none() {
            self.pending_error = Some(error);
        }
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        self.keep_alive
    }

    fn poll(
        &mut self,
    ) -> Poll<
        ProtocolsHandlerEvent<Self::OutboundProtocol, Self::OutboundOpenInfo, Self::OutEvent>,
        Self::Error,
    > {
        if let Some(err) = self.pending_error.take() {
            // Returning an error here will result in dropping any peer that doesn't support any of
            // the RPC protocols. For our immediate purposes we permit this and simply log that an
            // upgrade was not supported.
            // TODO: Add a logger to the handler for trace output.
            dbg!(&err);
        }

        // return any events that need to be reported
        if !self.events_out.is_empty() {
            return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                self.events_out.remove(0),
            )));
        } else {
            self.events_out.shrink_to_fit();
        }

        // remove any streams that have expired
        self.waiting_substreams
            .retain(|_k, waiting_stream| Instant::now() <= waiting_stream.timeout);

        // drive streams that need to be processed
        for n in (0..self.substreams.len()).rev() {
            let stream = self.substreams.swap_remove(n);
            match stream {
                SubstreamState::ResponsePendingSend { mut substream } => {
                    match substream.poll() {
                        Ok(Async::Ready(_substream)) => {} // sent and flushed
                        Ok(Async::NotReady) => {
                            self.substreams
                                .push(SubstreamState::ResponsePendingSend { substream });
                        }
                        Err(e) => {
                            return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                                RPCEvent::Error(0, RPCError::Codec),
                            )))
                        }
                    }
                }
                SubstreamState::RequestPendingResponse {
                    mut substream,
                    rpc_event,
                    timeout,
                } => match substream.poll() {
                    Ok(Async::Ready(response)) => {
                        if let Some(response) = response {
                            return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                                RPCEvent::Response(rpc_event.id(), response),
                            )));
                        } else {
                            // stream closed early or nothing was sent
                            return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                                RPCEvent::Error(
                                    rpc_event.id(),
                                    RPCError::Custom("Stream closed early. Empty response".into()),
                                ),
                            )));
                        }
                    }
                    Ok(Async::NotReady) => {
                        if Instant::now() < timeout {
                            self.substreams
                                .push(SubstreamState::RequestPendingResponse {
                                    substream,
                                    rpc_event,
                                    timeout,
                                });
                        }
                    }
                    Err(e) => {
                        return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                            RPCEvent::Error(rpc_event.id(), RPCError::Codec),
                        )))
                    }
                },
            }
        }

        // establish outbound substreams
        if !self.dial_queue.is_empty() {
            if self.dial_negotiated < self.max_dial_negotiated {
                self.dial_negotiated += 1;
                let rpc_event = self.dial_queue.remove(0);
                if let RPCEvent::Request(id, req) = rpc_event {
                    return Ok(Async::Ready(
                        ProtocolsHandlerEvent::OutboundSubstreamRequest {
                            protocol: SubstreamProtocol::new(
								RPCOutbound(req.clone(), self.protocol.clone())
							),
                            info: RPCEvent::Request(id, req),
                        },
                    ));
                }
            }
        } else {
            self.dial_queue.shrink_to_fit();
        }
        Ok(Async::NotReady)
    }
}
