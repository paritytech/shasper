//! Ethereum 2.0 (Serenity) beacon chain state transition implementation.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

/// Primitive types for integer and bytes.
pub mod primitives;
/// Types for operations and blocks.
pub mod types;
/// Constants used in beacon chain.
pub mod consts;
/// Exported beacon chain utilities.
pub mod utils;

mod error;
mod config;
mod executive;
mod genesis;

pub use self::error::*;
pub use self::config::*;
pub use self::executive::*;
pub use self::genesis::*;

use self::primitives::*;
use self::types::*;
use core::cmp::min;
use bm_le::tree_root;

/// Given a block, execute based on a parent state.
pub fn execute_block<C: Config, BLS: BLSConfig>(
	block: &BeaconBlock<C>,
	state: &mut BeaconState<C>
) -> Result<(), Error> {
	state.state_transition::<_, BLS>(block)
}

/// Get genesis domain.
pub fn genesis_domain(domain_type: u64) -> u64 {
	utils::bls_domain(domain_type, Default::default())
}

/// Beacon block inherent.
pub struct Inherent {
	/// New RANDAO reveal.
	pub randao_reveal: H768,
	/// New eth1 data.
	pub eth1_data: Eth1Data,
}

/// Beacon block transaction.
pub enum Transaction<C: Config> {
	/// Proposer slashing.
	ProposerSlashing(ProposerSlashing),
	/// Attester slashing.
	AttesterSlashing(AttesterSlashing<C>),
	/// Attestation.
	Attestation(Attestation<C>),
	/// Deposit.
	Deposit(Deposit),
	/// Voluntary exit.
	VoluntaryExit(VoluntaryExit),
	/// Transfer.
	Transfer(Transfer),
}

/// Initialize a block, and apply inherents.
pub fn initialize_block<C: Config>(
	state: &mut BeaconState<C>,
	target_slot: u64
) -> Result<(), Error> {
	state.process_slots(target_slot)
}

/// Apply inherent to a block.
pub fn apply_inherent<C: Config, BLS: BLSConfig>(
	parent_block: &BeaconBlock<C>,
	state: &mut BeaconState<C>,
	inherent: Inherent
) -> Result<UnsealedBeaconBlock<C>, Error> {
	let body = BeaconBlockBody {
		randao_reveal: inherent.randao_reveal,
		eth1_data: inherent.eth1_data,
		..Default::default()
	};

	let mut block = UnsealedBeaconBlock {
		slot: state.slot,
		parent_root: H256::default(),
		state_root: parent_block.state_root,
		body,
	};

	block.parent_root = tree_root::<C::Digest, _>(
		&SigningBeaconBlockHeader::from(state.latest_block_header.clone())
	);

	state.process_randao::<BLS>(block.body())?;
	state.process_eth1_data(block.body());

	Ok(block)
}

/// Apply a transaction to the block.
pub fn apply_transaction<C: Config, BLS: BLSConfig>(
	block: &mut UnsealedBeaconBlock<C>,
	state: &mut BeaconState<C>,
	extrinsic: Transaction<C>,
) -> Result<(), Error> {
	match extrinsic {
		Transaction::ProposerSlashing(slashing) => {
			if block.body.proposer_slashings.len() >= C::max_proposer_slashings() as usize {
				return Err(Error::TooManyProposerSlashings)
			}
			state.process_proposer_slashing::<BLS>(slashing.clone())?;
			block.body.proposer_slashings.push(slashing);
		},
		Transaction::AttesterSlashing(slashing) => {
			if block.body.attester_slashings.len() >= C::max_attester_slashings() as usize {
				return Err(Error::TooManyAttesterSlashings)
			}
			state.process_attester_slashing::<BLS>(slashing.clone())?;
			block.body.attester_slashings.push(slashing);
		},
		Transaction::Attestation(attestation) => {
			if block.body.attestations.len() >= C::max_attestations() as usize {
				return Err(Error::TooManyAttestations)
			}
			state.process_attestation::<BLS>(attestation.clone())?;
			block.body.attestations.push(attestation);
		},
		Transaction::Deposit(deposit) => {
			if block.body.deposits.len() >= C::max_deposits() as usize {
				return Err(Error::TooManyDeposits)
			}
			state.process_deposit::<BLS>(deposit.clone())?;
			block.body.deposits.push(deposit);
		},
		Transaction::VoluntaryExit(voluntary_exit) => {
			if block.body.voluntary_exits.len() >= C::max_voluntary_exits() as usize {
				return Err(Error::TooManyVoluntaryExits)
			}
			state.process_voluntary_exit::<BLS>(voluntary_exit.clone())?;
			block.body.voluntary_exits.push(voluntary_exit);
		},
		Transaction::Transfer(transfer) => {
			if block.body.transfers.len() >= C::max_transfers() as usize {
				return Err(Error::TooManyTransfers)
			}
			state.process_transfer::<BLS>(transfer.clone())?;
			block.body.transfers.push(transfer);
		},
	}
	Ok(())
}

/// Finalize an unsealed block.
pub fn finalize_block<C: Config, BLS: BLSConfig>(
	block: &mut UnsealedBeaconBlock<C>,
	state: &mut BeaconState<C>
) -> Result<(), Error> {
	if state.eth1_data.deposit_count < state.eth1_deposit_index {
		return Err(Error::InvalidEth1Data)
	}

	if block.body.deposits.len() != min(
		C::max_deposits(),
		state.eth1_data.deposit_count - state.eth1_deposit_index
	) as usize {
		return Err(Error::TooManyDeposits)
	}

	state.process_block_header::<_, BLS>(block)?;

	block.state_root = tree_root::<C::Digest, _>(state);

	Ok(())
}
