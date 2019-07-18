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
use serde::{Serialize, Deserialize};
use crate::primitives::{H256, Uint, Signature, ValidatorId};

/// BLS operations
pub trait BLSVerification: Default + Clone + 'static {
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
#[derive(Default, Clone, PartialEq, Eq)]
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
pub trait Config: Default + Clone + 'static {
	/// Digest hash function.
	type Digest: Digest<OutputSize=typenum::U32>;
	type MaxValidatorsPerCommittee: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type SlotsPerHistoricalRoot: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type MaxProposerSlashings: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type MaxAttesterSlashings: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type MaxAttestations: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type MaxDeposits: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type MaxVoluntaryExits: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type MaxTransfers: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type HistoricalRootsLimit: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type ShardCount: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type SlotsPerEpoch: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type SlotsPerEth1VotingPeriod: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type ValidatorRegistryLimit: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type EpochsPerHistoricalVector: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type EpochsPerSlashingsVector: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;
	type MaxAttestationsPerEpoch: Unsigned + core::fmt::Debug + Clone + Eq + PartialEq + Default;

	// === Misc ===
	/// Shard count.
	fn shard_count() -> Uint { Self::ShardCount::to_u64() }
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
	fn slots_per_epoch() -> Uint { Self::SlotsPerEpoch::to_u64() }
	/// Minimum seed lookahead.
	fn min_seed_lookahead() -> Uint;
	/// Activation exit delay.
	fn activation_exit_delay() -> Uint;
	/// Slots per eth1 voting period.
	fn slots_per_eth1_voting_period() -> Uint { Self::SlotsPerEth1VotingPeriod::to_u64() }
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
	fn epochs_per_historical_vector() -> Uint { Self::EpochsPerHistoricalVector::to_u64() }
	/// Epochs per slashings vector
	fn epochs_per_slashings_vector() -> Uint { Self::EpochsPerSlashingsVector::to_u64() }
	/// Historical roots limit
	fn historical_roots_limit() -> Uint { Self::HistoricalRootsLimit::to_u64() }
	/// Validator registry limit
	fn validator_registry_limit() -> Uint { Self::ValidatorRegistryLimit::to_u64() }

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
	fn max_proposer_slashings() -> Uint { Self::MaxProposerSlashings::to_u64() }
	/// Maximum attester slashings per block.
	fn max_attester_slashings() -> Uint { Self::MaxAttesterSlashings::to_u64() }
	/// Maximum attestations per block.
	fn max_attestations() -> Uint { Self::MaxAttestations::to_u64() }
	/// Maximum deposits per block.
	fn max_deposits() -> Uint { Self::MaxDeposits::to_u64() }
	/// Maximum voluntary exits per block.
	fn max_voluntary_exits() -> Uint { Self::MaxVoluntaryExits::to_u64() }
	/// Maximum transfers per block.
	fn max_transfers() -> Uint { Self::MaxTransfers::to_u64() }

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

#[derive(Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MinimalConfig<BLS>(PhantomData<BLS>);

impl<BLS: BLSVerification> Config for MinimalConfig<BLS> {
	type Digest = sha2::Sha256;
	type MaxValidatorsPerCommittee = typenum::U4096;
	type SlotsPerHistoricalRoot = typenum::U64;
	type MaxProposerSlashings = typenum::U16;
	type MaxAttesterSlashings = typenum::U1;
	type MaxAttestations = typenum::U128;
	type MaxDeposits = typenum::U16;
	type MaxVoluntaryExits = typenum::U16;
	type MaxTransfers = typenum::U0;
	type HistoricalRootsLimit = typenum::U16777216;
	type ShardCount = typenum::U8;
	type SlotsPerEpoch = typenum::U8;
	type SlotsPerEth1VotingPeriod = typenum::U16;
	type ValidatorRegistryLimit = typenum::U1099511627776;
	type EpochsPerHistoricalVector = typenum::U64;
	type EpochsPerSlashingsVector = typenum::U64;
	type MaxAttestationsPerEpoch = typenum::Prod<Self::MaxAttestations, Self::SlotsPerEpoch>;

	// === Misc ===
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
	fn min_seed_lookahead() -> Uint { 1 }
	fn activation_exit_delay() -> Uint { 4 }
	fn min_validator_withdrawability_delay() -> Uint { 256 }
	fn persistent_committee_period() -> Uint { 2048 }
	fn max_epochs_per_crosslink() -> Uint { 4 }
	fn min_epochs_to_inactivity_penalty() -> Uint { 4 }

	// == Reward and penalty quotients ==
	fn base_reward_factor() -> Uint { 64 }
	fn whistleblower_reward_quotient() -> Uint { 512 }
	fn proposer_reward_quotient() -> Uint { 8 }
	fn inactivity_penalty_quotient() -> Uint { 33554432 }
	fn min_slashing_penalty_quotient() -> Uint { 32 }

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

#[derive(Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MainnetConfig<BLS>(PhantomData<BLS>);

impl<BLS: BLSVerification> Config for MainnetConfig<BLS> {
	type Digest = sha2::Sha256;
	type MaxValidatorsPerCommittee = typenum::U4096;
	type SlotsPerHistoricalRoot = typenum::U8192;
	type MaxProposerSlashings = typenum::U16;
	type MaxAttesterSlashings = typenum::U1;
	type MaxAttestations = typenum::U128;
	type MaxDeposits = typenum::U16;
	type MaxVoluntaryExits = typenum::U16;
	type MaxTransfers = typenum::U0;
	type HistoricalRootsLimit = typenum::U16777216;
	type ShardCount = typenum::U1024;
	type SlotsPerEpoch = typenum::U64;
	type SlotsPerEth1VotingPeriod = typenum::U1024;
	type ValidatorRegistryLimit = typenum::U1099511627776;
	type EpochsPerHistoricalVector = typenum::U65536;
	type EpochsPerSlashingsVector = typenum::U8192;
	type MaxAttestationsPerEpoch = typenum::Prod<Self::MaxAttestations, Self::SlotsPerEpoch>;

	// === Misc ===
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
	fn min_seed_lookahead() -> Uint { 1 }
	fn activation_exit_delay() -> Uint { 4 }
	fn min_validator_withdrawability_delay() -> Uint { 256 }
	fn persistent_committee_period() -> Uint { 2048 }
	fn max_epochs_per_crosslink() -> Uint { 64 }
	fn min_epochs_to_inactivity_penalty() -> Uint { 4 }

	// == Reward and penalty quotients ==
	fn base_reward_factor() -> Uint { 64 }
	fn whistleblower_reward_quotient() -> Uint { 512 }
	fn proposer_reward_quotient() -> Uint { 8 }
	fn inactivity_penalty_quotient() -> Uint { 33554432 }
	fn min_slashing_penalty_quotient() -> Uint { 32 }

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
