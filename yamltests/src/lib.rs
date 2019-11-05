pub mod description;
pub mod ssz_static;

use std::io;
use std::path::Path;
use description::{TestDescription, TestType};

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
		_ => unimplemented!(),
	}
}
