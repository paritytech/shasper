use beacon::{H256, KeccakHasher, BeaconState, BeaconBlock, Error as BeaconError, NoVerificationConfig, Inherent, Transaction, Signature};
use blockchain::traits::{Block as BlockT, ExecuteContext, ImportContext, BlockExecutor, ExternalitiesOf, BlockOf, BuilderExecutor};
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

#[derive(Clone, Copy)]
pub enum Status {
	ApplyInherent,
	ApplyTransaction,
}

pub trait StateExternalities {
	fn state(&mut self) -> &mut BeaconState;
	fn status(&self) -> Status;
	fn set_status(&mut self, status: Status);
}

pub struct State {
	state: BeaconState,
	status: Status,
}

impl StateExternalities for State {
	fn state(&mut self) -> &mut BeaconState {
		&mut self.state
	}

	fn status(&self) -> Status {
		self.status
	}

	fn set_status(&mut self, status: Status) {
		self.status = status;
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
	InvalidStatus,
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

pub enum Extrinsic {
	Inherent(Inherent),
	Transaction(Transaction),
}

impl BuilderExecutor for Executor {
	type Error = Error;
	type Context = Context;
	type Extrinsic = Extrinsic;

	fn initialize_block(
		&self,
		block: &mut BlockOf<Self::Context>,
		_state: &mut ExternalitiesOf<Self::Context>,
	) -> Result<(), Self::Error> {
		block.0.signature = Signature::default();
		Ok(())
	}

	fn apply_extrinsic(
		&self,
		block: &mut BlockOf<Self::Context>,
		extrinsic: Self::Extrinsic,
		state: &mut ExternalitiesOf<Self::Context>,
	) -> Result<(), Self::Error> {
		let config = NoVerificationConfig::full();

		match state.status() {
			Status::ApplyInherent => {
				match extrinsic {
					Extrinsic::Inherent(inherent) => {
						beacon::initialize_block(&mut block.0, state.state(), inherent, &config)
							.map_err(|e| Error::Beacon(e))?;
					},
					Extrinsic::Transaction(_) => {
						return Err(Error::InvalidStatus)
					},
				}
				state.set_status(Status::ApplyTransaction);
			},
			Status::ApplyTransaction => {
				match extrinsic {
					Extrinsic::Inherent(_) => {
						return Err(Error::InvalidStatus)
					},
					Extrinsic::Transaction(transaction) => {
						beacon::apply_transaction(&mut block.0, state.state(), transaction, &config)
							.map_err(|e| Error::Beacon(e))?;
					},
				}
			},
		}

		Ok(())
	}

	fn finalize_block(
		&self,
		block: &mut BlockOf<Self::Context>,
		state: &mut ExternalitiesOf<Self::Context>,
	) -> Result<(), Self::Error> {
		let config = NoVerificationConfig::full();

		beacon::finalize_block(&mut block.0, state.state(), &config)
			.map_err(|e| Error::Beacon(e))?;

		Ok(())
	}
}
