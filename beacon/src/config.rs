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

use core::cmp::max;
use digest::Digest;
use crate::primitives::{H256, Uint, Epoch, Slot, ValidatorIndex, Signature, ValidatorId};
use crate::utils::to_uint;

/// Constants used in beacon block.
pub trait Config {
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
			let flip = (pivot - index) % index_count;
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
