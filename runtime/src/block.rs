use runtime_primitives;

use header::Header;
use extrinsic::Extrinsic;

pub type Block = runtime_primitives::generic::Block<Header, Extrinsic>;

pub trait BlockExt {

}

impl BlockExt for Block {

}
