//! This module contains the [`OrderPreservingHasher`] type, which is a helper struct for defining a
//! hash function that preserves integer key ordering modulo a reduced universe.
//!
//! See the documentation for [`OrderPreservingHasher`] for more information.

use crate::utils::*;

/// The default universe size for 64-bit unsigned integers, which is equivalent to [`u64::MAX`].
pub const MAX_UNIVERSE_SIZE: u64 = u64::MAX;

/// An error type representing if the parameters of an [`OrderPreservingHasher`] are invalid for any
/// reason.
#[derive(Debug, Clone, Copy)]
pub enum ParamError {
    /// If the input `epsilon` is not strictly in between `0.0` and `1.0`. Stores the invalid
    /// `epsilon`.
    InvalidEpsilon(f64),
    /// If the input maximum interval is too large. Stores the maximum range interval (note that
    /// this is _not_ the same as the input interval that raised the error).
    InvalidMaxInterval(u64),
    /// If overflow occurs in the calculation of the reduced universe size, or if the bits used per
    /// key is invalid.
    Overflow,
}

/// A struct to help manage the order-preserving hash function used for the Grafite range filter.
///
/// The intended use of this struct is simply to be constructed, moved, and stored into the
/// [`RangeFilter`](crate::RangeFilter) type.
///
/// Instead of manually calculating carrying around the constants for the hash function, we can
/// group them into a struct and use a [`Self::hash`] method to hash all of the input values.
///
/// See the [`Self::new`] and [`Self::hash`] methods for more information.
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

impl OrderPreservingHasher {
    /// Creates a new hash function helper struct with specific parameters and guarantees.
    ///
    /// If the parameters are invalid for any reason, this function will return a [`ParamError`].
    ///
    /// TODO more docs.
    ///
    /// See Section 3 of the original paper for more information on how the hash function works and
    /// behaves.
    pub fn new(num_elements: usize, epsilon: f64, max_interval: u64) -> Result<Self, ParamError> {
        if epsilon <= 0.0 || 1.0 <= epsilon {
            return Err(ParamError::InvalidEpsilon(epsilon));
        }

        let max_range_interval = Self::max_range_interval(MAX_UNIVERSE_SIZE, num_elements, epsilon);
        if max_interval > max_range_interval {
            return Err(ParamError::InvalidMaxInterval(max_range_interval));
        }

        let upper = (num_elements as u64)
            .checked_mul(max_interval)
            .ok_or(ParamError::Overflow)?;
        let lower = (1.0 / epsilon).floor() as u64;

        let reduced_universe_size = upper.checked_mul(lower).ok_or(ParamError::Overflow)?;

        // Generate `p > r`.
        let p = gen_prime(1 + reduced_universe_size..MAX_UNIVERSE_SIZE);

        // Generate two numbers `c1, c2 < p` with `c1 != 0`.
        let c1 = gen_random(1..p);
        let c2 = gen_random(0..p);

        Ok(Self {
            c1,
            c2,
            p,
            r: reduced_universe_size,
        })
    }

    /// Calculates the false positive rate of the [`RangeFilter`](crate::RangeFilter) given a
    /// maximum budget of bits per key and the maximum range interval that will be queried.
    ///
    /// If `bits_per_key` is not in the range (2, 64], this function will return a [`ParamError`].
    pub fn epsilon_with_budget(bits_per_key: u8, max_interval: u64) -> Result<f64, ParamError> {
        if bits_per_key <= 2 || bits_per_key > 64 {
            Err(ParamError::Overflow)
        } else {
            // We calculate the false positive rate with `L / 2^(B-2)`.
            Ok(max_interval as f64 / (1 << (bits_per_key - 2)) as f64)
        }
    }

    /// Creates a hash function given a budget of `bits_per_key` bits per key for every distinct
    /// value that is input into the range filter.
    ///
    /// Internally, this function will just calculate the false positive rate via
    /// `epsilon_with_budget` and use that `epsilon` as the parameter for the [`new`](Self::new)
    /// method above.
    pub fn new_with_budget(
        num_elements: usize,
        bits_per_key: u8,
        max_interval: u64,
    ) -> Result<Self, ParamError> {
        let epsilon = Self::epsilon_with_budget(bits_per_key, max_interval)?;
        Self::new(num_elements, epsilon, max_interval)
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
        ((self.c1.overflowing_mul(x).0.overflowing_add(self.c2)).0 % self.p) % self.r
    }

    /// A hash function that preserves locality and ordering modulo the reduced universe of integer
    /// items.
    ///
    /// TODO more docs.
    pub fn hash(&self, x: u64) -> u64 {
        let inner = x / self.r;
        let q = self.inner_hash(inner);

        (q.overflowing_add(x).0) % self.r
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
    ///
    /// # Panics
    ///
    /// Panics if `epsilon` is not strictly in between `0.0` and `1.0`.
    fn max_range_interval(universe_size: u64, num_elements: usize, epsilon: f64) -> u64 {
        assert!(
            0.0 < epsilon && epsilon < 1.0,
            "epsilon must be between 0.0 and 1.0"
        );

        ((universe_size as f64) * epsilon) as u64 / num_elements as u64
    }
}
