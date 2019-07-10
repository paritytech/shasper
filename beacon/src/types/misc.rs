#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use ssz::Ssz;
use bm_le::{IntoTree, FromTree, VariableVec, DefaultWithConfig};
use crate::*;
use crate::primitives::*;

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Fork information.
pub struct Fork {
	/// Previous fork version
	pub previous_version: Version,
	/// Current fork version
	pub current_version: Version,
	/// Fork epoch number
	pub epoch: Uint,
}

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Checkpoint
pub struct Checkpoint {
	/// Epoch
	pub epoch: Uint,
	/// Root of the checkpoint
	pub root: H256,
}

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Validator record.
pub struct Validator {
	/// BLS public key
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Effective balance
	pub effective_balance: Uint,
	/// Was the validator slashed
	pub slashed: bool,

	// == Status epochs ==
	/// Epoch when became eligible for activation
	pub activation_eligibility_epoch: Uint,
	/// Epoch when validator activated
	pub activation_epoch: Uint,
	/// Epoch when validator exited
	pub exit_epoch: Uint,
	/// Epoch when validator is eligible to withdraw
	pub withdrawable_epoch: Uint,
}

impl Validator {
	/// Whether it is active validator.
	pub fn is_active(&self, epoch: Uint) -> bool {
		self.activation_epoch <= epoch && epoch < self.exit_epoch
	}

	/// Whether it is slashable.
	pub fn is_slashable(&self, epoch: Uint) -> bool {
		self.slashed == false &&
			self.activation_epoch <= epoch && epoch < self.withdrawable_epoch
	}
}

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Crosslink.
pub struct Crosslink {
	/// Shard number
	pub shard: Uint,
	/// Root of the previous crosslink
	pub parent_root: H256,

	// == Crosslinking data ==
	/// Crosslinking data from epoch start
	pub start_epoch: Uint,
	/// Crosslinking data to epoch end
	pub end_epoch: Uint,
	/// Root of the crosslinked shard data since the previous crosslink
	pub data_root: H256,
}

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Attestation data.
pub struct AttestationData {
	// == LMD-GHOST vote ==
	/// Root of the signed beacon block
	pub beacon_block_root: H256,

	// == FFG vote ==
	/// Source
	pub source: Checkpoint,
	/// Target
	pub target: Checkpoint,

	/// Crosslink vote
	pub crosslink: Crosslink,
}

impl AttestationData {
	/// Is slashable.
	pub fn is_slashable(&self, other: &AttestationData) -> bool {
		(self != other && self.target.epoch == other.target.epoch) ||
			(self.source.epoch < other.source.epoch &&
			 other.target.epoch < self.target.epoch)
	}
}

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Attestation data with custody bit.
pub struct AttestationDataAndCustodyBit {
	/// Attestation data
	pub data: AttestationData,
	/// Custody bit
	pub custody_bit: bool,
}

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[bm(config_trait = "Config")]
/// Indexed attestation.
pub struct IndexedAttestation {
	/// Validator indices of custody bit 0.
	pub custody_bit_0_indices: VariableVec<Uint, MaxValidatorsPerCommitteeFromConfig>,
	/// Validator indices of custody bit 1
	pub custody_bit_1_indices: VariableVec<Uint, MaxValidatorsPerCommitteeFromConfig>,
	/// Attestation data
	pub data: AttestationData,
	/// Aggregate signature
	pub signature: Signature,
}

impl<C: Config> DefaultWithConfig<C> for IndexedAttestation {
	fn default_with_config(config: &C) -> Self {
		Self {
			custody_bit_0_indices: VariableVec::default_with_config(config),
			custody_bit_1_indices: VariableVec::default_with_config(config),
			data: Default::default(),
			signature: Default::default(),
		}
	}
}

impl From<IndexedAttestation> for SigningIndexedAttestation {
	fn from(indexed: IndexedAttestation) -> Self {
		Self {
			custody_bit_0_indices: indexed.custody_bit_0_indices,
			custody_bit_1_indices: indexed.custody_bit_1_indices,
			data: indexed.data
		}
	}
}

#[derive(Ssz, IntoTree, FromTree, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[bm(config_trait = "Config")]
/// Signing indexed attestation.
pub struct SigningIndexedAttestation {
	/// Validator indices of custody bit 0.
	pub custody_bit_0_indices: VariableVec<Uint, MaxValidatorsPerCommitteeFromConfig>,
	/// Validator indices of custody bit 1
	pub custody_bit_1_indices: VariableVec<Uint, MaxValidatorsPerCommitteeFromConfig>,
	/// Attestation data
	pub data: AttestationData,
}

#[derive(Ssz, FromTree, IntoTree, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[bm(config_trait = "Config")]
/// Pending attestation.
pub struct PendingAttestation {
	/// Attester aggregation bitfield
	pub aggregation_bits: VariableVec<bool, MaxValidatorsPerCommitteeFromConfig>,
	/// Attestation data
	pub data: AttestationData,
	/// Inclusion delay
	pub inclusion_delay: Uint,
	/// Proposer index
	pub proposer_index: Uint,
}

#[derive(Ssz, FromTree, IntoTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Eth1 data.
pub struct Eth1Data {
	/// Root of the deposit tree
	pub deposit_root: H256,
	/// Total number of deposits
	pub deposit_count: Uint,
	/// Block hash
	pub block_hash: H256,
}

#[derive(Ssz, FromTree, IntoTree, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[bm(config_trait = "Config")]
/// Historical batch information.
pub struct HistoricalBatch {
	/// Block roots
	pub block_roots: VariableVec<H256, SlotsPerHistoricalRootFromConfig>,
	/// State roots
	pub state_roots: VariableVec<H256, SlotsPerHistoricalRootFromConfig>,
}

impl<C: Config> DefaultWithConfig<C> for HistoricalBatch {
	fn default_with_config(config: &C) -> Self {
		Self {
			block_roots: VariableVec::default_with_config(config),
			state_roots: VariableVec::default_with_config(config),
		}
	}
}

#[derive(Ssz, FromTree, IntoTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Deposit data.
pub struct DepositData {
	/// BLS pubkey
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Amount in Gwei
	pub amount: Uint,
	/// Container self-signature
	pub signature: Signature,
}

impl From<DepositData> for SigningDepositData {
	fn from(data: DepositData) -> Self {
		Self {
			pubkey: data.pubkey,
			withdrawal_credentials: data.withdrawal_credentials,
			amount: data.amount,
		}
	}
}

#[derive(Ssz, FromTree, IntoTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Deposit data.
pub struct SigningDepositData {
	/// BLS pubkey
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Amount in Gwei
	pub amount: Uint,
}

#[derive(Ssz, FromTree, IntoTree, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[bm(config_trait = "Config")]
/// Compact committee
pub struct CompactCommittee {
	/// BLS pubkeys
	pub pubkeys: VariableVec<ValidatorId, MaxValidatorsPerCommitteeFromConfig>,
	/// Compact validators
	pub compact_validators: VariableVec<Uint, MaxValidatorsPerCommitteeFromConfig>,
}

impl<C: Config> DefaultWithConfig<C> for CompactCommittee {
	fn default_with_config(config: &C) -> Self {
		Self {
			pubkeys: VariableVec::default_with_config(config),
			compact_validators: VariableVec::default_with_config(config),
		}
	}
}

#[derive(Ssz, FromTree, IntoTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Beacon block header.
pub struct BeaconBlockHeader {
	/// Slot of the block.
    pub slot: Uint,
	/// Previous block root.
    pub parent_root: H256,
	/// State root.
    pub state_root: H256,
	/// Block body root.
    pub body_root: H256,
	/// Signature.
    pub signature: Signature,
}

impl From<BeaconBlockHeader> for SigningBeaconBlockHeader {
	fn from(header: BeaconBlockHeader) -> Self {
		Self {
			slot: header.slot,
			parent_root: header.parent_root,
			state_root: header.state_root,
			body_root: header.body_root,
		}
	}
}

#[derive(Ssz, FromTree, IntoTree, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(deny_unknown_fields))]
#[cfg_attr(feature = "std", derive(Debug))]
/// Beacon block header.
pub struct SigningBeaconBlockHeader {
	/// Slot of the block.
    pub slot: Uint,
	/// Previous block root.
    pub parent_root: H256,
	/// State root.
    pub state_root: H256,
	/// Block body root.
    pub body_root: H256,
}
