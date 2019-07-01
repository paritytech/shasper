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

use ssz::Digestible;
use crate::primitives::{H256, Uint};
use crate::types::{Deposit, Eth1Data, BeaconState, BeaconBlock, BeaconBlockHeader, BeaconBlockBody};
use crate::{Config, Executive, Error};

/// Generate genesis state and genesis block from given deposits, timestamp and eth1 data.
pub fn genesis<C: Config>(
	eth1_block_hash: H256,
	eth1_timestamp: Uint,
	deposits: &[Deposit],
	config: &C,
) -> Result<(BeaconBlock, BeaconState), Error> {
	let state = genesis_beacon_state(
		eth1_block_hash, eth1_timestamp, deposits, config
	)?;

	Ok((BeaconBlock {
		state_root: H256::from_slice(
			Digestible::<C::Digest>::hash(&state).as_slice()
		),
		..Default::default()
	}, state))
}

/// Generate genesis state from given deposits, timestamp, and eth1 data.
pub fn genesis_beacon_state<C: Config>(
	eth1_block_hash: H256,
	eth1_timestamp: Uint,
	deposits: &[Deposit],
	config: &C,
) -> Result<BeaconState, Error> {
	let mut state = BeaconState {
		genesis_time: eth1_timestamp - eth1_timestamp % consts::SECONDS_PER_DAY + 2 * SECONDS_PER_DAY,
		eth1_data: Eth1Data {
			block_hash: eth1_block_hash,
			deposit_count: deposits.len() as u64,
			..Default::default()
		},
		latest_block_header: BeaconBlockHeader {
			body_root: H256::from_slice(
				Digestible::<C::Digest>::hash(
					&BeaconBlockBody::default()
				).as_slice()
			),
			..Default::default()
		},
		..BeaconState::default_with_config(config)
	};

	{
		let mut executive = Executive {
			state: &mut state,
			config,
		};
		let leaves = deposits.iter().map(|d| d.data.clone()).collect::<Vec<_>>();
		for (index, deposit) in deposits.into_iter().enumerate() {
			let deposit_data_list = leaves[..(index + 1)].iter().cloned().collect::<Vec<_>>();
			executive.state.eth1_data.deposit_root = H256::from_slice(
				Digestible::<C::Digest>::hash(&deposit_data_list).as_slice()
			);
			executive.process_deposit(deposit.clone())?;
		}

		for (index, validator) in (&mut executive.state.validators).into_iter().enumerate() {
			let balance = state.balances[index];
			validator.effective_balance = min(
				balance - balance % config.effective_balance_increment(),
				config.max_effective_balance()
			);
			if validator.effective_balance >= config.max_effective_balance() {
				validator.activation_eligibility_epoch = config.genesis_epoch();
				validator.activation_epoch = config.genesis_epoch();
			}
		}

		// Populate latest_active_index_roots
		let active_index_root = H256::from_slice(
			Digestible::<C::Digest>::hash(
				&executive.active_validator_indices(config.genesis_epoch())
			).as_slice()
		);
		let committee_root = executive.compact_committees_root(config.genesis_epoch());
		for index in 0..config.epochs_per_historical_vector() {
			executive.state.active_index_roots[index as usize] =
				active_index_root;
			executive.state.compact_committees_roots[index as usize] =
				committee_root;
		}
	}

	Ok(state)
}
