//! TODO docs.

use rand::prelude::*;
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
fn is_prime(n: u64) -> bool {
    match n {
        0 | 1 => false,
        2 => true,
        _ if n % 2 == 0 => false,
        _ => !(1..)
            .map(|x| 2 * x + 1)
            .take_while(|&x| x * x <= n)
            .any(|factor| n % factor == 0),
    }
}

/// Generates a random number until it generates a prime, and then returns that prime number.
fn gen_prime() -> u64 {
    let mut rng = rand::thread_rng();

    loop {
        let attempt: u32 = rng.gen();
        dbg!(attempt);

        if is_prime(attempt as u64) {
            println!("Found prime {}", attempt);
            return attempt as u64;
        }
    }
}

/// TODO docs.
impl GrafiteHasher {
    /// Creates a new hash function. TODO more docs.
    pub fn new(num_elements: usize, max_interval: u64, epsilon: f64) -> Self {
        assert!(
            epsilon < 0.99,
            "Why are you trying to create a false positive rate of above 0.99?"
        );
        assert!(epsilon > 0.0, "epsilon must be greater than 0.0");
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

        dbg!(upper, lower, reduced_universe_size);

        let mut rng = rand::thread_rng();

        Self {
            c1: rng.gen(),
            c2: rng.gen(),
            p: gen_prime(), // TODO
            r: reduced_universe_size,
        }
    }

    /// TODO docs.
    fn max_range_interval(num_elements: usize, epsilon: f64) -> u64 {
        ((u64::MAX as f64) * epsilon) as u64 / num_elements as u64
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
    pub fn new<I>(values: I, hasher: GrafiteHasher) -> Self
    where
        I: Iterator<Item = u64>,
    {
        // Hash all items in the input set.
        let mut hashes: Vec<u64> = values.map(|x| hasher.hash(x)).collect();

        // Sort and then remove all duplicates.
        hashes.sort_unstable();
        hashes.dedup();

        assert!(hashes[hashes.len() - 1] < hasher.r);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prime() {
        let primes = [2, 3, 5, 7, 11, 13, 17, 19];

        assert!(primes.iter().copied().all(is_prime));
    }

    #[test]
    fn test_basic() {
        let values = [1, 2, 3, 7, 8, 9, 15, 20];

        let hasher = GrafiteHasher::new(values.len(), 20, 0.01);

        let rf = RangeFilter::new(values.iter().copied(), hasher);

        assert!(rf.query(0..20));
        assert!(rf.query(0..10));
        assert!(rf.query(0..5));
        assert!(rf.query(3..5));
        assert!(!rf.query(4..5));
        assert!(!rf.query(4..6));
        assert!(rf.query(4..7));
        assert!(rf.query(4..8));
        assert!(rf.query(4..10));
        assert!(!rf.query(10..14));
        assert!(rf.query(10..15));
    }
}
