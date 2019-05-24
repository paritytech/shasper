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
	pub use self::verification::Verification;

	mod verification {
		use crate::bls;
		use beacon::primitives::{H256, Signature, ValidatorId};
		use beacon::BLSVerification;

		#[derive(Clone, PartialEq, Eq, Debug)]
		pub struct Verification;

		impl BLSVerification for Verification {
			fn verify(pubkey: &ValidatorId, message: &H256, signature: &Signature, domain: u64) -> bool {
				let pubkey = match bls::Public::from_bytes(&pubkey[..]) {
					Ok(value) => value,
					Err(_) => return false,
				};
				let signature = match bls::Signature::from_bytes(&signature[..]) {
					Ok(value) => value,
					Err(_) => return false,
				};
				signature.verify(&message[..], domain, &pubkey)
			}
			fn aggregate_pubkeys(pubkeys: &[ValidatorId]) -> ValidatorId {
				let mut aggregated = bls::AggregatePublic::new();
				for pubkey in pubkeys {
					let pubkey = match bls::Public::from_bytes(&pubkey[..]) {
						Ok(value) => value,
						Err(_) => return ValidatorId::default(),
					};
					aggregated.add(&pubkey);
				}
				ValidatorId::from_slice(&aggregated.as_bytes()[..])
			}
			fn verify_multiple(pubkeys: &[ValidatorId], messages: &[H256], signature: &Signature, domain: u64) -> bool {
				let mut bls_messages = Vec::new();
				for message in messages {
					bls_messages.append(&mut (&message[..]).to_vec());
				}

				let bls_signature = match bls::AggregateSignature::from_bytes(&signature[..]) {
					Ok(value) => value,
					Err(_) => return false,
				};

				let mut bls_pubkeys = Vec::new();
				for pubkey in pubkeys {
					bls_pubkeys.push(match bls::AggregatePublic::from_bytes(&pubkey[..]) {
						Ok(value) => value,
						Err(_) => return false,
					});
				}

				bls_signature.verify_multiple(
					&bls_messages, domain, &bls_pubkeys.iter().collect::<Vec<_>>())
			}
		}
	}
}
