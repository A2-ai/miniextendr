# Plan: Builder-First Parallel Column Chunks for DataFrames

## Summary

Replace a closure-only API with a builder-style API that feels like `itertools` and Rayon:

1. Create a builder from a function (`par_columns(...)` / `par_columns_mut(...)`).
2. Configure behavior by chaining methods (or by passing an options struct).
3. Call `.build()` to materialize a reusable plan object.
4. Consume the plan object as a Rayon parallel iterator (`IntoParallelIterator`) or via convenience runners.

This keeps configuration ergonomic while preserving an explicit options data model.

## Prerequisite

Depends on `plans/materialized-sexp.md` for:
- `TypedSlice` / `TypedSliceMut`
- `NamedSlice` / `NamedSliceMut`
- `MaterializeBatch`
- `ColumnSliceError`

## Goals

- Builder ergonomics for caller-facing API.
- Deterministic grouping (build-time), nondeterministic execution order (runtime).
- No R API calls on Rayon threads.
- Support both options struct and fluent chain.

## Public API

### Constructor functions

```rust
#[cfg(feature = "rayon")]
pub fn par_columns<'a>(df: &'a DataFrameView) -> ParColumnsBuilder<'a>;

#[cfg(feature = "rayon")]
pub fn par_columns_mut<'a>(df: &'a mut DataFrameView) -> ParColumnsMutBuilder<'a>;
```

### Options struct

```rust
#[derive(Clone, Debug)]
pub struct ParColumnsOptions {
    pub n_groups: Option<usize>,
    pub row_chunk_rows: Option<usize>,
    pub include_strings: bool,
    pub min_rows_per_job: Option<usize>,
    pub max_rows_per_job: Option<usize>,
}

impl Default for ParColumnsOptions {
    // n_groups = None => default to rayon::current_num_threads() at build time
    // row_chunk_rows = None => no row sub-chunking
    // include_strings = true for immutable, ignored for mutable
}
```

### Builders

```rust
pub struct ParColumnsBuilder<'a> {
    df: &'a DataFrameView,
    columns: Option<Vec<String>>, // None => all sliceable columns
    options: ParColumnsOptions,
}

pub struct ParColumnsMutBuilder<'a> {
    df: &'a mut DataFrameView,
    columns: Option<Vec<String>>,
    options: ParColumnsOptions,
}
```

Builder methods:

```rust
impl<'a> ParColumnsBuilder<'a> {
    pub fn columns(mut self, names: &[&str]) -> Self;
    pub fn all_columns(mut self) -> Self;
    pub fn options(mut self, opts: ParColumnsOptions) -> Self;
    pub fn n_groups(mut self, n: usize) -> Self;
    pub fn row_chunk_rows(mut self, rows: usize) -> Self;
    pub fn include_strings(mut self, yes: bool) -> Self;
    pub fn build(self) -> Result<ParColumnsPlan<'a>, ColumnSliceError>;
}

impl<'a> ParColumnsMutBuilder<'a> {
    pub fn columns(mut self, names: &[&str]) -> Self;
    pub fn all_columns(mut self) -> Self;
    pub fn options(mut self, opts: ParColumnsOptions) -> Self;
    pub fn n_groups(mut self, n: usize) -> Self;
    pub fn row_chunk_rows(mut self, rows: usize) -> Self;
    pub fn build(self) -> Result<ParColumnsMutPlan<'a>, ColumnSliceError>;
}
```

### Built plan objects

```rust
pub struct ColumnGroup<'a> {
    pub columns: Vec<NamedSlice<'a>>,
    pub total_bytes: usize,
}

pub struct ColumnGroupMut<'a> {
    pub columns: Vec<NamedSliceMut<'a>>,
    pub total_bytes: usize,
}

pub struct ParColumnsPlan<'a> {
    groups: Vec<ColumnGroup<'a>>,
    options: ParColumnsOptions,
}

pub struct ParColumnsMutPlan<'a> {
    groups: Vec<ColumnGroupMut<'a>>,
    options: ParColumnsOptions,
}
```

Expose both iterator and convenience APIs:

```rust
impl<'a> ParColumnsPlan<'a> {
    pub fn groups(&self) -> &[ColumnGroup<'a>];
    pub fn into_groups(self) -> Vec<ColumnGroup<'a>>;
    pub fn into_par_iter(self) -> impl rayon::iter::IndexedParallelIterator<Item = (usize, ColumnGroup<'a>)>;
}

impl<'a> ParColumnsMutPlan<'a> {
    pub fn groups(&self) -> &[ColumnGroupMut<'a>];
    pub fn into_groups(self) -> Vec<ColumnGroupMut<'a>>;
    pub fn into_par_iter(self) -> impl rayon::iter::IndexedParallelIterator<Item = (usize, ColumnGroupMut<'a>)>;
}

impl<'a> rayon::iter::IntoParallelIterator for ParColumnsPlan<'a> { /* index + group */ }
impl<'a> rayon::iter::IntoParallelIterator for ParColumnsMutPlan<'a> { /* index + group */ }
```

Optional convenience runners:

```rust
impl<'a> ParColumnsPlan<'a> {
    pub fn try_for_each<E, F>(self, f: F) -> Result<(), E>
    where
        F: Fn(usize, &ColumnGroup<'a>) -> Result<(), E> + Send + Sync,
        E: Send;
}

impl<'a> ParColumnsMutPlan<'a> {
    pub fn try_for_each<E, F>(self, f: F) -> Result<(), E>
    where
        F: Fn(usize, &mut ColumnGroupMut<'a>) -> Result<(), E> + Send + Sync,
        E: Send;
}
```

## Behavior and Semantics

## Build phase

`build()` does all R-facing materialization work before any Rayon execution:

1. Resolve selected columns.
2. Materialize to `NamedSlice` / `NamedSliceMut`.
3. In mutable mode, reject aliasing by pointer identity.
4. Partition by byte weight into groups.
5. Apply optional row sub-chunking metadata.

## Partitioning

Use deterministic LPT-style grouping:

1. Sort columns by `byte_weight DESC`.
2. Tie-break by original column index.
3. Assign each column to the currently lightest group.
4. Tie-break group choice by lowest `group_idx`.

This makes group membership deterministic for a fixed input/options set.

## Execution phase

- Group processing order is not guaranteed (Rayon work stealing).
- Closures must be pure Rust: no `ffi::*`, no `with_r_thread`, no `.into_sexp()` inside parallel tasks.

## Example usage

```rust
use miniextendr_api::rayon_bridge::rayon::prelude::*;

let plan = par_columns(&df)
    .columns(&["x", "y", "z"])
    .n_groups(8)
    .row_chunk_rows(20_000)
    .build()?;

plan.into_par_iter().try_for_each(|(group_idx, group)| -> Result<(), MyErr> {
    // process group.columns
    Ok(())
})?;
```

Mutable:

```rust
let mut_plan = par_columns_mut(&mut df)
    .all_columns()
    .n_groups(4)
    .build()?;

mut_plan.into_par_iter().for_each(|(_idx, mut group)| {
    for col in &mut group.columns {
        // mutate numeric slices only
    }
});
```

## Files to Modify

- `miniextendr-api/src/optionals/rayon_bridge.rs`
- `miniextendr-api/src/lib.rs` (exports)
- `rpkg/src/rust/dataframe_rayon_tests.rs`
- `rpkg/tests/testthat/test-dataframe-rayon.R`
- `miniextendr-bench/benches/par_columns.rs`

## Test Cases

### Unit tests (api crate)

1. Builder default values and method override precedence.
2. `build()` rejects `n_groups == 0`.
3. Deterministic partition output for equal-weight and skewed columns.
4. Mutable alias rejection via pointer identity.
5. `into_par_iter()` yields exactly one item per group with stable `(group_idx, membership)`.

### Integration tests (rpkg)

1. Immutable builder path with selected columns.
2. Mutable builder path writes expected values and preserves untouched columns.
3. String inclusion/exclusion behavior in immutable mode.
4. Below-threshold/small-input behavior falls back to sensible grouping without panic.

### Benchmarks

1. Serial vs builder-parallel processing.
2. `n_groups` sweep.
3. Row chunk sweep.
4. Builder overhead vs direct function call overhead.

## Verification

1. `cargo check --workspace`
2. `cargo test -p miniextendr-api --features rayon`
3. `just devtools-test`
4. `cargo clippy --workspace --all-features`
5. `cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features rayon --bench par_columns`

## Assumptions and Defaults

- Global Rayon pool only for now.
- `n_groups` default is resolved at build time from `rayon::current_num_threads()`.
- `row_chunk_rows = None` means no row sub-chunking.
- Mutable mode never includes string columns.
- `materialized-sexp.md` is implemented first.
