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
use crate::types::{Deposit, Eth1Data, BeaconState, BeaconBlock};
use crate::{Config, Executive, Error};

/// Generate genesis state and genesis block from given deposits, timestamp and eth1 data.
pub fn genesis<C: Config>(
	genesis_validator_deposits: &[Deposit],
	genesis_time: Uint,
	genesis_eth1_data: Eth1Data,
	config: &C,
) -> Result<(BeaconBlock, BeaconState), Error> {
	let state = genesis_beacon_state(
		genesis_validator_deposits, genesis_time, genesis_eth1_data, config
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
	genesis_validator_deposits: &[Deposit],
	genesis_time: Uint,
	genesis_eth1_data: Eth1Data,
	config: &C,
) -> Result<BeaconState, Error> {
	let mut state = BeaconState {
		genesis_time,
		latest_eth1_data: genesis_eth1_data,
		..BeaconState::default_with_config(config)
	};

	{
		let mut executive = Executive {
			state: &mut state,
			config,
		};
		for deposit in genesis_validator_deposits {
			executive.process_deposit(deposit.clone())?;
		}

		for validator in &mut executive.state.validator_registry {
			if validator.effective_balance >= config.max_effective_balance() {
				validator.activation_eligibility_epoch = config.genesis_epoch();
				validator.activation_epoch = config.genesis_epoch();
			}
		}

		let genesis_active_index_root = H256::from_slice(
			Digestible::<C::Digest>::hash(
				&executive.active_validator_indices(config.genesis_epoch())
			).as_slice()
		);
		for index in 0..config.latest_active_index_roots_length() {
			executive.state.latest_active_index_roots[index as usize] =
				genesis_active_index_root;
		}
	}

	Ok(state)
}
