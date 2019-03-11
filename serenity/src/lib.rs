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

mod attestation;
mod block;
mod consts;
mod eth1;
mod slashing;
mod state;
mod validator;
mod util;
mod error;

pub use attestation::*;
pub use block::*;
pub use eth1::*;
pub use slashing::*;
pub use state::*;
pub use validator::*;
pub use error::*;

type Gwei = u64;
type Slot = u64;
type Epoch = u64;
type Shard = u64;
type Timestamp = u64;
type ValidatorIndex = u64;
