use primitives::H256;
use runtime_primitives::traits::BlakeTwo256;
use runtime_primitives;

use super::BlockNumber;

pub type DigestItem = runtime_primitives::generic::DigestItem<H256, ()>;
pub type Digest = runtime_primitives::generic::Digest<DigestItem>;
pub type Header = runtime_primitives::generic::Header<BlockNumber, BlakeTwo256, DigestItem>;
