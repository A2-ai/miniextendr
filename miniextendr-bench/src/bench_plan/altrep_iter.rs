//! Iterator-backed ALTREP benchmarks.
//!
//! Planned groups:
//! - `sequential_access` (0..n)
//! - `random_access` (sparse indices)
//! - `materialize` (force full realization)
//! - `get_region` (bulk read)
//! - `coerce_variants` (IterIntCoerce, IterRealCoerce, bool->int)
//! - `option_iterators` (`Option<T>` -> NA)
//!
//! Compare:
//! - iterator-backed vs pre-materialized `Vec<T>`
//! - ExactSizeIterator vs explicit length
//! - cache hit vs miss performance
