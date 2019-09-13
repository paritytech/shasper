use core::marker::PhantomData;
use tokio::codec::{Encoder, Decoder};
use bytes::{BufMut, BytesMut};
use beacon::Config;
use ssz::{Encode, Decode};
use log::*;
use crate::{RPCType, RPCRequest, RPCResponse};

pub struct InboundCodec<C: Config> {
	typ: RPCType,
	_marker: PhantomData<C>,
}

impl<C: Config> InboundCodec<C> {
	pub fn new(typ: RPCType) -> Self {
		Self { typ, _marker: PhantomData }
	}
}

impl<C: Config> Encoder for InboundCodec<C> {
	type Item = RPCResponse<C>;
	type Error = ssz::Error;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		trace!("inbound encode item: {:?}", item);

		match item {
			RPCResponse::Hello(item) => {
				dst.put(self.typ as u8);
				dst.put(&item.encode()[..])
			},
			RPCResponse::BeaconBlocks(item) => {
				dst.put(self.typ as u8);
				dst.put(&item.encode()[..])
			},
			RPCResponse::RecentBeaconBlocks(item) => {
				dst.put(self.typ as u8);
				dst.put(&item.encode()[..])
			},
			RPCResponse::Unknown(id, value) => {
				dst.put(id);
				dst.put(&value[..])
			},
		}

		Ok(())
	}
}

impl<C: Config> Decoder for InboundCodec<C> {
	type Item = RPCRequest;
	type Error = ssz::Error;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		trace!("inbound decode src len: {}", src.len());

		Ok(Some(match self.typ {
			RPCType::Hello => RPCRequest::Hello(Decode::decode(&src)?),
			RPCType::Goodbye => RPCRequest::Goodbye(Decode::decode(&src)?),
			RPCType::BeaconBlocks => {
				let bytes: Vec<u8> = Decode::decode(&src)?;
				RPCRequest::BeaconBlocks(Decode::decode(&bytes[..])?)
			},
			RPCType::RecentBeaconBlocks => {
				let bytes: Vec<u8> = Decode::decode(&src)?;
				RPCRequest::RecentBeaconBlocks(Decode::decode(&bytes[..])?)
			},
		}))
	}
}

pub struct OutboundCodec<C: Config> {
	typ: RPCType,
	_marker: PhantomData<C>,
}

impl<C: Config> OutboundCodec<C> {
	pub fn new(typ: RPCType) -> Self {
		Self { typ, _marker: PhantomData }
	}
}

impl<C: Config> Encoder for OutboundCodec<C> {
	type Item = RPCRequest;
	type Error = ssz::Error;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		trace!("outbound encode type: {:?}, item: {:?}", self.typ, item);

		match (self.typ, item) {
			(RPCType::Hello, RPCRequest::Hello(item)) => dst.put(&item.encode()[..]),
			(RPCType::Goodbye, RPCRequest::Goodbye(item)) => dst.put(&item.encode()[..]),
			(RPCType::BeaconBlocks, RPCRequest::BeaconBlocks(item)) => {
				let bytes = item.encode();
				dst.put(&bytes.encode()[..])
			},
			(RPCType::RecentBeaconBlocks, RPCRequest::RecentBeaconBlocks(item)) => {
				let bytes = item.encode();
				dst.put(&bytes.encode()[..])
			},
			_ => return Err(ssz::Error::Other("outbound codec invalid type")),
		}

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

		Ok(Some(if self.typ as u8 == code {
			match self.typ {
				RPCType::Hello => RPCResponse::Hello(Decode::decode(&src)?),
				RPCType::BeaconBlocks => RPCResponse::BeaconBlocks(Decode::decode(&src)?),
				RPCType::RecentBeaconBlocks =>
					RPCResponse::RecentBeaconBlocks(Decode::decode(&src)?),
				RPCType::Goodbye => RPCResponse::Unknown(code, src.to_vec()),
			}
		} else {
			RPCResponse::Unknown(code, src.to_vec())
		}))
	}
}
