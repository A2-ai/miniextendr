# Plan: Parallel Column Processing for DataFrames

Depends on:
- [`plans/sexp-not-send.md`](sexp-not-send.md) — AltrepSexp, ensure_materialized
- [`plans/materialized-sexp.md`](materialized-sexp.md) — TypedSlice, NamedSlice extraction

## Goal

Column-group partitioning and rayon dispatch for data-frame processing. All ALTREP
materialization happens on the R main thread before rayon tasks start. Since SEXP is
Send + Sync, we use `rayon::in_place_scope` so the setup runs on the current (R) thread
and only spawned tasks go to pool threads.

## Design

### Column Groups

```rust
pub struct ColumnGroup<'a> {
    pub columns: Vec<NamedSlice<'a>>,
    pub total_bytes: usize,
}

pub struct ColumnGroupMut<'a> {
    pub columns: Vec<NamedSliceMut<'a>>,
    pub total_bytes: usize,
}
```

### Weighted Partitioner

```rust
pub fn partition_by_weight<T>(
    items: &[T],
    weight: impl Fn(&T) -> usize,
    n_groups: usize,
) -> Vec<Vec<usize>>;
```

Greedy LPT: sort descending by weight, assign each to the lightest group.

### Dispatch APIs (rayon feature-gated)

```rust
#[cfg(feature = "rayon")]
pub unsafe fn par_columns<F>(
    df: &DataFrameView,
    columns: Option<&[&str]>,
    f: F,
) -> Result<(), ColumnSliceError>
where
    F: Fn(usize, &ColumnGroup<'_>) + Send + Sync;

#[cfg(feature = "rayon")]
pub unsafe fn par_columns_mut<F>(
    df: &mut DataFrameView,
    columns: Option<&[&str]>,
    f: F,
) -> Result<(), ColumnSliceError>
where
    F: Fn(usize, &mut ColumnGroupMut<'_>) + Send + Sync;
```

### Execution Flow

```rust
pub unsafe fn par_columns<F>(df: &DataFrameView, columns: Option<&[&str]>, f: F)
    -> Result<(), ColumnSliceError>
where F: Fn(usize, &ColumnGroup<'_>) + Send + Sync
{
    // Phase 1: Extract + materialize on R main thread
    // typed_columns calls ensure_materialized internally for each column
    let slices = match columns {
        Some(names) => df.typed_columns(names)?,
        None => df.all_typed_columns(),
    };

    // Phase 2: Partition by byte weight
    let n = rayon::current_num_threads();
    let indices = partition_by_weight(&slices, |s| s.data.byte_weight(), n);
    let groups = build_groups(slices, indices);

    // Phase 3: Dispatch via in_place_scope
    // Outer closure runs on current (R) thread.
    // s.spawn() tasks run on pool threads — only touch Send slices.
    rayon::in_place_scope(|s| {
        for (idx, group) in groups.iter().enumerate() {
            s.spawn(move |_| f(idx, group));
        }
    });

    Ok(())
}
```

No R API calls inside spawned tasks. ALTREP is resolved before `in_place_scope`.
The slices are plain `&[T]` — Send + Sync, safe on any thread.

## Mutable Safety

For `par_columns_mut`:
- Alias check by data pointer identity (not column name)
- Each mutable column in exactly one group
- Non-overlapping mutable slices

## Optional: Row Sub-chunking

For tall/narrow frames, `TypedSlice::sub_slice` enables row-band splitting:

```rust
impl<'a> TypedSlice<'a> {
    pub fn sub_slice(&self, start: usize, len: usize) -> TypedSlice<'a>;
}
```

## Usage Example

```rust
#[miniextendr]
pub fn normalize_columns(mut df: DataFrameView) -> DataFrameView {
    unsafe {
        par_columns_mut(&mut df, None, |_idx, group| {
            for col in &mut group.columns {
                if let TypedSliceMut::Real(data) = &mut col.data {
                    let mean = data.iter().sum::<f64>() / data.len() as f64;
                    let std = (data.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                        / data.len() as f64).sqrt();
                    if std > 0.0 {
                        data.iter_mut().for_each(|x| *x = (*x - mean) / std);
                    }
                }
            }
        }).unwrap();
    }
    df
}
```

## Files to Modify

- `miniextendr-api/src/optionals/rayon_bridge.rs`
- `miniextendr-api/src/lib.rs` — re-exports
- `rpkg/src/rust/dataframe_rayon_tests.rs`
- `rpkg/tests/testthat/test-dataframe-rayon.R`

## Benchmarks

`miniextendr-bench/benches/par_columns.rs` (feature `rayon`):
- Partitioner overhead
- Materialization cost
- `par_columns` vs serial

## Verification

1. `cargo check --workspace`
2. `cargo test -p miniextendr-api --features rayon`
3. `just devtools-test`
4. `cargo clippy --workspace --all-features`

## Success Criteria

- No R API calls on rayon threads
- ALTREP materialized before parallel dispatch
- No aliasing UB in mutable mode
- Measurable speedup on wide/tall workloads
