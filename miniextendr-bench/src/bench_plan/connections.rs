//! Connection framework benchmarks (feature = "connections").
//!
//! Planned groups:
//! - `open_close` (create/destroy connection)
//! - `read_small` / `read_large`
//! - `write_small` / `write_large`
//! - `seek` / `tell` (if supported)
//!
//! Parameters:
//! - payload size
//! - buffering on/off
//! - encoding variations
