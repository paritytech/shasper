pub mod archive;
pub mod pruning;

use blockchain::traits::Block;
use core::hash::Hash;

pub trait LmdGhostExternalities<BI, VI> {
	fn justified_active_validators(&self) -> Vec<VI>;
	fn justified_block_id(&self) -> BI;
}

pub trait VotedBlock: Block {
	type ValidatorIdentifier: Eq + Hash;

	fn votes(&self) -> Vec<(Self::ValidatorIdentifier, Self::Identifier)>;
}
