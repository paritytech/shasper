use beacon::Config;
use beacon::primitives::H256;
use beacon::types::{Attestation, AttestationDataAndCustodyBit};
use ssz::Digestible;
use std::collections::HashMap;

pub struct AttestationPool<'config, C: Config> {
	pool: HashMap<H256, Vec<Attestation>>,
	_config: &'config C,
}

impl<'config, C: Config> AttestationPool<'config, C> {
	pub fn new(_config: &'config C) -> Self {
		Self {
			pool: Default::default(),
			_config,
		}
	}

	pub fn push(&mut self, attestation: Attestation) {
		let hash = H256::from_slice(Digestible::<C::Digest>::hash(&AttestationDataAndCustodyBit {
			data: attestation.data.clone(),
			custody_bit: false,
		}).as_slice());

		self.pool.entry(hash)
			.and_modify(|existings| {
				let attestation = attestation.clone();
				let mut aggregated = false;

				for existing in existings.iter_mut() {
					let has_duplicate = {
						let mut has_duplicate = false;
						for i in 0..(existing.aggregation_bitfield.0.len() * 8) {
							if attestation.aggregation_bitfield.get_bit(i) {
								has_duplicate = true;
							}
						}
						has_duplicate
					};

					if has_duplicate {
						continue
					}

					existing.aggregation_bitfield |= attestation.aggregation_bitfield.clone();
					for i in 0..attestation.custody_bitfield.0.len() {
						assert_eq!(attestation.custody_bitfield.0[i], 0);
					}
					existing.custody_bitfield |= attestation.custody_bitfield.clone();
					existing.signature = C::aggregate_signatures(&[
						existing.signature, attestation.signature.clone()
					]);

					aggregated = true;
					break;
				}

				if !aggregated {
					existings.push(attestation);
				}
			})
			.or_insert(vec![attestation]);
    }

	pub fn pop(&mut self, key: &H256) {
		self.pool.remove(key);
	}

	pub fn iter(&self) -> impl Iterator<Item=(&H256, &Attestation)> {
		self.pool.iter().flat_map(|(h, ats)| ats.iter().map(move |at| (h, at)))
	}
}
