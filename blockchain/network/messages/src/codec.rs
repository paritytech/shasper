use core::marker::PhantomData;
use tokio::codec::{Encoder, Decoder};
use bytes::{BufMut, BytesMut};
use beacon::Config;
use ssz::{Encode, Decode};
use crate::{RPCType, RPCRequest, RPCResponse};

pub struct InboundCodec<C: Config> {
	typ: RPCType,
	_marker: PhantomData<C>,
}

impl<C: Config> Encoder for InboundCodec<C> {
	type Item = RPCResponse<C>;
	type Error = ssz::Error;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
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
		Ok(Some(match self.typ {
			RPCType::Hello => RPCRequest::Hello(Decode::decode(&src)?),
			RPCType::Goodbye => RPCRequest::Goodbye(Decode::decode(&src)?),
			RPCType::BeaconBlocks => RPCRequest::BeaconBlocks(Decode::decode(&src)?),
			RPCType::RecentBeaconBlocks => RPCRequest::RecentBeaconBlocks(Decode::decode(&src)?),
		}))
	}
}

pub struct OutboundCodec<C: Config> {
	typ: RPCType,
	_marker: PhantomData<C>,
}

impl<C: Config> Encoder for OutboundCodec<C> {
	type Item = RPCRequest;
	type Error = ssz::Error;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		match (self.typ, item) {
			(RPCType::Hello, RPCRequest::Hello(item)) => dst.put(&item.encode()[..]),
			(RPCType::Goodbye, RPCRequest::Goodbye(item)) => dst.put(&item.encode()[..]),
			(RPCType::BeaconBlocks, RPCRequest::BeaconBlocks(item)) => dst.put(&item.encode()[..]),
			(RPCType::RecentBeaconBlocks, RPCRequest::RecentBeaconBlocks(item)) =>
				dst.put(&item.encode()[..]),
			_ => return Err(ssz::Error::Other),
		}

		Ok(())
	}
}

impl<C: Config> Decoder for OutboundCodec<C> {
	type Item = RPCResponse<C>;
	type Error = ssz::Error;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		if src.is_empty() {
			return Err(ssz::Error::Other)
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
