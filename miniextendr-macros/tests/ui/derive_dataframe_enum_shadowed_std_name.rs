//! Test: the derive matches `std::string::String` to a bare `String` without
//! resolving names (#2084). When a local type shadows `String`, the two are
//! different types, so rustc rejects the shared column instead of the derive
//! storing one variant's value as the other's type.

use miniextendr_api::{DataFrameRow, IntoR};

// Convertible on its own, so only the merged column fails.
#[derive(IntoR)]
pub struct String(std::string::String);

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum Event {
    Std { label: std::string::String },
    Local { label: String },
}

fn main() {}
