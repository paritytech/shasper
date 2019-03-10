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

use primitives::{Signature, ValidatorId, H256};

pub struct Validator {
	/// BLS public key
	pub pubkey: ValidatorId,
	/// Withdrawal credentials
	pub withdrawal_credentials: H256,
	/// Epoch when validator activated
	pub activation_epoch: u64,
	/// Epoch when validator exited
	pub exit_epoch: u64,
	/// Epoch when validator is eligible to withdraw
	pub withdrawable_epoch: u64,
	/// Did the validator initiate an exit
	pub initiated_exit: bool,
	/// Was the validator slashed
	pub slashed: bool,
}

pub struct VoluntaryExit {
	/// Minimum epoch for processing exit
	pub epoch: u64,
	/// Index of the exiting validator
	pub validator_index: u64,
	/// Validator signature
	pub signature: Signature,
}

pub struct Transfer {
	/// Sender index
	pub from: u64,
	/// Recipient index
	pub to: u64,
	/// Amount in Gwei
	pub amount: u64,
	/// Fee in Gwei for block proposer
	pub fee: u64,
	/// Inclusion slot
	pub slot: u64,
	/// Sender withdrawal pubkey
	pub pubkey: ValidatorId,
	/// Sender signature
	pub signature: Signature,
}
