// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

mod helpers;
mod transition;
mod genesis;

pub use self::genesis::*;

use core::cmp::min;
use crate::types::{BeaconState, BeaconBlock};
use crate::{Config, Error};

/// Beacon state executive.
pub struct Executive<'state, 'config, C: Config> {
	/// Beacon state.
	pub state: &'state mut BeaconState,
	/// Beacon config.
	pub config: &'config C,
}

/// Given a block, execute based on a parent state.
pub fn execute_block<C: Config>(block: &BeaconBlock, state: &mut BeaconState, config: &C) -> Result<(), Error> {
	let mut executive = Executive {
		state, config
	};

	while executive.state.slot < block.slot {
		executive.cache_state();

		if (executive.state.slot + 1) % config.slots_per_epoch() == 0 {
			executive.process_justification_and_finalization()?;
			executive.process_crosslinks()?;
			executive.process_rewards_and_penalties()?;
			executive.process_registry_updates()?;
			executive.process_slashings();
			executive.process_final_updates();
		}

		executive.advance_slot();
	}

	executive.process_block_header(block)?;
	executive.process_randao(block)?;
	executive.process_eth1_data(block);

	if block.body.proposer_slashings.len() > config.max_proposer_slashings() as usize {
		return Err(Error::TooManyProposerSlashings)
	}
	for slashing in &block.body.proposer_slashings {
		executive.process_proposer_slashing(slashing.clone())?;
	}

	if block.body.attester_slashings.len() > config.max_attester_slashings() as usize {
		return Err(Error::TooManyAttesterSlashings)
	}
	for slashing in &block.body.attester_slashings {
		executive.process_attester_slashing(slashing.clone())?;
	}

	if block.body.attestations.len() > config.max_attestations() as usize {
		return Err(Error::TooManyAttestations)
	}
	for attestation in &block.body.attestations {
		executive.process_attestation(attestation.clone())?;
	}

	if block.body.deposits.len() != min(
		config.max_deposits(),
		executive.state.latest_eth1_data.deposit_count - executive.state.deposit_index
	) as usize {
		return Err(Error::TooManyDeposits)
	}
	for deposit in &block.body.deposits {
		executive.process_deposit(deposit.clone())?;
	}

	if block.body.voluntary_exits.len() > config.max_voluntary_exits() as usize {
		return Err(Error::TooManyVoluntaryExits)
	}
	for voluntary_exit in &block.body.voluntary_exits {
		executive.process_voluntary_exit(voluntary_exit.clone())?;
	}

	if block.body.transfers.len() > config.max_transfers() as usize{
		return Err(Error::TooManyTransfers)
	}
	for transfer in &block.body.transfers {
		executive.process_transfer(transfer.clone())?;
	}

	executive.verify_block_state_root(block)?;

	Ok(())
}
