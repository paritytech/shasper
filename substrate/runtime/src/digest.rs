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

use primitives::{H256, ValidatorId};
use runtime_primitives::traits;
use rstd::prelude::*;

use codec_derive::{Encode, Decode};
#[cfg(feature = "std")]
use serde_derive::Serialize;

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize))]
/// Shasper digest items.
pub enum DigestItem {
	/// System digest item announcing that authorities set has been changed
	/// in the block. Contains the new set of authorities.
	AuthoritiesChange(Vec<ValidatorId>),
	/// System digest item that contains the root of changes trie at given
	/// block. It is created for every block iff runtime supports changes
	/// trie creation.
	ChangesTrieRoot(H256),
	/// Put a Seal on it
	Seal(u64, Vec<u8>),
	/// Any 'non-system' digest item, opaque to the native code.
	Other(Vec<u8>),
}

impl traits::DigestItem for DigestItem {
	type Hash = H256;
	type AuthorityId = ValidatorId;

	fn as_authorities_change(&self) -> Option<&[Self::AuthorityId]> {
		match self {
			DigestItem::AuthoritiesChange(ref validators) => Some(validators),
			_ => None,
		}
	}

	fn as_changes_trie_root(&self) -> Option<&Self::Hash> {
		match self {
			DigestItem::ChangesTrieRoot(ref root) => Some(root),
			_ => None,
		}
	}
}
