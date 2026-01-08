//! Bench plan: `benches/list.rs`
//!
//! Groups:
//!
//! 1) `get_named` vs `get_index`
//!    - best-case (first element) vs worst-case (last element)
//!    - sizes from `NAMED_LIST_SIZES`
//!
//! 2) derive-driven conversions
//!    - `#[derive(IntoList)]` named vs tuple structs
//!    - `#[derive(TryFromList)]` named vs tuple structs
//!    - `#[into_list(ignore)]` field skipping impact (reads + bounds)
//!
//! Notes:
//! - Keep list fixtures protected for the entire benchmark process.
//! - For `TryFromList`, avoid including list allocation cost unless explicitly measuring it
//!   (use protected fixture `SEXP`s or `divan::Bencher::with_inputs`).
