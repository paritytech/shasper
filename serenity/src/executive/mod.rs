use ssz::Hashable;

use primitives::H256;
use crate::{
	BeaconState, Config, Timestamp, Eth1Data, Fork, Error, Crosslink, BeaconBlockHeader,
	Deposit, BeaconBlock,
};

mod cache;
mod per_block;
mod per_epoch;
mod per_slot;

pub struct Executive<'state, 'config, C: Config> {
	state: &'state mut BeaconState,
	config: &'config C,
}

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	pub fn new(state: &'state mut BeaconState, config: &'config C) -> Self {
		Self { state, config }
	}

	pub fn state(&self) -> &BeaconState {
		self.state
	}

	pub fn config(&self) -> &C {
		self.config
	}
}

pub fn genesis<C: Config>(deposits: Vec<Deposit>, genesis_time: Timestamp, latest_eth1_data: Eth1Data, config: &C) -> Result<(BeaconBlock, BeaconState), Error> {
	let state = genesis_state(deposits, genesis_time, latest_eth1_data, config)?;
	let mut block = BeaconBlock::empty();
	block.state_root = state.hash::<C::Hasher>();

	Ok((block, state))
}

pub fn genesis_state<C: Config>(deposits: Vec<Deposit>, genesis_time: Timestamp, latest_eth1_data: Eth1Data, config: &C) -> Result<BeaconState, Error> {
	let mut state = BeaconState {
		slot: config.genesis_slot(),
		genesis_time,
		fork: Fork::default(),

		validator_registry: Vec::new(),
		validator_balances: Vec::new(),
		validator_registry_update_epoch: config.genesis_epoch(),

		latest_randao_mixes: {
			let mut ret = Vec::new();
			for _ in 0..config.latest_randao_mixes_length() {
				ret.push(H256::default());
			}
			ret
		},
		previous_shuffling_start_shard: config.genesis_start_shard(),
		current_shuffling_start_shard: config.genesis_start_shard(),
		previous_shuffling_epoch: config.genesis_epoch() - 1,
		current_shuffling_epoch: config.genesis_epoch(),
		previous_shuffling_seed: H256::default(),
		current_shuffling_seed: H256::default(),

		previous_epoch_attestations: Vec::new(),
		current_epoch_attestations: Vec::new(),
		previous_justified_epoch: config.genesis_epoch() - 1,
		current_justified_epoch: config.genesis_epoch(),
		previous_justified_root: H256::default(),
		current_justified_root: H256::default(),
		justification_bitfield: 0,
		finalized_epoch: config.genesis_epoch(),
		finalized_root: H256::default(),

		latest_crosslinks: {
			let mut ret = Vec::new();
			for _ in 0..config.shard_count() {
				ret.push(Crosslink::default());
			}
			ret
		},
		latest_block_roots: {
			let mut ret = Vec::new();
			for _ in 0..config.slots_per_historical_root() {
				ret.push(H256::default());
			}
			ret
		},
		latest_state_roots: {
			let mut ret = Vec::new();
			for _ in 0..config.slots_per_historical_root() {
				ret.push(H256::default());
			}
			ret
		},
		latest_active_index_roots: {
			let mut ret = Vec::new();
			for _ in 0..config.latest_active_index_roots_length() {
				ret.push(H256::default());
			}
			ret
		},
		latest_slashed_balances: {
			let mut ret = Vec::new();
			for _ in 0..config.latest_slashed_exit_length() {
				ret.push(0);
			}
			ret
		},
		latest_block_header: BeaconBlockHeader::with_state_root(&BeaconBlock::empty(), H256::default()),
		historical_roots: Vec::new(),

		latest_eth1_data,
		eth1_data_votes: Vec::new(),
		deposit_index: 0,
	};

	{
		let mut executive = Executive::new(&mut state, config);
		for deposit in deposits {
			executive.push_deposit(deposit)?;
		}
	}

	for validator_index in 0..(state.validator_registry.len() as u64) {
		if state.effective_balance(validator_index) >= config.max_deposit_amount() {
			state.activate_validator(validator_index, true);
		}
	}

	let genesis_active_index_root = state.active_validator_indices(config.genesis_epoch()).hash::<C::Hasher>();
	for index in 0..config.latest_active_index_roots_length() {
		state.latest_active_index_roots[index] = genesis_active_index_root;
	}
	state.current_shuffling_seed = state.seed(config.genesis_epoch())?;

	Ok(state)
}
