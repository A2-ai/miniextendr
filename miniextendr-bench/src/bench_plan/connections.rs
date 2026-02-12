//! Connection framework benchmarks (feature = "connections").
//!
//! Implemented groups:
//! - `open_close`: create/destroy connection
//! - `read_small` / `read_large`: 128 bytes / 4096 bytes
//! - `write_small` / `write_large`: 128 bytes / 4096 bytes
//!
//! Remaining gaps:
//! - `seek` / `tell` benchmarks
//! - Buffering on/off variants
//! - Encoding variations
