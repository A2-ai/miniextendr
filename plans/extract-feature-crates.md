# Plan: Extract Optional Integrations into `miniextendr-*` Crates (No Cycles)

## Goal

Move optional integration logic out of `miniextendr-api` into dedicated crates while keeping `miniextendr-api` as the user-facing R boundary.

## Critical Constraint

There must be **no dependency cycles**.

### Required dependency direction

```text
miniextendr-api --optional dep--> miniextendr-<feature> --> third-party crate
```

Allowed:
- `miniextendr-api -> miniextendr-ndarray -> ndarray`

Not allowed:
- `miniextendr-api -> miniextendr-ndarray -> miniextendr-api`

## Architecture Rules

1. `miniextendr-*` feature crates are **pure Rust integration crates**.
2. Feature crates **must not depend on `miniextendr-api`**.
3. `miniextendr-api` keeps all R-boundary code (`SEXP`, `TryFromSexp`, `IntoR`, `TryCoerce`, allocation/protection).
4. Feature crates can expose:
   - helper conversions (Rust <-> Rust),
   - extension/adapter traits,
   - re-exports of third-party types used by API stubs.
5. `miniextendr-api` optional modules call feature-crate helpers.

## Example: `ndarray`

Feature crate (`miniextendr-ndarray`) owns pure helpers:

```rust
// miniextendr-ndarray/src/lib.rs
pub use ndarray::{Array1, Array2, ArrayD};

pub fn array2_from_col_major<T: Copy>(data: &[T], nrow: usize, ncol: usize) -> Array2<T> {
    ndarray::Array2::from_shape_vec((nrow, ncol).f(), data.to_vec()).expect("shape mismatch")
}
```

API crate keeps R boundary:

```rust
// miniextendr-api/src/optionals/ndarray_impl.rs
#[cfg(feature = "ndarray")]
impl<T: crate::ffi::RNativeType + Copy> TryFromSexp for miniextendr_ndarray::Array2<T> {
    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        // read dims + pointer via api helpers
        // call miniextendr_ndarray::array2_from_col_major(...)
        // return typed ndarray value
    }
}
```

This avoids any `feature -> api` edge.

## Crate Layout

Planned crates:
- `miniextendr-ndarray`
- `miniextendr-nalgebra`
- `miniextendr-serde`
- `miniextendr-rayon`
- `miniextendr-json` (serde_json + toml)
- `miniextendr-rand`
- `miniextendr-time`
- `miniextendr-numeric`
- `miniextendr-string`
- `miniextendr-collections`
- `miniextendr-binary`
- `miniextendr-format`

## What Stays in `miniextendr-api`

Always stays in API:
- `TryFromSexp` / `IntoR` impls.
- `TryCoerce` impls.
- direct `ffi::*` calls.
- GC/protection/thread-routing interactions.

## Macro Compatibility (`rayon`)

`miniextendr-macros` currently generates paths using `::miniextendr_api::rayon_bridge::ColumnWriter`.

Phase 1 (safe migration):
- keep `ColumnWriter` in `miniextendr-api`.
- extract non-macro-coupled rayon helpers first.

Phase 2 (optional):
- move `ColumnWriter` to `miniextendr-rayon`.
- update macros and keep `miniextendr-api` re-export for backward compatibility.

## Cargo/Feature Wiring

In `miniextendr-api/Cargo.toml`:
- replace direct optional third-party deps with optional `miniextendr-*` deps where feasible.
- keep API feature names stable (`ndarray`, `nalgebra`, etc.) for user compatibility.

Example:

```toml
[features]
ndarray = ["dep:miniextendr-ndarray"]

[dependencies]
miniextendr-ndarray = { path = "../miniextendr-ndarray", optional = true }
```

## `rpkg` + Vendoring Changes

Use this repo's existing patch source convention:
- `rpkg/src/rust/Cargo.toml` uses `[patch."https://github.com/CGMossa/miniextendr"]`
- `minirextendr/R/vendor.R` manages vendored crate list and patch insertion.

Do **not** switch to `[patch.crates-io]` for this workflow.

## API Helper Visibility Policy

Do not make low-level helpers (`r_slice`, `r_slice_mut`, `charsxp_to_str`) broadly public just for extraction.

Preferred:
- keep them `pub(crate)`.
- add small, purpose-built bridge functions/modules where cross-module reuse is needed.

## Phased Execution

1. Create `codex/extract-feature-crates` branch.
2. Add one pilot crate (`miniextendr-ndarray`) with no `api` dependency.
3. Rewire `miniextendr-api` `ndarray` feature to optional dependency on that crate.
4. Validate:
   - `cargo check -p miniextendr-ndarray`
   - `cargo check -p miniextendr-api --features ndarray`
   - `cargo test -p rpkg --features ndarray` (or repo equivalent target)
5. Repeat feature-by-feature.
6. Update vendoring scripts (`vendor.R`) and templates once at the end.
7. Run full verification.

## Verification

1. `cargo check --workspace --all-features`
2. `cargo clippy --workspace --all-features`
3. `cargo test --workspace`
4. `just devtools-test`
5. `just cross-test`
6. `just vendor-sync-check`

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| accidental cycle between api and feature crate | enforce rule: feature crates never depend on api |
| macro path regressions (rayon) | keep `ColumnWriter` in api for phase 1 |
| vendoring drift | update `vendor.R` crate list + run vendor sync checks |
| API breakage from feature renames | keep existing feature names in `miniextendr-api` |
