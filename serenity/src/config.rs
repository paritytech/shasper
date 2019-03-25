use hash_db::Hasher;

use primitives::{Signature, H256, ValidatorId, Version};
use crate::{Epoch, Slot, Gwei, Shard, Fork, ValidatorIndex};
use crate::util::{split_offset, permuted_index};

pub trait Config {
	type Hasher: Hasher<Out=H256>;

	fn shard_count(&self) -> usize;
	fn target_committee_size(&self) -> usize;
	fn max_balance_churn_quotient(&self) -> Gwei;
	fn max_indices_per_slashable_vote(&self) -> usize;
	fn max_exit_dequeues_per_epoch(&self) -> usize;
	fn shuffle_round_count(&self) -> usize;
	fn deposit_contract_tree_depth(&self) -> usize;
	fn min_deposit_amount(&self) -> Gwei;
	fn max_deposit_amount(&self) -> Gwei;
	fn fork_choice_balance_increment(&self) -> Gwei;
	fn ejection_balance(&self) -> Gwei;
	fn genesis_fork_version(&self) -> Version;
	fn genesis_slot(&self) -> Slot;
	fn genesis_epoch(&self) -> Epoch { self.slot_to_epoch(self.genesis_slot()) }
	fn genesis_start_shard(&self) -> Shard;
	fn bls_withdrawal_prefix_byte(&self) -> u8;
	fn seconds_per_slot(&self) -> u64;
	fn min_attestation_inclusion_delay(&self) -> Slot;
	fn slots_per_epoch(&self) -> Slot;
	fn min_seed_lookahead(&self) -> Epoch;
	fn activation_exit_delay(&self) -> Epoch;
	fn epochs_per_eth1_voting_period(&self) -> Epoch;
	fn slots_per_historical_root(&self) -> usize;
	fn min_validator_withdrawability_delay(&self) -> Epoch;
	fn persistent_committee_period(&self) -> Epoch;
	fn latest_randao_mixes_length(&self) -> usize;
	fn latest_active_index_roots_length(&self) -> usize;
	fn latest_slashed_exit_length(&self) -> usize;
	fn base_reward_quotient(&self) -> Gwei;
	fn whistleblower_reward_quotient(&self) -> Gwei;
	fn attestation_inclusion_reward_quotient(&self) -> Gwei;
	fn inactivity_penalty_quotient(&self) -> Gwei;
	fn min_penalty_quotient(&self) -> Gwei;
	fn max_proposer_slashings(&self) -> usize;
	fn max_attester_slashings(&self) -> usize;
	fn max_attestations(&self) -> usize;
	fn max_deposits(&self) -> usize;
	fn max_voluntary_exits(&self) -> usize;
	fn max_transfers(&self) -> usize;
	fn domain_beacon_block(&self) -> u64;
	fn domain_randao(&self) -> u64;
	fn domain_attestation(&self) -> u64;
	fn domain_deposit(&self) -> u64;
	fn domain_voluntary_exit(&self) -> u64;
	fn domain_transfer(&self) -> u64;
	fn far_future_epoch(&self) -> Epoch;

	fn domain_id(&self, fork: &Fork, epoch: Epoch, typ: u64) -> u64;
	fn bls_verify(&self, pubkey: &ValidatorId, message: &H256, signature: &Signature, domain: u64) -> bool;
	fn bls_aggregate_pubkeys(&self, pubkeys: &[ValidatorId]) -> Option<ValidatorId>;
	fn bls_verify_multiple(&self, pubkeys: &[ValidatorId], messages: &[H256], signature: &Signature, domain: u64) -> bool;

	/// Hash bytes with a hasher.
	fn hash(&self, seed: &[u8]) -> H256 {
		Self::Hasher::hash(seed)
	}

	/// Hash two bytes with a hasher.
	fn hash2(&self, seed: &[u8], a: &[u8]) -> H256 {
		let mut v = seed.to_vec();
		let mut a = a.to_vec();
		v.append(&mut a);
		Self::Hasher::hash(&v)
	}

	/// Hash three bytes with a hasher.
	fn hash3(&self, seed: &[u8], a: &[u8], b: &[u8]) -> H256 {
		let mut v = seed.to_vec();
		let mut a = a.to_vec();
		let mut b = b.to_vec();
		v.append(&mut a);
		v.append(&mut b);
		Self::Hasher::hash(&v)
	}

	fn slot_to_epoch(&self, slot: Slot) -> Epoch {
		slot / self.slots_per_epoch()
	}

	fn epoch_start_slot(&self, epoch: Epoch) -> Slot {
		epoch.saturating_mul(self.slots_per_epoch())
	}

	fn compute_committee(&self, validators: &[ValidatorIndex], seed: &H256, index: usize, total_committees: usize) -> Vec<ValidatorIndex> {
		let start_offset = split_offset(validators.len(), total_committees, index);
		let end_offset = split_offset(validators.len(), total_committees, index + 1);

		let mut ret = Vec::new();
		for i in start_offset..end_offset {
			ret.push(permuted_index(i, seed, validators.len(), self.shuffle_round_count()) as ValidatorIndex);
		}
		ret
	}

	fn epoch_committee_count(&self, active_validator_count: usize) -> usize {
		core::cmp::max(
			1,
			core::cmp::min(
				self.shard_count() / self.slots_per_epoch() as usize,
				active_validator_count / self.slots_per_epoch() as usize / self.target_committee_size(),
			)
		) * self.slots_per_epoch() as usize
	}
}

pub struct NoVerificationConfig {
	pub shard_count: usize,
	pub target_committee_size: usize,
	pub max_balance_churn_quotient: Gwei,
	pub max_indices_per_slashable_vote: usize,
	pub max_exit_dequeues_per_epoch: usize,
	pub shuffle_round_count: usize,
	pub deposit_contract_tree_depth: usize,
	pub min_deposit_amount: Gwei,
	pub max_deposit_amount: Gwei,
	pub fork_choice_balance_increment: Gwei,
	pub ejection_balance: Gwei,
	pub genesis_fork_version: [u8; 4],
	pub genesis_slot: Slot,
	pub genesis_start_shard: Shard,
	pub bls_withdrawal_prefix_byte: u8,
	pub seconds_per_slot: u64,
	pub min_attestation_inclusion_delay: Slot,
	pub slots_per_epoch: Slot,
	pub min_seed_lookahead: Epoch,
	pub activation_exit_delay: Epoch,
	pub epochs_per_eth1_voting_period: Epoch,
	pub slots_per_historical_root: usize,
	pub min_validator_withdrawability_delay: Epoch,
	pub persistent_committee_period: Epoch,
	pub latest_randao_mixes_length: usize,
	pub latest_active_index_roots_length: usize,
	pub latest_slashed_exit_length: usize,
	pub base_reward_quotient: Gwei,
	pub whistleblower_reward_quotient: Gwei,
	pub attestation_inclusion_reward_quotient: Gwei,
	pub inactivity_penalty_quotient: Gwei,
	pub min_penalty_quotient: Gwei,
	pub max_proposer_slashings: usize,
	pub max_attester_slashings: usize,
	pub max_attestations: usize,
	pub max_deposits: usize,
	pub max_voluntary_exits: usize,
	pub max_transfers: usize,
	pub domain_beacon_block: u64,
	pub domain_randao: u64,
	pub domain_attestation: u64,
	pub domain_deposit: u64,
	pub domain_voluntary_exit: u64,
	pub domain_transfer: u64,
	pub far_future_epoch: Epoch,
}

impl Config for NoVerificationConfig {
	type Hasher = keccak_hasher::KeccakHasher;

	fn shard_count(&self) -> usize { self.shard_count }
	fn target_committee_size(&self) -> usize { self.target_committee_size }
	fn max_balance_churn_quotient(&self) -> Gwei { self.max_balance_churn_quotient }
	fn max_indices_per_slashable_vote(&self) -> usize { self.max_indices_per_slashable_vote }
	fn max_exit_dequeues_per_epoch(&self) -> usize { self.max_exit_dequeues_per_epoch }
	fn shuffle_round_count(&self) -> usize { self.shuffle_round_count }
	fn deposit_contract_tree_depth(&self) -> usize { self.deposit_contract_tree_depth }
	fn min_deposit_amount(&self) -> Gwei { self.min_deposit_amount }
	fn max_deposit_amount(&self) -> Gwei { self.max_deposit_amount }
	fn fork_choice_balance_increment(&self) -> Gwei { self.fork_choice_balance_increment }
	fn ejection_balance(&self) -> Gwei { self.ejection_balance }
	fn genesis_fork_version(&self) -> Version { Version::from(self.genesis_fork_version) }
	fn genesis_slot(&self) -> Slot { self.genesis_slot }
	fn genesis_start_shard(&self) -> Shard { self.genesis_start_shard }
	fn bls_withdrawal_prefix_byte(&self) -> u8 { self.bls_withdrawal_prefix_byte }
	fn seconds_per_slot(&self) -> u64 { self.seconds_per_slot }
	fn min_attestation_inclusion_delay(&self) -> Slot { self.min_attestation_inclusion_delay }
	fn slots_per_epoch(&self) -> Slot { self.slots_per_epoch }
	fn min_seed_lookahead(&self) -> Epoch { self.min_seed_lookahead }
	fn activation_exit_delay(&self) -> Epoch { self.activation_exit_delay }
	fn epochs_per_eth1_voting_period(&self) -> Epoch { self.epochs_per_eth1_voting_period }
	fn slots_per_historical_root(&self) -> usize { self.slots_per_historical_root }
	fn min_validator_withdrawability_delay(&self) -> Epoch { self.min_validator_withdrawability_delay }
	fn persistent_committee_period(&self) -> Epoch { self.persistent_committee_period }
	fn latest_randao_mixes_length(&self) -> usize { self.latest_randao_mixes_length }
	fn latest_active_index_roots_length(&self) -> usize { self.latest_active_index_roots_length }
	fn latest_slashed_exit_length(&self) -> usize { self.latest_slashed_exit_length }
	fn base_reward_quotient(&self) -> Gwei { self.base_reward_quotient }
	fn whistleblower_reward_quotient(&self) -> Gwei { self.whistleblower_reward_quotient }
	fn attestation_inclusion_reward_quotient(&self) -> Gwei { self.attestation_inclusion_reward_quotient }
	fn inactivity_penalty_quotient(&self) -> Gwei { self.inactivity_penalty_quotient }
	fn min_penalty_quotient(&self) -> Gwei { self.min_penalty_quotient }
	fn max_proposer_slashings(&self) -> usize { self.max_proposer_slashings }
	fn max_attester_slashings(&self) -> usize { self.max_attester_slashings }
	fn max_attestations(&self) -> usize { self.max_attestations }
	fn max_deposits(&self) -> usize { self.max_deposits }
	fn max_voluntary_exits(&self) -> usize { self.max_voluntary_exits }
	fn max_transfers(&self) -> usize { self.max_transfers }
	fn domain_beacon_block(&self) -> u64 { self.domain_beacon_block }
	fn domain_randao(&self) -> u64 { self.domain_randao }
	fn domain_attestation(&self) -> u64 { self.domain_attestation }
	fn domain_deposit(&self) -> u64 { self.domain_deposit }
	fn domain_voluntary_exit(&self) -> u64 { self.domain_voluntary_exit }
	fn domain_transfer(&self) -> u64 { self.domain_transfer }
	fn far_future_epoch(&self) -> Epoch { self.far_future_epoch }

	fn domain_id(&self, fork: &Fork, epoch: Epoch, typ: u64) -> u64 {
		let version = if epoch < fork.epoch {
			&fork.previous_version
		} else {
			&fork.current_version
		};

		let mut bytes = [0u8; 8];
		(&mut bytes[0..4]).copy_from_slice(version.as_ref());
		(&mut bytes[4..8]).copy_from_slice(&typ.to_le_bytes()[0..4]);

		u64::from_le_bytes(bytes)
	}
	fn bls_verify(&self, _pubkey: &ValidatorId, _message: &H256, _signature: &Signature, _domain: u64) -> bool {
		true
	}
	fn bls_aggregate_pubkeys(&self, _pubkeys: &[ValidatorId]) -> Option<ValidatorId> {
		Some(ValidatorId::default())
	}
	fn bls_verify_multiple(&self, _pubkeys: &[ValidatorId], _messages: &[H256], _signature: &Signature, _domain: u64) -> bool {
		true
	}
}

impl NoVerificationConfig {
	pub fn small() -> Self {
		Self {
			shard_count: 8,
			target_committee_size: 4,
			max_balance_churn_quotient: 32,
			max_indices_per_slashable_vote: 4096,
			max_exit_dequeues_per_epoch: 4,
			shuffle_round_count: 90,
			deposit_contract_tree_depth: 32,
			min_deposit_amount: 1_000_000_000,
			max_deposit_amount: 32_000_000_000,
			fork_choice_balance_increment: 1_000_000_000,
			ejection_balance: 16_000_000_000,
			genesis_fork_version: [0, 0, 0, 0],
			genesis_slot: 4294967296,
			genesis_start_shard: 0,
			bls_withdrawal_prefix_byte: 0,
			seconds_per_slot: 6,
			min_attestation_inclusion_delay: 2,
			slots_per_epoch: 8,
			min_seed_lookahead: 1,
			activation_exit_delay: 4,
			epochs_per_eth1_voting_period: 16,
			slots_per_historical_root: 64,
			min_validator_withdrawability_delay: 256,
			persistent_committee_period: 2048,
			latest_randao_mixes_length: 64,
			latest_active_index_roots_length: 64,
			latest_slashed_exit_length: 64,
			base_reward_quotient: 32,
			whistleblower_reward_quotient: 512,
			attestation_inclusion_reward_quotient: 8,
			inactivity_penalty_quotient: 16_777_216,
			min_penalty_quotient: 32,
			max_proposer_slashings: 16,
			max_attester_slashings: 1,
			max_attestations: 128,
			max_deposits: 16,
			max_voluntary_exits: 16,
			max_transfers: 16,
			domain_beacon_block: 0,
			domain_randao: 1,
			domain_attestation: 2,
			domain_deposit: 3,
			domain_voluntary_exit: 4,
			domain_transfer: 5,
			far_future_epoch: u64::max_value(),
		}
	}
}
