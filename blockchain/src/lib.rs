use beacon::primitives::{H256, ValidatorId, Signature};
use beacon::types::{BeaconState, BeaconBlock, UnsealedBeaconBlock, BeaconBlockHeader};
use beacon::{Error as BeaconError, Config, Inherent, Transaction, BLSVerification};
use blockchain::traits::{Block as BlockT, BlockExecutor, AsExternalities};
use lmd_ghost::JustifiableExecutor;
use parity_codec::{Encode, Decode};
use ssz::Digestible;
use bls_aggregates as bls;

#[derive(Eq, PartialEq, Clone, Debug, Encode, Decode)]
pub struct Block(pub BeaconBlock);

impl BlockT for Block {
	type Identifier = H256;

	fn id(&self) -> H256 {
		let header = BeaconBlockHeader {
			slot: self.0.slot,
			previous_block_root: self.0.previous_block_root,
			state_root: self.0.state_root,
			block_body_root: if self.0.previous_block_root == H256::default() {
				H256::default()
			} else {
				H256::from_slice(
					Digestible::<sha2::Sha256>::hash(&self.0.body).as_slice()
				)
			},
			..Default::default()
		};

		H256::from_slice(
			Digestible::<sha2::Sha256>::truncated_hash(&header).as_slice()
		)
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

#[derive(Clone)]
pub struct State {
	state: BeaconState,
}

impl From<BeaconState> for State {
	fn from(state: BeaconState) -> Self {
		Self { state }
	}
}

impl Into<BeaconState> for State {
	fn into(self) -> BeaconState {
		self.state
	}
}

impl StateExternalities for State {
	fn state(&mut self) -> &mut BeaconState {
		&mut self.state
	}
}

impl AsExternalities<dyn StateExternalities> for State {
	fn as_externalities(&mut self) -> &mut (dyn StateExternalities + 'static) {
		self
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AMCLVerification;

impl BLSVerification for AMCLVerification {
	fn verify(pubkey: &ValidatorId, message: &H256, signature: &Signature, domain: u64) -> bool {
		let pubkey = match bls::PublicKey::from_bytes(&pubkey[..]) {
			Ok(value) => value,
			Err(_) => return false,
		};
		let signature = match bls::Signature::from_bytes(&signature[..]) {
			Ok(value) => value,
			Err(_) => return false,
		};
		signature.verify(&message[..], domain, &pubkey)
	}
	fn aggregate_pubkeys(pubkeys: &[ValidatorId]) -> ValidatorId {
		let mut aggregated = bls::AggregatePublicKey::new();
		for pubkey in pubkeys {
			let pubkey = match bls::PublicKey::from_bytes(&pubkey[..]) {
				Ok(value) => value,
				Err(_) => return ValidatorId::default(),
			};
			aggregated.add(&pubkey);
		}
		ValidatorId::from_slice(&aggregated.as_bytes()[..])
	}
	fn verify_multiple(pubkeys: &[ValidatorId], messages: &[H256], signature: &Signature, domain: u64) -> bool {
		let mut bls_messages = Vec::new();
		for message in messages {
			bls_messages.append(&mut (&message[..]).to_vec());
		}

		let bls_signature = match bls::AggregateSignature::from_bytes(&signature[..]) {
			Ok(value) => value,
			Err(_) => return false,
		};

		let mut bls_pubkeys = Vec::new();
		for pubkey in pubkeys {
			bls_pubkeys.push(match bls::AggregatePublicKey::from_bytes(&pubkey[..]) {
				Ok(value) => value,
				Err(_) => return false,
			});
		}

		bls_signature.verify_multiple(
			&bls_messages, domain, &bls_pubkeys.iter().collect::<Vec<_>>())
	}
}

#[derive(Debug)]
pub enum Error {
	Beacon(BeaconError),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error { }

impl From<BeaconError> for Error {
	fn from(error: BeaconError) -> Error {
		Error::Beacon(error)
	}
}

impl From<Error> for blockchain::chain::Error {
	fn from(error: Error) -> blockchain::chain::Error {
		blockchain::chain::Error::Executor(Box::new(error))
	}
}

#[derive(Clone)]
pub struct Executor<C: Config> {
	config: C,
}

impl<C: Config> Executor<C> {
	pub fn new(config: C) -> Self {
		Self { config }
	}

	pub fn proposer_index(
		&self,
		state: &mut <Self as BlockExecutor>::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Result<u64, Error> {
		Ok(beacon::beacon_proposer_index(state.state(), &self.config)?)
	}

	pub fn validator_pubkey(
		&self,
		index: u64,
		state: &mut <Self as BlockExecutor>::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Option<ValidatorId> {
		beacon::validator_pubkey(index, state.state(), &self.config)
	}

	pub fn validator_index(
		&self,
		pubkey: &ValidatorId,
		state: &mut <Self as BlockExecutor>::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Result<Option<u64>, Error> {
		Ok(beacon::validator_index(pubkey, state.state(), &self.config))
	}

	pub fn current_epoch(
		&self,
		state: &mut <Self as BlockExecutor>::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Result<u64, Error> {
		Ok(beacon::current_epoch(state.state(), &self.config))
	}

	pub fn domain(
		&self,
		state: &mut <Self as BlockExecutor>::Externalities, // FIXME: replace `&mut` with `&`.
		domain_type: u64,
		message_epoch: Option<u64>
	) -> Result<u64, Error> {
		Ok(beacon::domain(state.state(), domain_type, message_epoch, &self.config))
	}

	pub fn initialize_block(
		&self,
		state: &mut <Self as BlockExecutor>::Externalities,
		target_slot: u64,
	) -> Result<(), Error> {
		Ok(beacon::initialize_block(state.state(), target_slot, &self.config)?)
	}

	pub fn committee_assignment(
		&self,
		state: &mut <Self as BlockExecutor>::Externalities,
		epoch: u64,
		validator_id: u64,
	) -> Result<Option<beacon::CommitteeAssignment>, Error> {
		Ok(beacon::committee_assignment(epoch, validator_id, state.state(), &self.config)?)
	}

	pub fn apply_inherent(
		&self,
		parent_block: &Block,
		state: &mut <Self as BlockExecutor>::Externalities,
		inherent: Inherent,
	) -> Result<UnsealedBeaconBlock, Error> {
		Ok(beacon::apply_inherent(&parent_block.0, state.state(), inherent, &self.config)?)
	}

	pub fn apply_extrinsic(
		&self,
		block: &mut UnsealedBeaconBlock,
		extrinsic: Transaction,
		state: &mut <Self as BlockExecutor>::Externalities,
	) -> Result<(), Error> {
		Ok(beacon::apply_transaction(block, state.state(), extrinsic, &self.config)?)
	}

	pub fn finalize_block(
		&self,
		block: &mut UnsealedBeaconBlock,
		state: &mut <Self as BlockExecutor>::Externalities,
	) -> Result<(), Error> {
		Ok(beacon::finalize_block(block, state.state(), &self.config)?)
	}
}

impl<C: Config> BlockExecutor for Executor<C> {
	type Error = Error;
	type Block = Block;
	type Externalities = dyn StateExternalities + 'static;

	fn execute_block(
		&self,
		block: &Block,
		state: &mut Self::Externalities,
	) -> Result<(), Error> {
		Ok(beacon::execute_block(&block.0, state.state(), &self.config)?)
	}
}

impl<C: Config> JustifiableExecutor for Executor<C> {
	type ValidatorIndex = u64;

	fn justified_active_validators(
		&self,
		state: &mut Self::Externalities,
	) -> Result<Vec<Self::ValidatorIndex>, Self::Error> {
		Ok(beacon::justified_active_validators(state.state(), &self.config))
	}

	fn justified_block_id(
		&self,
		state: &mut Self::Externalities,
	) -> Result<Option<<Self::Block as BlockT>::Identifier>, Self::Error> {
		let justified_root = beacon::justified_root(state.state(), &self.config);
		if justified_root == H256::default() {
			Ok(None)
		} else {
			Ok(Some(justified_root))
		}
	}

	fn votes(
		&self,
		block: &Self::Block,
		state: &mut Self::Externalities,
	) -> Result<Vec<(Self::ValidatorIndex, <Self::Block as BlockT>::Identifier)>, Self::Error> {
		Ok(beacon::block_vote_targets(&block.0, state.state(), &self.config)?)
	}
}
