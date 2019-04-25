use hash_db::Hasher;
use tiny_keccak::Keccak;
use plain_hasher::PlainHasher;

#[cfg(feature = "serde")]
use serde_derive::{Serialize, Deserialize};
#[cfg(feature = "parity-codec")]
use codec::{Encode, Decode};

use crate::primitives::{Signature, H256, ValidatorId, Version};
use crate::{Epoch, Slot, Gwei, Shard, Fork, ValidatorIndex};
use crate::utils::{split_offset, to_usize};

/// Constants used in beacon block.
pub trait Config {
	/// Hash function.
	type Hasher: Hasher<Out=H256>;

	/// Shard count.
	fn shard_count(&self) -> usize;
	/// Target committee size.
	fn target_committee_size(&self) -> usize;
	/// Maximum balance churn quotient.
	fn max_balance_churn_quotient(&self) -> Gwei;
	/// Maximum indices per slashable vote.
	fn max_indices_per_slashable_vote(&self) -> usize;
	/// Maximum exit dequeues per epoch.
	fn max_exit_dequeues_per_epoch(&self) -> usize;
	/// Shuffle round count.
	fn shuffle_round_count(&self) -> usize;
	/// Deposit contract tree depth.
	fn deposit_contract_tree_depth(&self) -> usize;
	/// Minimum deposit amount.
	fn min_deposit_amount(&self) -> Gwei;
	/// Maximum deposit amount.
	fn max_deposit_amount(&self) -> Gwei;
	/// Fork choice balance increment.
	fn fork_choice_balance_increment(&self) -> Gwei;
	/// Ejection balance.
	fn ejection_balance(&self) -> Gwei;
	/// Genesis fork version.
	fn genesis_fork_version(&self) -> Version;
	/// Genesis slot.
	fn genesis_slot(&self) -> Slot;
	/// Genesis epoch.
	fn genesis_epoch(&self) -> Epoch { self.slot_to_epoch(self.genesis_slot()) }
	/// Genesis start shard.
	fn genesis_start_shard(&self) -> Shard;
	/// BLS withdrawal prefix byte.
	fn bls_withdrawal_prefix_byte(&self) -> u8;
	/// Seconds per slot.
	fn seconds_per_slot(&self) -> u64;
	/// Minimum attestation inclusion delay.
	fn min_attestation_inclusion_delay(&self) -> Slot;
	/// Slots per epoch.
	fn slots_per_epoch(&self) -> Slot;
	/// Minimum seed lookahead.
	fn min_seed_lookahead(&self) -> Epoch;
	/// Activation exit delay.
	fn activation_exit_delay(&self) -> Epoch;
	/// Epoch per eth1 voting period.
	fn epochs_per_eth1_voting_period(&self) -> Epoch;
	/// Slots per historical root.
	fn slots_per_historical_root(&self) -> usize;
	/// Minimal validator withdrawability delay.
	fn min_validator_withdrawability_delay(&self) -> Epoch;
	/// Persistent committee period.
	fn persistent_committee_period(&self) -> Epoch;
	/// Latest randao mixes length.
	fn latest_randao_mixes_length(&self) -> usize;
	/// Latest active index roots length.
	fn latest_active_index_roots_length(&self) -> usize;
	/// Latest slashed exit length.
	fn latest_slashed_exit_length(&self) -> usize;
	/// Base reward quotient.
	fn base_reward_quotient(&self) -> Gwei;
	/// Whistleblower reward quotient.
	fn whistleblower_reward_quotient(&self) -> Gwei;
	/// Attestation inclusion reward quotient.
	fn attestation_inclusion_reward_quotient(&self) -> Gwei;
	/// Inactivity penalty quotient.
	fn inactivity_penalty_quotient(&self) -> Gwei;
	/// Minimal penalty quotient.
	fn min_penalty_quotient(&self) -> Gwei;
	/// Maximum proposer slashings per block.
	fn max_proposer_slashings(&self) -> usize;
	/// Maximum attester slashings per block.
	fn max_attester_slashings(&self) -> usize;
	/// Maximum attestations per block.
	fn max_attestations(&self) -> usize;
	/// Maximum deposits per block.
	fn max_deposits(&self) -> usize;
	/// Maximum voluntary exits per block.
	fn max_voluntary_exits(&self) -> usize;
	/// Maximum transfers per block.
	fn max_transfers(&self) -> usize;
	/// Beacon block domain.
	fn domain_beacon_block(&self) -> u64;
	/// Randao domain.
	fn domain_randao(&self) -> u64;
	/// Attestation domain.
	fn domain_attestation(&self) -> u64;
	/// Deposit domain.
	fn domain_deposit(&self) -> u64;
	/// Voluntary exit domain.
	fn domain_voluntary_exit(&self) -> u64;
	/// Transfer domain.
	fn domain_transfer(&self) -> u64;
	/// Far future epoch.
	fn far_future_epoch(&self) -> Epoch;

	/// Get domain id for given fork, epoch and type.
	fn domain_id(&self, fork: &Fork, epoch: Epoch, typ: u64) -> u64;
	/// Verify BLS signature.
	fn bls_verify(&self, pubkey: &ValidatorId, message: &H256, signature: &Signature, domain: u64) -> bool;
	/// Aggregate BLS public keys.
	fn bls_aggregate_pubkeys(&self, pubkeys: &[ValidatorId]) -> Option<ValidatorId>;
	/// Verify multiple BLS signatures.
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

	/// Convert slot into epoch.
	fn slot_to_epoch(&self, slot: Slot) -> Epoch {
		slot / self.slots_per_epoch()
	}

	/// Get start slot for an epoch.
	fn epoch_start_slot(&self, epoch: Epoch) -> Slot {
		epoch.saturating_mul(self.slots_per_epoch())
	}

	/// Get the permuted index.
	fn permuted_index(&self, mut index: usize, seed: &H256, len: usize) -> usize {
		if index >= len {
			index = index % len;
		}

		let usize_len = 0usize.to_le_bytes().len();

		for round in 0..self.shuffle_round_count() {
			let pivot = to_usize(
				&self.hash2(&seed[..], &round.to_le_bytes()[..1]).as_ref()[..usize_len]
			) % len;
			let flip = if pivot >= index { (pivot - index) % len } else { len - (index - pivot) % len };
			let position = core::cmp::max(index, flip);
			let source = self.hash3(
				&seed[..],
				&round.to_le_bytes()[..1],
				&(position / 256).to_le_bytes()[..4]
			);
			let byte = source.as_ref()[(position % 256) / 8];
			let bit = (byte >> (position % 8 )) % 2;
			index = if bit == 1 { flip } else { index };
		}

		index
	}

	/// Compute committee.
	fn compute_committee(&self, validators: &[ValidatorIndex], seed: &H256, index: usize, total_committees: usize) -> Vec<ValidatorIndex> {
		let start_offset = split_offset(validators.len(), total_committees, index);
		let end_offset = split_offset(validators.len(), total_committees, index + 1);

		let mut ret = Vec::new();
		for i in start_offset..end_offset {
			ret.push(self.permuted_index(i, seed, validators.len()) as ValidatorIndex);
		}
		ret
	}

	/// Get epoch committee count.
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

/// Keccak256 hasher.
pub struct KeccakHasher;

impl Hasher for KeccakHasher {
	type Out = H256;
	type StdHasher = PlainHasher;
	const LENGTH: usize = 32;

	fn hash(x: &[u8]) -> Self::Out {
		let mut out = [0; 32];
		Keccak::keccak256(x, &mut out);
		out.into()
	}
}

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "parity-codec", derive(Encode, Decode))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Config that does not verify BLS signature.
pub struct NoVerificationConfig {
	/// Shard count.
	pub shard_count: usize,
	/// Target committee size.
	pub target_committee_size: usize,
	/// Maximum balance churn quotient.
	pub max_balance_churn_quotient: Gwei,
	/// Maximum indices per slashable vote.
	pub max_indices_per_slashable_vote: usize,
	/// Maximum exit dequeues per epoch.
	pub max_exit_dequeues_per_epoch: usize,
	/// Shuffle round count.
	pub shuffle_round_count: usize,
	/// Deposit contract tree depth.
	pub deposit_contract_tree_depth: usize,
	/// Minimum deposit amount.
	pub min_deposit_amount: Gwei,
	/// Maximum deposit amount.
	pub max_deposit_amount: Gwei,
	/// Fork choice balance increment.
	pub fork_choice_balance_increment: Gwei,
	/// Ejection balance.
	pub ejection_balance: Gwei,
	/// Genesis fork version.
	pub genesis_fork_version: [u8; 4],
	/// Genesis slot.
	pub genesis_slot: Slot,
	/// Genesis start shard.
	pub genesis_start_shard: Shard,
	/// BLS withdrawal prefix byte.
	pub bls_withdrawal_prefix_byte: [u8; 1],
	/// Seconds per slot.
	pub seconds_per_slot: u64,
	/// Minimum attestation inclusion delay.
	pub min_attestation_inclusion_delay: Slot,
	/// Slots per epoch.
	pub slots_per_epoch: Slot,
	/// Minimum seed lookahead.
	pub min_seed_lookahead: Epoch,
	/// Activation exit delay.
	pub activation_exit_delay: Epoch,
	/// Epoch per eth1 voting period.
	pub epochs_per_eth1_voting_period: Epoch,
	/// Slots per historical root.
	pub slots_per_historical_root: usize,
	/// Minimal validator withdrawability delay.
	pub min_validator_withdrawability_delay: Epoch,
	/// Persistent committee period.
	pub persistent_committee_period: Epoch,
	/// Latest randao mixes length.
	pub latest_randao_mixes_length: usize,
	/// Latest active index roots length.
	pub latest_active_index_roots_length: usize,
	/// Latest slashed exit length.
	pub latest_slashed_exit_length: usize,
	/// Base reward quotient.
	pub base_reward_quotient: Gwei,
	/// Whistleblower reward quotient.
	pub whistleblower_reward_quotient: Gwei,
	/// Attestation inclusion reward quotient.
	pub attestation_inclusion_reward_quotient: Gwei,
	/// Inactivity penalty quotient.
	pub inactivity_penalty_quotient: Gwei,
	/// Minimal penalty quotient.
	pub min_penalty_quotient: Gwei,
	/// Maximum proposer slashings per block.
	pub max_proposer_slashings: usize,
	/// Maximum attester slashings per block.
	pub max_attester_slashings: usize,
	/// Maximum attestations per block.
	pub max_attestations: usize,
	/// Maximum deposits per block.
	pub max_deposits: usize,
	/// Maximum voluntary exits per block.
	pub max_voluntary_exits: usize,
	/// Maximum transfers per block.
	pub max_transfers: usize,
	/// Beacon block domain.
	pub domain_beacon_block: u64,
	/// Randao domain.
	pub domain_randao: u64,
	/// Attestation domain.
	pub domain_attestation: u64,
	/// Deposit domain.
	pub domain_deposit: u64,
	/// Voluntary exit domain.
	pub domain_voluntary_exit: u64,
	/// Transfer domain.
	pub domain_transfer: u64,
	/// Far future epoch.
	pub far_future_epoch: Epoch,
}

impl Config for NoVerificationConfig {
	type Hasher = KeccakHasher;

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
	fn bls_withdrawal_prefix_byte(&self) -> u8 { self.bls_withdrawal_prefix_byte[0] }
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
	/// Small config with 8 shards.
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
			bls_withdrawal_prefix_byte: [0],
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

	/// Full config with 1024 shards.
	pub fn full() -> Self {
		Self {
			shard_count: 1024,
			target_committee_size: 128,
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
			bls_withdrawal_prefix_byte: [0],
			seconds_per_slot: 6,
			min_attestation_inclusion_delay: 4,
			slots_per_epoch: 64,
			min_seed_lookahead: 1,
			activation_exit_delay: 4,
			epochs_per_eth1_voting_period: 16,
			slots_per_historical_root: 8192,
			min_validator_withdrawability_delay: 256,
			persistent_committee_period: 2048,
			latest_randao_mixes_length: 8192,
			latest_active_index_roots_length: 8192,
			latest_slashed_exit_length: 8192,
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
