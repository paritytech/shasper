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

#![cfg_attr(not(feature = "std"), no_std, feature(alloc), feature(alloc_prelude), feature(prelude_import))]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
pub(crate) mod prelude {
	pub use core::prelude::v1::*;
	pub use alloc::prelude::v1::*;
}

#[cfg(not(feature = "std"))]
#[allow(unused)]
#[prelude_import]
use crate::prelude::*;

#[cfg(feature = "parity-codec")]
extern crate parity_codec as codec;

mod attestation;
mod block;
mod eth1;
mod slashing;
mod state;
mod validator;
mod util;
mod error;
mod config;
mod executive;
mod primitives;

pub use attestation::*;
pub use block::*;
pub use eth1::*;
pub use slashing::*;
pub use state::*;
pub use validator::*;
pub use error::*;
pub use config::*;
pub use executive::*;
pub use primitives::*;

pub type Gwei = u64;
pub type Slot = u64;
pub type Epoch = u64;
pub type Shard = u64;
pub type Timestamp = u64;
pub type ValidatorIndex = u64;

pub fn execute_block<C: Config>(block: &BeaconBlock, state: &mut BeaconState, config: &C) -> Result<(), Error> {
	let mut executive = Executive::new(state, config);

	while executive.state().slot < block.slot {
		executive.update_cache();

		if (executive.state().slot + 1) % config.slots_per_epoch() == 0 {
			executive.update_justification_and_finalization()?;
			executive.update_crosslinks()?;
			executive.update_eth1_period();
			executive.update_rewards()?;
			executive.update_ejections();
			executive.update_registry_and_shuffling_data()?;
			executive.update_slashings();
			executive.update_exit_queue();
			executive.update_finalize()?;
		}

		executive.advance_slot();

		if executive.state().slot == block.slot {
			executive.process_block_header(block)?;
			executive.process_randao(block)?;
			executive.process_eth1_data(block);

			if block.body.proposer_slashings.len() > config.max_proposer_slashings() {
				return Err(Error::TooManyProposerSlashings)
			}
			for slashing in &block.body.proposer_slashings {
				executive.push_proposer_slashing(slashing.clone())?;
			}

			if block.body.attester_slashings.len() > config.max_attester_slashings() {
				return Err(Error::TooManyAttesterSlashings)
			}
			for slashing in &block.body.attester_slashings {
				executive.push_attester_slashing(slashing.clone())?;
			}

			if block.body.attestations.len() > config.max_attestations() {
				return Err(Error::TooManyAttestations)
			}
			for attestation in &block.body.attestations {
				executive.push_attestation(attestation.clone())?;
			}

			if block.body.deposits.len() > config.max_deposits() {
				return Err(Error::TooManyDeposits)
			}
			for deposit in &block.body.deposits {
				executive.push_deposit(deposit.clone())?;
			}

			if block.body.voluntary_exits.len() > config.max_voluntary_exits() {
				return Err(Error::TooManyVoluntaryExits)
			}
			for voluntary_exit in &block.body.voluntary_exits {
				executive.push_voluntary_exit(voluntary_exit.clone())?;
			}

			if block.body.transfers.len() > config.max_transfers() {
				return Err(Error::TooManyTransfers)
			}
			for transfer in &block.body.transfers {
				executive.push_transfer(transfer.clone())?;
			}
		}
	}

	Ok(())
}
