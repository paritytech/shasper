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

use crate::types::BeaconState;
use crate::Config;

/// Beacon state executive.
pub struct Executive<'state, 'config, C: Config> {
	/// Beacon state.
	pub state: &'state mut BeaconState,
	/// Beacon config.
	pub config: &'config C,
}
