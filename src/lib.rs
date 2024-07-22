#![doc = include_str!("../README.md")]

pub mod hash;
pub use crate::hash::OrderPreservingHasher;

mod utils;

use std::ops::RangeBounds;
use vers_vecs::EliasFanoVec;

/// The Grafite Range Filter.
#[derive(Debug, Clone)]
pub struct RangeFilter {
    /// The hash function used to encode the hash values.
    pub hasher: OrderPreservingHasher,
    /// A succinct encoding of a non-decreasing sequence of integer hash values.
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
    pub fn query<R>(&self, range: R) -> bool
    where
        R: RangeBounds<u64>,
    {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&i) => i,
            std::ops::Bound::Excluded(_) => unreachable!("Somehow had an exclusive start bound"),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(&i) => i,
            std::ops::Bound::Excluded(&e) => e - 1,
            std::ops::Bound::Unbounded => u64::MAX,
        };

        let start_hash = self.hasher.hash(start);
        let end_hash = self.hasher.hash(end);

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
