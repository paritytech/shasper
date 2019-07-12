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

use core::marker::PhantomData;
use digest::Digest;
use typenum::Unsigned;
use generic_array::ArrayLength;
use serde::{Serialize, Deserialize};
use crate::primitives::{H256, Uint, Signature, ValidatorId};

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
	type MaxValidatorsPerCommittee: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq;
	type SlotsPerHistoricalRoot: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + ArrayLength<H256>;

	// === Misc ===
	/// Shard count.
	fn shard_count() -> Uint;
	/// Target committee size.
	fn target_committee_size() -> Uint;
	/// Maximum indices per attestation.
	fn max_validators_per_committee() -> Uint { Self::MaxValidatorsPerCommittee::to_u64() }
	/// Minimum per-epoch churn limit.
	fn min_per_epoch_churn_limit() -> Uint;
	/// Churn limit quotient.
	fn churn_limit_quotient() -> Uint;
	/// Shuffle round count.
	fn shuffle_round_count() -> Uint;
	/// Min genesis active validator count.
	fn min_genesis_active_validator_count() -> Uint;
	/// Min genesis time.
	fn min_genesis_time() -> Uint;

	// == Gwei values ==
	/// Minimum deposit amount.
	fn min_deposit_amount() -> Uint;
	/// Maximum effective balance.
	fn max_effective_balance() -> Uint;
	/// Ejection balance.
	fn ejection_balance() -> Uint;
	/// Effective balance increment.
	fn effective_balance_increment() -> Uint;

	// == Initial values ==
	/// Genesis slot.
	fn genesis_slot() -> Uint;
	/// Genesis epoch.
	fn genesis_epoch() -> Uint;
	/// BLS withdrawal prefix byte.
	fn bls_withdrawal_prefix_byte() -> u8;

	// == Time parameters ==
	/// Minimum attestation inclusion delay.
	fn min_attestation_inclusion_delay() -> Uint;
	/// Slots per epoch.
	fn slots_per_epoch() -> Uint;
	/// Minimum seed lookahead.
	fn min_seed_lookahead() -> Uint;
	/// Activation exit delay.
	fn activation_exit_delay() -> Uint;
	/// Slots per eth1 voting period.
	fn slots_per_eth1_voting_period() -> Uint;
	/// Slots per historical root.
	fn slots_per_historical_root() -> Uint { Self::SlotsPerHistoricalRoot::to_u64() }
	/// Minimal validator withdrawability delay.
	fn min_validator_withdrawability_delay() -> Uint;
	/// Persistent committee period.
	fn persistent_committee_period() -> Uint;
	/// Maximum crosslink epochs.
	fn max_epochs_per_crosslink() -> Uint;
	/// Minimum epochs to inactivity penalty.
	fn min_epochs_to_inactivity_penalty() -> Uint;

	// == State list lengths ==
	/// Epochs per historical vector
	fn epochs_per_historical_vector() -> Uint;
	/// Epochs per slashings vector
	fn epochs_per_slashings_vector() -> Uint;
	/// Historical roots limit
	fn historical_roots_limit() -> Uint;
	/// Validator registry limit
	fn validator_registry_limit() -> Uint;

	// == Reward and penalty quotients ==
	/// Base reward quotient.
	fn base_reward_factor() -> Uint;
	/// Whistleblowing reward quotient.
	fn whistleblower_reward_quotient() -> Uint;
	/// Proposer reward quotient.
	fn proposer_reward_quotient() -> Uint;
	/// Inactivity penalty quotient.
	fn inactivity_penalty_quotient() -> Uint;
	/// Minimal slashing penalty quotient.
	fn min_slashing_penalty_quotient() -> Uint;

	// == Max operations per block ==
	/// Maximum proposer slashings per block.
	fn max_proposer_slashings() -> Uint;
	/// Maximum attester slashings per block.
	fn max_attester_slashings() -> Uint;
	/// Maximum attestations per block.
	fn max_attestations() -> Uint;
	/// Maximum deposits per block.
	fn max_deposits() -> Uint;
	/// Maximum voluntary exits per block.
	fn max_voluntary_exits() -> Uint;
	/// Maximum transfers per block.
	fn max_transfers() -> Uint;

	// == Signature domains ==
	/// Beacon proposer domain.
	fn domain_beacon_proposer() -> Uint;
	/// Randao domain.
	fn domain_randao() -> Uint;
	/// Attestation domain.
	fn domain_attestation() -> Uint;
	/// Deposit domain.
	fn domain_deposit() -> Uint;
	/// Voluntary exit domain.
	fn domain_voluntary_exit() -> Uint;
	/// Transfer domain.
	fn domain_transfer() -> Uint;

	// == BLS ==
	/// Verify BLS signature.
	fn bls_verify(
		pubkey: &ValidatorId,
		message: &H256,
		signature: &Signature,
		domain: u64
	) -> bool;
	/// Verify multiple BLS signatures.
	fn bls_verify_multiple(
		pubkeys: &[ValidatorId],
		messages: &[H256],
		signature: &Signature,
		domain: u64
	) -> bool;
	/// Aggregate BLS public keys.
	fn bls_aggregate_pubkeys(pubkeys: &[ValidatorId]) -> ValidatorId;

	// == Helpers ==
	/// Hash function.
	fn hash<A: AsRef<[u8]>, I: IntoIterator<Item=A>>(
		inputs: I
	) -> H256 {
		let mut digest = Self::Digest::new();
		for input in inputs {
			digest.input(input);
		}
		H256::from_slice(digest.result().as_slice())
	}
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MinimalConfig<BLS>(PhantomData<BLS>);

impl<BLS: BLSVerification> Config for MinimalConfig<BLS> {
	type Digest = sha2::Sha256;
	type MaxValidatorsPerCommittee = typenum::U4096;
	type SlotsPerHistoricalRoot = typenum::U64;

	// === Misc ===
	fn shard_count() -> Uint { 8 }
	fn target_committee_size() -> Uint { 4 }
	fn min_per_epoch_churn_limit() -> Uint { 4 }
	fn churn_limit_quotient() -> Uint { 65536 }
	fn shuffle_round_count() -> Uint { 10 }
	fn min_genesis_active_validator_count() -> Uint { 64 }
	fn min_genesis_time() -> Uint { 1578009600 }

	// == Gwei values ==
	fn min_deposit_amount() -> Uint { 1000000000 }
	fn max_effective_balance() -> Uint { 32000000000 }
	fn ejection_balance() -> Uint { 16000000000 }
	fn effective_balance_increment() -> Uint { 1000000000 }

	// == Initial values ==
	fn genesis_slot() -> Uint { 0 }
	fn genesis_epoch() -> Uint { 0 }
	fn bls_withdrawal_prefix_byte() -> u8 { 0x00 }

	// == Time parameters ==
	fn min_attestation_inclusion_delay() -> Uint { 1 }
	fn slots_per_epoch() -> Uint { 8 }
	fn min_seed_lookahead() -> Uint { 1 }
	fn activation_exit_delay() -> Uint { 4 }
	fn slots_per_eth1_voting_period() -> Uint { 16 }
	fn min_validator_withdrawability_delay() -> Uint { 256 }
	fn persistent_committee_period() -> Uint { 2048 }
	fn max_epochs_per_crosslink() -> Uint { 4 }
	fn min_epochs_to_inactivity_penalty() -> Uint { 4 }

	// == State list lengths ==
	fn epochs_per_historical_vector() -> Uint { 64 }
	fn epochs_per_slashings_vector() -> Uint { 64 }
	fn historical_roots_limit() -> Uint { 16777216 }
	fn validator_registry_limit() -> Uint { 1099511627776 }

	// == Reward and penalty quotients ==
	fn base_reward_factor() -> Uint { 64 }
	fn whistleblower_reward_quotient() -> Uint { 512 }
	fn proposer_reward_quotient() -> Uint { 8 }
	fn inactivity_penalty_quotient() -> Uint { 33554432 }
	fn min_slashing_penalty_quotient() -> Uint { 32 }

	// == Max operations per block ==
	fn max_proposer_slashings() -> Uint { 16 }
	fn max_attester_slashings() -> Uint { 1 }
	fn max_attestations() -> Uint { 128 }
	fn max_deposits() -> Uint { 16 }
	fn max_voluntary_exits() -> Uint { 16 }
	fn max_transfers() -> Uint { 0 }

	// == Signature domains ==
	fn domain_beacon_proposer() -> Uint { 0x00000000 }
	fn domain_randao() -> Uint { 0x01000000 }
	fn domain_attestation() -> Uint { 0x02000000 }
	fn domain_deposit() -> Uint { 0x03000000 }
	fn domain_voluntary_exit() -> Uint { 0x04000000 }
	fn domain_transfer() -> Uint { 0x05000000 }

	// == BLS ==
	/// Verify BLS signature.
	fn bls_verify(
		pubkey: &ValidatorId,
		message: &H256,
		signature: &Signature,
		domain: u64
	) -> bool { BLS::verify(pubkey, message, signature, domain) }
	/// Verify multiple BLS signatures.
	fn bls_verify_multiple(
		pubkeys: &[ValidatorId],
		messages: &[H256],
		signature: &Signature,
		domain: u64
	) -> bool { BLS::verify_multiple(pubkeys, messages, signature, domain) }
	/// Aggregate BLS public keys.
	fn bls_aggregate_pubkeys(pubkeys: &[ValidatorId]) -> ValidatorId {
		BLS::aggregate_pubkeys(pubkeys)
	}
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MainnetConfig<BLS>(PhantomData<BLS>);

impl<BLS: BLSVerification> Config for MainnetConfig<BLS> {
	type Digest = sha2::Sha256;
	type MaxValidatorsPerCommittee = typenum::U4096;
	type SlotsPerHistoricalRoot = typenum::U8192;

	// === Misc ===
	fn shard_count() -> Uint { 1024 }
	fn target_committee_size() -> Uint { 128 }
	fn min_per_epoch_churn_limit() -> Uint { 4 }
	fn churn_limit_quotient() -> Uint { 65536 }
	fn shuffle_round_count() -> Uint { 90 }
	fn min_genesis_active_validator_count() -> Uint { 65536 }
	fn min_genesis_time() -> Uint { 1578009600 }

	// == Gwei values ==
	fn min_deposit_amount() -> Uint { 1000000000 }
	fn max_effective_balance() -> Uint { 32000000000 }
	fn ejection_balance() -> Uint { 16000000000 }
	fn effective_balance_increment() -> Uint { 1000000000 }

	// == Initial values ==
	fn genesis_slot() -> Uint { 0 }
	fn genesis_epoch() -> Uint { 0 }
	fn bls_withdrawal_prefix_byte() -> u8 { 0x00 }

	// == Time parameters ==
	fn min_attestation_inclusion_delay() -> Uint { 1 }
	fn slots_per_epoch() -> Uint { 64 }
	fn min_seed_lookahead() -> Uint { 1 }
	fn activation_exit_delay() -> Uint { 4 }
	fn slots_per_eth1_voting_period() -> Uint { 1024 }
	fn min_validator_withdrawability_delay() -> Uint { 256 }
	fn persistent_committee_period() -> Uint { 2048 }
	fn max_epochs_per_crosslink() -> Uint { 64 }
	fn min_epochs_to_inactivity_penalty() -> Uint { 4 }

	// == State list lengths ==
	fn epochs_per_historical_vector() -> Uint { 65536 }
	fn epochs_per_slashings_vector() -> Uint { 8192 }
	fn historical_roots_limit() -> Uint { 16777216 }
	fn validator_registry_limit() -> Uint { 1099511627776 }

	// == Reward and penalty quotients ==
	fn base_reward_factor() -> Uint { 64 }
	fn whistleblower_reward_quotient() -> Uint { 512 }
	fn proposer_reward_quotient() -> Uint { 8 }
	fn inactivity_penalty_quotient() -> Uint { 33554432 }
	fn min_slashing_penalty_quotient() -> Uint { 32 }

	// == Max operations per block ==
	fn max_proposer_slashings() -> Uint { 16 }
	fn max_attester_slashings() -> Uint { 1 }
	fn max_attestations() -> Uint { 128 }
	fn max_deposits() -> Uint { 16 }
	fn max_voluntary_exits() -> Uint { 16 }
	fn max_transfers() -> Uint { 0 }

	// == Signature domains ==
	fn domain_beacon_proposer() -> Uint { 0x00000000 }
	fn domain_randao() -> Uint { 0x01000000 }
	fn domain_attestation() -> Uint { 0x02000000 }
	fn domain_deposit() -> Uint { 0x03000000 }
	fn domain_voluntary_exit() -> Uint { 0x04000000 }
	fn domain_transfer() -> Uint { 0x05000000 }

	// == BLS ==
	/// Verify BLS signature.
	fn bls_verify(
		pubkey: &ValidatorId,
		message: &H256,
		signature: &Signature,
		domain: u64
	) -> bool { BLS::verify(pubkey, message, signature, domain) }
	/// Verify multiple BLS signatures.
	fn bls_verify_multiple(
		pubkeys: &[ValidatorId],
		messages: &[H256],
		signature: &Signature,
		domain: u64
	) -> bool { BLS::verify_multiple(pubkeys, messages, signature, domain) }
	/// Aggregate BLS public keys.
	fn bls_aggregate_pubkeys(pubkeys: &[ValidatorId]) -> ValidatorId {
		BLS::aggregate_pubkeys(pubkeys)
	}
}
