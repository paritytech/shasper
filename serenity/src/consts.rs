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

use crate::{Slot, Epoch, Shard, Gwei};
use crate::util::slot_to_epoch;

pub const DEPOSIT_CONTRACT_TREE_DEPTH: usize = 32;
pub const LATEST_RANDAO_MIXES_LENGTH: usize = 8192;
pub const SHARD_COUNT: usize = 1024;
pub const SLOTS_PER_HISTORICAL_ROOT: usize = 8192;
pub const LATEST_ACTIVE_INDEX_ROOTS_LENGTH: usize = 8192;
pub const LATEST_SLASHED_EXIT_LENGTH: usize = 8192;
pub const SLOTS_PER_EPOCH: Slot = 64;
pub const DOMAIN_BEACON_BLOCK: u64 = 0;
pub const DOMAIN_RANDAO: u64 = 1;
pub const DOMAIN_ATTESTATION: u64 = 2;
pub const DOMAIN_DEPOSIT: u64 = 3;
pub const DOMAIN_VOLUNTARY_EXIT: u64 = 4;
pub const DOMAIN_TRANSFER: u64 = 5;
pub const GENESIS_SLOT: Slot = 4294967296;
pub const GENESIS_EPOCH: Epoch = slot_to_epoch(GENESIS_SLOT);
pub const ACTIVATION_EXIT_DELAY: Epoch = 4;
pub const MIN_SEED_LOOKAHEAD: Epoch = 1;
pub const GENESIS_START_SHARD: Shard = 0;
pub const GENESIS_FORK_VERSION: u64 = 0;
pub const MIN_DEPOSIT_AMOUNT: Gwei = 1_000_000_000;
pub const MAX_DEPOSIT_AMOUNT: Gwei = 32_000_000_000;
pub const SHUFFLE_ROUND_COUNT: usize = 90;
pub const MAX_INDICES_PER_SLASHABLE_VOTE: usize = 4096;
pub const WHISTLEBLOWER_REWARD_QUOTIENT: Gwei = 512;
pub const MIN_VALIDATOR_WITHDRAWABILITY_DELAY: Epoch = 256;
pub const TARGET_COMMITTEE_SIZE: usize = 128;
pub const EPOCHS_PER_ETH1_VOTING_PERIOD: Epoch = 16;
pub const BASE_REWARD_QUOTIENT: Gwei = 32;
pub const MIN_PENALTY_QUOTIENT: Gwei = 32;
pub const INACTIVITY_PENALTY_QUOTIENT: Gwei = 16_777_216;
pub const ATTESTATION_INCLUSION_REWARD_QUOTIENT: Gwei = 8;
pub const MIN_ATTESTATION_INCLUSION_DELAY: Slot = 4;
pub const EJECTION_BALANCE: Gwei = 16_000_000_000;
pub const FAR_FUTURE_EPOCH: Epoch = u64::max_value();
pub const MAX_BALANCE_CHURN_QUOTIENT: Gwei = 32;
pub const MAX_EXIT_DEQUEUES_PER_EPOCH: usize = 4;
