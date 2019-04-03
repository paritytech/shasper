#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
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
	/// Block slot is invalid.
	BlockSlotInvalid,
	/// Block previous root is invalid.
	BlockPreviousRootInvalid,
	/// Block signature is invalid.
	BlockSignatureInvalid,
	/// Randao signature is invalid.
	RandaoSignatureInvalid,
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
}
