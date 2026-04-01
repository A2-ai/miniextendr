//! Test fixtures for growth_debug module (requires `growth-debug` feature).

use miniextendr_api::growth_debug;
use miniextendr_api::prelude::*;

/// Record growth events and return the count.
#[miniextendr]
pub fn growth_debug_test() -> i32 {
    growth_debug::reset();
    growth_debug::record_growth("test_collection");
    growth_debug::record_growth("test_collection");
    growth_debug::record_growth("test_collection");
    growth_debug::get_count("test_collection") as i32
}
