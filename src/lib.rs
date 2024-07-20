//! TODO docs.

mod hash;
mod utils;

use crate::hash::*;
use std::ops::Range;
use vers_vecs::EliasFanoVec;

/// The Grafite Range Filter.
#[derive(Debug, Clone)]
pub struct RangeFilter {
    /// The hash function used to encode the hash values.
    pub hasher: OrderPreservingHasher,
    /// A succinct encoding of a monotonic non-decreasing sequence of hash values.
    pub ef: EliasFanoVec,
}

/// The `RangeFilter` must be built on items that are able to be turned into a 64-bit integer.
impl RangeFilter {
    /// Creates a new `RangeFilter` given a slice of values.
    pub fn new<I>(values: I, hasher: OrderPreservingHasher) -> Self
    where
        I: Iterator<Item = u64>,
    {
        // Hash all items in the input set.
        let mut hashes: Vec<u64> = values.map(|x| hasher.hash(x)).collect();

        // Sort and then remove all duplicates.
        hashes.sort_unstable();
        hashes.dedup();

        assert!(hashes[hashes.len() - 1] < hasher.reduced_universe());

        Self {
            hasher,
            ef: EliasFanoVec::from_slice(&hashes),
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
    fn test_basic() {
        let values = [1, 2, 3, 7, 8, 9, 15, 20];

        let hasher = OrderPreservingHasher::new(values.len(), 20, 0.01);

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
