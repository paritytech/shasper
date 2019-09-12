use core::cmp::Ordering;
use ssz::{Codec, Decode, Encode};
use beacon::primitives::{Epoch, H256, Slot, Version};

/// The HELLO request/response handshake message.
#[derive(Codec, Encode, Decode, Clone, Debug, Eq, PartialEq)]
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

impl PartialOrd for HelloMessage {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for HelloMessage {
	fn cmp(&self, other: &Self) -> Ordering {
		self.head_slot.cmp(&other.head_slot)
	}
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
