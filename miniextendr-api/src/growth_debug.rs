//! Debug instrumentation for tracking collection growth events.
//!
//! When the `growth-debug` feature is enabled, this module provides thread-local
//! counters that track how many times collections (Vec, Arena backing, etc.)
//! reallocate. This helps diagnose missing `with_capacity` calls and unexpected
//! growth during hot paths.
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::{track_growth, report_growth};
//!
//! fn process(data: &[i32]) {
//!     let mut out = Vec::new();
//!     for &x in data {
//!         let old_cap = out.capacity();
//!         out.push(x * 2);
//!         if out.capacity() != old_cap {
//!             track_growth!("process_output");
//!         }
//!     }
//!     report_growth!();
//! }
//! ```
//!
//! When the feature is disabled, both macros compile to no-ops with zero overhead.

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static GROWTH_COUNTERS: RefCell<HashMap<&'static str, u64>> = RefCell::new(HashMap::new());
}

/// Increment the growth counter for the named collection.
#[inline]
pub fn record_growth(name: &'static str) {
    GROWTH_COUNTERS.with(|counters| {
        *counters.borrow_mut().entry(name).or_insert(0) += 1;
    });
}

/// Print all growth counters to stderr and reset them.
pub fn report_and_reset() {
    GROWTH_COUNTERS.with(|counters| {
        let mut map = counters.borrow_mut();
        if map.is_empty() {
            return;
        }
        eprintln!("[growth-debug] Collection growth events:");
        let mut entries: Vec<_> = map.drain().collect();
        entries.sort_by_key(|(name, _)| *name);
        for (name, count) in entries {
            eprintln!("  {}: {} reallocation(s)", name, count);
        }
    });
}

/// Get the current growth count for a named collection (for testing).
pub fn get_count(name: &'static str) -> u64 {
    GROWTH_COUNTERS.with(|counters| counters.borrow().get(name).copied().unwrap_or(0))
}

/// Reset all growth counters.
pub fn reset() {
    GROWTH_COUNTERS.with(|counters| {
        counters.borrow_mut().clear();
    });
}
