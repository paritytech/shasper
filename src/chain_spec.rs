use shasper_primitives::{ValidatorId, AccountId};
use shasper_runtime::{GenesisConfig};
use substrate_service;
use crypto::bls;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialised `ChainSpec`. This is a specialisation of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,
	/// Whatever the current runtime is, with simple Alice/Bob auths.
	LocalTestnet,
}

impl Alternative {
	/// Get an actual chain config from one of the alternatives.
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Development => ChainSpec::from_genesis(
				"Development",
				"development",
				|| {
					let alice_sec = bls::Secret::from_bytes(b"Alice").unwrap();
					let alice = bls::Pair::from_secret(alice_sec);
					let alice_id = ValidatorId::from_public(alice.public.clone());

					testnet_genesis(
						vec![
							alice_id
						], vec![
							alice_id.into()
						],
						alice_id.into()
					)
				},
				vec![],
				None,
				None,
				None,
				None
			),
			Alternative::LocalTestnet => ChainSpec::from_genesis(
				"Local Testnet",
				"local_testnet",
				|| {
					let alice_sec = bls::Secret::from_bytes(b"Alice").unwrap();
					let bob_sec = bls::Secret::from_bytes(b"Bob").unwrap();

					let alice = bls::Pair::from_secret(alice_sec);
					let bob = bls::Pair::from_secret(bob_sec);

					let alice_id = ValidatorId::from_public(alice.public.clone());
					let bob_id = ValidatorId::from_public(bob.public.clone());

					testnet_genesis(
						vec![
							alice_id,
							bob_id,
						], vec![
							alice_id.into(),
							bob_id.into(),
						],
						alice_id.into(),
					)
				},
				vec![],
				None,
				None,
				None,
				None
			),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			"local" => Some(Alternative::LocalTestnet),
			_ => None,
		}
	}
}

fn testnet_genesis(initial_authorities: Vec<ValidatorId>, _endowed_accounts: Vec<AccountId>, _upgrade_key: AccountId) -> GenesisConfig {
	GenesisConfig {
		code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/shasper_runtime.compact.wasm").to_vec(),
		authorities: initial_authorities.clone(),
	}
}
