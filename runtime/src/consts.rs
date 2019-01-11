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

pub const CYCLE_LENGTH: usize = 64;
pub const MIN_COMMITTEE_SIZE: usize = 128;
pub const SHARD_COUNT: u16 = 1024;
pub const WEI_PER_ETH: u128 = 1000000000000000000;
pub const BASE_REWARD_QUOTIENT: u128 = 32768;
pub const SQRT_E_DROP_TIME: u128 = 1048576;
pub const SLOT_DURATION: u128 = 8;
pub const MIN_DYNASTY_LENGTH: u64 = 256;

pub const TIMESTAMP_POSITION: u32 = 0;
pub const SLOT_POSITION: u32 = 1;
pub const RANDAO_REVEAL_POSITION: u32 = 2;
pub const POW_CHAIN_REF_POSITION: u32 = 3;
