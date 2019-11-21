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

use crate::types::BeaconBlockBody;
use crate::{Config, BeaconExecutive, Error, BLSConfig};
use bm_le::tree_root;

impl<'a, C: Config> BeaconExecutive<'a, C> {
	/// Process randao information given in a block.
	pub fn process_randao<BLS: BLSConfig>(&mut self, body: &BeaconBlockBody<C>) -> Result<(), Error> {
		let proposer = &self.validators[
			self.beacon_proposer_index()? as usize
		];

		if !BLS::verify(
			&proposer.pubkey,
			&tree_root::<C::Digest, _>(&self.current_epoch()),
			&body.randao_reveal,
			self.domain(C::domain_randao(), None)
		) {
			return Err(Error::RandaoSignatureInvalid)
		}

		let current_epoch = self.current_epoch();
		self.state.randao_mixes[
			(current_epoch % C::epochs_per_historical_vector()) as usize
		] = self.randao_mix(current_epoch) ^
			C::hash(&[&body.randao_reveal[..]]);

		Ok(())
	}
}
