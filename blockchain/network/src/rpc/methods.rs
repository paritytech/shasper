// Copyright 2019 Parity Technologies (UK) Ltd.
// Copyright 2019 Sigma Prime.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.

//! Available RPC methods types and ids.

use ssz::{Codec, Decode, Encode};
use beacon_primitives::{Epoch, H256, Slot, Version};

/* Request/Response data structures for RPC methods */

/* Requests */

pub type RequestId = usize;

/// The HELLO request/response handshake message.
#[derive(Codec, Encode, Decode, Clone, Debug)]
pub struct HelloMessage {
    /// The fork version of the chain we are broadcasting.
    pub fork_version: Version,

    /// Latest finalized root.
    pub finalized_root: H256,

    /// Latest finalized epoch.
    pub finalized_epoch: Epoch,

    /// The latest block root.
    pub head_root: H256,

    /// The slot associated with the latest block root.
    pub head_slot: Slot,
}

/// The reason given for a `Goodbye` message.
///
/// Note: any unknown `u64::into(n)` will resolve to `Goodbye::Unknown` for any unknown `n`,
/// however `GoodbyeReason::Unknown.into()` will go into `0_u64`. Therefore de-serializing then
/// re-serializing may not return the same bytes.
#[derive(Debug, Clone, Copy)]
pub enum GoodbyeReason {
    /// This node has shutdown.
    ClientShutdown = 1,

    /// Incompatible networks.
    IrrelevantNetwork = 2,

    /// Error/fault in the RPC.
    Fault = 3,

    /// Unknown reason.
    Unknown = 0,
}

impl From<u64> for GoodbyeReason {
    fn from(id: u64) -> GoodbyeReason {
        match id {
            1 => GoodbyeReason::ClientShutdown,
            2 => GoodbyeReason::IrrelevantNetwork,
            3 => GoodbyeReason::Fault,
            _ => GoodbyeReason::Unknown,
        }
    }
}

impl Into<u64> for GoodbyeReason {
    fn into(self) -> u64 {
        self as u64
    }
}

impl ssz::Codec for GoodbyeReason {
	type Size = <u64 as ssz::Codec>::Size;
}

impl ssz::Encode for GoodbyeReason {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		let uint: u64 = (*self).into();
		uint.using_encoded(f)
	}
}

impl ssz::Decode for GoodbyeReason {
	fn decode(value: &[u8]) -> Result<Self, ssz::Error> {
		u64::decode(value).map(Into::into)
	}
}

/// Request a number of beacon block roots from a peer.
#[derive(Codec, Encode, Decode, Clone, Debug, PartialEq)]
pub struct BeaconBlocksRequest {
    /// The hash tree root of a block on the requested chain.
    pub head_block_root: H256,

    /// The starting slot to request blocks.
    pub start_slot: Slot,

    /// The number of blocks from the start slot.
    pub count: u64,

    /// The step increment to receive blocks.
    ///
    /// A value of 1 returns every block.
    /// A value of 2 returns every second block.
    /// A value of 3 returns every third block and so on.
    pub step: u64,
}

/// Request a number of beacon block bodies from a peer.
#[derive(Codec, Encode, Decode, Clone, Debug, PartialEq)]
pub struct RecentBeaconBlocksRequest {
    /// The list of beacon block bodies being requested.
    pub block_roots: Vec<H256>,
}

/* RPC Handling and Grouping */
// Collection of enums and structs used by the Codecs to encode/decode RPC messages

#[derive(Debug, Clone)]
pub enum RPCResponse {
    /// A HELLO message.
    Hello(HelloMessage),
    /// A response to a get BEACON_BLOCKS request.
    BeaconBlocks(Vec<u8>),
    /// A response to a get RECENT_BEACON_BLOCKS request.
    RecentBeaconBlocks(Vec<u8>),
}

#[derive(Debug)]
pub enum RPCErrorResponse {
    Success(RPCResponse),
    InvalidRequest(ErrorMessage),
    ServerError(ErrorMessage),
    Unknown(ErrorMessage),
}

impl RPCErrorResponse {
    /// Used to encode the response.
    pub fn as_u8(&self) -> u8 {
        match self {
            RPCErrorResponse::Success(_) => 0,
            RPCErrorResponse::InvalidRequest(_) => 1,
            RPCErrorResponse::ServerError(_) => 2,
            RPCErrorResponse::Unknown(_) => 255,
        }
    }

    /// Tells the codec whether to decode as an RPCResponse or an error.
    pub fn is_response(response_code: u8) -> bool {
        match response_code {
            0 => true,
            _ => false,
        }
    }

    /// Builds an RPCErrorResponse from a response code and an ErrorMessage
    pub fn from_error(response_code: u8, err: ErrorMessage) -> Self {
        match response_code {
            1 => RPCErrorResponse::InvalidRequest(err),
            2 => RPCErrorResponse::ServerError(err),
            _ => RPCErrorResponse::Unknown(err),
        }
    }
}

#[derive(Codec, Encode, Decode, Debug)]
pub struct ErrorMessage {
    /// The UTF-8 encoded Error message string.
    pub error_message: Vec<u8>,
}

impl ErrorMessage {
    pub fn as_string(&self) -> String {
        String::from_utf8(self.error_message.clone()).unwrap_or_else(|_| "".into())
    }
}