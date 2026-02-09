//! Worker-thread and dispatch benchmarks.
//!
//! Implemented groups:
//! - `run_on_worker`: pure Rust closure overhead
//! - `with_r_thread`: round-trip latency (worker → main → worker)
//! - `channel_saturation`: 20 sequential worker round-trips
//! - `batching`: single worker hop with 10 batched R thread requests
//!
//! Remaining gap:
//! - Payload size and batch count are hardcoded, not parameterized via divan
