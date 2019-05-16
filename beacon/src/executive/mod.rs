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
use ssz::Digestible;
use crate::primitives::{H768, H256};
use crate::types::{BeaconState, BeaconBlock, UnsealedBeaconBlock, BeaconBlockBody, ProposerSlashing, AttesterSlashing, Deposit, Attestation, Transfer, VoluntaryExit, Eth1Data};
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

/// Get justified active validators from current state.
// FIXME: change `&mut` to `&`.
pub fn justified_active_validators<C: Config>(state: &mut BeaconState, config: &C) -> Vec<u64> {
	let executive = Executive {
		state, config
	};
	let current_justified_epoch = executive.state.current_justified_epoch;

	executive.active_validator_indices(current_justified_epoch)
}

/// Get current justified block root.
// FIXME: change `&mut` to `&`.
pub fn justified_root<C: Config>(state: &mut BeaconState, _config: &C) -> H256 {
	state.current_justified_root
}

/// Get block attestation vote targets.
// FIXME: change `&mut` to `&`.
pub fn block_vote_targets<C: Config>(
	block: &BeaconBlock,
	state: &mut BeaconState,
	config: &C
) -> Result<Vec<(u64, H256)>, Error> {
	let executive = Executive {
		state, config
	};

	let mut ret = Vec::new();
	for attestation in block.body.attestations.clone() {
		let indexed = executive.convert_to_indexed(attestation)?;

		for v in indexed.custody_bit_0_indices.into_iter()
			.chain(indexed.custody_bit_1_indices.into_iter())
		{
			ret.push((v, indexed.data.target_root));
		}
	}

	Ok(ret)
}

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
		..Default::default()
	};
	let mut block = UnsealedBeaconBlock {
		slot: inherent.slot,
		previous_block_root: H256::default(),
		state_root: parent_block.state_root,
		body,
	};

	let mut executive = Executive { state, config };

	while executive.state.slot < block.slot {
		executive.cache_state();

		block.previous_block_root = H256::from_slice(
			Digestible::<C::Digest>::truncated_hash(&executive.state.latest_block_header).as_slice()
		);

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

	assert!(executive.state.slot == block.slot);
	executive.process_randao(&block)?;
	executive.process_eth1_data(&block);

	Ok(block)
}

/// Apply a transaction to the block.
pub fn apply_transaction<C: Config>(block: &mut UnsealedBeaconBlock, state: &mut BeaconState, extrinsic: Transaction, config: &C) -> Result<(), Error> {
	let mut executive = Executive { state, config };

	match extrinsic {
		Transaction::ProposerSlashing(slashing) => {
			if block.body.proposer_slashings.len() >= config.max_proposer_slashings() as usize {
				return Err(Error::TooManyProposerSlashings)
			}
			executive.process_proposer_slashing(slashing.clone())?;
			block.body.proposer_slashings.push(slashing);
		},
		Transaction::AttesterSlashing(slashing) => {
			if block.body.attester_slashings.len() >= config.max_attester_slashings() as usize {
				return Err(Error::TooManyAttesterSlashings)
			}
			executive.process_attester_slashing(slashing.clone())?;
			block.body.attester_slashings.push(slashing);
		},
		Transaction::Attestation(attestation) => {
			if block.body.attestations.len() >= config.max_attestations() as usize {
				return Err(Error::TooManyAttestations)
			}
			executive.process_attestation(attestation.clone())?;
			block.body.attestations.push(attestation);
		},
		Transaction::Deposit(deposit) => {
			if block.body.deposits.len() >= config.max_deposits() as usize {
				return Err(Error::TooManyDeposits)
			}
			executive.process_deposit(deposit.clone())?;
			block.body.deposits.push(deposit);
		},
		Transaction::VoluntaryExit(voluntary_exit) => {
			if block.body.voluntary_exits.len() >= config.max_voluntary_exits() as usize {
				return Err(Error::TooManyVoluntaryExits)
			}
			executive.process_voluntary_exit(voluntary_exit.clone())?;
			block.body.voluntary_exits.push(voluntary_exit);
		},
		Transaction::Transfer(transfer) => {
			if block.body.transfers.len() >= config.max_transfers() as usize {
				return Err(Error::TooManyTransfers)
			}
			executive.process_transfer(transfer.clone())?;
			block.body.transfers.push(transfer);
		},
	}
	Ok(())
}

/// Finalize an unsealed block.
pub fn finalize_block<C: Config>(block: &mut UnsealedBeaconBlock, state: &mut BeaconState, config: &C) -> Result<(), Error> {
	let mut executive = Executive { state, config };

	if block.body.deposits.len() != min(
		config.max_deposits(),
		executive.state.latest_eth1_data.deposit_count - executive.state.deposit_index
	) as usize {
		return Err(Error::TooManyDeposits)
	}

	executive.process_block_header(block)?;

	block.state_root = H256::from_slice(
		Digestible::<C::Digest>::hash(executive.state).as_slice()
	);

	Ok(())
}
