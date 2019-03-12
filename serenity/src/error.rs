pub enum Error {
	DepositIndexMismatch,
	DepositMerkleInvalid,
	DepositProofInvalid,
	DepositWithdrawalCredentialsMismatch,
	EpochOutOfRange,
	SlotOutOfRange,
	AttestationShardInvalid,
	AttestationBitFieldInvalid,
	ValidatorNotWithdrawable,
}
