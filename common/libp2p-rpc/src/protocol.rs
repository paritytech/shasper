use core::time::Duration;
use tokio::codec::{Encoder, Decoder, Framed};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::timer::timeout;
use tokio::util::FutureExt;
use tokio_io_timeout::TimeoutStream;
use libp2p::{InboundUpgrade, OutboundUpgrade};
use libp2p::core::upgrade::{Negotiated, UpgradeInfo};
use futures::{
	future::{self, Future, FutureResult}, stream::{self, Stream},
	sink::{self, Sink}
};
use crate::{RPCError, RPCRequest};

/// Time allowed for the first byte of a request to arrive before we time out (Time To First Byte).
const TTFB_TIMEOUT: u64 = 5;
/// The number of seconds to wait for the first bytes of a request once a protocol has been
/// established before the stream is terminated.
const REQUEST_TIMEOUT: u64 = 15;

pub trait RPCProtocol: UpgradeInfo {
	type Request: RPCRequest + Clone;
	type Response: Clone;

	type InboundCodec: Encoder<Item=Self::Response> + Decoder<Item=Self::Request>;
	fn inbound_codec(&self, protocol: <Self as UpgradeInfo>::Info) -> Self::InboundCodec;

	type OutboundCodec: Encoder<Item=Self::Request> + Decoder<Item=Self::Response>;
	fn outbound_codec(&self, protocol: <Self as UpgradeInfo>::Info) -> Self::OutboundCodec;
}

pub type InboundFramed<P, TSocket> = Framed<TimeoutStream<Negotiated<TSocket>>,
											<P as RPCProtocol>::InboundCodec>;
pub type InboundOutput<P, TSocket> = (<P as RPCProtocol>::Request, InboundFramed<P, TSocket>);

#[derive(Default, Clone)]
pub struct RPCInbound<P>(pub P);

impl<P: RPCProtocol> UpgradeInfo for RPCInbound<P> {
	type Info = P::Info;
	type InfoIter = P::InfoIter;

	fn protocol_info(&self) -> Self::InfoIter {
		self.0.protocol_info()
	}
}

type RPCInboundFnAndThen<P, TSocket> = fn(
    (Option<<P as RPCProtocol>::Request>, InboundFramed<P, TSocket>),
) -> FutureResult<InboundOutput<P, TSocket>, RPCError>;
type RPCInboundFnMapErr<P, TSocket> = fn(timeout::Error<(<<P as RPCProtocol>::InboundCodec as Decoder>::Error, InboundFramed<P, TSocket>)>) -> RPCError;

impl<P, TSocket> InboundUpgrade<TSocket> for RPCInbound<P> where
	P: RPCProtocol,
	TSocket: AsyncRead + AsyncWrite,
{
	type Output = InboundOutput<P, TSocket>;
	type Error = RPCError;

	type Future = future::AndThen<
        future::MapErr<
            timeout::Timeout<stream::StreamFuture<InboundFramed<P, TSocket>>>,
            RPCInboundFnMapErr<P, TSocket>,
        >,
        FutureResult<InboundOutput<P, TSocket>, RPCError>,
        RPCInboundFnAndThen<P, TSocket>,
    >;

	fn upgrade_inbound(
		self,
        socket: Negotiated<TSocket>,
        protocol: P::Info,
	) -> Self::Future {
		let codec = self.0.inbound_codec(protocol);
		let mut timed_socket = TimeoutStream::new(socket);
		timed_socket.set_read_timeout(Some(Duration::from_secs(TTFB_TIMEOUT)));
        Framed::new(timed_socket, codec)
            .into_future()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT))
            .map_err(RPCError::from as RPCInboundFnMapErr<P, TSocket>)
            .and_then({
                |(req, stream)| match req {
                    Some(req) => futures::future::ok((req, stream)),
                    None => futures::future::err(RPCError::Custom(
                        "Stream terminated early".into(),
                    )),
                }
            } as RPCInboundFnAndThen<P, TSocket>)
	}
}

pub type OutboundFramed<P, TSocket> = Framed<Negotiated<TSocket>,
											 <P as RPCProtocol>::OutboundCodec>;

pub struct RPCOutbound<P: RPCProtocol>(pub P::Request, pub P);

impl<P: RPCProtocol> UpgradeInfo for RPCOutbound<P> {
	type Info = P::Info;
	type InfoIter = P::InfoIter;

	fn protocol_info(&self) -> Self::InfoIter {
		self.1.protocol_info()
	}
}

type RPCOutboundFnMapErr<P> = fn(<<P as RPCProtocol>::OutboundCodec as Encoder>::Error) -> RPCError;

impl<P, TSocket> OutboundUpgrade<TSocket> for RPCOutbound<P> where
	P: RPCProtocol,
	TSocket: AsyncRead + AsyncWrite,
{
	type Output = OutboundFramed<P, TSocket>;
	type Error = RPCError;
	type Future = future::MapErr<
		sink::Send<OutboundFramed<P, TSocket>>,
		RPCOutboundFnMapErr<P>,
	>;

	fn upgrade_outbound(
        self,
        socket: Negotiated<TSocket>,
        protocol: P::Info,
    ) -> Self::Future {
		let codec = self.1.outbound_codec(protocol);
		Framed::new(socket, codec).send(self.0)
			.map_err(|_| RPCError::Codec)
	}
}
