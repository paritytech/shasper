use crate::primitives::*;
use crate::types::*;
use crate::{Config, BeaconState, Error, utils};
use bm_le::tree_root;
use core::cmp::Ordering;

impl<C: Config> BeaconState<C> {
	/// Get attestations with matching source at given epoch.
	pub fn matching_source_attestations(
		&self,
		epoch: Epoch
	) -> Result<Vec<PendingAttestation<C>>, Error> {
		if epoch == self.current_epoch() {
			Ok(self.current_epoch_attestations.clone().into())
		} else if epoch == self.previous_epoch() {
			Ok(self.previous_epoch_attestations.clone().into())
		} else {
			Err(Error::EpochOutOfRange)
		}
	}

	/// Get attestations with matching target at given epoch.
	pub fn matching_target_attestations(
		&self,
		epoch: Epoch
	) -> Result<Vec<PendingAttestation<C>>, Error> {
		let block_root = self.block_root(epoch)?;
		Ok(self.matching_source_attestations(epoch)?.into_iter()
		   .filter(|a| a.data.target.root == block_root)
		   .collect())
	}

	/// Get attestations with matching head at given epoch.
	pub fn matching_head_attestations(
		&self,
		epoch: Epoch
	) -> Result<Vec<PendingAttestation<C>>, Error> {
		self.matching_source_attestations(epoch)?.into_iter()
			.map(|a| {
				Ok((a.data.beacon_block_root == self.block_root_at_slot(
					self.attestation_data_slot(&a.data)?
				)?, a))
			})
			.collect::<Result<Vec<_>, _>>()
			.map(|r| {
				r.into_iter()
					.filter(|(c, _)| *c)
					.map(|(_, v)| v)
					.collect::<Vec<_>>()
			})
	}

	/// Get indices of all unslashed validators within attestations.
	pub fn unslashed_attesting_indices(
		&self, attestations: &[PendingAttestation<C>]
	) -> Result<Vec<ValidatorIndex>, Error> {
		let mut ret = Vec::new();
		for a in attestations {
			for index in self.attesting_indices(&a.data, &a.aggregation_bits)? {
				if !ret.contains(&index) {
					ret.push(index);
				}
			}
		}
		ret.retain(|index| {
			!self.validators[*index as usize].slashed
		});
		ret.sort();
		Ok(ret)
	}

	/// Get the total attesting balance given a list of attestations.
	pub fn attesting_balance(
		&self, attestations: &[PendingAttestation<C>]
	) -> Result<Gwei, Error> {
		Ok(self.total_balance(&self.unslashed_attesting_indices(attestations)?))
	}

	/// Get the winning crosslink and attesting indices.
	pub fn winning_crosslink_and_attesting_indices(
		&self, epoch: Epoch, shard: Shard
	) -> Result<(Crosslink, Vec<ValidatorIndex>), Error> {
		let attestations = self.matching_source_attestations(epoch)?.into_iter()
			.filter(|a| a.data.crosslink.shard == shard)
			.collect::<Vec<_>>();
		let crosslinks = attestations.clone().into_iter()
			.map(|a| a.data.crosslink)
			.filter(|c| {
				let current_root = tree_root::<C::Digest, _>(
					&self.current_crosslinks[shard as usize]
				);
				let root = tree_root::<C::Digest, _>(c);

				current_root == root || current_root == c.parent_root
			})
			.collect::<Vec<_>>();

		let attestations_for = |crosslink: &Crosslink| {
			attestations.clone().into_iter()
				.filter(|a| &a.data.crosslink == crosslink)
				.collect::<Vec<_>>()
		};
		let winning_crosslink = if crosslinks.len() == 0 {
			Crosslink::default()
		} else {
			crosslinks
				.iter()
				.fold(Ok(crosslinks[0].clone()), |a, b| {
					let a = a?;
					let cmp1 = self.attesting_balance(&attestations_for(&a))?
						.cmp(&self.attesting_balance(&attestations_for(b))?);
					let cmp2 = utils::compare_hash(&a.data_root, &b.data_root);

					Ok(match (cmp1, cmp2) {
						(Ordering::Greater, _) |
						(Ordering::Equal, Ordering::Greater) => a,
						_ => b.clone(),
					})
				})?
		};
		let winning_attestations = attestations_for(&winning_crosslink);

		Ok((winning_crosslink, self.unslashed_attesting_indices(&winning_attestations)?))
	}
}
