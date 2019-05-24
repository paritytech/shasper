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

	pub type Public = bls_crate::PublicKey;
	pub type Secret = bls_crate::SecretKey;
	pub type Pair = bls_crate::Keypair;
	pub type Signature = bls_crate::Signature;
	pub type AggregatePublic = bls_crate::AggregatePublicKey;
	pub type AggregateSignature = bls_crate::AggregateSignature;
}
