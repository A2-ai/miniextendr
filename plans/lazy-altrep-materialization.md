# Plan: `Lazy<T>` — Opt-in ALTREP lazy materialization for all conversion types

## Goal

Introduce `Lazy<T>` as an opt-in wrapper that returns Rust data to R as ALTREP vectors
instead of copying. R reads elements on demand; full materialization only happens if R
needs a contiguous pointer (e.g., for `.Internal(inspect(x))`).

**Key principle**: Opt-in, not default.
- `fn f() -> Vec<f64>` — eager copy (unchanged)
- `fn f() -> Lazy<Vec<f64>>` — ALTREP, R reads on demand

## `Lazy<T>` type

```rust
pub type Lazy<T> = Altrep<T>;
```

Just a type alias for the existing `Altrep<T>` wrapper. No new struct needed.

## Types to support

### Already working (Vec/Box — no work needed)
- `Lazy<Vec<i32/f64/u8/bool/String>>`, `Lazy<Vec<Option<i32/f64/bool>>>`
- `Lazy<Box<[i32/f64/u8/bool/String]>>`

### New: Arrow arrays
| Type | ALTREP family | Dataptr? | Notes |
|------|-------------|----------|-------|
| `Lazy<Float64Array>` | real | YES (zero-copy, `null_count==0`) | Arrow buffer pointer directly |
| `Lazy<Int32Array>` | integer | YES (zero-copy, `null_count==0`) | Arrow buffer pointer directly |
| `Lazy<UInt8Array>` | raw | YES (zero-copy) | No NAs in raw |
| `Lazy<BooleanArray>` | logical | NO (bit-packed) | Elt callback unpacks bits |
| `Lazy<StringArray>` | string | NO | Elt creates CHARSXP on demand |
| `Lazy<RecordBatch>` | list | NO | Per-column lazy materialization |

Arrow NA for Dataptr: Arrow's null bitmap marks nulls but data buffer has garbage at those positions.
If `null_count > 0`, Dataptr returns `None` → R materializes via Elt (which handles NA correctly).
If `null_count == 0`, Dataptr returns Arrow buffer pointer directly (true zero-copy, O(1)).

### New: ndarray
| Type | ALTREP family | Dataptr? | Notes |
|------|-------------|----------|-------|
| `Lazy<Array1<f64>>` | real | YES (if standard layout) | ndarray's contiguous buffer |
| `Lazy<Array1<i32>>` | integer | YES | Same |
| `Lazy<Array2<f64>>` | real | YES (if Fortran order) | Column-major matches R |
| `Lazy<Array2<i32>>` | integer | YES (if Fortran order) | Same |

Row-major ndarray: Dataptr returns `None`, Elt computes index translation on demand.

### New: nalgebra
| Type | ALTREP family | Dataptr? | Notes |
|------|-------------|----------|-------|
| `Lazy<DVector<f64>>` | real | YES | VecStorage is contiguous |
| `Lazy<DVector<i32>>` | integer | YES | Same |
| `Lazy<DMatrix<f64>>` | real | YES | Column-major by default |
| `Lazy<DMatrix<i32>>` | integer | YES | Same |

## Implementation per type

Each type needs: `TypedExternal` + `AltrepLen` + `Alt*Data` + optionally `AltrepDataptr` + `impl_inferbase_*!` macro.

### Example: Arrow Float64Array

```rust
impl TypedExternal for Float64Array {
    const TYPE_NAME: &'static str = "arrow::Float64Array";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::Float64Array\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::Float64Array\0";
}

impl AltrepLen for Float64Array {
    fn length(&self) -> usize { self.len() }
}

impl AltRealData for Float64Array {
    fn elt(&self, i: usize) -> f64 {
        if self.is_null(i) { NA_REAL } else { self.value(i) }
    }
}

impl AltrepDataptr<f64> for Float64Array {
    fn dataptr(&self) -> Option<*const f64> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr())
        } else {
            None
        }
    }
}

impl_inferbase_real!(Float64Array);
```

## Files to modify

1. `miniextendr-api/src/into_r.rs` — add `pub type Lazy<T> = Altrep<T>;`
2. `miniextendr-api/src/optionals/arrow_impl.rs` — ALTREP impls for Arrow arrays
3. `miniextendr-api/src/optionals/ndarray_impl.rs` — ALTREP impls for Array1/Array2
4. `miniextendr-api/src/optionals/nalgebra_impl.rs` — ALTREP impls for DVector/DMatrix
5. `miniextendr-api/src/prelude.rs` — re-export `Lazy`
6. `rpkg/src/rust/lazy_tests.rs` (new) — test fixtures
7. `rpkg/tests/testthat/test-lazy.R` (new) — R tests
8. `rpkg/src/rust/lib.rs` — add `mod lazy_tests;`

## Verification

- `cargo check --features arrow,ndarray,nalgebra -p miniextendr-api`
- `just rcmdinstall && devtools::test("rpkg", filter = "lazy")`
- R: `is.altrep(lazy_f64_result())` returns TRUE
