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

//! RANDAO constructs.
//!
//! RANAO is for generating random numbers in a decentralized fashion.
//! In RANDAO, participants publish "onion" of hashed chains. Each time
//! when the participant is required to add entropy into the system, it
//! reveals one layer of the onion.

use hash_db::Hasher;
use rstd::prelude::*;
use rstd::ops::BitXor;
use codec::{Encode, Decode};
use codec_derive::{Encode, Decode};
#[cfg(feature = "std")]
use std::path::Path;
use crate::utils::hash2;

/// RANDAO config.
#[derive(Default, Encode, Decode, Clone)]
pub struct RandaoConfig {
	/// Seed lookahead.
	pub lookahead: usize,
}

/// RANDAO producer.
#[derive(Default, Encode, Decode, Clone)]
pub struct RandaoProducer<H: Hasher> where
	H::Out: Encode + Decode
{
	history: Vec<H::Out>,
	offset: usize,
	mix: RandaoMix<H>,
	config: RandaoConfig,
}

impl<H: Hasher> RandaoProducer<H> where
	H::Out: Encode + Decode
{
	/// Mix the current value with a new reveal.
	pub fn mix(&mut self, reveal: &H::Out) where
		H::Out: BitXor<Output=H::Out>
	{
		self.mix.mix(reveal)
	}

	/// Advance the epoch.
	pub fn advance_epoch(&mut self, f: &H::Out, update: bool) where
		H::Out: BitXor<Output=H::Out>
	{
		let mix = hash2::<H>(self.mix.get().as_ref(), f.as_ref());
		self.history.insert(0, mix);

		if update {
			self.offset = 0;
			self.history.truncate(self.config.lookahead + 1);
		} else {
			self.offset += 1;
		}
	}

	/// Get the current seed.
	pub fn current(&self) -> H::Out {
		self.history[self.offset + self.config.lookahead]
	}

	/// Get the previous seed.
	pub fn previous(&self) -> H::Out {
		self.history[self.offset + self.config.lookahead + 1]
	}

	/// Create a new RANDAO producer.
	pub fn new(val: H::Out, config: RandaoConfig) -> Self {
		let mut history = Vec::new();
		for _ in 0..(config.lookahead + 1) {
			history.push(val);
		}

		Self {
			history, config,
			offset: 0,
			mix: RandaoMix::new(val)
		}
	}
}

/// A RANDAO mix. Combine revealed values together.
#[derive(Default, Encode, Decode, Clone)]
pub struct RandaoMix<H: Hasher> where
	H::Out: Encode + Decode
{
	data: H::Out
}

impl<H: Hasher> RandaoMix<H> where
	H::Out: Encode + Decode
{
	/// Create a new mix.
	pub fn new(val: H::Out) -> Self {
		RandaoMix { data: val }
	}

	/// Mix the current value with a new reveal.
	pub fn mix(&mut self, reveal: &H::Out) where
		H::Out: BitXor<Output=H::Out>,
	{
		let input = self.data ^ *reveal;
		self.data = H::hash(input.as_ref());
	}

	/// Get the inner randao value.
	pub fn get(&self) -> H::Out {
		self.data
	}
}

impl<H: Hasher> AsRef<H::Out> for RandaoMix<H> where
	H::Out: Encode + Decode
{
	fn as_ref(&self) -> &H::Out {
		&self.data
	}
}

/// A RANDAO commitment.
pub struct RandaoCommitment<H: Hasher> where
	H::Out: Encode + Decode
{
	data: H::Out
}

impl<H: Hasher> RandaoCommitment<H> where
	H::Out: Encode + Decode
{
	/// Create a new commitment.
	pub fn new(val: H::Out) -> Self {
		RandaoCommitment { data: val }
	}

	/// Reveal the commitment, with the given revealed value, and how many
	/// layers to be revealed. Returns whether the reveal is successful.
	pub fn reveal(&mut self, reveal: &H::Out, layers: usize) -> bool {
		let mut revealed = *reveal;
		for _ in 0..layers {
			revealed = H::hash(revealed.as_ref());
		}

		if revealed != self.data {
			false
		} else {
			self.data = *reveal;
			true
		}
	}
}

impl<H: Hasher> AsRef<H::Out> for RandaoCommitment<H> where
	H::Out: Encode + Decode
{
	fn as_ref(&self) -> &H::Out {
		&self.data
	}
}

/// An onion for RANDAO.
pub struct RandaoOnion<H: Hasher> {
	data: Vec<H::Out>,
}

impl<H: Hasher> RandaoOnion<H> {
	/// Generate a new onion.
	pub fn generate(seed: H::Out, n: usize) -> Self {
		let mut data = Vec::new();
		data.push(seed);

		for _ in 0..n {
			data.push(H::hash(data.last().expect("Seed is pushed; data at least has one item; qed").as_ref()));
		}

		Self { data }
	}

	/// Pop a value from the onion, skip given layers.
	pub fn pop(&mut self, skip: usize) -> Option<H::Out> {
		for _ in 0..skip {
			self.data.pop();
		}

		self.data.pop()
	}

	/// Save the onion to a file.
	#[cfg(feature = "std")]
	pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> where
		H::Out: serde::Serialize
	{
		use std::fs::File;

		let f = File::create(path)?;
		serde_json::to_writer(&f, &self.data)?;
		f.sync_all()
	}

	/// Load the onion from a file.
	#[cfg(feature = "std")]
	pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> where
		H::Out: serde::de::DeserializeOwned
	{
		use std::fs::File;

		let f = File::open(path)?;
		let data: Vec<H::Out> = serde_json::from_reader(&f)
			.map_err(|_| std::io::Error::from(std::io::ErrorKind::Other))?;
		Ok(Self { data })
	}
}

#[cfg(test)]
mod tests {
	use hash_db::Hasher;
	use plain_hasher::PlainHasher;
	use super::*;

	/// A dummy hasher for `[u8; 1]`. A hash for `n` is `n + 1`.
	struct DummyHasher;

	impl Hasher for DummyHasher {
		type Out = [u8; 1];
		type StdHasher = PlainHasher;
		const LENGTH: usize = 1;

		fn hash(x: &[u8]) -> Self::Out {
			assert!(x.len() == 1);
			[x[0] + 1]
		}
	}

	#[test]
	fn reveal_commitment_255_layers() {
		let mut commitment = RandaoCommitment::<DummyHasher>::new([255]);
		assert!(!commitment.reveal(&[0], 254));
		assert_eq!(commitment.as_ref(), &[255]);
		assert!(commitment.reveal(&[0], 255));
		assert_eq!(commitment.as_ref(), &[0]);
	}
}
