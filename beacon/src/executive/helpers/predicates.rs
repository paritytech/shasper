use crate::types::*;
use crate::primitives::*;
use crate::{BeaconState, Config, Error};
use bm_le::tree_root;

impl<C: Config> BeaconState<C> {
	/// Check if ``indexed_attestation`` has valid indices and signature.
	pub fn is_valid_indexed_attestation(
		&self,
		indexed_attestation: &IndexedAttestation<C>
	) -> bool {
		let bit_0_indices = &indexed_attestation.custody_bit_0_indices;
		let bit_1_indices = &indexed_attestation.custody_bit_1_indices;

		if bit_1_indices.len() > 0 {
			return false
		}

		let total_len =
			(bit_0_indices.len() + bit_1_indices.len()) as u64;
		if !(total_len <= C::max_validators_per_committee()) {
			return false
		}

		// Ensure no duplicate indices across custody bits
		for index in bit_0_indices.as_ref() {
			if bit_1_indices.contains(index) {
				return false
			}
		}

		if !bit_0_indices.windows(2).all(|w| w[0] <= w[1]) {
			return false
		}

		if !bit_1_indices.windows(2).all(|w| w[0] <= w[1]) {
			return false
		}

		C::bls_verify_multiple(
			&[
				C::bls_aggregate_pubkeys(
					&bit_0_indices
						.iter()
						.map(|i| self.validators[*i as usize].pubkey)
						.collect::<Vec<_>>()[..]
				),
				C::bls_aggregate_pubkeys(
					&bit_1_indices
						.iter()
						.map(|i| self.validators[*i as usize].pubkey)
						.collect::<Vec<_>>()[..]
				),
			],
			&[
				tree_root::<C::Digest, _>(&AttestationDataAndCustodyBit {
					data: indexed_attestation.data.clone(),
					custody_bit: false,
				}),
				tree_root(&AttestationDataAndCustodyBit {
					data: indexed_attestation.data.clone(),
					custody_bit: true,
				}),
			],
			&indexed_attestation.signature,
			self.domain(
				C::domain_attestation(),
				Some(indexed_attestation.data.target.epoch)
			)
		)
	}
}
