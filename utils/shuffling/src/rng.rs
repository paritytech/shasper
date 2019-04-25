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

use blake2::Blake2s;
use blake2::crypto_mac::{Mac, MacResult};
use blake2::digest::generic_array::typenum::U32;

const SEED_SIZE_BYTES: usize = 32;
const RAND_BYTES: usize = 3;      // 24 / 8
const RAND_MAX: u32 = 16_777_216;   // 2**24

/// A pseudo-random number generator which given a seed
/// uses successive blake2s hashing to generate "entropy".
pub struct ShuffleRng {
    seed: MacResult<U32>,
    idx: usize,
    pub rand_max: u32,
}

impl ShuffleRng {
    /// Create a new instance given some "seed" bytes.
    pub fn new(initial_seed: &[u8]) -> Self {
        Self {
            seed: hash(initial_seed),
            idx: 0,
            rand_max: RAND_MAX,
        }
    }

    /// "Regenerates" the seed by hashing it.
    fn rehash_seed(&mut self) {
        self.seed  = hash(&self.seed.clone().code());
        self.idx = 0;
    }

    /// Extracts 3 bytes from the `seed`. Rehashes seed if required.
    fn rand(&mut self) -> u32 {
        self.idx += RAND_BYTES;
        if self.idx >= SEED_SIZE_BYTES {
            self.rehash_seed();
            self.rand()
        } else {
            int_from_byte_slice(
                &self.seed.clone().code(),
                self.idx - RAND_BYTES,
            )
        }
    }

    /// Generate a random u32 below the specified maximum `n`.
    ///
    /// Provides a filtered result from a higher-level rng, by discarding
    /// results which may bias the output. Because of this, execution time is
    /// not linear and may potentially be infinite.
    pub fn rand_range(&mut self, n: u32) -> u32 {
        assert!(n < RAND_MAX, "RAND_MAX exceed");
        let mut x = self.rand();
        while x >= self.rand_max - (self.rand_max % n) {
            x = self.rand();
        }
        x % n
    }
}

/// Reads the next three bytes of `source`, starting from `offset` and
/// interprets those bytes as a 24 bit big-endian integer.
/// Returns that integer.
fn int_from_byte_slice(source: &[u8], offset: usize) -> u32 {
    (
         u32::from(source[offset + 2])) |
        (u32::from(source[offset + 1]) << 8) |
        (u32::from(source[offset    ]) << 16
    )
}

/// Peform a blake2s hash on the given bytes.
fn hash(bytes: &[u8]) -> MacResult<U32> {
    let mut hasher = Blake2s::new_keyed(&[], SEED_SIZE_BYTES);
    hasher.input(bytes);
    hasher.result()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffling_int_from_slice() {
        let mut x = int_from_byte_slice(
            &[0, 0, 1],
            0);
        assert_eq!((x as u32), 1);

        x = int_from_byte_slice(
            &[0, 1, 1],
            0);
        assert_eq!(x, 257);

        x = int_from_byte_slice(
            &[1, 1, 1],
            0);
        assert_eq!(x, 65793);

        x = int_from_byte_slice(
            &[255, 1, 1],
            0);
        assert_eq!(x, 16711937);

        x = int_from_byte_slice(
            &[255, 255, 255],
            0);
        assert_eq!(x, 16777215);

        x = int_from_byte_slice(
            &[0x8f, 0xbb, 0xc7],
            0);
        assert_eq!(x, 9419719);
    }

    #[test]
    fn test_shuffling_hash_fn() {
        let digest = hash(&hash(b"4kn4driuctg8").code());  // double-hash is intentional
        let digest_bytes: &[u8] = &digest.code();
        let expected = [
            0xff, 0xff, 0xff, 0x8f, 0xbb, 0xc7, 0xab, 0x64, 0x43, 0x9a,
            0xe5, 0x12, 0x44, 0xd8, 0x70, 0xcf, 0xe5, 0x79, 0xf6, 0x55,
            0x6b, 0xbd, 0x81, 0x43, 0xc5, 0xcd, 0x70, 0x2b, 0xbe, 0xe3,
            0x87, 0xc7,
        ];
        assert_eq!(digest_bytes.len(), expected.len());
        assert_eq!(&digest_bytes, &expected)
    }
}
