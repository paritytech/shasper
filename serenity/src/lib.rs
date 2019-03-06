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

use primitives::{H256, BitField, Signature, ValidatorId};

pub const DEPOSIT_CONTRACT_TREE_DEPTH: usize = 32;
pub const LATEST_RANDAO_MIXES_LENGTH: usize = 8192;
pub const SHARD_COUNT: usize = 1024;
pub const SLOTS_PER_HISTORICAL_ROOT: usize = 1024;
pub const LATEST_ACTIVE_INDEX_ROOTS_LENGTH: usize = 8192;
pub const LATEST_SLASHED_EXIT_LENGTH: usize = 8192;

pub struct ProposerSlashing {
	/// Proposer index
	pub proposer_index: u64,
	/// First proposal
	pub proposal_a: Proposal,
	/// First proposal
	pub proposal_b: Proposal,
}

pub struct AttesterSlashing {
	/// First slashable attestation
	pub slashable_attestation_a: SlashableAttestation,
	/// Second slashable attestation
	pub slashable_attestation_b: SlashableAttestation,
}

pub struct SlashableAttestation {
	/// Validator indices
	pub validator_indices: Vec<u64>,
	/// Attestation data
	pub data: AttestationData,
	/// Custody bitfield
	pub custody_bitfield: BitField,
	/// Aggregate signature
	pub aggregate_signature: Signature,
}

pub struct Attestation {
	/// Attester aggregation bitfield
	pub aggregation_bitfield: BitField,
	/// Attestation data
	pub data: AttestationData,
	/// Custody bitfield
	pub custody_bitfield: BitField,
	/// BLS aggregate signature
	pub aggregate_signature: Signature,
}

pub struct AttestationData {
	/// Slot number
	pub slot: u64,
	/// Shard number
	pub shard: u64,
	/// Root of the signed beacon block
	pub beacon_block_root: H256,
	/// Root of the ancestor at the epoch boundary
	pub epoch_boundary_root: H256,
	/// Data from the shard since the last attestation
	pub crosslink_data_root: H256,
	/// Last crosslink
	pub latest_crosslink: Crosslink,
	/// Last justified epoch in the beacon state
	pub justified_epoch: u64,
	/// Hash of the last justified beacon block
	pub justified_block_root: H256,
}

pub struct AttestationDataAndCustodyBit {
	/// Attestation data
	pub data: AttestationData,
	/// Custody bit
	pub custody_bit: bool,
}

pub struct Deposit {
	/// Branch in the deposit tree
	pub proof: [H256; DEPOSIT_CONTRACT_TREE_DEPTH],
	/// Index in the deposit tree
	pub index: u64,
	/// Data
	pub deposit_data: DepositData,
}

pub struct DepositData {
	/// Amount in Gwei
	pub amount: u64,
	/// Timestamp from deposit contract
	pub timestamp: u64,
	/// Deposit input
	pub deposit_input: DepositInput,
}

pub struct DepositInput {
	/// BLS pubkey
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// A BLS signature of this `DepositInput`
	pub proof_of_possession: Signature,
}

pub struct VoluntaryExit {
	/// Minimum epoch for processing exit
	pub epoch: u64,
	/// Index of the exiting validator
	pub validator_index: u64,
	/// Validator signature
	pub signature: Signature,
}

pub struct Transfer {
	/// Sender index
	pub from: u64,
	/// Recipient index
	pub to: u64,
	/// Amount in Gwei
	pub amount: u64,
	/// Fee in Gwei for block proposer
	pub fee: u64,
	/// Inclusion slot
	pub slot: u64,
	/// Sender withdrawal pubkey
	pub pubkey: ValidatorId,
	/// Sender signature
	pub signature: Signature,
}

pub struct BeaconBlock {
	// Header
	pub slot: u64,
	pub parent_root: H256,
	pub state_root: H256,
	pub randao_reveal: Signature,
	pub eth1_data: Eth1Data,

	/// Body
	pub body: BeaconBlockBody,
	/// Signature
	pub signature: Signature,
}

pub struct BeaconBlockBody {
	pub proposer_slashings: Vec<ProposerSlashing>,
	pub attester_slashings: Vec<AttesterSlashing>,
	pub attestations: Vec<Attestation>,
	pub deposits: Vec<Deposit>,
	pub voluntary_exits: Vec<VoluntaryExit>,
	pub transfers: Vec<Transfer>,
}

pub struct Proposal {
	/// Slot number
	pub slot: u64,
	/// Shard number (`BEACON_CHAIN_SHARD_NUMBER` for beacon chain)
	pub shard: u64,
	/// Block root
	pub block_root: H256,
	/// Signature
	pub signature: Signature,
}

pub struct BeaconState {
	// Misc
	pub slot: u64,
	pub genesis_time: u64,
	pub fork: Fork, // For versioning hard forks

	// Validator registry
	pub validator_registry: Vec<Validator>,
	pub validator_balances: Vec<u64>,
	pub validator_registry_update_epoch: u64,

	// Randomness and committees
	pub latest_randao_mixes: [H256; LATEST_RANDAO_MIXES_LENGTH],
	pub previous_shuffling_start_shard: u64,
	pub current_shuffling_start_shard: u64,
	pub previous_shuffling_epoch: u64,
	pub current_shuffling_epoch: u64,
	pub previous_shuffling_seed: H256,
	pub current_shuffling_seed: H256,

	// Finality
	pub previous_justified_epoch: u64,
	pub justified_epoch: u64,
	pub justification_bitfield: u64,
	pub finalized_epoch: u64,

	// Recent state
	pub latest_crosslinks: [Crosslink; SHARD_COUNT],
	pub latest_block_roots: [H256; SLOTS_PER_HISTORICAL_ROOT],
	pub latest_active_index_roots: [H256; LATEST_ACTIVE_INDEX_ROOTS_LENGTH],
	pub latest_slashed_balances: [u64; LATEST_SLASHED_EXIT_LENGTH], // Balances slashed at every withdrawal period
	pub latest_attestations: Vec<PendingAttestation>,
	pub batched_block_roots: Vec<H256>,

	// Ethereum 1.0 chain data
	pub latest_eth1_data: Eth1Data,
	pub eth1_data_votes: Vec<Eth1DataVote>,
	pub deposit_index: u64,
}

pub struct Validator {
	/// BLS public key
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Epoch when validator activated
	pub activation_epoch: u64,
	/// Epoch when validator exited
	pub exit_epoch: u64,
	/// Epoch when validator is eligible to withdraw
	pub withdrawable_epoch: u64,
	/// Did the validator initiate an exit
	pub initiated_exit: bool,
	/// Was the validator slashed
	pub slashed: bool,
}

pub struct Crosslink {
	/// Epoch number
	pub epoch: u64,
	/// Shard data since the previous crosslink
	pub crosslink_data_root: H256,
}

pub struct PendingAttestation {
	/// Attester aggregation bitfield
	pub aggregation_bitfield: BitField,
	/// Attestation data
	pub data: AttestationData,
	/// Custody bitfield
	pub custody_bitfield: BitField,
	/// Inclusion slot
	pub inclusion_slot: u64,
}

pub struct Fork {
	/// Previous fork version
	pub previous_version: u64,
	/// Current fork version
	pub current_version: u64,
	/// Fork epoch number
	pub epoch: u64,
}

pub struct Eth1Data {
	/// Root of the deposit tree
	pub deposit_root: H256,
	/// Block hash
	pub block_hash: H256,
}

pub struct Eth1DataVote {
	/// Data being voted for
	pub eth1_data: Eth1Data,
	/// Vote count
	pub vote_count: u64,
}
