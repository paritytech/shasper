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

use ssz::Digestible;
use crate::primitives::H256;
use crate::types::BeaconBlockBody;
use crate::{Config, Executive, Error};

impl<'state, 'config, C: Config> Executive<'state, 'config, C> {
	/// Process randao information given in a block.
	pub fn process_randao(&mut self, body: &BeaconBlockBody) -> Result<(), Error> {
		let proposer = &self.state.validators[
			self.beacon_proposer_index()? as usize
		];

		if !self.config.bls_verify(
			&proposer.pubkey,
			&H256::from_slice(
				Digestible::<C::Digest>::hash(&self.current_epoch()).as_slice()
			),
			&body.randao_reveal,
			self.domain(self.config.domain_randao(), None)
		) {
			return Err(Error::RandaoSignatureInvalid)
		}

		let current_epoch = self.current_epoch();
		self.state.randao_mixes[
			(current_epoch % self.config.epochs_per_historical_vector()) as usize
		] = self.randao_mix(current_epoch) ^
			self.config.hash(&[&body.randao_reveal[..]]);

		Ok(())
	}
}
