# TODO: ALTREP Lazy DataFrame Columns

From plan 02 phase 4 (deferred).

## Idea

DataFrameView columns are currently all eager (materialized SEXPs).
Add a `LazyDataFrame` variant where columns can be ALTREP-backed,
computing values on access.

```rust
pub struct LazyDataFrame {
    columns: Vec<LazyColumn>,
    names: Vec<String>,
    nrow: usize,
}

enum LazyColumn {
    Eager(SEXP),                       // Already materialized
    Altrep(Box<dyn AltrepColumnData>), // Compute on access
}
```

## Context

Phases 1-3 of the DataFrame plan are done (`DataFrameView` in
`miniextendr-api/src/dataframe.rs`). This is the remaining phase.

## Priority

Low — current eager materialization works for most cases. Should come
after DataFrameView is battle-tested.
