# Grafite

**Grafite** is a _range filter_ with a simple design and clear theoretical guarantees
that hold regardless of the input data and query distribution.

This library is a Rust implementation of the data structure introduced
by this paper: [Grafite: Taming Adversarial Queries with Optimal Range Filters](https://arxiv.org/pdf/2311.15380).

The authors of this paper also created a C++ implementation for Grafite, which can be found on one of the author's GitHub: [`grafite`](https://github.com/marcocosta97/grafite).

The Grafite data structure relies on the Elias-Fano encoding of non-decreasing integer sequences. This library uses the [`vers_vecs`] library for their implementation of the encoding.

# Usage

```rust
let values = [1, 2, 3, 7, 8, 9, 15, 20];

let epsilon = 0.01;
let max_query_range = 20;
let hasher = OrderPreservingHasher::new(values.len(), max_query_range, epsilon).expect("TODO");

let rf = RangeFilter::new(values.iter().copied(), hasher);

// If there are any values in the range, it will return `true`.
assert!(rf.query(0..20));

// Start is inclusive.
assert!(rf.query(3..5));
assert!(rf.query(9..16));

// End is exclusive. Note that false positives are possible depending on `epsilon`.
assert!(!rf.query(10..15));
assert!(rf.query(10..16));
```
