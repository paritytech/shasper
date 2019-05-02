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

//! Minimal beacon chain state transition implementation for Serenity.

#![cfg_attr(not(feature = "std"), no_std, feature(alloc), feature(alloc_prelude), feature(prelude_import))]

#![warn(missing_docs)]

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
mod utils;
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

use ssz::Hashable;
use core::cmp;

/// Gwei as in the currency ETH.
pub type Gwei = u64;
/// Slot type.
pub type Slot = u64;
/// Epoch type.
pub type Epoch = u64;
/// Shard type.
pub type Shard = u64;
/// Timestamp type.
pub type Timestamp = u64;
/// Index type for validators.
pub type ValidatorIndex = u64;

/// Beacon block inherent.
pub struct Inherent {
	/// New slot.
	pub slot: u64,
	/// New RANDAO reveal.
	pub randao_reveal: H768,
	/// New eth1 data.
	pub eth1_data: Eth1Data,
}

/// Beacon block transaction.
pub enum Transaction {
	/// Proposer slashing.
	ProposerSlashing(ProposerSlashing),
	/// Attester slashing.
	AttesterSlashing(AttesterSlashing),
	/// Attestation.
	Attestation(Attestation),
	/// Deposit.
	Deposit(Deposit),
	/// Voluntary exit.
	VoluntaryExit(VoluntaryExit),
	/// Transfer.
	Transfer(Transfer),
}

/// Initialize a block, and apply inherents.
pub fn initialize_block<C: Config>(parent_block: &BeaconBlock, state: &mut BeaconState, inherent: Inherent, config: &C) -> Result<UnsealedBeaconBlock, Error> {
	let body = BeaconBlockBody {
		randao_reveal: inherent.randao_reveal,
		eth1_data: inherent.eth1_data,
		proposer_slashings: Vec::new(),
		attester_slashings: Vec::new(),
		attestations: Vec::new(),
		deposits: Vec::new(),
		voluntary_exits: Vec::new(),
		transfers: Vec::new(),
	};
	let block = UnsealedBeaconBlock {
		slot: inherent.slot,
		previous_block_root: Hashable::<C::Hasher>::hash(parent_block),
		state_root: parent_block.state_root,
		body,
	};

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
	}

	assert!(executive.state().slot == block.slot);
	executive.process_randao(&block)?;
	executive.process_eth1_data(&block);

	Ok(block)
}

/// Apply a transaction to the block.
pub fn apply_transaction<C: Config>(block: &mut UnsealedBeaconBlock, state: &mut BeaconState, extrinsic: Transaction, config: &C) -> Result<(), Error> {
	let mut executive = Executive::new(state, config);

	match extrinsic {
		Transaction::ProposerSlashing(slashing) => {
			if block.body.proposer_slashings.len() >= config.max_proposer_slashings() {
				return Err(Error::TooManyProposerSlashings)
			}
			executive.push_proposer_slashing(slashing.clone())?;
			block.body.proposer_slashings.push(slashing);
		},
		Transaction::AttesterSlashing(slashing) => {
			if block.body.attester_slashings.len() >= config.max_attester_slashings() {
				return Err(Error::TooManyAttesterSlashings)
			}
			executive.push_attester_slashing(slashing.clone())?;
			block.body.attester_slashings.push(slashing);
		},
		Transaction::Attestation(attestation) => {
			if block.body.attestations.len() >= config.max_attestations() {
				return Err(Error::TooManyAttestations)
			}
			executive.push_attestation(attestation.clone())?;
			block.body.attestations.push(attestation);
		},
		Transaction::Deposit(deposit) => {
			if block.body.deposits.len() >= config.max_deposits() {
				return Err(Error::TooManyDeposits)
			}
			executive.push_deposit(deposit.clone())?;
			block.body.deposits.push(deposit);
		},
		Transaction::VoluntaryExit(voluntary_exit) => {
			if block.body.voluntary_exits.len() >= config.max_voluntary_exits() {
				return Err(Error::TooManyVoluntaryExits)
			}
			executive.push_voluntary_exit(voluntary_exit.clone())?;
			block.body.voluntary_exits.push(voluntary_exit);
		},
		Transaction::Transfer(transfer) => {
			if block.body.transfers.len() >= config.max_transfers() {
				return Err(Error::TooManyTransfers)
			}
			executive.push_transfer(transfer.clone())?;
			block.body.transfers.push(transfer);
		},
	}
	Ok(())
}

/// Finalize an unsealed block.
pub fn finalize_block<C: Config>(block: &mut UnsealedBeaconBlock, state: &mut BeaconState, config: &C) -> Result<(), Error> {
	let mut executive = Executive::new(state, config);

	if block.body.deposits.len() != cmp::min(config.max_deposits(), (executive.state().latest_eth1_data.deposit_count - executive.state().deposit_index) as usize) {
		return Err(Error::TooFewDeposits)
	}

	executive.process_block_header(block)?;
	Ok(())
}

/// Given a block, execute based on a parent state.
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
			if block.body.deposits.len() != cmp::min(config.max_deposits(), (executive.state().latest_eth1_data.deposit_count - executive.state().deposit_index) as usize) {
				return Err(Error::TooFewDeposits)
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

			executive.process_block_header(block)?;
		}
	}

	Ok(())
}
