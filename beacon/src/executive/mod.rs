use ssz::Hashable;

use crate::{
	BeaconState, Config, Timestamp, Eth1Data, Fork, Error, Crosslink, BeaconBlockHeader,
	Deposit, BeaconBlock, Slot, ValidatorIndex, Shard, PendingAttestation, AttestationData,
	Epoch, Gwei, BeaconBlockBody,
};
use crate::primitives::{H256, BitField, Version, Signature};
use crate::utils::{is_power_of_two, to_bytes};

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
			return Err(Error::EpochOutOfRange)
		}
		let active_validators = self.state.active_validator_indices(shuffling_epoch);

		let start_shard = if epoch == current_epoch {
			self.state.latest_start_shard
		} else if epoch == previous_epoch {
			let previous_shard_delta = self.shard_delta(previous_epoch);
			(self.state.latest_start_shard - previous_shard_delta) % self.config.shard_count()
		} else if epoch == next_epoch {
			let current_shard_delta = self.shard_delta(current_epoch);
			(self.state.latest_start_shard + current_shard_delta) % self.config.shard_count()
		} else {
			return Err(Error::EpochOutOfRange)
		};

		let committees_per_epoch = self.epoch_committee_count(epoch);
		let committees_per_slot = committees_per_epoch as u64 / self.config.slots_per_epoch();
		let offset = slot % self.config.slots_per_epoch();

		let slot_start_shard = (shuffling_start_shard + committees_per_slot * offset) % self.config.shard_count() as u64;
		let seed = self.seed(epoch)?;

		let mut ret = Vec::new();
		for i in 0..committees_per_slot {
			ret.push(
				(self.config.compute_committee(&active_validators, &seed, (committees_per_slot * offset + i) as usize, committees_per_epoch), (slot_start_shard + i as u64) % self.config.shard_count() as u64)
			);
		}
		Ok(ret)
	}

	fn beacon_proposer_index(&self) -> Result<ValidatorIndex, Error> {
		let current_epoch = self.current_epoch();

		let (first_committee, _) = self.crosslink_committees_at_slot(self.state.slot)?[0].clone();
		let max_random_byte = u8::max_value();
		let mut i = 0;
		loop {
			let candidate_index = first_committee[(current_epoch + i) % first_committee.len()];
			let random_byte = self.config.hash2(
				self.seed(current_epoch),
				to_bytes(i / 32)
			)[i % 32];

			let effective_balance = self.state.validator_registry[candidate_index].effective_balance;
			if effective_balance * max_random_byte >= self.config.max_effective_balance() * random_byte {
				return Ok(candidate_index)
			}

			i+= 1
		}
	}

	fn attesting_indices(&self, attestation: &AttestationData, bitfield: &BitField) -> Result<Vec<ValidatorIndex>, Error> {
		let crosslink_committees = self.crosslink_committees_at_slot(attestation.slot, false)?;

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
		participants.sort();
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

	fn epoch_committee_count(&self, epoch: Epoch) -> usize {
		let active_validators = self.state.active_validator_indices(epoch);
		self.config.epoch_committee_count(active_validators.len())
	}

	fn shard_delta(&self, epoch: Epoch) -> Shard {
		cmp::min(
			self.epoch_committee_count(epoch),
			self.config.shard_count() - self.config.shard_count() / self.config.slots_per_epoch()
		)
	}

	fn inclusion_slot(&self, index: ValidatorIndex) -> Result<Slot, Error> {
		Ok(self.earliest_attestation(index)?.inclusion_slot)
	}

	fn total_balance(&self, indices: &[ValidatorIndex]) -> Gwei {
		indices.iter().fold(0, |sum, index| {
			sum + self.validator_registry[*index].effective_balance
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

	fn block_root_at_slot(&self, slot: Slot) -> Result<H256, Error> {
		if slot >= self.state.slot || self.state.slot > slot + self.config.slots_per_historical_root() as u64 {
			return Err(Error::SlotOutOfRange)
		}
		Ok(self.state.latest_block_roots[(slot % self.config.slots_per_historical_root() as u64) as usize])
	}

	fn block_root_at_epoch(&self, epoch: Epoch) -> Result<H256, Error> {
		self.block_root_at_slot(self.config.epoch_start_slot(epoch))
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
		let current_epoch = self.current_epoch();
		if current_epoch > self.config.genesis_epoch() {
			current_epoch.saturating_sub(1)
		} else {
			current_epoch
		}
	}

	fn previous_epoch_committee_count(&self) -> usize {
		let previous_active_validators = self.state.active_validator_indices(self.state.previous_shuffling_epoch);
		self.config.epoch_committee_count(previous_active_validators.len())
	}

	fn current_epoch_committee_count(&self) -> usize {
		let current_active_validators = self.state.active_validator_indices(self.state.current_shuffling_epoch);
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

	fn initiate_validator_exit(&mut self, index: ValidatorIndex) {
		if self.state.validator_registry[index].exit_epoch != self.config.far_future_epoch() {
			return
		}

		let exit_epochs = {
			let ret = Vec::new();
			for validator in &self.state.validator_registry {
				if validator.exit_epoch != self.config.far_future_epoch() {
					ret.push(validator.exit_epoch);
				}
			}
			ret
		};
		let mut exit_queue_epoch = cmp::max(
			exit_epochs.fold(0, cmp::max),
			self.delayed_activation_exit_epoch(self.current_epoch())
		);
		let exit_queue_churn = exit_epochs.fold(0, |sum, i| {
			if i == exit_queue_epoch {
				sum + 1
			} else {
				sum
			}
		});

		if exit_queue_churn >= self.churn_limit() {
			exit_queue_epoch += 1;
		}

		let validator = &mut self.state.validator_registry[index];
		validator.exit_epoch = exit_queue_epoch;
		validator.withdrawable_epoch = validator.exit_epoch + self.config.min_validator_withdrawability_delay();
	}

	fn slash_validator(&mut self, slashed_index: ValidatorIndex, whistleblower_index: Option<ValidatorIndex>) {
		let current_epoch = self.current_epoch();
		self.initiate_validator_exit(slashed_index);

		self.state.validator_registry[slashed_index].slashed = true;
		self.state.validator_registry[slashed_index].withdrawable_epoch = current_epoch + self.config.latest_slashed_exit_length();
		let slashed_balance = self.state.validator_registry[slashed_index].effective_balance;
		self.state.latest_slashed_balances[current_epoch % self.config.latest_slashed_exit_length] += slahsed_balance;

		let proposer_index = self.beacon_proposer_index();
		let whistleblower_index = whistleblower_index.unwrap_or(proposer_index);
		let whistleblower_reward = slashed_balance / self.config.whistleblowing_reward_quotient();
		let proposer_reward = whistleblowing_reward / self.config.proposer_reward_quotient();

		self.state.decrease_balance(slashed_index, whistleblowing_reward);
		self.state.increase_balance(proposer_index, proposer_reward);
		self.state.increase_balance(whistleblower_index, whistleblowing_reward - proposer_reward);
	}

	fn domain_id(&self, domain_type: u64, message_epoch: Option<u64>) -> u64 {
		let epoch = message_epoch.unwarp_or(self.current_epoch());
		let fork_version = if epoch < self.state.fork.epoch {
			self.state.fork.previous_version
		} else {
			self.state.fork.current_version
		};

		let mut bytes = [0u8; 8];
		(&mut bytes[0..4]).copy_from_slice(fork_version.as_ref());
		(&mut bytes[4..8]).copy_from_slice(&domain_type.to_le_bytes()[0..4]);

		u64::from_le_bytes(bytes)
	}

	fn to_indexed(&self, attestation: Attestation) -> IndexedAttestation {
		let attesting_indices = self.attesting_indices(&attestation.data, &attestation.aggregation_bitfield);
		let custody_bit_1_indices = self.attesting_indices(&attestation.data, &attestation.custody_bitfield);
		let custody_bit_0_indices = {
			let mut ret = attesting_indices.clone();
			ret.retain(|v| !custody_bit_1_indices.contains(&v));
			ret
		};

		IndexedAttestation {
			data: attestation.data,
			signature: attestation.signature,
			custody_bit_0_indices, custody_bit_1_indices,
		}
	}

	fn churn_limit(&self) -> usize {
		cmp::max(
			self.config.min_per_epoch_churn_limit(),
			self.state.active_validator_indices(self.current_epoch()) / self.config.churn_limit_quotient()
		)
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
		previous_shuffling_epoch: config.genesis_epoch(),
		current_shuffling_epoch: config.genesis_epoch(),
		previous_shuffling_seed: H256::default(),
		current_shuffling_seed: H256::default(),

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
		executive.state.current_shuffling_seed = executive.seed(config.genesis_epoch())?;
	}

	Ok(state)
}
