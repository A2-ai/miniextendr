+++
title = "ColumnarDataFrame: all-None Option columns"
weight = 50
description = "ColumnarDataFrame::from_rows discovers column types by probing runtime values. When every row has None for an Option<T> field the probe never sees a Some, the column stays ColumnBuffer::Generic, and R received list(NULL, NULL, …) instead of an atomic vector with NA. Tibble and dplyr treat list(NULL, …) as a list-column — it cannot be compared to scalars, does not coerce cleanly, and appears as <list> rather than <lgl>/<int>/<dbl>/<chr> in str()."
+++

## The old failure

`ColumnarDataFrame::from_rows` discovers column types by probing runtime values.
When every row has `None` for an `Option<T>` field the probe never sees a `Some`,
the column stays `ColumnBuffer::Generic`, and R received `list(NULL, NULL, …)`
instead of an atomic vector with `NA`. Tibble and dplyr treat `list(NULL, …)` as
a list-column — it cannot be compared to scalars, does not coerce cleanly, and
appears as `<list>` rather than `<lgl>/<int>/<dbl>/<chr>` in `str()`.

## The new behaviour

At assembly time, if a `ColumnBuffer::Generic` column has *every* entry as `None`,
the column is emitted as an `LGLSXP` of length `nrow` filled with `NA_logical_`
rather than a `VECSXP` of `NULL` elements. This is the assembly-time downgrade.
No user hint, schema annotation, or derive macro is involved.

The discriminator is in the buffer: `Vec<Option<SEXP>>` where `push_na` (pad for
missing rows) stores `None`, and `push_value(&None::<T>)` serializes through
`RSerializer::serialize_none` → returns `SEXP::nil()` → stores `Some(SEXP::nil())`.
Both represent "no value" in the generic-list context. The downgrade checks
`v.iter().all(|e| e.is_none() || e.map_or(false, |s| s.is_nil()))` — all entries are
either missing or NULL. Only this condition fires the downgrade.

## The R coercion guarantee

R's coercion rules make logical NA invisible downstream:

```r
c(NA, 1L)      # integer NA + integer → integer vector
c(NA, "x")     # logical NA + character → character vector
c(NA, 3.14)    # logical NA + double → double vector
```

`dplyr::bind_rows()`, `tibble::as_tibble()`, `mutate()`, and `coalesce()` all
coerce on contact. An all-NA logical column is indistinguishable from an all-NA
typed column for everything users do downstream.

## When this is not what you want

In the rare case where you need a specific typed NA column (for example, R
metadata systems that inspect the column type before any values arrive), use
`with_column` to inject a typed NA vector explicitly after assembly:

```rust
use miniextendr_api::IntoR;

let na_integer = vec![Option::<i32>::None; nrow].into_sexp(); // INTSXP of NA_integer_
df.with_column("stored_size", na_integer)
```

This pattern is already described in the issue body for `stored_size: Option<u64>`.
