use beacon::{H256, KeccakHasher, BeaconState, BeaconBlock, Error as BeaconError, NoVerificationConfig};
use blockchain::traits::{Block as BlockT, ExecuteContext, ImportContext, BlockExecutor, ExternalitiesOf};
use ssz::Hashable;

#[derive(Eq, PartialEq, Clone)]
pub struct Block(BeaconBlock);

impl BlockT for Block {
	type Identifier = H256;

	fn id(&self) -> H256 {
		Hashable::<KeccakHasher>::hash(&self.0)
	}

	fn parent_id(&self) -> Option<H256> {
		if self.0.previous_block_root == H256::default() {
			None
		} else {
			Some(self.0.previous_block_root)
		}
	}
}

pub trait StateExternalities {
	fn state(&mut self) -> &mut BeaconState;
}

impl StateExternalities for BeaconState {
	fn state(&mut self) -> &mut BeaconState {
		self
	}
}

pub struct Context;

impl ExecuteContext for Context {
	type Block = Block;
	type Externalities = dyn StateExternalities + 'static;
}

impl ImportContext for Context {
	type Auxiliary = ();
}

#[derive(Debug)]
pub enum Error {
	Backend(Box<std::error::Error>),
	Beacon(BeaconError),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error { }

pub struct Executor;

impl BlockExecutor for Executor {
	type Error = Error;
	type Context = Context;

	fn execute_block(
		&self,
		block: &Block,
		state: &mut ExternalitiesOf<Context>,
	) -> Result<(), Error> {
		let config = NoVerificationConfig::full();

		beacon::execute_block(&block.0, state.state(), &config)
			.map_err(|e| Error::Beacon(e))
	}
}
