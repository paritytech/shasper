// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

#[cfg(feature = "serde")]
use serde_derive::{Serialize, Deserialize};

use core::cmp::max;
use core::marker::PhantomData;
use digest::Digest;
use crate::primitives::{H256, Uint, Epoch, Slot, ValidatorIndex, Signature, ValidatorId};
use crate::utils::to_uint;

/// BLS operations
pub trait BLSVerification {
	/// Verify BLS signature.
	fn verify(pubkey: &ValidatorId, message: &H256, signature: &Signature, domain: u64) -> bool;
	/// Aggregate BLS public keys.
	fn aggregate_pubkeys(pubkeys: &[ValidatorId]) -> ValidatorId;
	/// Verify multiple BLS signatures.
	fn verify_multiple(pubkeys: &[ValidatorId], messages: &[H256], signature: &Signature, domain: u64) -> bool;
}

/// Run bls without any verification.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BLSNoVerification;

impl BLSVerification for BLSNoVerification {
	fn verify(_pubkey: &ValidatorId, _message: &H256, _signature: &Signature, _domain: u64) -> bool {
		true
	}
	fn aggregate_pubkeys(_pubkeys: &[ValidatorId]) -> ValidatorId {
		ValidatorId::default()
	}
	fn verify_multiple(_pubkeys: &[ValidatorId], _messages: &[H256], _signature: &Signature, _domain: u64) -> bool {
		true
	}
}

/// Constants used in beacon block.
pub trait Config {
	/// Digest hash function.
	type Digest: Digest;

	// === Misc ===
	/// Shard count.
	fn shard_count(&self) -> Uint;
	/// Target committee size.
	fn target_committee_size(&self) -> Uint;
	/// Maximum indices per attestation.
	fn max_indices_per_attestation(&self) -> Uint;
	/// Minimum per-epoch churn limit.
	fn min_per_epoch_churn_limit(&self) -> Uint;
	/// Churn limit quotient.
	fn churn_limit_quotient(&self) -> Uint;
	/// Base rewards per epoch.
	fn base_rewards_per_epoch(&self) -> Uint;
	/// Shuffle round count.
	fn shuffle_round_count(&self) -> Uint;

	// == Deposit contract ==
	/// Deposit contract tree depth.
	fn deposit_contract_tree_depth(&self) -> Uint;

	// == Gwei values ==
	/// Minimum deposit amount.
	fn min_deposit_amount(&self) -> Uint;
	/// Maximum effective balance.
	fn max_effective_balance(&self) -> Uint;
	/// Ejection balance.
	fn ejection_balance(&self) -> Uint;
	/// Effective balance increment.
	fn effective_balance_increment(&self) -> Uint;

	// == Initial values ==
	/// Genesis slot.
	fn genesis_slot(&self) -> Uint;
	/// Genesis epoch.
	fn genesis_epoch(&self) -> Uint;
	/// Far future epoch.
	fn far_future_epoch(&self) -> Uint { u64::max_value() }
	/// BLS withdrawal prefix byte.
	fn bls_withdrawal_prefix_byte(&self) -> u8;

	// == Time parameters ==
	/// Minimum attestation inclusion delay.
	fn min_attestation_inclusion_delay(&self) -> Uint;
	/// Slots per epoch.
	fn slots_per_epoch(&self) -> Uint;
	/// Minimum seed lookahead.
	fn min_seed_lookahead(&self) -> Uint;
	/// Activation exit delay.
	fn activation_exit_delay(&self) -> Uint;
	/// Slots per eth1 voting period.
	fn slots_per_eth1_voting_period(&self) -> Uint;
	/// Slots per historical root.
	fn slots_per_historical_root(&self) -> Uint;
	/// Minimal validator withdrawability delay.
	fn min_validator_withdrawability_delay(&self) -> Uint;
	/// Persistent committee period.
	fn persistent_committee_period(&self) -> Uint;
	/// Maximum crosslink epochs.
	fn max_crosslink_epochs(&self) -> Uint;
	/// Minimum epochs to inactivity penalty.
	fn min_epochs_to_inactivity_penalty(&self) -> Uint;

	// == State list lengths ==
	/// Latest randao mixes length.
	fn latest_randao_mixes_length(&self) -> Uint;
	/// Latest active index roots length.
	fn latest_active_index_roots_length(&self) -> Uint;
	/// Latest slashed exit length.
	fn latest_slashed_exit_length(&self) -> Uint;

	// == Reward and penalty quotients ==
	/// Base reward quotient.
	fn base_reward_quotient(&self) -> Uint;
	/// Whistleblowing reward quotient.
	fn whistleblowing_reward_quotient(&self) -> Uint;
	/// Proposer reward quotient.
	fn proposer_reward_quotient(&self) -> Uint;
	/// Inactivity penalty quotient.
	fn inactivity_penalty_quotient(&self) -> Uint;
	/// Minimal slashing penalty quotient.
	fn min_slashing_penalty_quotient(&self) -> Uint;

	// == Max operations per block ==
	/// Maximum proposer slashings per block.
	fn max_proposer_slashings(&self) -> Uint;
	/// Maximum attester slashings per block.
	fn max_attester_slashings(&self) -> Uint;
	/// Maximum attestations per block.
	fn max_attestations(&self) -> Uint;
	/// Maximum deposits per block.
	fn max_deposits(&self) -> Uint;
	/// Maximum voluntary exits per block.
	fn max_voluntary_exits(&self) -> Uint;
	/// Maximum transfers per block.
	fn max_transfers(&self) -> Uint;

	// == Signature domains ==
	/// Beacon proposer domain.
	fn domain_beacon_proposer(&self) -> Uint;
	/// Randao domain.
	fn domain_randao(&self) -> Uint;
	/// Attestation domain.
	fn domain_attestation(&self) -> Uint;
	/// Deposit domain.
	fn domain_deposit(&self) -> Uint;
	/// Voluntary exit domain.
	fn domain_voluntary_exit(&self) -> Uint;
	/// Transfer domain.
	fn domain_transfer(&self) -> Uint;

	// == BLS ==
	/// Verify BLS signature.
	fn bls_verify(
		&self,
		pubkey: &ValidatorId,
		message: &H256,
		signature: &Signature,
		domain: u64
	) -> bool;
	/// Aggregate BLS public keys.
	fn bls_aggregate_pubkeys(&self, pubkeys: &[ValidatorId]) -> ValidatorId;
	/// Verify multiple BLS signatures.
	fn bls_verify_multiple(
		&self,
		pubkeys: &[ValidatorId],
		messages: &[H256],
		signature: &Signature,
		domain: u64
	) -> bool;

	// == Helpers ==
	/// Hash function.
	fn hash<A: AsRef<[u8]>, I: IntoIterator<Item=A>>(
		&self, inputs: I
	) -> H256 {
		let mut digest = Self::Digest::new();
		for input in inputs {
			digest.input(input);
		}
		H256::from_slice(digest.result().as_slice())
	}
	/// Convert slot into epoch.
	fn slot_to_epoch(&self, slot: Slot) -> Epoch {
		slot / self.slots_per_epoch()
	}
	/// Get start slot for an epoch.
	fn epoch_start_slot(&self, epoch: Epoch) -> Slot {
		epoch.saturating_mul(self.slots_per_epoch())
	}
	/// Verify merkle branch.
	fn verify_merkle_branch(
		&self, leaf: H256, proof: &[H256], depth: u64, index: u64, root: H256,
	) -> bool {
		if proof.len() as u64 != depth {
			return false
		}

		let mut value = leaf;
		for i in 0..depth {
			if index / 2u64.pow(i as u32) % 2 != 0 {
				value = self.hash(&[&proof[i as usize][..], &value[..]]);
			} else {
				value = self.hash(&[&value[..], &proof[i as usize][..]]);
			}
		}
		value == root
	}
	/// Shuffled index.
	fn shuffled_index(
		&self, mut index: Uint, index_count: Uint, seed: H256
	) -> Option<ValidatorIndex> {
		if !(index < index_count && index_count <= 2u64.pow(40)) {
			return None
		}

		// Swap or not
		// (https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf)
		// See the 'generalized domain' algorithm on page 3

		for round in 0..self.shuffle_round_count() {
			let pivot = to_uint(
				&self.hash(&[
					&seed[..],
					&round.to_le_bytes()[..1]
				])[..8]
			) % index_count;
			let flip = ((((pivot as i128 - index as i128) % index_count as i128) + index_count as i128)
				as u64) % index_count;
			let position = max(index, flip);
			let source = self.hash(&[
				&seed[..],
				&round.to_le_bytes()[..1],
				&(position / 256).to_le_bytes()[..4]
			]);
			let byte = source[((position % 256) / 8) as usize];
			let bit = (byte >> (position % 8)) % 2;
			index = if bit != 0 { flip } else { index };
		}

		Some(index)
	}
	/// Delayed activation exit epoch.
	fn delayed_activation_exit_epoch(&self, epoch: Epoch) -> Epoch {
		epoch + 1 + self.activation_exit_delay()
	}
}

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Config that does not verify BLS signature.
pub struct ParameteredConfig<BLS: BLSVerification> {
	// === Misc ===
	/// Shard count.
	pub shard_count: Uint,
	/// Target committee size.
	pub target_committee_size: Uint,
	/// Maximum indices per attestation.
	pub max_indices_per_attestation: Uint,
	/// Minimum per-epoch churn limit.
	pub min_per_epoch_churn_limit: Uint,
	/// Churn limit quotient.
	pub churn_limit_quotient: Uint,
	/// Base rewards per epoch.
	pub base_rewards_per_epoch: Uint,
	/// Shuffle round count.
	pub shuffle_round_count: Uint,

	// == Deposit contract ==
	/// Deposit contract tree depth.
	pub deposit_contract_tree_depth: Uint,

	// == Gwei values ==
	/// Minimum deposit amount.
	pub min_deposit_amount: Uint,
	/// Maximum effective balance.
	pub max_effective_balance: Uint,
	/// Ejection balance.
	pub ejection_balance: Uint,
	/// Effective balance increment.
	pub effective_balance_increment: Uint,

	// == Initial values ==
	/// Genesis slot.
	pub genesis_slot: Uint,
	/// Genesis epoch.
	pub genesis_epoch: Uint,
	/// BLS withdrawal prefix byte.
	pub bls_withdrawal_prefix_byte: u8,

	// == Time parameters ==
	/// Minimum attestation inclusion delay.
	pub min_attestation_inclusion_delay: Uint,
	/// Slots per epoch.
	pub slots_per_epoch: Uint,
	/// Minimum seed lookahead.
	pub min_seed_lookahead: Uint,
	/// Activation exit delay.
	pub activation_exit_delay: Uint,
	/// Slots per eth1 voting period.
	pub slots_per_eth1_voting_period: Uint,
	/// Slots per historical root.
	pub slots_per_historical_root: Uint,
	/// Minimal validator withdrawability delay.
	pub min_validator_withdrawability_delay: Uint,
	/// Persistent committee period.
	pub persistent_committee_period: Uint,
	/// Maximum crosslink epochs.
	pub max_crosslink_epochs: Uint,
	/// Minimum epochs to inactivity penalty.
	pub min_epochs_to_inactivity_penalty: Uint,

	// == State list lengths ==
	/// Latest randao mixes length.
	pub latest_randao_mixes_length: Uint,
	/// Latest active index roots length.
	pub latest_active_index_roots_length: Uint,
	/// Latest slashed exit length.
	pub latest_slashed_exit_length: Uint,

	// == Reward and penalty quotients ==
	/// Base reward quotient.
	pub base_reward_quotient: Uint,
	/// Whistleblowing reward quotient.
	pub whistleblowing_reward_quotient: Uint,
	/// Proposer reward quotient.
	pub proposer_reward_quotient: Uint,
	/// Inactivity penalty quotient.
	pub inactivity_penalty_quotient: Uint,
	/// Minimal slashing penalty quotient.
	pub min_slashing_penalty_quotient: Uint,

	// == Max operations per block ==
	/// Maximum proposer slashings per block.
	pub max_proposer_slashings: Uint,
	/// Maximum attester slashings per block.
	pub max_attester_slashings: Uint,
	/// Maximum attestations per block.
	pub max_attestations: Uint,
	/// Maximum deposits per block.
	pub max_deposits: Uint,
	/// Maximum voluntary exits per block.
	pub max_voluntary_exits: Uint,
	/// Maximum transfers per block.
	pub max_transfers: Uint,

	// == Signature domains ==
	/// Beacon proposer domain.
	pub domain_beacon_proposer: Uint,
	/// Randao domain.
	pub domain_randao: Uint,
	/// Attestation domain.
	pub domain_attestation: Uint,
	/// Deposit domain.
	pub domain_deposit: Uint,
	/// Voluntary exit domain.
	pub domain_voluntary_exit: Uint,
	/// Transfer domain.
	pub domain_transfer: Uint,

	#[serde(skip)]
	_marker: PhantomData<BLS>,
}

impl<BLS: BLSVerification> Config for ParameteredConfig<BLS> {
	type Digest = sha2::Sha256;

	fn shard_count(&self) -> Uint { self.shard_count }
	fn target_committee_size(&self) -> Uint { self.target_committee_size }
	fn shuffle_round_count(&self) -> Uint { self.shuffle_round_count }
	fn deposit_contract_tree_depth(&self) -> Uint { self.deposit_contract_tree_depth }
	fn min_deposit_amount(&self) -> Uint { self.min_deposit_amount }
	fn ejection_balance(&self) -> Uint { self.ejection_balance }
	fn genesis_slot(&self) -> Slot { self.genesis_slot }
	fn bls_withdrawal_prefix_byte(&self) -> u8 { self.bls_withdrawal_prefix_byte }
	fn min_attestation_inclusion_delay(&self) -> Slot { self.min_attestation_inclusion_delay }
	fn slots_per_epoch(&self) -> Slot { self.slots_per_epoch }
	fn min_seed_lookahead(&self) -> Uint { self.min_seed_lookahead }
	fn activation_exit_delay(&self) -> Uint { self.activation_exit_delay }
	fn slots_per_historical_root(&self) -> Uint { self.slots_per_historical_root }
	fn min_validator_withdrawability_delay(&self) -> Uint { self.min_validator_withdrawability_delay }
	fn persistent_committee_period(&self) -> Uint { self.persistent_committee_period }
	fn latest_randao_mixes_length(&self) -> Uint { self.latest_randao_mixes_length }
	fn latest_active_index_roots_length(&self) -> Uint { self.latest_active_index_roots_length }
	fn latest_slashed_exit_length(&self) -> Uint { self.latest_slashed_exit_length }
	fn base_reward_quotient(&self) -> Uint { self.base_reward_quotient }
	fn inactivity_penalty_quotient(&self) -> Uint { self.inactivity_penalty_quotient }
	fn max_proposer_slashings(&self) -> Uint { self.max_proposer_slashings }
	fn max_attester_slashings(&self) -> Uint { self.max_attester_slashings }
	fn max_attestations(&self) -> Uint { self.max_attestations }
	fn max_deposits(&self) -> Uint { self.max_deposits }
	fn max_voluntary_exits(&self) -> Uint { self.max_voluntary_exits }
	fn max_transfers(&self) -> Uint { self.max_transfers }
	fn domain_beacon_proposer(&self) -> Uint { self.domain_beacon_proposer }
	fn domain_randao(&self) -> Uint { self.domain_randao }
	fn domain_attestation(&self) -> Uint { self.domain_attestation }
	fn domain_deposit(&self) -> Uint { self.domain_deposit }
	fn domain_voluntary_exit(&self) -> Uint { self.domain_voluntary_exit }
	fn domain_transfer(&self) -> Uint { self.domain_transfer }

	fn max_indices_per_attestation(&self) -> Uint { self.max_indices_per_attestation }
	fn min_per_epoch_churn_limit(&self) -> Uint { self.min_per_epoch_churn_limit }
	fn churn_limit_quotient(&self) -> Uint { self.churn_limit_quotient }
	fn base_rewards_per_epoch(&self) -> Uint { self.base_rewards_per_epoch }
	fn max_effective_balance(&self) -> Uint { self.max_effective_balance }
	fn effective_balance_increment(&self) -> Uint { self.effective_balance_increment }
	fn genesis_epoch(&self) -> Uint { self.genesis_epoch }
	fn slots_per_eth1_voting_period(&self) -> Uint { self.slots_per_eth1_voting_period }
	fn max_crosslink_epochs(&self) -> Uint { self.max_crosslink_epochs }
	fn min_epochs_to_inactivity_penalty(&self) -> Uint { self.min_epochs_to_inactivity_penalty }
	fn whistleblowing_reward_quotient(&self) -> Uint { self.whistleblowing_reward_quotient }
	fn proposer_reward_quotient(&self) -> Uint { self.proposer_reward_quotient }
	fn min_slashing_penalty_quotient(&self) -> Uint { self.min_slashing_penalty_quotient }

	fn bls_verify(&self, pubkey: &ValidatorId, message: &H256, signature: &Signature, domain: u64) -> bool {
		BLS::verify(pubkey, message, signature, domain)
	}
	fn bls_aggregate_pubkeys(&self, pubkeys: &[ValidatorId]) -> ValidatorId {
		BLS::aggregate_pubkeys(pubkeys)
	}
	fn bls_verify_multiple(&self, pubkeys: &[ValidatorId], messages: &[H256], signature: &Signature, domain: u64) -> bool {
		BLS::verify_multiple(pubkeys, messages, signature, domain)
	}
}

impl<BLS: BLSVerification> ParameteredConfig<BLS> {
	/// Small config with 8 shards.
	pub fn small() -> Self {
		Self {
			shard_count: 8,
			target_committee_size: 4,
			max_indices_per_attestation: 4096,
			min_per_epoch_churn_limit: 4,
			churn_limit_quotient: 65536,
			base_rewards_per_epoch: 5,
			shuffle_round_count: 10,

			deposit_contract_tree_depth: 32,

			min_deposit_amount: 1000000000,
			max_effective_balance: 32000000000,
			ejection_balance: 16000000000,
			effective_balance_increment: 1000000000,

			genesis_slot: 0,
			genesis_epoch: 0,
			bls_withdrawal_prefix_byte: 0,

			min_attestation_inclusion_delay: 2,
			slots_per_epoch: 8,
			min_seed_lookahead: 1,
			activation_exit_delay: 4,
			slots_per_eth1_voting_period: 16,
			slots_per_historical_root: 64,
			min_validator_withdrawability_delay: 256,
			persistent_committee_period: 2048,
			max_crosslink_epochs: 64,
			min_epochs_to_inactivity_penalty: 4,

			latest_randao_mixes_length: 64,
			latest_active_index_roots_length: 64,
			latest_slashed_exit_length: 64,

			base_reward_quotient: 32,
			whistleblowing_reward_quotient: 512,
			proposer_reward_quotient: 8,
			inactivity_penalty_quotient: 33554432,
			min_slashing_penalty_quotient: 32,

			max_proposer_slashings: 16,
			max_attester_slashings: 1,
			max_attestations: 128,
			max_deposits: 16,
			max_voluntary_exits: 16,
			max_transfers: 0,

			domain_beacon_proposer: 0,
			domain_randao: 1,
			domain_attestation: 2,
			domain_deposit: 3,
			domain_voluntary_exit: 4,
			domain_transfer: 5,

			_marker: PhantomData,
		}
	}

	/// Full config with 1024 shards.
	pub fn full() -> Self {
		Self {
			shard_count: 1024,
			target_committee_size: 128,
			max_indices_per_attestation: 4096,
			min_per_epoch_churn_limit: 4,
			churn_limit_quotient: 65536,
			base_rewards_per_epoch: 5,
			shuffle_round_count: 90,

			deposit_contract_tree_depth: 32,

			min_deposit_amount: 1000000000,
			max_effective_balance: 32000000000,
			ejection_balance: 16000000000,
			effective_balance_increment: 1000000000,

			genesis_slot: 0,
			genesis_epoch: 0,
			bls_withdrawal_prefix_byte: 0,

			min_attestation_inclusion_delay: 4,
			slots_per_epoch: 64,
			min_seed_lookahead: 1,
			activation_exit_delay: 4,
			slots_per_eth1_voting_period: 1024,
			slots_per_historical_root: 8192,
			min_validator_withdrawability_delay: 256,
			persistent_committee_period: 2048,
			max_crosslink_epochs: 64,
			min_epochs_to_inactivity_penalty: 4,

			latest_randao_mixes_length: 8192,
			latest_active_index_roots_length: 8192,
			latest_slashed_exit_length: 8192,

			base_reward_quotient: 32,
			whistleblowing_reward_quotient: 512,
			proposer_reward_quotient: 8,
			inactivity_penalty_quotient: 33554432,
			min_slashing_penalty_quotient: 32,

			max_proposer_slashings: 16,
			max_attester_slashings: 1,
			max_attestations: 128,
			max_deposits: 16,
			max_voluntary_exits: 16,
			max_transfers: 0,

			domain_beacon_proposer: 0,
			domain_randao: 1,
			domain_attestation: 2,
			domain_deposit: 3,
			domain_voluntary_exit: 4,
			domain_transfer: 5,

			_marker: PhantomData,
		}
	}
}

/// Parametered no verification config.
pub type NoVerificationConfig = ParameteredConfig<BLSNoVerification>;
