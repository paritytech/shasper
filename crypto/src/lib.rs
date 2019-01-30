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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod bls {
	use bls_crate;
	use pairing::bls12_381::Bls12;

	pub type Public = bls_crate::Public<Bls12>;
	pub type Secret = bls_crate::Secret<Bls12>;
	pub type Pair = bls_crate::Pair<Bls12>;
	pub type Signature = bls_crate::Signature<Bls12>;
	pub type AggregatePublic = bls_crate::AggregatePublic<Bls12>;
	pub type AggregateSignature = bls_crate::AggregateSignature<Bls12>;
}
