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
use serde::{Serialize, Deserialize};

use core::marker::PhantomData;
use digest::Digest;
use crate::primitives::{H256, Uint, Signature, ValidatorId};

/// Traits that allows creation from any other config.
pub trait FromConfig {
	/// Create self from another config.
	fn from_config<C: Config>(config: &C) -> Self;
}

/// BLS operations
pub trait BLSVerification {
	/// Verify BLS signature.
	fn verify(pubkey: &ValidatorId, message: &H256, signature: &Signature, domain: u64) -> bool;
	/// Aggregate BLS public keys.
	fn aggregate_pubkeys(pubkeys: &[ValidatorId]) -> ValidatorId;
	/// Aggregate BLS signatures.
	fn aggregate_signatures(signatures: &[Signature]) -> Signature;
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
	fn aggregate_signatures(_signatures: &[Signature]) -> Signature {
		Signature::default()
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
	fn max_validators_per_committee(&self) -> Uint;
	/// Minimum per-epoch churn limit.
	fn min_per_epoch_churn_limit(&self) -> Uint;
	/// Churn limit quotient.
	fn churn_limit_quotient(&self) -> Uint;
	/// Shuffle round count.
	fn shuffle_round_count(&self) -> Uint;
	/// Min genesis active validator count.
	fn min_genesis_active_validator_count(&self) -> Uint;
	/// Min genesis time.
	fn min_genesis_time(&self) -> Uint;

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
	fn max_epochs_per_crosslink(&self) -> Uint;
	/// Minimum epochs to inactivity penalty.
	fn min_epochs_to_inactivity_penalty(&self) -> Uint;

	// == State list lengths ==
	/// Epochs per historical vector
	fn epochs_per_historical_vector(&self) -> Uint;
	/// Epochs per slashings vector
	fn epochs_per_slashings_vector(&self) -> Uint;
	/// Historical roots limit
	fn historical_roots_limit(&self) -> Uint;
	/// Validator registry limit
	fn validator_registry_limit(&self) -> Uint;

	// == Reward and penalty quotients ==
	/// Base reward quotient.
	fn base_reward_factor(&self) -> Uint;
	/// Whistleblowing reward quotient.
	fn whistleblower_reward_quotient(&self) -> Uint;
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
	/// Verify multiple BLS signatures.
	fn bls_verify_multiple(
		&self,
		pubkeys: &[ValidatorId],
		messages: &[H256],
		signature: &Signature,
		domain: u64
	) -> bool;
	/// Aggregate BLS public keys.
	fn bls_aggregate_pubkeys(&self, pubkeys: &[ValidatorId]) -> ValidatorId;

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
	pub max_validators_per_committee: Uint,
	/// Minimum per-epoch churn limit.
	pub min_per_epoch_churn_limit: Uint,
	/// Churn limit quotient.
	pub churn_limit_quotient: Uint,
	/// Shuffle round count.
	pub shuffle_round_count: Uint,
	/// Min genesis active validator count.
	pub min_genesis_active_validator_count: Uint,
	/// Min genesis time.
	pub min_genesis_time: Uint,

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
	pub bls_withdrawal_prefix_byte: [u8; 1],

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
	pub max_epochs_per_crosslink: Uint,
	/// Minimum epochs to inactivity penalty.
	pub min_epochs_to_inactivity_penalty: Uint,

	// == State list lengths ==
	/// Epochs per historical vector
	pub epochs_per_historical_vector: Uint,
	/// Epochs per slashings vector
	pub epochs_per_slashings_vector: Uint,
	/// Historical roots limit
	pub historical_roots_limit: Uint,
	/// Validator registry limit
	pub validator_registry_limit: Uint,

	// == Reward and penalty quotients ==
	/// Base reward quotient.
	pub base_reward_factor: Uint,
	/// Whistleblowing reward quotient.
	pub whistleblower_reward_quotient: Uint,
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

	#[cfg_attr(feature = "serde", serde(skip))]
	_marker: PhantomData<BLS>,
}

impl<BLS: BLSVerification> Config for ParameteredConfig<BLS> {
	type Digest = sha2::Sha256;

	// === Misc ===
	fn shard_count(&self) -> Uint { self.shard_count }
	fn target_committee_size(&self) -> Uint { self.target_committee_size }
	fn max_validators_per_committee(&self) -> Uint { self.max_validators_per_committee }
	fn min_per_epoch_churn_limit(&self) -> Uint { self.min_per_epoch_churn_limit }
	fn churn_limit_quotient(&self) -> Uint { self.churn_limit_quotient }
	fn shuffle_round_count(&self) -> Uint { self.shuffle_round_count }
	fn min_genesis_active_validator_count(&self) -> Uint { self.min_genesis_active_validator_count }
	fn min_genesis_time(&self) -> Uint { self.min_genesis_time }

	// == Gwei values ==
	fn min_deposit_amount(&self) -> Uint { self.min_deposit_amount }
	fn max_effective_balance(&self) -> Uint { self.max_effective_balance }
	fn ejection_balance(&self) -> Uint { self.ejection_balance }
	fn effective_balance_increment(&self) -> Uint { self.effective_balance_increment }

	// == Initial values ==
	fn genesis_slot(&self) -> Uint { self.genesis_slot }
	fn genesis_epoch(&self) -> Uint { self.genesis_epoch }
	fn bls_withdrawal_prefix_byte(&self) -> u8 { self.bls_withdrawal_prefix_byte[0] }

	// == Time parameters ==
	fn min_attestation_inclusion_delay(&self) -> Uint { self.min_attestation_inclusion_delay }
	fn slots_per_epoch(&self) -> Uint { self.slots_per_epoch }
	fn min_seed_lookahead(&self) -> Uint { self.min_seed_lookahead }
	fn activation_exit_delay(&self) -> Uint { self.activation_exit_delay }
	fn slots_per_eth1_voting_period(&self) -> Uint { self.slots_per_eth1_voting_period }
	fn slots_per_historical_root(&self) -> Uint { self.slots_per_historical_root }
	fn min_validator_withdrawability_delay(&self) -> Uint { self.min_validator_withdrawability_delay }
	fn persistent_committee_period(&self) -> Uint { self.persistent_committee_period }
	fn max_epochs_per_crosslink(&self) -> Uint { self.max_epochs_per_crosslink }
	fn min_epochs_to_inactivity_penalty(&self) -> Uint { self.min_epochs_to_inactivity_penalty }

	// == State list lengths ==
	fn epochs_per_historical_vector(&self) -> Uint { self.epochs_per_historical_vector }
	fn epochs_per_slashings_vector(&self) -> Uint { self.epochs_per_slashings_vector }
	fn historical_roots_limit(&self) -> Uint { self.historical_roots_limit }
	fn validator_registry_limit(&self) -> Uint { self.validator_registry_limit }

	// == Reward and penalty quotients ==
	fn base_reward_factor(&self) -> Uint { self.base_reward_factor }
	fn whistleblower_reward_quotient(&self) -> Uint { self.whistleblower_reward_quotient }
	fn proposer_reward_quotient(&self) -> Uint { self.proposer_reward_quotient }
	fn inactivity_penalty_quotient(&self) -> Uint { self.inactivity_penalty_quotient }
	fn min_slashing_penalty_quotient(&self) -> Uint { self.min_slashing_penalty_quotient }

	// == Max operations per block ==
	fn max_proposer_slashings(&self) -> Uint { self.max_proposer_slashings }
	fn max_attester_slashings(&self) -> Uint { self.max_attester_slashings }
	fn max_attestations(&self) -> Uint { self.max_attestations }
	fn max_deposits(&self) -> Uint { self.max_deposits }
	fn max_voluntary_exits(&self) -> Uint { self.max_voluntary_exits }
	fn max_transfers(&self) -> Uint { self.max_transfers }

	// == Signature domains ==
	fn domain_beacon_proposer(&self) -> Uint { self.domain_beacon_proposer }
	fn domain_randao(&self) -> Uint { self.domain_randao }
	fn domain_attestation(&self) -> Uint { self.domain_attestation }
	fn domain_deposit(&self) -> Uint { self.domain_deposit }
	fn domain_voluntary_exit(&self) -> Uint { self.domain_voluntary_exit }
	fn domain_transfer(&self) -> Uint { self.domain_transfer }

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

impl<BLS: BLSVerification> FromConfig for ParameteredConfig<BLS> {
	fn from_config<C: Config>(config: &C) -> Self {
		Self {
			// === Misc ===
			shard_count: config.shard_count(),
			target_committee_size: config.target_committee_size(),
			max_validators_per_committee: config.max_validators_per_committee(),
			min_per_epoch_churn_limit: config.min_per_epoch_churn_limit(),
			churn_limit_quotient: config.churn_limit_quotient(),
			shuffle_round_count: config.shuffle_round_count(),
			min_genesis_active_validator_count: config.min_genesis_active_validator_count(),
			min_genesis_time: config.min_genesis_time(),

			// == Gwei values ==
			min_deposit_amount: config.min_deposit_amount(),
			max_effective_balance: config.max_effective_balance(),
			ejection_balance: config.ejection_balance(),
			effective_balance_increment: config.effective_balance_increment(),

			// == Initial values ==
			genesis_slot: config.genesis_slot(),
			genesis_epoch: config.genesis_epoch(),
			bls_withdrawal_prefix_byte: [config.bls_withdrawal_prefix_byte()],

			// == Time parameters ==
			min_attestation_inclusion_delay: config.min_attestation_inclusion_delay(),
			slots_per_epoch: config.slots_per_epoch(),
			min_seed_lookahead: config.min_seed_lookahead(),
			activation_exit_delay: config.activation_exit_delay(),
			slots_per_eth1_voting_period: config.slots_per_eth1_voting_period(),
			slots_per_historical_root: config.slots_per_historical_root(),
			min_validator_withdrawability_delay: config.min_validator_withdrawability_delay(),
			persistent_committee_period: config.persistent_committee_period(),
			max_epochs_per_crosslink: config.max_epochs_per_crosslink(),
			min_epochs_to_inactivity_penalty: config.min_epochs_to_inactivity_penalty(),

			// == State list lengths ==
			epochs_per_historical_vector: config.epochs_per_historical_vector(),
			epochs_per_slashings_vector: config.epochs_per_slashings_vector(),
			historical_roots_limit: config.historical_roots_limit(),
			validator_registry_limit: config.validator_registry_limit(),

			// == Reward and penalty quotients ==
			base_reward_factor: config.base_reward_factor(),
			whistleblower_reward_quotient: config.whistleblower_reward_quotient(),
			proposer_reward_quotient: config.proposer_reward_quotient(),
			inactivity_penalty_quotient: config.inactivity_penalty_quotient(),
			min_slashing_penalty_quotient: config.min_slashing_penalty_quotient(),

			// == Max operations per block ==
			max_proposer_slashings: config.max_proposer_slashings(),
			max_attester_slashings: config.max_attester_slashings(),
			max_attestations: config.max_attestations(),
			max_deposits: config.max_deposits(),
			max_voluntary_exits: config.max_voluntary_exits(),
			max_transfers: config.max_transfers(),

			// == Signature domains ==
			domain_beacon_proposer: config.domain_beacon_proposer(),
			domain_randao: config.domain_randao(),
			domain_attestation: config.domain_attestation(),
			domain_deposit: config.domain_deposit(),
			domain_voluntary_exit: config.domain_voluntary_exit(),
			domain_transfer: config.domain_transfer(),

			_marker: PhantomData,
		}
	}
}

impl<BLS: BLSVerification> ParameteredConfig<BLS> {
	/// Small config with 8 shards.
	pub fn small() -> Self {
		Self {
			// === Misc ===
			shard_count: 8,
			target_committee_size: 4,
			max_validators_per_committee: 4096,
			min_per_epoch_churn_limit: 4,
			churn_limit_quotient: 65536,
			shuffle_round_count: 10,
			min_genesis_active_validator_count: 64,
			min_genesis_time: 1578009600,

			// == Gwei values ==
			min_deposit_amount: 1000000000,
			max_effective_balance: 32000000000,
			ejection_balance: 16000000000,
			effective_balance_increment: 1000000000,

			// == Initial values ==
			genesis_slot: 0,
			genesis_epoch: 0,
			bls_withdrawal_prefix_byte: [0x00],

			// == Time parameters ==
			min_attestation_inclusion_delay: 1,
			slots_per_epoch: 8,
			min_seed_lookahead: 1,
			activation_exit_delay: 4,
			slots_per_eth1_voting_period: 16,
			slots_per_historical_root: 64,
			min_validator_withdrawability_delay: 256,
			persistent_committee_period: 2048,
			max_epochs_per_crosslink: 4,
			min_epochs_to_inactivity_penalty: 4,

			// == State list lengths ==
			epochs_per_historical_vector: 64,
			epochs_per_slashings_vector: 64,
			historical_roots_limit: 16777216,
			validator_registry_limit: 1099511627776,

			// == Reward and penalty quotients ==
			base_reward_factor: 64,
			whistleblower_reward_quotient: 512,
			proposer_reward_quotient: 8,
			inactivity_penalty_quotient: 33554432,
			min_slashing_penalty_quotient: 32,

			// == Max operations per block ==
			max_proposer_slashings: 16,
			max_attester_slashings: 1,
			max_attestations: 128,
			max_deposits: 16,
			max_voluntary_exits: 16,
			max_transfers: 0,

			// == Signature domains ==
			domain_beacon_proposer: 0x00000000,
			domain_randao: 0x01000000,
			domain_attestation: 0x02000000,
			domain_deposit: 0x03000000,
			domain_voluntary_exit: 0x04000000,
			domain_transfer: 0x05000000,

			_marker: PhantomData,
		}
	}

	/// Full config with 1024 shards.
	pub fn full() -> Self {
		Self {
			// === Misc ===
			shard_count: 1024,
			target_committee_size: 128,
			max_validators_per_committee: 4096,
			min_per_epoch_churn_limit: 4,
			churn_limit_quotient: 65536,
			shuffle_round_count: 90,
			min_genesis_active_validator_count: 65536,
			min_genesis_time: 1578009600,

			// == Gwei values ==
			min_deposit_amount: 1000000000,
			max_effective_balance: 32000000000,
			ejection_balance: 16000000000,
			effective_balance_increment: 1000000000,

			// == Initial values ==
			genesis_slot: 0,
			genesis_epoch: 0,
			bls_withdrawal_prefix_byte: [0x00],

			// == Time parameters ==
			min_attestation_inclusion_delay: 1,
			slots_per_epoch: 64,
			min_seed_lookahead: 1,
			activation_exit_delay: 4,
			slots_per_eth1_voting_period: 1024,
			slots_per_historical_root: 8192,
			min_validator_withdrawability_delay: 256,
			persistent_committee_period: 2048,
			max_epochs_per_crosslink: 64,
			min_epochs_to_inactivity_penalty: 4,

			// == State list lengths ==
			epochs_per_historical_vector: 65536,
			epochs_per_slashings_vector: 8192,
			historical_roots_limit: 16777216,
			validator_registry_limit: 1099511627776,

			// == Reward and penalty quotients ==
			base_reward_factor: 64,
			whistleblower_reward_quotient: 512,
			proposer_reward_quotient: 8,
			inactivity_penalty_quotient: 33554432,
			min_slashing_penalty_quotient: 32,

			// == Max operations per block ==
			max_proposer_slashings: 16,
			max_attester_slashings: 1,
			max_attestations: 128,
			max_deposits: 16,
			max_voluntary_exits: 16,
			max_transfers: 0,

			// == Signature domains ==
			domain_beacon_proposer: 0x00000000,
			domain_randao: 0x01000000,
			domain_attestation: 0x02000000,
			domain_deposit: 0x03000000,
			domain_voluntary_exit: 0x04000000,
			domain_transfer: 0x05000000,

			_marker: PhantomData,
		}
	}
}

/// Parametered no verification config.
pub type NoVerificationConfig = ParameteredConfig<BLSNoVerification>;
