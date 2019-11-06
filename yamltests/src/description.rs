use std::str::FromStr;
use std::path::{Path, PathBuf};
use std::fs;
use crate::{test_name, Error};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TestPhase {
	Phase0,
}

impl FromStr for TestPhase {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"phase0" => Ok(Self::Phase0),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TestNetwork {
	General,
	Mainnet,
	Minimal,
}

impl FromStr for TestNetwork {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"general" => Ok(Self::General),
			"mainnet" => Ok(Self::Mainnet),
			"minimal" => Ok(Self::Minimal),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TestType {
	Bls(BLSType),
	SszGeneric(SszGenericType),
	EpochProcessing(EpochProcessingType),
	Genesis(GenesisType),
	Operations(OperationsType),
	Sanity(SanityType),
	Shuffling(ShufflingType),
	SszStatic(SszStaticType),
}

impl FromStr for TestType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		let s = s.split("/").collect::<Vec<_>>();
		if s.len() != 2 {
			return Err(Error::InvalidType)
		}

		match s[0] {
			"bls" => Ok(Self::Bls(FromStr::from_str(s[1])?)),
			"ssz_generic" => Ok(Self::SszGeneric(FromStr::from_str(s[1])?)),
			"epoch_processing" => Ok(Self::EpochProcessing(FromStr::from_str(s[1])?)),
			"genesis" => Ok(Self::Genesis(FromStr::from_str(s[1])?)),
			"operations" => Ok(Self::Operations(FromStr::from_str(s[1])?)),
			"sanity" => Ok(Self::Sanity(FromStr::from_str(s[1])?)),
			"shuffling" => Ok(Self::Shuffling(FromStr::from_str(s[1])?)),
			"ssz_static" => Ok(Self::SszStatic(FromStr::from_str(s[1])?)),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BLSType {
	AggregatePubkeys,
	AggregateSigs,
	MsgHashCompressed,
	MsgHashUncompressed,
	PrivToPub,
	SignMsg,
}

impl FromStr for BLSType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"aggregate_pubkeys" => Ok(Self::AggregatePubkeys),
			"aggregate_sigs" => Ok(Self::AggregateSigs),
			"msg_hash_compressed" => Ok(Self::MsgHashCompressed),
			"msg_hash_uncompressed" => Ok(Self::MsgHashUncompressed),
			"priv_to_pub" => Ok(Self::PrivToPub),
			"sign_msg" => Ok(Self::SignMsg),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SszGenericType {
	BasicVector,
	Bitlist,
	Bitvector,
	Boolean,
	Containers,
	Uints,
}

impl FromStr for SszGenericType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"basic_vector" => Ok(Self::BasicVector),
			"bitlist" => Ok(Self::Bitlist),
			"bitvector" => Ok(Self::Bitvector),
			"boolean" => Ok(Self::Boolean),
			"containers" => Ok(Self::Containers),
			"uints" => Ok(Self::Uints),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpochProcessingType {
	Crosslinks,
	FinalUpdates,
	JustificationAndFinalization,
	RegistryUpdates,
	Slashings,
}

impl FromStr for EpochProcessingType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"crosslinks" => Ok(Self::Crosslinks),
			"final_updates" => Ok(Self::FinalUpdates),
			"justification_and_finalization" => Ok(Self::JustificationAndFinalization),
			"registry_updates" => Ok(Self::RegistryUpdates),
			"slashings" => Ok(Self::Slashings),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GenesisType {
	Initialization,
	Validity,
}

impl FromStr for GenesisType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"initialization" => Ok(Self::Initialization),
			"validity" => Ok(Self::Validity),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OperationsType {
	Attestation,
	AttesterSlashing,
	BlockHeader,
	Deposit,
	ProposerSlashing,
	Transfer,
	VoluntaryExit,
}

impl FromStr for OperationsType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"attestation" => Ok(Self::Attestation),
			"attester_slashing" => Ok(Self::AttesterSlashing),
			"block_header" => Ok(Self::BlockHeader),
			"deposit" => Ok(Self::Deposit),
			"proposer_slashing" => Ok(Self::ProposerSlashing),
			"transfer" => Ok(Self::Transfer),
			"voluntary_exit" => Ok(Self::VoluntaryExit),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SanityType {
	Blocks,
	Slots,
}

impl FromStr for SanityType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"blocks" => Ok(Self::Blocks),
			"slots" => Ok(Self::Slots),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShufflingType {
	Core,
}

impl FromStr for ShufflingType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"core" => Ok(Self::Core),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SszStaticType {
	Attestation,
	AttestationData,
	AttestationDataAndCustodyBit,
	AttesterSlashing,
	BeaconBlock,
	BeaconBlockBody,
	BeaconBlockHeader,
	BeaconState,
	Checkpoint,
	CompactCommittee,
	Crosslink,
	Deposit,
	DepositData,
	Eth1Data,
	Fork,
	HistoricalBatch,
	IndexedAttestation,
	PendingAttestation,
	ProposerSlashing,
	Transfer,
	Validator,
	VoluntaryExit,
}

impl FromStr for SszStaticType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> {
		match s {
			"Attestation" => Ok(Self::Attestation),
			"AttestationData" => Ok(Self::AttestationData),
			"AttestationDataAndCustodyBit" => Ok(Self::AttestationDataAndCustodyBit),
			"AttesterSlashing" => Ok(Self::AttesterSlashing),
			"BeaconBlock" => Ok(Self::BeaconBlock),
			"BeaconBlockBody" => Ok(Self::BeaconBlockBody),
			"BeaconBlockHeader" => Ok(Self::BeaconBlockHeader),
			"BeaconState" => Ok(Self::BeaconState),
			"Checkpoint" => Ok(Self::Checkpoint),
			"CompactCommittee" => Ok(Self::CompactCommittee),
			"Crosslink" => Ok(Self::Crosslink),
			"Deposit" => Ok(Self::Deposit),
			"DepositData" => Ok(Self::DepositData),
			"Eth1Data" => Ok(Self::Eth1Data),
			"Fork" => Ok(Self::Fork),
			"HistoricalBatch" => Ok(Self::HistoricalBatch),
			"IndexedAttestation" => Ok(Self::IndexedAttestation),
			"PendingAttestation" => Ok(Self::PendingAttestation),
			"ProposerSlashing" => Ok(Self::ProposerSlashing),
			"Transfer" => Ok(Self::Transfer),
			"Validator" => Ok(Self::Validator),
			"VoluntaryExit" => Ok(Self::VoluntaryExit),
			_ => Err(Error::InvalidType),
		}
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TestDescription {
	pub network: TestNetwork,
	pub phase: TestPhase,
	pub typ: TestType,
	pub origin: String,
	pub name: String,
	pub path: Option<PathBuf>,
}

impl FromStr for TestDescription {
	type Err = Error;

	fn from_str(s0: &str) -> Result<Self, Error> {
		let s = s0.split("/").collect::<Vec<_>>();
		if s.len() != 6 {
			return Err(Error::InvalidType)
		}

		let network = TestNetwork::from_str(s[0])?;
		let phase = TestPhase::from_str(s[1])?;
		let typ = TestType::from_str(&(s[2].to_owned() + "/" + s[3]))?;
		let origin = s[4].to_owned();
		let name = s[5].to_owned();

		Ok(Self {
			network, phase, typ, origin, name,
			path: None,
		})
	}
}

pub fn read_descriptions<P: AsRef<Path> + Clone>(root: P) -> Result<Vec<TestDescription>, Error> {
	let mut ret = Vec::new();
	let mut is_all_files = true;

	for entry in fs::read_dir(root.clone())? {
		let entry = entry?;
		let path = entry.path();

		if path.is_dir() {
			is_all_files = false;
			ret.append(&mut read_descriptions(path)?);
		}
	}

	if is_all_files {
		let path = fs::canonicalize(root)?;
		let mut desc = TestDescription::from_str(&test_name(&path)?)?;
		desc.path = Some(path);
		ret.push(desc);
	}

	Ok(ret)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[test]
	fn all_test_types_supported() {
		let root = {
			let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
			root.push("res/ethtests/tests");
			root
		};

		let descs = read_descriptions(root);
		assert!(descs.is_ok());
	}
}
