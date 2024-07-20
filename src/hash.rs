//! This module contains the [`OrderPreservingHasher`] type, which is a helper struct for defining a
//! hash function that preserves order modulo a reduced universe.
//!
//! See the documentation for [`OrderPreservingHasher`] for more information.

use crate::utils::*;

/// TODO docs.
#[derive(Debug, Clone, Copy)]
pub struct OrderPreservingHasher {
    /// The first arbitrary constant.
    c1: u64,
    /// The second arbitrary constant.
    c2: u64,
    /// A large prime.
    p: u64,
    /// The size of the reduced universe.
    r: u64,
}

/// TODO docs.
impl OrderPreservingHasher {
    /// Creates a new hash function.
    ///
    /// TODO more docs.
    ///
    /// # Panics
    ///
    /// This function will panic if `epsilon` is not between `0.0` or `1.0`, or if the
    /// `max_interval` parameter is too large.
    pub fn new(num_elements: usize, max_interval: u64, epsilon: f64) -> Self {
        assert!(
            0.0 < epsilon && epsilon < 1.0,
            "The false positive rate (epsilon) must be between 0.0 and 1.0"
        );
        assert!(
            max_interval <= Self::max_range_interval(num_elements, epsilon),
            "The input maximum interval is impossible given the other parameters"
        );

        let upper = (num_elements as u64)
            .checked_mul(max_interval)
            .expect("We should not have any overflow on calculating the reduced universe size.");
        let lower = (1.0 / epsilon).floor() as u64;

        let reduced_universe_size = upper
            .checked_mul(lower)
            .expect("We should not have any overflow on calculating the reduced universe size.");

        Self {
            c1: gen_random(),
            c2: gen_random(),
            p: gen_prime(), // TODO
            r: reduced_universe_size,
        }
    }

    // A hash function taken from a pairwise-independent family.
    fn inner_hash(&self, x: u64) -> u64 {
        ((self.c1 * x + self.c2) % self.p) % self.r
    }

    /// A hash function that preserves locality and ordering modulo the reduced universe of integer
    /// items.
    pub fn hash(&self, x: u64) -> u64 {
        let inner = x / self.r;
        let q = self.inner_hash(inner);

        (q + x) % self.r
    }

    /// Returns the size of the reduced universe that the hash function maps to.
    pub fn reduced_universe(&self) -> u64 {
        self.r
    }

    /// Returns the maximum range interval given the number of elements in the set and the false
    /// positive rate.
    ///
    /// TODO docs.
    fn max_range_interval(num_elements: usize, epsilon: f64) -> u64 {
        ((u64::MAX as f64) * epsilon) as u64 / num_elements as u64
    }
}
