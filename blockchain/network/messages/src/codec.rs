use core::marker::PhantomData;
use tokio::codec::{Encoder, Decoder};
use bytes::{BufMut, Bytes, BytesMut};
use beacon::Config;
use ssz::{Encode, Decode};
use log::*;
use unsigned_varint::codec::UviBytes;
use crate::{RPCType, RPCRequest, RPCResponse};

pub struct InboundCodec<C: Config> {
	typ: RPCType,
	uvi: UviBytes,
	_marker: PhantomData<C>,
}

impl<C: Config> InboundCodec<C> {
	pub fn new(typ: RPCType) -> Self {
		Self { typ, uvi: UviBytes::default(), _marker: PhantomData }
	}
}

impl<C: Config> Encoder for InboundCodec<C> {
	type Item = RPCResponse<C>;
	type Error = ssz::Error;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		trace!("inbound encode item: {:?}", item);

		match item {
			RPCResponse::Unknown(id, _) => dst.put(id),
			_ => dst.put(0u8),
		}

		let bytes = match item {
			RPCResponse::Hello(item) => item.encode(),
			RPCResponse::BeaconBlocks(item) => item.encode(),
			RPCResponse::RecentBeaconBlocks(item) => item.encode(),
			RPCResponse::Unknown(_, value) => value,
		};

		self.uvi.encode(Bytes::from(bytes), dst)?;

		Ok(())
	}
}

impl<C: Config> Decoder for InboundCodec<C> {
	type Item = RPCRequest;
	type Error = ssz::Error;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		trace!("inbound decode src len: {}", src.len());

		match self.uvi.decode(src)? {
			Some(bytes) => Ok(Some(match self.typ {
				RPCType::Hello => RPCRequest::Hello(Decode::decode(&bytes[..])?),
				RPCType::Goodbye => RPCRequest::Goodbye(Decode::decode(&bytes[..])?),
				RPCType::BeaconBlocks => RPCRequest::BeaconBlocks(Decode::decode(&bytes[..])?),
				RPCType::RecentBeaconBlocks =>
					RPCRequest::RecentBeaconBlocks(Decode::decode(&bytes[..])?),
			})),
			None => Ok(None),
		}
	}
}

pub struct OutboundCodec<C: Config> {
	typ: RPCType,
	uvi: UviBytes,
	_marker: PhantomData<C>,
}

impl<C: Config> OutboundCodec<C> {
	pub fn new(typ: RPCType) -> Self {
		Self { typ, uvi: UviBytes::default(), _marker: PhantomData }
	}
}

impl<C: Config> Encoder for OutboundCodec<C> {
	type Item = RPCRequest;
	type Error = ssz::Error;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		trace!("outbound encode type: {:?}, item: {:?}", self.typ, item);

		let bytes = match (self.typ, item) {
			(RPCType::Hello, RPCRequest::Hello(item)) => item.encode(),
			(RPCType::Goodbye, RPCRequest::Goodbye(item)) => item.encode(),
			(RPCType::BeaconBlocks, RPCRequest::BeaconBlocks(item)) => item.encode(),
			(RPCType::RecentBeaconBlocks, RPCRequest::RecentBeaconBlocks(item)) => item.encode(),
			_ => return Err(ssz::Error::Other("outbound codec invalid type")),
		};

		self.uvi.encode(Bytes::from(bytes), dst)?;

		Ok(())
	}
}

impl<C: Config> Decoder for OutboundCodec<C> {
	type Item = RPCResponse<C>;
	type Error = ssz::Error;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		trace!("outbound decode src len: {}", src.len());

		if src.is_empty() {
			return Ok(None)
		}

		let code = src.split_to(1)[0];
		let bytes = match self.uvi.decode(src)? {
			Some(bytes) => bytes,
			None => return Ok(None),
		};

		Ok(Some(if code == 0 {
			match self.typ {
				RPCType::Hello => RPCResponse::Hello(Decode::decode(&bytes[..])?),
				RPCType::BeaconBlocks => RPCResponse::BeaconBlocks(Decode::decode(&bytes[..])?),
				RPCType::RecentBeaconBlocks =>
					RPCResponse::RecentBeaconBlocks(Decode::decode(&bytes[..])?),
				RPCType::Goodbye => RPCResponse::Unknown(code, bytes.to_vec()),
			}
		} else {
			RPCResponse::Unknown(code, bytes.to_vec())
		}))
	}
}
