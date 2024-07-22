//! This module contains the [`OrderPreservingHasher`] type, which is a helper struct for defining a
//! hash function that preserves integer key ordering modulo a reduced universe.
//!
//! See the documentation for [`OrderPreservingHasher`] for more information.

use crate::utils::*;

/// The default universe size for 64-bit unsigned integers, which is equivalent to [`u64::MAX`].
pub const MAX_UNIVERSE_SIZE: u64 = u64::MAX;

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
    /// Creates a new hash function helper struct with specific parameters and guarantees.
    ///
    /// TODO more docs.
    ///
    /// See Section 3 of the original paper for more information on how the hash function works and
    /// behaves.
    pub fn new(num_elements: usize, epsilon: f64, max_interval: u64) -> Option<Self> {
        if epsilon <= 0.0 || 1.0 <= epsilon {
            return None;
        }

        if max_interval > Self::max_range_interval(MAX_UNIVERSE_SIZE, num_elements, epsilon) {
            return None;
        }

        let upper = (num_elements as u64)
            .checked_mul(max_interval)
            .expect("We should not have any overflow on calculating the reduced universe size.");
        let lower = (1.0 / epsilon).floor() as u64;

        let reduced_universe_size = upper
            .checked_mul(lower)
            .expect("We should not have any overflow on calculating the reduced universe size.");

        // Generate `p > r`.
        let p = gen_prime(1 + reduced_universe_size..MAX_UNIVERSE_SIZE);

        // Generate two numbers `c1, c2 < p` with `c1 != 0`.
        let c1 = gen_random(1..p);
        let c2 = gen_random(0..p);

        Some(Self {
            c1,
            c2,
            p,
            r: reduced_universe_size,
        })
    }

    /// Creates a new hash function helper struct where the caller can pass in a custom reduced
    /// universe size.
    ///
    /// The [`Self::new`] method will calculate a good reduced universe size depending on the number
    /// of input items, the necessary false positive rate and maximum query interval, whereas this
    /// method will use whatever reduced universe size is passed in.
    ///
    /// The caller must take care to ensure `r` is optimal for their expected workloads.
    ///
    /// See the [`Self::new`] method for more information on how the hash function works and
    /// behaves.
    pub fn new_with_reduced(r: u64) -> Self {
        let p = gen_prime(1 + r..MAX_UNIVERSE_SIZE);

        // Generate two numbers `c1, c2 < p` with `c1 != 0`.
        let c1 = gen_random(1..p);
        let c2 = gen_random(0..p);

        Self { c1, c2, p, r }
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
    /// The maximum range interval is defined as `(u * e) / n`, where the variables are defined as:
    /// -   `u`: The size of the universe of keys
    /// -   `e`: The false positive rate `epsilon`
    /// -   `n`: The number of elements in the input set
    ///
    /// If the universe size is not known, [`MAX_UNIVERSE_SIZE`] should be used.
    fn max_range_interval(universe_size: u64, num_elements: usize, epsilon: f64) -> u64 {
        ((universe_size as f64) * epsilon) as u64 / num_elements as u64
    }
}
