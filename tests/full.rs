use grafite::{OrderPreservingHasher, RangeFilter};
use rand::prelude::*;
use rayon::prelude::*;

#[test]
fn full_bench() {
    const NUM_ELEMENTS: usize = 200_000_000;
    const BITS_PER_KEY: u8 = 16;
    const MAX_INTERVAL: u64 = 1 << 5;

    let mut values: Vec<u64> = (0..NUM_ELEMENTS)
        .into_par_iter()
        .map(|_| thread_rng().gen())
        .collect();

    println!("Finished generating 200M values");

    let hasher =
        OrderPreservingHasher::new_with_budget(NUM_ELEMENTS, BITS_PER_KEY, MAX_INTERVAL).unwrap();

    let rf = RangeFilter::new(values.iter().copied(), hasher);

    println!(
        "Finished constructing RangeFilter, taking {} bytes of space",
        rf.heap_size()
    );

    values.sort_unstable();

    println!("Finished sorting 200M values");
}
