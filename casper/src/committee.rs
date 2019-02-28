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

//! Committee for Casper

use hash_db::Hasher;
use rstd::cmp;
use rstd::ops::BitXor;
use crate::utils::{hash2, hash3, to_usize};
use crate::randao::RandaoProducer;

/// Shuffle config.
pub struct ShuffleConfig {
	/// Rounds for the shuffling algorithm.
	pub round: usize,
	/// Target committee size.
	pub target_committee_len: usize,
	/// Shard count.
	pub shard_count: usize,
	/// The number of splits we should get for all shards. In beacon chain, this is the number of slots per epoch.
	pub split_count: usize,
}

/// Shuffle update.
pub enum ShuffleUpdate {
	/// Update nothing.
	None,
	/// Update seed only.
	Seed,
	/// Update both seed and shard offset.
	Len(usize),
}

/// Committee assigner.
pub struct CommitteeProcess<H: Hasher> {
	current_len: usize,
	previous_len: usize,
	previous_shard_offset: usize,
	current_shard_offset: usize,
	randao: RandaoProducer<H>,
	config: ShuffleConfig,
}

impl<H: Hasher> CommitteeProcess<H> {
	/// Mix a reveal into the RANDAO.
	pub fn mix(&mut self, reveal: &H::Out) where
		H::Out: BitXor<Output=H::Out>
	{
		self.randao.mix(reveal);
	}

	/// Advance the epoch for the process.
	pub fn advance_epoch(&mut self, f: &H::Out, update: ShuffleUpdate) where
		H::Out: BitXor<Output=H::Out>
	{
		self.previous_shard_offset = self.current_shard_offset;
		self.previous_len = self.current_len;

		match update {
			ShuffleUpdate::None => {
				self.randao.advance_epoch(f, false);
			},
			ShuffleUpdate::Seed => {
				self.randao.advance_epoch(f, true);
			},
			ShuffleUpdate::Len(new_len) => {
				self.randao.advance_epoch(f, true);
				self.current_shard_offset = (self.current_shard_offset + committee_count(new_len, &self.config)) % self.config.shard_count;
				self.current_len = new_len;
			},
		}
	}

	fn committees_at(&self, offset: usize, is_current: bool) -> Vec<Vec<usize>> {
		let len = if is_current { self.current_len } else { self.previous_len };
		let committee_count = committee_count(len, &self.config);
		let committee_per_slot_count = committee_count / self.config.split_count;
		let committee_size = len / committee_count;

		let mut committees = Vec::new();
		for i in 0..committee_per_slot_count {
			let mut committee = Vec::new();
			for j in 0..committee_size {
				let index = (committee_per_slot_count * offset + i) * committee_size + j;
				if index < self.current_len {
					committee.push(permuted_index::<H>(
						index,
						if is_current {
							self.randao.current()
						} else {
							self.randao.previous()
						}.as_ref(),
						len,
						self.config.round
					));
				}
			}
			committees.push(committee);
		}
		committees
	}

	/// Get current committees at a particular slot.
	pub fn current_committees_at(&self, offset: usize) -> Vec<Vec<usize>> {
		self.committees_at(offset, true)
	}

	/// Get previous committees at a particular slot.
	pub fn previous_committees_at(&self, offset: usize) -> Vec<Vec<usize>> {
		self.committees_at(offset, false)
	}
}

fn permuted_index<H: Hasher>(mut index: usize, seed: &[u8], len: usize, round: usize) -> usize {
	assert!(index < len);

	for round in 0..round {
		let pivot = to_usize(
			hash2::<H>(seed, &round.to_le_bytes()[..1]).as_ref()
		) % len;
		let flip = (pivot - index) % len;
		let position = cmp::max(index, flip);
		let source = hash3::<H>(
			seed,
			&round.to_le_bytes()[..1],
			&(position / 256).to_le_bytes()[..4]
		);
		let byte = source.as_ref()[(position % 256) / 8];
		let bit = (byte >> (position % 8 )) % 2;
		index = if bit == 1 { flip } else { index }
	}

	index
}

fn committee_count(len: usize, config: &ShuffleConfig) -> usize {
	cmp::max(
		1,
		cmp::min(
			config.shard_count / config.split_count,
			len / config.split_count / config.target_committee_len,
		)
	) * config.split_count
}
