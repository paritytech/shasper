// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.

#[derive(Clone, PartialEq, Eq, Debug)]
/// Error type for beacon chain.
pub enum Error {
	/// Deposit index mismatch.
	DepositIndexMismatch,
	/// Deposit merkle is invalid.
	DepositMerkleInvalid,
	/// Deposit proof is invalid.
	DepositProofInvalid,
	/// Deposit withdrawal credentials does not match.
	DepositWithdrawalCredentialsMismatch,
	/// Duplicate indexes.
	DuplicateIndexes,
	/// Duplicate transfer.
	DuplicateTransfer,
	/// Index is out of range.
	IndexOutOfRange,
	/// Epoch is out of range.
	EpochOutOfRange,
	/// Slot is out of range.
	SlotOutOfRange,
	/// Attestation shard is invalid.
	AttestationShardInvalid,
	/// Attestation bitfield is invalid.
	AttestationBitFieldInvalid,
	/// Validator is not yet withdrawable.
	ValidatorNotWithdrawable,
	/// Validator's attestation not found.
	ValidatorAttestationNotFound,
	/// Block state root is invalid.
	BlockStateRootInvalid,
	/// Block slot is invalid.
	BlockSlotInvalid,
	/// Block proposer has been slashed.
	BlockProposerSlashed,
	/// Block previous root is invalid.
	BlockPreviousRootInvalid,
	/// Block signature is invalid.
	BlockSignatureInvalid,
	/// Randao signature is invalid.
	RandaoSignatureInvalid,
	/// Proposer slashing contains invalid proposer index.
	ProposerSlashingInvalidProposerIndex,
	/// Proposer slashing contains invalid slot.
	ProposerSlashingInvalidSlot,
	/// Proposer slashing is on same header.
	ProposerSlashingSameHeader,
	/// Proposer slahsing has already been slashed.
	ProposerSlashingAlreadySlashed,
	/// Proposer slahsing contains invalid signature.
	ProposerSlashingInvalidSignature,
	/// Attester slashing is on same attestation.
	AttesterSlashingSameAttestation,
	/// Attester slashing is not slashable.
	AttesterSlashingNotSlashable,
	/// Attester slashing is invalid.
	AttesterSlashingInvalid,
	/// Attester slashing is on empty indices.
	AttesterSlashingEmptyIndices,
	/// Attestation is too far in the past.
	AttestationTooFarInHistory,
	/// Attestation submitted to quickly.
	AttestationSubmittedTooQuickly,
	/// Attestation contains incorrect justified epoch or block root.
	AttestationIncorrectJustifiedEpochOrBlockRoot,
	/// Attestation contains incorrect crosslink data.
	AttestationIncorrectCrosslinkData,
	/// Attestation has empty aggregation.
	AttestationEmptyAggregation,
	/// Attestation has empty custody.
	AttestationEmptyCustody,
	/// Attestation data is invalid.
	AttestationInvalidData,
	/// Attestation is on invalid shard.
	AttestationInvalidShard,
	/// Attestation has invalid custody.
	AttestationInvalidCustody,
	/// Attestation has invalid signature.
	AttestationInvalidSignature,
	/// Attestation has invalid crosslink.
	AttestationInvalidCrosslink,
	/// Voluntary exit has already exited.
	VoluntaryExitAlreadyExited,
	/// Voluntary exit has already been initiated.
	VoluntaryExitAlreadyInitiated,
	/// Voluntary exit is not yet valid.
	VoluntaryExitNotYetValid,
	/// Voluntary exit is not long enough.
	VoluntaryExitNotLongEnough,
	/// Voluntary exit contains invalid signature.
	VoluntaryExitInvalidSignature,
	/// Transfer does not have enough fund.
	TransferNoFund,
	/// Transfer is not on valid slot.
	TransferNotValidSlot,
	/// Transfer is not withdrawable.
	TransferNotWithdrawable,
	/// Transfer has invalid public key.
	TransferInvalidPublicKey,
	/// Transfer has invalid signature.
	TransferInvalidSignature,
	/// Too many proposer slashings in a block.
	TooManyProposerSlashings,
	/// Too many attester slashings in a block.
	TooManyAttesterSlashings,
	/// Too many attestations in a block.
	TooManyAttestations,
	/// Too many deposits in a block.
	TooManyDeposits,
	/// Too many voluntary exits in a block.
	TooManyVoluntaryExits,
	/// Too many transfers in a block.
	TooManyTransfers,
	/// Invalid eth1 data.
	InvalidEth1Data,
}
