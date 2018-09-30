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

use primitives::H256;
use runtime_primitives::traits::{DigestItem as DigestItemT};
use rstd::prelude::*;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum Never { }

impl DigestItemT for Never {
	type Hash = H256;
	type AuthorityId = Never;
}

pub fn split(list: Vec<usize>, n: usize) -> Vec<Vec<usize>> {
	let mut ret = Vec::new();
	for i in 0..n {
		let cur = list[(list.len() * i / n)..(list.len() * (i + 1) / n)]
			.iter().cloned().collect();
		ret.push(cur);
	}
	ret
}
