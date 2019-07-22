use serde_derive::{Serialize, Deserialize};
use beacon::types::*;
use beacon::{BeaconState, Config, BLSConfig};
use crate::{TestWithBLS, run_test_with};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AttestationTest<C: Config> {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState<C>,
	pub attestation: Attestation<C>,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> TestWithBLS for AttestationTest<C> {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<BLS: BLSConfig>(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_attestation::<BLS>(self.attestation.clone())
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AttesterSlashingTest<C: Config> {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState<C>,
	pub attester_slashing: AttesterSlashing<C>,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> TestWithBLS for AttesterSlashingTest<C> {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<BLS: BLSConfig>(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_attester_slashing::<BLS>(self.attester_slashing.clone())
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "C: Config + serde::Serialize + Clone + serde::de::DeserializeOwned + 'static")]
#[serde(deny_unknown_fields)]
pub struct BlockHeaderTest<C: Config> where
	C: serde::Serialize + serde::de::DeserializeOwned
{
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState<C>,
	pub block: BeaconBlock<C>,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> TestWithBLS for BlockHeaderTest<C> where
	C: serde::Serialize + serde::de::DeserializeOwned
{
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<BLS: BLSConfig>(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_block_header::<_, BLS>(&self.block)
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DepositTest<C: Config> {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState<C>,
	pub deposit: Deposit,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> TestWithBLS for DepositTest<C> {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<BLS: BLSConfig>(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_deposit::<BLS>(self.deposit.clone())
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProposerSlashingTest<C: Config> {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState<C>,
	pub proposer_slashing: ProposerSlashing,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> TestWithBLS for ProposerSlashingTest<C> {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<BLS: BLSConfig>(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_proposer_slashing::<BLS>(self.proposer_slashing.clone())
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferTest<C: Config> {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState<C>,
	pub transfer: Transfer,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> TestWithBLS for TransferTest<C> {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<BLS: BLSConfig>(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_transfer::<BLS>(self.transfer.clone())
		});
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct VoluntaryExitTest<C: Config> {
	pub bls_setting: Option<usize>,
	pub description: String,
	pub pre: BeaconState<C>,
	pub voluntary_exit: VoluntaryExit,
	pub post: Option<BeaconState<C>>,
}

impl<C: Config> TestWithBLS for VoluntaryExitTest<C> {
	fn bls_setting(&self) -> Option<usize> { self.bls_setting }

	fn run<BLS: BLSConfig>(&self) {
		run_test_with(&self.description, &self.pre, self.post.as_ref(), |state| {
			state.process_voluntary_exit::<BLS>(self.voluntary_exit.clone())
		});
	}
}
