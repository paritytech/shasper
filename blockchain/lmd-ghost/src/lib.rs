pub mod archive;
pub mod pruning;

use blockchain::traits::{Block, BlockExecutor};
use core::hash::Hash;

pub trait JustifiableExecutor: BlockExecutor {
	type ValidatorIndex: Eq + Hash;

	fn justified_active_validators(
		&self,
		state: &mut Self::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Result<Vec<Self::ValidatorIndex>, Self::Error>;
	fn justified_block_id(
		&self,
		state: &mut Self::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Result<<Self::Block as Block>::Identifier, Self::Error>;
	fn votes(
		&self,
		block: &Self::Block,
		state: &mut Self::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Result<Vec<(Self::ValidatorIndex, <Self::Block as Block>::Identifier)>, Self::Error>;
}
