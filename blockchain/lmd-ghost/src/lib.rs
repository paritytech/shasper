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







pub mod archive;

use blockchain::traits::{Block, BlockExecutor};
use core::hash::Hash;

pub trait JustifiableExecutor: BlockExecutor {
	type ValidatorIndex: Eq + Hash;

	fn justified_active_validators(
		&self,
		state: &mut Self::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Result<Vec<Self::ValidatorIndex>, Self::Error>;
	fn justified_block_id(
		&self,
		state: &mut Self::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Result<Option<<Self::Block as Block>::Identifier>, Self::Error>;
	fn votes(
		&self,
		block: &Self::Block,
		state: &mut Self::Externalities, // FIXME: replace `&mut` with `&`.
	) -> Result<Vec<(Self::ValidatorIndex, <Self::Block as Block>::Identifier)>, Self::Error>;
}
