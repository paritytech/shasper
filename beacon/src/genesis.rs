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

use crate::primitives::*;
use crate::types::*;
use crate::{Config, BeaconState, BLSConfig, Error};
use bm_le::{MaxVec, Compact, tree_root};

/// Generate genesis state and genesis block from given deposits, timestamp and eth1 data.
pub fn genesis<C: Config, BLS: BLSConfig>(
	genesis_validator_deposits: &[Deposit],
	genesis_time: Uint,
	genesis_eth1_data: Eth1Data,
) -> Result<(BeaconBlock<C>, BeaconState<C>), Error> {
	let state = genesis_beacon_state::<C, BLS>(
		genesis_validator_deposits, genesis_time, genesis_eth1_data
	)?;

	Ok((BeaconBlock {
		state_root: tree_root::<C::Digest, _>(&state),
		..Default::default()
	}, state))
}

/// Generate genesis state from given deposits, timestamp, and eth1 data.
pub fn genesis_beacon_state<C: Config, BLS: BLSConfig>(
	deposits: &[Deposit],
	genesis_time: Uint,
	genesis_eth1_data: Eth1Data,
) -> Result<BeaconState<C>, Error> {
	let mut state = BeaconState {
		genesis_time,
		eth1_data: genesis_eth1_data,
		latest_block_header: BeaconBlockHeader {
			body_root: tree_root::<C::Digest, _>(
				&BeaconBlockBody::<C>::default()
			),
			..Default::default()
		},
		..BeaconState::<C>::default()
	};

	for deposit in deposits {
		state.process_deposit::<BLS>(deposit.clone())?;
	}

	for validator in state.validators.iter_mut() {
		if validator.effective_balance >= C::max_effective_balance() {
			validator.activation_eligibility_epoch = C::genesis_epoch();
			validator.activation_epoch = C::genesis_epoch();
		}
	}

	// Populate latest_active_index_roots
	let genesis_active_index_root = tree_root::<C::Digest, _>(
		&Compact(MaxVec::<_, C::ValidatorRegistryLimit>::from(
			state.active_validator_indices(C::genesis_epoch())
		))
	);
	for index in 0..C::epochs_per_historical_vector() {
		state.active_index_roots[index as usize] =
			genesis_active_index_root;
	}

	Ok(state)
}
