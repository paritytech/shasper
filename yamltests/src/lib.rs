pub mod description;
pub mod ssz_static;
pub mod operations;
pub mod sanity;
pub mod epoch_processing;

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use ssz::Encode;
use serde::de::DeserializeOwned;
use description::{TestDescription, TestType};
use beacon::{BeaconState, BeaconExecutive, Config};

#[derive(Debug)]
pub enum Error {
	InvalidType,
	IO(io::Error),
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Error {
		Error::IO(err)
	}
}

pub fn test_name<P: AsRef<Path>>(path: P) -> Result<String, Error> {
	let s = path.as_ref().to_str().ok_or(Error::InvalidType)?;
	let ss = s.split("/").collect::<Vec<_>>();

	if ss.len() < 6 {
		return Err(Error::InvalidType)
	}

	let s0 = &ss[(ss.len() - 6)..];

	Ok(s0[0].to_owned() + "/" + s0[1] + "/"
	   + s0[2] + "/" + s0[3] + "/"
	   + s0[4] + "/" + s0[5])
}

pub fn test(desc: TestDescription) {
	match desc.typ {
		TestType::SszStatic(typ) => ssz_static::test(typ, desc),
		TestType::Operations(typ) => operations::test(typ, desc),
		TestType::Sanity(typ) => sanity::test(typ, desc),
		TestType::EpochProcessing(typ) => epoch_processing::test(typ, desc),
		_ => println!("Skipped {}", test_name(desc.path.unwrap()).unwrap()),
	}
}

pub fn test_state_with<C: Config, F: FnOnce(&mut BeaconExecutive<C>) -> Result<(), beacon::Error>>(
	description: &str, pre: &BeaconState<C>, post: Option<&BeaconState<C>>, f: F
) {
	print!("Running test: {} ...", description);

	let mut state = pre.clone();
	let mut executive = BeaconExecutive::new(&mut state);

	match f(&mut executive) {
		Ok(()) => {
			print!(" accepted");

			let post = post.unwrap().clone();
			assert_eq!(state, post);
			print!(" passed");
		}
		Err(e) => {
			print!(" rejected({:?})", e);

			assert!(post.is_none());
			print!(" passed");
		}
	}

	println!("");
}

pub fn read_raw_unwrap<P: AsRef<Path>>(path: P) -> Vec<u8> {
	File::open(path).expect("Open serialized failed")
		.bytes()
		.map(|v| v.unwrap())
		.collect::<Vec<_>>()
}

pub fn read_value_unwrap<P: AsRef<Path>, T>(path: P) -> T where
	T: DeserializeOwned
{
	let file = File::open(path).expect("Open roots failed");
	let reader = BufReader::new(file);
	serde_yaml::from_reader::<_, T>(reader).expect("Parse roots failed")
}

pub fn read_pre_post_unwrap<C: Config>(path: PathBuf) -> (BeaconState<C>, Option<BeaconState<C>>) where
	C: DeserializeOwned,
{
	let pre = {
		let mut path = path.clone();
		path.push("pre.yaml");

		read_value_unwrap::<_, BeaconState<C>>(path)
	};

	let pre_ssz = {
		let mut path = path.clone();
		path.push("pre.ssz");

		read_raw_unwrap(path)
	};

	assert_eq!(Encode::encode(&pre), pre_ssz);

	let post = {
		let mut path = path.clone();
		path.push("post.yaml");

		if path.exists() {
			Some(read_value_unwrap::<_, BeaconState<C>>(path))
		} else {
			None
		}
	};

	if let Some(post) = post.as_ref() {
		let post_ssz = {
			let mut path = path.clone();
			path.push("post.ssz");

			read_raw_unwrap(path)
		};

		assert_eq!(Encode::encode(post), post_ssz);
	}

	(pre, post)
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	#[test]
	fn ethtests_all() {
		let dir = {
			let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
			path.push("res/ethtests/tests");
			path
		};

		let descs = crate::description::read_descriptions(dir).unwrap();
		for desc in descs {
			crate::test(desc);
		}
	}
}
