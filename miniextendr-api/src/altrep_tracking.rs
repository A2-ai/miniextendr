//! ALTREP materialization tracking (feature-gated).
//!
//! When the `materialization-tracking` feature is enabled, this module logs
//! every ALTREP `Dataptr` call — which is when R forces a lazy/compact vector
//! to materialize into a contiguous memory buffer.
//!
//! This is useful for diagnosing unexpected materializations that negate the
//! performance benefits of ALTREP.
//!
//! # Usage
//!
//! Enable in your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! miniextendr-api = { ..., features = ["materialization-tracking"] }
//! ```
//!
//! Then from R:
//! ```r
//! miniextendr:::altrep_materialization_count()
//! miniextendr:::altrep_materialization_reset()
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};

static MATERIALIZATION_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Record a materialization event. Called from the `__impl_altvec_dataptr!` macro.
#[inline]
pub fn record_materialization(type_name: &str, writable: bool) {
    let n = MATERIALIZATION_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    eprintln!(
        "[ALTREP materialization #{n}] {type_name}, writable={writable}"
    );
}

/// Get the total number of materializations since last reset.
pub fn materialization_count() -> usize {
    MATERIALIZATION_COUNT.load(Ordering::Relaxed)
}

/// Reset the materialization counter to zero.
pub fn materialization_reset() {
    MATERIALIZATION_COUNT.store(0, Ordering::Relaxed);
}
