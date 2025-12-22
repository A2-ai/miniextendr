//! Benchmark harness plan.
//!
//! Goal: ensure every benchmark runs with consistent fixtures, sizes, and
//! thread safety. This module describes the shared helpers that should exist
//! in `miniextendr-bench` (not implemented here).
//!
//! ---------------------------------------------------------------------------
//! Planned helpers
//!
//! 1) `init()`
//!    - Initialize embedded R once via `miniextendr_engine`.
//!    - Record the init thread and enforce single-threaded R access.
//!
//! 2) `assert_on_init_thread()`
//!    - Panic if any R API call is made from a non-init thread.
//!
//! 3) `Fixtures` struct
//!    - Pre-allocated R vectors for each type and size class.
//!    - Matching Rust Vec<T> inputs for IntoR benchmarks.
//!    - Named lists for map conversion benches.
//!    - ExternalPtr fixtures (typed and type-erased).
//!    - ALTREP classes for data and iterator-backed variants.
//!    - String fixtures: UTF-8, Latin-1, ASCII-only, empty, NA.
//!
//! 4) `Param` types
//!    - Size enum: tiny / small / medium / large.
//!    - NA density enum: none / sparse / moderate / heavy.
//!    - Encoding enum: ascii / utf8 / latin1 / bytes.
//!    - Flags: include_alloc (yes/no), include_gc (yes/no).
//!
//! ---------------------------------------------------------------------------
//! Execution guidelines
//!
//! - Benchmarks should not allocate within the hot loop unless explicitly
//!   measuring allocation cost.
//! - Use divan parameterization and label all cases (type, size, NA density).
//! - Warm up R by running a small no-op .Call once per bench file.
//! - Keep all R objects protected or preserved for the full benchmark run.
//! - Prefer precomputed fixtures to avoid R GC noise.
//! - Use explicit “with allocation” and “without allocation” variants.
