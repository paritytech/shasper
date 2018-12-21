// Copyright 2018 Paul Hauner.
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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
extern crate alloc;

extern crate blake2;

mod rng;

use rng::ShuffleRng;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[derive(Debug)]
pub enum ShuffleErr {
    ExceedsListLength,
}

/// Performs a deterministic, in-place shuffle of a vector of bytes.
/// The final order of the shuffle is determined by successive hashes
/// of the supplied `seed`.
pub fn shuffle(
    seed: &[u8],
    mut list: Vec<usize>)
    -> Result<Vec<usize>, ShuffleErr>
{
    let mut rng = ShuffleRng::new(seed);
    if list.len() > rng.rand_max as usize {
        return Err(ShuffleErr::ExceedsListLength);
    }
    for i in 0..(list.len() - 1) {
        let n = list.len() - i;
        let j = rng.rand_range(n as u32) as usize + i;
        list.swap(i, j);
    }
    Ok(list)
}


#[cfg(test)]
mod tests {
    use super::*;
    use blake2::Blake2s;
	use blake2::crypto_mac::{Mac, MacResult};
	use blake2::digest::generic_array::typenum::U32;

    fn hash(seed: &[u8]) -> MacResult<U32> {
		let mut hasher = Blake2s::new_keyed(&[], 32);
		hasher.input(seed);
		hasher.result()
    }

    #[test]
    fn test_shuffling() {
        let seed = hash(b"4kn4driuctg8");
        let list: Vec<usize> = (0..12).collect();
        let s = shuffle(&seed.code(), list).unwrap();
        assert_eq!(
            s,
            vec![7, 4, 8, 6, 5, 3, 0, 11, 1, 2, 10, 9],
        )
    }
}
