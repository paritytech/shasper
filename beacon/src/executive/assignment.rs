use crate::{Config, BeaconState, Error, utils};

/// Committee assignment.
pub struct CommitteeAssignment {
	/// List of validators in the committee.
	pub validators: Vec<u64>,
	/// Shard to which the committee is assigned.
	pub shard: u64,
	/// Slot at which the committee is assigned.
	pub slot: u64,
}

impl<C: Config> BeaconState<C> {
	/// Find committee assignment at slot.
	pub fn committee_assignment(
		&self,
		epoch: u64,
		validator_index: u64
	) -> Result<Option<CommitteeAssignment>, Error> {
		let next_epoch = self.current_epoch() + 1;
		if epoch > next_epoch {
			return Err(Error::EpochOutOfRange)
		}

		let committees_per_slot =
			self.committee_count(epoch) / C::slots_per_epoch();
		let epoch_start_slot = utils::start_slot_of_epoch::<C>(epoch);
		for slot in epoch_start_slot..(epoch_start_slot + C::slots_per_epoch()) {
			let offset = committees_per_slot *
				(slot % C::slots_per_epoch());
			let slot_start_shard =
				(self.start_shard(epoch)? + offset) % C::shard_count();
			for i in 0..committees_per_slot {
				let shard = (slot_start_shard + i) % C::shard_count();
				let committee = self.crosslink_committee(epoch, shard)?;
				if committee.contains(&validator_index) {
					return Ok(Some(CommitteeAssignment {
						validators: committee,
						shard, slot,
					}))
				}
			}
		}
		Ok(None)
	}
}
