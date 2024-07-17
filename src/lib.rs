//! TODO need to be able to take control of the Elias-Fano encoding parameters.

use std::ops::Range;
use vers_vecs::EliasFanoVec;

#[derive(Clone, Copy)]
pub struct GrafiteHasher {
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
impl GrafiteHasher {
    /// Creates a new hash function. TODO more docs.
    pub fn new(num_elements: u64, max_interval: u64, epsilon: f64) -> Self {
        let upper = num_elements
            .checked_mul(max_interval)
            .expect("We should not have any overflow on calculating the reduced universe size.");

        let lower = (1.0 / epsilon).floor() as u64;

        let reduced_universe_size = upper
            .checked_mul(lower)
            .expect("We should not have any overflow on calculating the reduced universe size.");

        Self {
            c1: 0,
            c2: 0,
            p: 3,
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
}

/// The Grafite Range Filter.
pub struct RangeFilter {
    /// A succinct encoding of a monotonic non-decreasing sequence of hash values.
    ef: EliasFanoVec,
    /// The hash function used to encode the hash values.
    hasher: GrafiteHasher,
}

/// The `RangeFilter` must be built on items that are able to be turned into a 64-bit integer.
impl RangeFilter {
    /// Creates a new `RangeFilter` given a slice of values.
    pub fn new(values: &[u64], hasher: GrafiteHasher) -> Self {
        // Hash the input set.
        let mut hashes: Vec<u64> = values.iter().cloned().map(|x| hasher.hash(x)).collect();

        // Sort and then remove all duplicates.
        hashes.sort_unstable();
        hashes.dedup();

        Self {
            ef: EliasFanoVec::from_slice(&hashes),
            hasher,
        }
    }

    /// Gets the minimum hash value in the sorted hash codes.
    fn min_hash(&self) -> u64 {
        self.ef.get_unchecked(0)
    }

    /// Gets the maximum hash value in the sorted hash codes.
    fn max_hash(&self) -> u64 {
        self.ef.get_unchecked(self.ef.len() - 1)
    }

    /// Checks if there are any elements within the given range among the original input set.
    pub fn query(&self, range: Range<u64>) -> bool {
        let start_hash = self.hasher.hash(range.start);
        let end_hash = self.hasher.hash(range.end);

        // If the start hash is greater than the end hash, then the range has wrapped around due to
        // the reduced universe. Thus we can just check the min and max hashes to see if there is an
        // element between the endpoints.
        if start_hash > end_hash {
            return self.min_hash() <= end_hash || self.max_hash() >= start_hash;
        }

        match self.ef.predecessor(end_hash) {
            // If the end hash has no predecessor, then there can't be any elements in the set less
            // than the input range end, which means there is no element in between start and end.
            None => false,
            // If the predecessor is less than the start hash, then there cannot be any elements in
            // between start and end.
            Some(predecessor) => predecessor >= start_hash,
        }
    }
}
