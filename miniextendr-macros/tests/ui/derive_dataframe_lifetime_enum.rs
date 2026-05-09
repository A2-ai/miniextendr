//! Test: #[derive(DataFrameRow)] on an enum with a borrowed `&'a str` field
//! fails at the IntoR bound level, not at the derive-macro rejection level.
//!
//! The DataFrameRow derive macro does not explicitly reject enum types with lifetime
//! parameters — it calls `split_for_impl()` identically to the struct path and threads
//! the lifetime through the companion struct and all impl blocks.
//!
//! However, enum variant fields that borrow `&'a str` produce a `Vec<Option<&str>>`
//! companion column, which currently has no `IntoR` impl (only `Vec<&str>` is
//! implemented). This is a downstream type-system gap, not a macro rejection.
//!
//! Workaround: use `String` for text fields in enum DataFrameRow types.
//! See issue #460 for tracking `Vec<Option<&str>>: IntoR`.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
enum Event<'a> {
    Row { label: &'a str, value: f64 },
}

fn main() {}
