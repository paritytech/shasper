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

//! Non-configurable constants used throughout the specification.

use crate::primitives::Uint;

/// Far future epoch.
pub const FAR_FUTURE_EPOCH: Uint = Uint::max_value();
/// Base rewards per epoch.
pub const BASE_REWARDS_PER_EPOCH: Uint = 5;
/// Deposit contract tree depth.
pub const DEPOSIT_CONTRACT_TREE_DEPTH: Uint = 32;
/// Deposit contract tree depth type.
pub type DepositContractTreeDepth = typenum::U32;
/// Seconds per day.
pub const SECONDS_PER_DAY: Uint = 86400;
/// Justification bits length;
pub type JustificationBitsLength = typenum::U4;
