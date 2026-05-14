//! Compile-pass test: DataFrameRow on a generic unit-only enum.
//!
//! Issue #487 / PR #546 — verify that `#[derive(DataFrameRow)]` auto-emits
//! `UnitEnumFactor`, `IntoR`, and `IntoList` for enums with generic parameters
//! (the `impl_generics.to_token_stream().is_empty() == false` branch in
//! `enum_expansion.rs`).
//!
//! # Why const params only
//!
//! Rust E0392 forbids type parameters that appear in no variant field.  A
//! truly unit-only enum (`enum Tag<T> { A, B }`) is therefore rejected by the
//! Rust compiler *before* the proc-macro runs, making a type-param fixture
//! impossible without adding a non-unit (PhantomData) variant — which in turn
//! would fail the unit-only guard in `enum_expansion.rs`.
//!
//! Const parameters are exempt from E0392 (Rust permits unused const params)
//! and exercise the identical generated code path: `impl_generics` is
//! non-empty in both cases, `ty_generics` carries the param, and `where_clause`
//! propagates correctly.  The companion struct `PhantomData<T>` emission path
//! (lines 722-729 of `enum_expansion.rs`) is only reachable when `type_params`
//! is non-empty, which requires a non-unit variant — this is documented
//! in the `phantom_field` comment in `enum_expansion.rs`.

use miniextendr_macros::DataFrameRow;

/// Three-variant const-generic unit-only enum — the same shape as the
/// `enum Status<T>` example from issue #487, exercised via const param.
#[allow(dead_code)]
#[derive(Clone, Copy, DataFrameRow)]
enum Tag<const V: usize> {
    A,
    B,
    C,
}

/// Verify the companion struct and `from_rows` method compile.
fn _use_companion() {
    let _df = TagDataFrame::<0>::from_rows(vec![Tag::<0>::A, Tag::<0>::B]);
}

fn main() {}
