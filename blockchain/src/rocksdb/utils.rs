use blockchain::traits::Block;
use parity_codec::{Encode, Decode};
use rocksdb::DB;
use super::Error;

pub const COLUMN_BLOCKS: &str = "blocks";
pub const COLUMN_CANON_DEPTH_MAPPINGS: &str = "canon_depth_mappings";
pub const COLUMN_AUXILIARIES: &str = "auxiliaries";
pub const COLUMN_INFO: &str = "info";
pub const KEY_HEAD: &str = "head";
pub const KEY_GENESIS: &str = "genesis";

#[derive(Encode, Decode)]
pub struct BlockData<B: Block, S> {
	pub block: B,
	pub state: S,
	pub depth: u64,
	pub children: Vec<B::Identifier>,
	pub is_canon: bool,
}

pub fn fetch_block_data<B: Block, S>(
	db:
	&DB, id: &B::Identifier
) -> Result<Option<BlockData<B, S>>, Error> where
	B::Identifier: Encode + Decode,
	B: Decode,
	S: Decode
{
	let cf = db.cf_handle(COLUMN_BLOCKS).ok_or(Error::Corrupted)?;
	let raw = match db.get_cf(cf, id.encode())? {
		Some(raw) => raw,
		None => return Ok(None),
	};
	Ok(Some(BlockData::decode(&mut raw.as_ref()).ok_or(Error::Corrupted)?))
}

pub fn fetch_head<I: Decode>(db: &DB) -> Result<Option<I>, Error> {
	let cf = db.cf_handle(COLUMN_INFO).ok_or(Error::Corrupted)?;
	let raw = match db.get_cf(cf, KEY_HEAD.encode())? {
		Some(raw) => raw,
		None => return Ok(None),
	};
	Ok(Some(I::decode(&mut raw.as_ref()).ok_or(Error::Corrupted)?))
}

pub fn fetch_genesis<I: Decode>(db: &DB) -> Result<Option<I>, Error> {
	let cf = db.cf_handle(COLUMN_INFO).ok_or(Error::Corrupted)?;
	let raw = match db.get_cf(cf, KEY_GENESIS.encode())? {
		Some(raw) => raw,
		None => return Ok(None),
	};
	Ok(Some(I::decode(&mut raw.as_ref()).ok_or(Error::Corrupted)?))
}
