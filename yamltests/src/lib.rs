pub mod description;
pub mod ssz_static;
pub mod operations;

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use serde::de::DeserializeOwned;
use description::{TestDescription, TestType};
use beacon::{BeaconState, Config};

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
		_ => unimplemented!(),
	}
}

pub fn test_state_with<C: Config, F: FnOnce(&mut BeaconState<C>) -> Result<(), beacon::Error>>(
	description: &str, pre: &BeaconState<C>, post: Option<&BeaconState<C>>, f: F
) {
	print!("Running test: {} ...", description);

	let mut state = pre.clone();

	match f(&mut state) {
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
