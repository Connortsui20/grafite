//! TODO need to be able to take control of the Elias-Fano encoding parameters.

use std::{marker::PhantomData, ops::Range};
use vers_vecs::EliasFanoVec;

/// The Grafite Range Filter.
pub struct RangeFilter<T, H: Fn(&T) -> u64> {
    ef: EliasFanoVec,
    hasher: H,
    _phantom: PhantomData<T>,
}

impl<T, H: Fn(&T) -> u64> RangeFilter<T, H> {
    /// Creates a new `RangeFilter` given a slice of values.
    pub fn new(values: &[T], hasher: H) -> Self {
        let mut hashes: Vec<u64> = values.iter().map(&hasher).collect();
        hashes.sort_unstable();

        Self {
            ef: EliasFanoVec::from_slice(&hashes),
            hasher,
            _phantom: PhantomData,
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
    pub fn query(&self, range: Range<T>) -> bool {
        let start_hash = (self.hasher)(&range.start);
        let end_hash = (self.hasher)(&range.end);

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
