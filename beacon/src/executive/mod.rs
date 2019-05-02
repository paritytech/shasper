use ssz::Hashable;

use crate::{
	BeaconState, Config, Timestamp, Eth1Data, Fork, Error, Crosslink, BeaconBlockHeader,
	Deposit, BeaconBlock, Slot, ValidatorIndex, Shard, PendingAttestation, AttestationData,
	Epoch, Gwei, BeaconBlockBody,
};
use crate::primitives::{H256, BitField, Version, Signature};
use crate::utils::to_bytes;

mod cache;
mod per_block;
mod per_epoch;
mod per_slot;

/// Beacon state executive.
pub struct Executive<'state, 'config, C: Config> {
	state: &'state mut BeaconState,
	config: &'config C,
}

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Create a new executive from given state and config.
	pub fn new(state: &'state mut BeaconState, config: &'config C) -> Self {
		Self { state, config }
	}

	/// Get a reference to the underlying state.
	pub fn state(&self) -> &BeaconState {
		self.state
	}

	/// Get a reference to the config.
	pub fn config(&self) -> &C {
		self.config
	}

	fn crosslink_committees_at_slot(&self, slot: Slot) -> Result<Vec<(Vec<ValidatorIndex>, Shard)>, Error> {
		let epoch = self.config.slot_to_epoch(slot);
		let current_epoch = self.current_epoch();
		let previous_epoch = self.previous_epoch();
		let next_epoch = current_epoch + 1;

		if previous_epoch > epoch || epoch > next_epoch {
			return Err(Error::EpochOutOfRange);
		}

		let active_validators = self.state.active_validator_indices(epoch);
		let committees_per_epoch = self.config.epoch_committee_count(active_validators.len()) as u64;

		let start_shard = if epoch == current_epoch {
			self.state.latest_start_shard
		} else if epoch == previous_epoch {
			(self.state.latest_start_shard - committees_per_epoch) % self.config.shard_count() as u64
		} else {
			let current_epoch_committees = self.current_epoch_committee_count();
			(self.state.latest_start_shard + current_epoch_committees as u64) % self.config.shard_count() as u64
		};

		let committees_per_slot = committees_per_epoch / self.config.slots_per_epoch() as u64;
		let offset = slot % self.config.slots_per_epoch();
		let slot_start_shard = (start_shard + committees_per_slot * offset) % self.config.shard_count() as u64;
		let seed = self.seed(epoch)?;

		let mut ret = Vec::new();
		for i in 0..committees_per_slot {
			ret.push(
				(self.config.compute_committee(&active_validators, &seed, (committees_per_slot * offset + i) as usize, committees_per_epoch as usize), (slot_start_shard + i as u64) % self.config.shard_count() as u64)
			);
		}
		Ok(ret)
	}

	fn beacon_proposer_index(&self, slot: Slot) -> Result<ValidatorIndex, Error> {
		let current_epoch = self.current_epoch();

		if self.config.slot_to_epoch(slot) != current_epoch {
			return Err(Error::EpochOutOfRange)
		}

		let (first_committee, _) = self.crosslink_committees_at_slot(slot)?[0].clone();
		let mut i: usize = 0;
		loop {
			let rand_byte = self.config.hash2(
				&self.seed(current_epoch)?[..],
				&to_bytes((i / 32) as u64)[..]
			)[i % 32];
			let candidate = first_committee[((current_epoch + i as u64) % first_committee.len() as u64) as usize];
			if self.effective_balance(candidate) * 256 > self.config.max_deposit_amount() * rand_byte as u64 {
				return Ok(candidate)
			}
			i += 1;
		}
	}

	fn attestation_participants(&self, attestation: &AttestationData, bitfield: &BitField) -> Result<Vec<ValidatorIndex>, Error> {
		let crosslink_committees = self.crosslink_committees_at_slot(attestation.slot)?;

		let matched_committees = crosslink_committees.iter().filter(|(_, s)| s == &attestation.shard).collect::<Vec<_>>();
		if matched_committees.len() == 0 {
			return Err(Error::AttestationShardInvalid);
		}

		let crosslink_committee = matched_committees[0];
		if !bitfield.verify(crosslink_committee.0.len()) {
			return Err(Error::AttestationBitFieldInvalid);
		}

		let mut participants = Vec::new();
		for (i, validator_index) in crosslink_committee.0.iter().enumerate() {
			if bitfield.has_voted(i) {
				participants.push(*validator_index);
			}
		}
		Ok(participants)
	}

	fn earliest_attestation(&self, index: ValidatorIndex) -> Result<PendingAttestation, Error> {
		let attestations = {
			let mut ret = Vec::new();
			for attestation in self.state.previous_epoch_attestations.clone() {
				if self.attestation_participants(&attestation.data, &attestation.aggregation_bitfield)?.contains(&index) {
					ret.push(attestation);
				}
			}
			ret
		};

		attestations.into_iter().min_by_key(|a| a.inclusion_slot).ok_or(Error::ValidatorAttestationNotFound)
	}

	fn inclusion_slot(&self, index: ValidatorIndex) -> Result<Slot, Error> {
		Ok(self.earliest_attestation(index)?.inclusion_slot)
	}

	fn balance(&self, index: ValidatorIndex) -> Gwei {
		self.state.balances[index as usize]
	}

	fn set_balance(&mut self, index: ValidatorIndex, balance: Gwei) {
		let half_increment = self.config.high_balance_increment() / 2;

		let validator = &mut self.state.validator_registry[index as usize];
		if validator.high_balance > balance || validator.high_balance + 3 * half_increment < balance {
			validator.high_balance = balance - balance % self.config.high_balance_increment();
		}
		self.state.balances[index as usize] = balance;
	}

	fn increase_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
		self.set_balance(index, self.balance(index) + delta);
	}

	fn decrease_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
		let cur_balance = self.balance(index);
		self.set_balance(index, if cur_balance >= delta {
			cur_balance - delta
		} else {
			0
		});
	}

	fn effective_balance(&self, index: ValidatorIndex) -> Gwei {
		core::cmp::min(self.balance(index), self.config.max_deposit_amount())
	}

	fn total_balance(&self, indices: &[ValidatorIndex]) -> Gwei {
		indices.iter().fold(0, |sum, index| {
			sum + self.effective_balance(*index)
		})
	}

	fn current_total_balance(&self) -> Gwei {
		self.total_balance(&self.state.active_validator_indices(self.current_epoch())[..])
	}

	fn previous_total_balance(&self) -> Gwei {
		self.total_balance(&self.state.active_validator_indices(self.previous_epoch())[..])
	}

	fn delayed_activation_exit_epoch(&self) -> u64 {
		self.current_epoch() + 1 + self.config.activation_exit_delay()
	}

	fn randao_mix(&self, epoch: Epoch) -> Result<H256, Error> {
		if self.current_epoch().saturating_sub(self.config.latest_randao_mixes_length() as u64) >= epoch ||
			epoch > self.current_epoch()
		{
			return Err(Error::EpochOutOfRange)
		}

		Ok(self.state.latest_randao_mixes[(epoch % self.config.latest_randao_mixes_length() as u64) as usize])
	}

	fn block_root(&self, slot: Slot) -> Result<H256, Error> {
		if slot >= self.state.slot || self.state.slot > slot + self.config.slots_per_historical_root() as u64 {
			return Err(Error::SlotOutOfRange)
		}
		Ok(self.state.latest_block_roots[(slot % self.config.slots_per_historical_root() as u64) as usize])
	}

	#[allow(dead_code)]
	fn state_root(&self, slot: Slot) -> Result<H256, Error> {
		if slot >= self.state.slot || self.state.slot > slot + self.config.slots_per_historical_root() as u64 {
			return Err(Error::SlotOutOfRange)
		}
		Ok(self.state.latest_state_roots[(slot % self.config.slots_per_historical_root() as u64) as usize])
	}

	fn active_index_root(&self, epoch: Epoch) -> Result<H256, Error> {
		if self.current_epoch().saturating_sub(
			self.config.latest_active_index_roots_length() as u64 - self.config.activation_exit_delay()
		) >= epoch || epoch > self.current_epoch() + self.config.activation_exit_delay() {
			return Err(Error::EpochOutOfRange)
		}

		Ok(self.state.latest_active_index_roots[(epoch % self.config.latest_active_index_roots_length() as u64) as usize])
	}

	fn seed(&self, epoch: Epoch) -> Result<H256, Error> {
		Ok(self.config.hash3(
			self.randao_mix(epoch.saturating_sub(self.config.min_seed_lookahead()))?.as_ref(),
			self.active_index_root(epoch)?.as_ref(),
			to_bytes(epoch).as_ref()
		))
	}

	fn current_epoch(&self) -> Epoch {
		self.config.slot_to_epoch(self.state.slot)
	}

	fn previous_epoch(&self) -> Epoch {
		self.current_epoch().saturating_sub(1)
	}

	fn current_epoch_committee_count(&self) -> usize {
		let current_active_validators = self.state.active_validator_indices(self.current_epoch());
		self.config.epoch_committee_count(current_active_validators.len())
	}

	fn current_epoch_boundary_attestations(&self) -> Result<Vec<PendingAttestation>, Error> {
		let block_root = self.block_root(self.config.epoch_start_slot(self.current_epoch()))?;
		Ok(self.state.current_epoch_attestations.clone().into_iter()
		   .filter(|a| a.data.target_root == block_root)
		   .collect())
	}

	fn previous_epoch_boundary_attestations(&self) -> Result<Vec<PendingAttestation>, Error> {
		let block_root = self.block_root(self.config.epoch_start_slot(self.previous_epoch()))?;
		Ok(self.state.previous_epoch_attestations.clone().into_iter()
		   .filter(|a| a.data.target_root == block_root)
		   .collect())
	}

	fn previous_epoch_matching_head_attestations(&self) -> Result<Vec<PendingAttestation>, Error> {
		let mut ret = Vec::new();
		for attestation in self.state.previous_epoch_attestations.clone() {
			if attestation.data.beacon_block_root == self.block_root(attestation.data.slot)? {
				ret.push(attestation);
			}
		}
		Ok(ret)
	}

	fn activate_validator(&mut self, index: ValidatorIndex, is_genesis: bool) {
		let delayed_activation_exit_epoch = self.delayed_activation_exit_epoch();
		self.state.validator_registry[index as usize].activate(delayed_activation_exit_epoch, is_genesis, self.config);
	}

	fn initiate_validator_exit(&mut self, index: ValidatorIndex) {
		self.state.validator_registry[index as usize].initiate_exit();
	}

	fn exit_validator(&mut self, index: ValidatorIndex) {
		let delayed_activation_exit_epoch = self.delayed_activation_exit_epoch();
		self.state.validator_registry[index as usize].exit(delayed_activation_exit_epoch);
	}
}

/// Generate genesis state and genesis block from given deposits, timestamp and eth1 data.
pub fn genesis<C: Config>(deposits: Vec<Deposit>, genesis_time: Timestamp, latest_eth1_data: Eth1Data, config: &C) -> Result<(BeaconBlock, BeaconState), Error> {
	let state = genesis_state(deposits, genesis_time, latest_eth1_data, config)?;
	let mut block = BeaconBlock {
		slot: config.genesis_slot(),
		previous_block_root: H256::default(),
		state_root: H256::default(),
		signature: Signature::default(),
		body: BeaconBlockBody::empty(),
	};
	block.state_root = Hashable::<C::Hasher>::hash(&state);

	Ok((block, state))
}

/// Generate genesis state from given deposits, timestamp, and eth1 data.
pub fn genesis_state<C: Config>(deposits: Vec<Deposit>, genesis_time: Timestamp, latest_eth1_data: Eth1Data, config: &C) -> Result<BeaconState, Error> {
	let mut state = BeaconState {
		slot: config.genesis_slot(),
		genesis_time,
		fork: Fork {
			previous_version: Version::from(config.genesis_fork_version()),
			current_version: Version::from(config.genesis_fork_version()),
			epoch: config.genesis_epoch(),
		},

		validator_registry: Vec::new(),
		balances: Vec::new(),
		validator_registry_update_epoch: config.genesis_epoch(),

		latest_randao_mixes: {
			let mut ret = Vec::new();
			for _ in 0..config.latest_randao_mixes_length() {
				ret.push(H256::default());
			}
			ret
		},
		latest_start_shard: config.genesis_start_shard(),

		previous_epoch_attestations: Vec::new(),
		current_epoch_attestations: Vec::new(),
		previous_justified_epoch: config.genesis_epoch(),
		current_justified_epoch: config.genesis_epoch(),
		previous_justified_root: H256::default(),
		current_justified_root: H256::default(),
		justification_bitfield: 0,
		finalized_epoch: config.genesis_epoch(),
		finalized_root: H256::default(),

		latest_crosslinks: {
			let mut ret = Vec::new();
			for _ in 0..config.shard_count() {
				ret.push(Crosslink {
					epoch: config.genesis_epoch(),
					crosslink_data_root: H256::default(),
				});
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
		latest_block_header: BeaconBlockHeader::with_state_root_no_signature::<_, C::Hasher>(&BeaconBlock {
			slot: config.genesis_slot(),
			previous_block_root: H256::default(),
			state_root: H256::default(),
			signature: Signature::default(),
			body: BeaconBlockBody::empty(),
		}, H256::default()),
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

		for validator_index in 0..(executive.state.validator_registry.len() as u64) {
			if executive.effective_balance(validator_index) >= config.max_deposit_amount() {
				executive.activate_validator(validator_index, true);
			}
		}

		let genesis_active_index_root = Hashable::<C::Hasher>::hash(&executive.state.active_validator_indices(config.genesis_epoch()));
		for index in 0..config.latest_active_index_roots_length() {
			executive.state.latest_active_index_roots[index] = genesis_active_index_root;
		}
	}

	Ok(state)
}
