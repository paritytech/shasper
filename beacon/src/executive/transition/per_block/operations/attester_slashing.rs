use crate::primitives::*;
use crate::types::*;
use crate::{Config, BeaconState, Error, BLSConfig, utils};
use bm_le::{tree_root, MaxVec};

impl<C: Config> BeaconState<C> {
	/// Push a new `AttesterSlashing` to the state.
	pub fn process_attester_slashing<BLS: BLSConfig>(&mut self, attester_slashing: AttesterSlashing<C>) -> Result<(), Error> {
		let attestation_1 = attester_slashing.attestation_1;
		let attestation_2 = attester_slashing.attestation_2;

		if !attestation_1.data.is_slashable(&attestation_2.data) {
			return Err(Error::AttesterSlashingNotSlashable)
		}

		if !self.is_valid_indexed_attestation::<BLS>(&attestation_1) {
			return Err(Error::AttestationInvalidSignature)
		}

		if !self.is_valid_indexed_attestation::<BLS>(&attestation_2) {
			return Err(Error::AttestationInvalidSignature)
		}

		let mut slashed_any = false;
		let attesting_indices_1 = attestation_1.custody_bit_0_indices.iter().cloned()
			.chain(attestation_1.custody_bit_1_indices.iter().cloned());
		let attesting_indices_2 = attestation_2.custody_bit_0_indices.iter().cloned()
			.chain(attestation_2.custody_bit_1_indices.iter().cloned());

		let mut full = Vec::new();
		for index in attesting_indices_1.chain(attesting_indices_2) {
			if !full.contains(&index) {
				full.push(index);
			}
		}
		full.sort();

		for index in full {
			if self.validators[index as usize]
				.is_slashable(self.current_epoch())
			{
				self.slash_validator(index, None)?;
				slashed_any = true;
			}
		}

		if !slashed_any {
			return Err(Error::AttesterSlashingEmptyIndices)
		}

		Ok(())
	}
}
