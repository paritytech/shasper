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
use runtime_primitives;

use header::Header;
use extrinsic::Extrinsic;
use spec::SpecHeader;

pub type Block = runtime_primitives::generic::Block<Header, Extrinsic>;
pub type BlockId = runtime_primitives::generic::BlockId<Block>;

pub trait BlockExt {
	fn spec_hash(&self, active_state_root: H256, crystallized_state_root: H256) -> H256;
}

impl BlockExt for Block {
	fn spec_hash(&self, active_state_root: H256, crystallized_state_root: H256) -> H256 {
		let extrinsic = &self.extrinsics[0];
		let header = &self.header;

		let spec_header = SpecHeader {
			parent_hash: header.parent_hash,
			slot_number: extrinsic.slot_number,
			randao_reveal: extrinsic.randao_reveal,
			attestations: extrinsic.attestations.clone(),
			pow_chain_ref: extrinsic.pow_chain_ref,
			active_state_root: active_state_root,
			crystallized_state_root: crystallized_state_root,
		};

		spec_header.spec_hash()
	}
}
