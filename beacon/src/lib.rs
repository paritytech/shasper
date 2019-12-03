// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.

//! Ethereum 2.0 (Serenity) beacon chain state transition implementation.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use beacon_primitives as primitives;

/// Types for operations and blocks.
pub mod types;
/// Constants used in beacon chain.
pub mod consts;
/// Exported beacon chain utilities.
pub mod utils;
/// Components for reuse.
pub mod components;

mod error;
mod config;
mod executive;
mod genesis;

pub use self::error::Error;
pub use self::config::{
	BLSConfig, BLSNoVerification,
	Config, MinimalConfig, MainnetConfig,
};
pub use self::executive::{BeaconState, BeaconExecutive};
pub use self::genesis::{genesis, genesis_beacon_state};

use self::primitives::{H256, H768};
use self::types::{
	Attestation, UnsealedBeaconBlock, BeaconBlockBody, BeaconBlock, VoluntaryExit, Deposit,
	Block, AttesterSlashing, ProposerSlashing, Eth1Data, SigningBeaconBlockHeader,
};
use core::cmp::min;
use bm_le::tree_root;

/// Given a block, execute based on a parent state.
pub fn execute_block<C: Config, BLS: BLSConfig>(
	block: &BeaconBlock<C>,
	state: &mut BeaconState<C>
) -> Result<(), Error> {
	let mut executive = BeaconExecutive::new(state);
	executive.state_transition::<_, BLS>(block)
}

/// Get genesis domain.
pub fn genesis_domain(domain_type: u32) -> u64 {
	utils::bls_domain(domain_type, Default::default())
}

/// Beacon block inherent.
pub struct Inherent {
	/// New RANDAO reveal.
	pub randao_reveal: H768,
	/// New eth1 data.
	pub eth1_data: Eth1Data,
}

/// Beacon block transaction.
pub enum Transaction<C: Config> {
	/// Proposer slashing.
	ProposerSlashing(ProposerSlashing),
	/// Attester slashing.
	AttesterSlashing(AttesterSlashing<C>),
	/// Attestation.
	Attestation(Attestation<C>),
	/// Deposit.
	Deposit(Deposit),
	/// Voluntary exit.
	VoluntaryExit(VoluntaryExit),
}

/// Initialize a block, and apply inherents.
pub fn initialize_block<C: Config>(
	state: &mut BeaconState<C>,
	target_slot: u64
) -> Result<(), Error> {
	let mut executive = BeaconExecutive::new(state);
	executive.process_slots(target_slot)
}

/// Apply inherent to a block.
pub fn apply_inherent<C: Config, BLS: BLSConfig>(
	parent_block: &BeaconBlock<C>,
	state: &mut BeaconState<C>,
	inherent: Inherent
) -> Result<UnsealedBeaconBlock<C>, Error> {
	let body = BeaconBlockBody {
		randao_reveal: inherent.randao_reveal,
		eth1_data: inherent.eth1_data,
		..Default::default()
	};

	let mut block = UnsealedBeaconBlock {
		slot: state.slot,
		parent_root: H256::default(),
		state_root: parent_block.state_root,
		body,
	};

	block.parent_root = tree_root::<C::Digest, _>(
		&SigningBeaconBlockHeader::from(state.latest_block_header.clone())
	);

	let mut executive = BeaconExecutive::new(state);
	executive.process_randao::<BLS>(block.body())?;
	executive.process_eth1_data(block.body());

	Ok(block)
}

/// Apply a transaction to the block.
pub fn apply_transaction<C: Config, BLS: BLSConfig>(
	block: &mut UnsealedBeaconBlock<C>,
	state: &mut BeaconState<C>,
	extrinsic: Transaction<C>,
) -> Result<(), Error> {
	let mut executive = BeaconExecutive::new(state);
	match extrinsic {
		Transaction::ProposerSlashing(slashing) => {
			if block.body.proposer_slashings.len() >= C::max_proposer_slashings() as usize {
				return Err(Error::TooManyProposerSlashings)
			}
			executive.process_proposer_slashing::<BLS>(slashing.clone())?;
			block.body.proposer_slashings.push(slashing);
		},
		Transaction::AttesterSlashing(slashing) => {
			if block.body.attester_slashings.len() >= C::max_attester_slashings() as usize {
				return Err(Error::TooManyAttesterSlashings)
			}
			executive.process_attester_slashing::<BLS>(slashing.clone())?;
			block.body.attester_slashings.push(slashing);
		},
		Transaction::Attestation(attestation) => {
			if block.body.attestations.len() >= C::max_attestations() as usize {
				return Err(Error::TooManyAttestations)
			}
			executive.process_attestation::<BLS>(attestation.clone())?;
			block.body.attestations.push(attestation);
		},
		Transaction::Deposit(deposit) => {
			if block.body.deposits.len() >= C::max_deposits() as usize {
				return Err(Error::TooManyDeposits)
			}
			executive.process_deposit::<BLS>(deposit.clone())?;
			block.body.deposits.push(deposit);
		},
		Transaction::VoluntaryExit(voluntary_exit) => {
			if block.body.voluntary_exits.len() >= C::max_voluntary_exits() as usize {
				return Err(Error::TooManyVoluntaryExits)
			}
			executive.process_voluntary_exit::<BLS>(voluntary_exit.clone())?;
			block.body.voluntary_exits.push(voluntary_exit);
		},
	}
	Ok(())
}

/// Finalize an unsealed block.
pub fn finalize_block<C: Config, BLS: BLSConfig>(
	block: &mut UnsealedBeaconBlock<C>,
	state: &mut BeaconState<C>
) -> Result<(), Error> {
	if state.eth1_data.deposit_count < state.eth1_deposit_index {
		return Err(Error::InvalidEth1Data)
	}

	if block.body.deposits.len() != min(
		C::max_deposits(),
		state.eth1_data.deposit_count - state.eth1_deposit_index
	) as usize {
		return Err(Error::TooManyDeposits)
	}

	let mut executive = BeaconExecutive::new(state);
	executive.process_block_header::<_, BLS>(block)?;

	block.state_root = tree_root::<C::Digest, _>(state);

	Ok(())
}
