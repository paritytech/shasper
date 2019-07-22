use crate::types::*;
use crate::primitives::*;
use crate::{BeaconState, Config, Error, utils};
use vecarray::VecArray;
use bm_le::tree_root;
use core::cmp::{max, min};

impl<C: Config> BeaconState<C> {
	pub fn current_epoch(&self) -> Epoch {
		utils::epoch_of_slot::<C>(self.slot)
	}

	pub fn previous_epoch(&self) -> Epoch {
		let current_epoch = self.current_epoch();
		if current_epoch == C::genesis_epoch() {
			C::genesis_epoch()
		} else {
			current_epoch.saturating_sub(1)
		}
	}

	pub fn block_root(&self, epoch: Epoch) -> Result<H256, Error> {
		self.block_root_at_slot(utils::start_slot_of_epoch::<C>(epoch))
	}

	pub fn block_root_at_slot(&self, slot: Slot) -> Result<H256, Error> {
		if !(slot < self.slot &&
			 self.slot <= slot + C::slots_per_historical_root())
		{
			return Err(Error::SlotOutOfRange)
		}

		Ok(self.block_roots[
			(slot % C::slots_per_historical_root()) as usize
		])
	}

	pub fn randao_mix(&self, epoch: Epoch) -> H256 {
		self.randao_mixes[
			(epoch % C::epochs_per_historical_vector()) as usize
		]
	}

	pub fn active_validator_indices(&self, epoch: Uint) -> Vec<ValidatorIndex> {
		self.validators
			.iter()
			.enumerate()
			.filter(move |(_, v)| v.is_active(epoch))
			.map(|(i, _)| i as u64)
			.collect()
	}

	pub fn validator_churn_limit(&self) -> Uint {
		max(
			C::min_per_epoch_churn_limit(),
			self.active_validator_indices(self.current_epoch()).len() as u64 /
				C::churn_limit_quotient()
		)
	}

	pub fn seed(&self, epoch: Epoch) -> H256 {
		C::hash(&[
			&self.randao_mix(epoch +
							 C::epochs_per_historical_vector() -
							 C::min_seed_lookahead() - 1)[..],
			&self.active_index_roots[(epoch % C::epochs_per_historical_vector()) as usize][..],
			&utils::to_bytes(epoch)[..],
		])
	}


	pub fn committee_count(&self, epoch: Epoch) -> Uint {
		let active_validator_indices = self.active_validator_indices(epoch);
		max(
			1,
			min(
				C::shard_count() / C::slots_per_epoch(),
				active_validator_indices.len() as u64 /
					C::slots_per_epoch() /
					C::target_committee_size(),
			)
		) * C::slots_per_epoch()
	}


	pub fn crosslink_committee(
		&self, epoch: Epoch, shard: Shard
	) -> Result<Vec<ValidatorIndex>, Error> {
		let indices = self.active_validator_indices(epoch);
		let seed = self.seed(epoch);
		let index = (shard +
					 C::shard_count() - self.start_shard(epoch)?) %
			C::shard_count();
		let count = self.committee_count(epoch);

		utils::compute_committee::<C>(&indices, seed, index, count)
	}

	pub fn start_shard(&self, epoch: Epoch) -> Result<Shard, Error> {
		if !(epoch <= self.current_epoch() + 1) {
			return Err(Error::EpochOutOfRange)
		}

		let mut check_epoch = self.current_epoch() + 1;
		let mut shard = (self.start_shard +
						 self.shard_delta(self.current_epoch())) %
			C::shard_count();

		while check_epoch > epoch {
			check_epoch -= 1;
			shard = (shard + C::shard_count() -
					 self.shard_delta(check_epoch)) %
				C::shard_count();
		}

		Ok(shard)
	}

	pub fn shard_delta(&self, epoch: Epoch) -> Uint {
		min(
			self.committee_count(epoch),
			C::shard_count() -
				C::shard_count() / C::slots_per_epoch()
		)
	}

	pub fn beacon_proposer_index(&self) -> Result<ValidatorIndex, Error> {
		let epoch = self.current_epoch();
		let committees_per_slot =
			self.committee_count(epoch) / C::slots_per_epoch();
		let offset = committees_per_slot *
			(self.slot % C::slots_per_epoch());
		let shard = (self.start_shard(epoch)? + offset) %
			C::shard_count();
		let first_committee = self.crosslink_committee(epoch, shard)?;
		let seed = self.seed(epoch);

		let mut i = 0;
		loop {
			let candidate_index = first_committee[
				((epoch + i) % first_committee.len() as u64) as usize
			];
			let random_byte = C::hash(&[
				&seed[..],
				&utils::to_bytes(i / 32)[..],
			])[(i % 32) as usize];
			let effective_balance = self.validators[candidate_index as usize].effective_balance;
			if effective_balance * u8::max_value() as u64 >=
				C::max_effective_balance() * random_byte as u64
			{
				return Ok(candidate_index)
			}

			i+= 1
		}
	}

	pub fn attestation_data_slot(&self, attestation: &AttestationData) -> Result<Slot, Error> {
		let committee_count = self.committee_count(
			attestation.target.epoch
		);
		let offset = (attestation.crosslink.shard + C::shard_count() -
					  self.start_shard(attestation.target.epoch)?) %
			C::shard_count();

		Ok(utils::start_slot_of_epoch::<C>(attestation.target.epoch) +
		   offset / (committee_count / C::slots_per_epoch()))
	}

	pub fn compact_committees_root(&self, epoch: Uint) -> Result<H256, Error> {
		let mut committees = VecArray::<CompactCommittee<C>, C::ShardCount>::default();
		let start_shard = self.start_shard(epoch)?;

		for committee_number in 0..self.committee_count(epoch) {
			let shard = (start_shard + committee_number) % C::shard_count();
			for index in self.crosslink_committee(epoch, shard)? {
				let validator = &self.validators[index as usize];
				committees[shard as usize].pubkeys.push(validator.pubkey.clone());
				let compact_balance = validator.effective_balance / C::effective_balance_increment();
				let compact_validator = (index << 16) +
					(if validator.slashed { 1 } else { 0 } << 15) + compact_balance;
				committees[shard as usize].compact_validators.push(compact_validator);
			}
		}

		Ok(tree_root::<C::Digest, _>(&committees))
	}

	pub fn total_balance(&self, indices: &[ValidatorIndex]) -> Gwei {
		max(
			indices.iter().fold(0, |sum, index| {
				sum + self.validators[*index as usize].effective_balance
			}),
			1
		)
	}

	pub fn total_active_balance(&self) -> Gwei {
		self.total_balance(&self.active_validator_indices(self.current_epoch()))
	}

	pub fn domain(&self, domain_type: Uint, message_epoch: Option<Uint>) -> Uint {
		let epoch = message_epoch.unwrap_or(self.current_epoch());
		let fork_version = if epoch < self.fork.epoch {
			self.fork.previous_version
		} else {
			self.fork.current_version
		};

		utils::bls_domain(domain_type, fork_version)
	}

	pub fn indexed_attestation(
		&self,
		attestation: Attestation<C>
	) -> Result<IndexedAttestation<C>, Error> {
		let attesting_indices = self.attesting_indices(
			&attestation.data, &attestation.aggregation_bits
		)?;
		let custody_bit_1_indices = self.attesting_indices(
			&attestation.data, &attestation.custody_bits
		)?;
		let custody_bit_0_indices = attesting_indices.clone()
			.into_iter()
			.filter(|index| !custody_bit_1_indices.contains(index))
			.collect::<Vec<_>>();

		Ok(IndexedAttestation {
			data: attestation.data,
			signature: attestation.signature,
			custody_bit_0_indices: custody_bit_0_indices.into(),
			custody_bit_1_indices: custody_bit_1_indices.into(),
		})
	}

	pub fn attesting_indices(
		&self, attestation_data: &AttestationData, bitfield: &[bool],
	) -> Result<Vec<ValidatorIndex>, Error> {
		let committee = self.crosslink_committee(
			attestation_data.target.epoch, attestation_data.crosslink.shard
		)?;

		let mut ret = committee.into_iter()
			.enumerate()
			.filter(|(i, _)| bitfield[*i])
			.map(|(_, val)| val)
			.collect::<Vec<_>>();
		ret.sort();
		Ok(ret)
	}
}
