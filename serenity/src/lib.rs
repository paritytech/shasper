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

mod attestation;
mod block;
mod consts;
mod eth1;
mod slashing;
mod state;
mod validator;
mod util;
mod error;

pub use attestation::*;
pub use block::*;
pub use eth1::*;
pub use slashing::*;
pub use state::*;
pub use validator::*;
pub use error::*;

type Gwei = u64;
type Slot = u64;
type Epoch = u64;
type Shard = u64;
type Timestamp = u64;
type ValidatorIndex = u64;

pub fn execute_block(block: &BeaconBlock, state: &mut BeaconState) -> Result<(), Error> {
	state.update_cache();

	if state.slot > consts::GENESIS_SLOT && (state.slot + 1) % consts::SLOTS_PER_EPOCH == 0 {
		state.update_justification_and_finalization()?;
		state.update_crosslinks()?;
		state.update_eth1_period();
		state.update_rewards()?;
		state.update_ejections();
		state.update_registry_and_shuffling_data()?;
		state.update_slashings();
		state.update_exit_queue();
		state.update_finalize()?;
	}

	if state.slot != block.slot {
		state.advance_slot();
	}

	state.process_block_header(block)?;
	state.process_randao(block)?;
	state.process_eth1_data(block);

	if block.body.proposer_slashings.len() > consts::MAX_PROPOSER_SLASHINGS {
		return Err(Error::TooManyProposerSlashings)
	}
	for slashing in &block.body.proposer_slashings {
		state.push_proposer_slashing(slashing.clone())?;
	}

	if block.body.attester_slashings.len() > consts::MAX_ATTESTER_SLASHINGS {
		return Err(Error::TooManyAttesterSlashings)
	}
	for slashing in &block.body.attester_slashings {
		state.push_attester_slashing(slashing.clone())?;
	}

	if block.body.attestations.len() > consts::MAX_ATTESTATIONS {
		return Err(Error::TooManyAttestations)
	}
	for attestation in &block.body.attestations {
		state.push_attestation(attestation.clone())?;
	}

	if block.body.deposits.len() > consts::MAX_DEPOSITS {
		return Err(Error::TooManyDeposits)
	}
	for deposit in &block.body.deposits {
		state.push_deposit(deposit.clone())?;
	}

	if block.body.voluntary_exits.len() > consts::MAX_VOLUNTARY_EXITS {
		return Err(Error::TooManyVoluntaryExits)
	}
	for voluntary_exit in &block.body.voluntary_exits {
		state.push_voluntary_exit(voluntary_exit.clone())?;
	}

	if block.body.transfers.len() > consts::MAX_TRANSFERS {
		return Err(Error::TooManyTransfers)
	}
	for transfer in &block.body.transfers {
		state.push_transfer(transfer.clone())?;
	}

	Ok(())
}
