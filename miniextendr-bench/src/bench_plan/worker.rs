//! Worker-thread and dispatch benchmarks.
//!
//! Planned groups:
//! - `run_on_worker` overhead vs direct function call
//! - `with_r_thread` round-trip latency (worker -> main -> worker)
//! - `channel_saturation` (many small R calls)
//! - `batching` (send N requests at once)
//!
//! Parameters:
//! - payload size (small vs large vectors)
//! - number of calls per batch
