# Plan: Typed Column Extraction for DataFrames

Depends on [`plans/sexp-not-send.md`](sexp-not-send.md) for `AltrepSexp` and `ensure_materialized`.

## Problem

Data frame columns are R vectors (SEXP). Some may be ALTREP, where `DATAPTR_RO` triggers
R internals. Users need plain Rust slices for parallel code. The extraction methods must
handle the ALTREP check and materialization transparently.

## Design

`DataFrameView::typed_column()` calls `ensure_materialized(sexp)` (runtime ALTREP check),
then `DATAPTR_RO` to get a data pointer, returning a plain `&[T]` slice. For heterogeneous
access, `TypedSlice<'a>` is a runtime-typed enum. Strings materialize into `Vec<Option<String>>`.

No wrapper types. No typestate. Just runtime check + plain slices.

## Types

### `TypedSlice<'a>`

```rust
pub enum TypedSlice<'a> {
    Integer(&'a [i32]),
    Real(&'a [f64]),
    Logical(&'a [RLogical]),
    Raw(&'a [u8]),
    Complex(&'a [Rcomplex]),
    String(Vec<Option<String>>),
}
```

Send + Sync automatically. Methods: `len()`, `element_size()`, `byte_weight()`,
`as_integer()`, `as_real()`, etc.

### `TypedSliceMut<'a>`

```rust
pub enum TypedSliceMut<'a> {
    Integer(&'a mut [i32]),
    Real(&'a mut [f64]),
    Logical(&'a mut [RLogical]),
    Raw(&'a mut [u8]),
    Complex(&'a mut [Rcomplex]),
}
```

No String variant. Send but not Sync.

### Named wrappers

```rust
pub struct NamedSlice<'a> {
    pub name: String,
    pub data: TypedSlice<'a>,
}

pub struct NamedSliceMut<'a> {
    pub name: String,
    pub data: TypedSliceMut<'a>,
}
```

## DataFrameView Methods

```rust
impl DataFrameView {
    /// Extract a typed slice. Materializes ALTREP if needed via ensure_materialized.
    pub unsafe fn typed_column<'a>(&'a self, name: &str)
        -> Result<NamedSlice<'a>, ColumnSliceError>;

    pub unsafe fn typed_columns<'a>(&'a self, names: &[&str])
        -> Result<Vec<NamedSlice<'a>>, ColumnSliceError>;

    pub unsafe fn all_typed_columns<'a>(&'a self) -> Vec<NamedSlice<'a>>;

    pub unsafe fn typed_column_mut<'a>(&'a mut self, name: &str)
        -> Result<NamedSliceMut<'a>, ColumnSliceError>;

    /// Rejects aliasing by pointer identity.
    pub unsafe fn typed_columns_mut<'a>(&'a mut self, names: &[&str])
        -> Result<Vec<NamedSliceMut<'a>>, ColumnSliceError>;

    pub unsafe fn all_typed_columns_mut<'a>(&'a mut self) -> Vec<NamedSliceMut<'a>>;
}
```

### Implementation sketch

```rust
pub unsafe fn typed_column<'a>(&'a self, name: &str)
    -> Result<NamedSlice<'a>, ColumnSliceError>
{
    let sexp = self.column_raw(name)
        .ok_or_else(|| ColumnSliceError::NotFound(name.into()))?;

    let sexp = ensure_materialized(sexp);  // ALTREP check + force
    let n = self.nrow;

    let data = match ffi::TYPEOF(sexp) {
        INTSXP => TypedSlice::Integer(from_r::r_slice(ffi::DATAPTR_RO(sexp) as *const i32, n)),
        REALSXP => TypedSlice::Real(from_r::r_slice(ffi::DATAPTR_RO(sexp) as *const f64, n)),
        LGLSXP => TypedSlice::Logical(from_r::r_slice(ffi::DATAPTR_RO(sexp) as *const RLogical, n)),
        RAWSXP => TypedSlice::Raw(from_r::r_slice(ffi::DATAPTR_RO(sexp) as *const u8, n)),
        CPLXSXP => TypedSlice::Complex(from_r::r_slice(ffi::DATAPTR_RO(sexp) as *const Rcomplex, n)),
        STRSXP => {
            let mut v = Vec::with_capacity(n);
            for i in 0..n as ffi::R_xlen_t {
                let elt = ffi::STRING_ELT(sexp, i);
                if elt == ffi::R_NaString {
                    v.push(None);
                } else {
                    let c = ffi::Rf_translateCharUTF8(elt);
                    v.push(Some(std::ffi::CStr::from_ptr(c).to_string_lossy().into_owned()));
                }
            }
            TypedSlice::String(v)
        }
        other => return Err(ColumnSliceError::UnsupportedType {
            name: name.into(), sexptype: other
        }),
    };
    Ok(NamedSlice { name: name.into(), data })
}
```

Factors (INTSXP + class "factor") treated as Integer.

## Error Type

```rust
pub enum ColumnSliceError {
    NotFound(String),
    UnsupportedType { name: String, sexptype: SEXPTYPE },
    DuplicateMutableAlias { name: String },
}
```

Implement `Display`, `Error`, `From<ColumnSliceError> for SexpError`.

## Files to Modify

- `miniextendr-api/src/dataframe.rs` — TypedSlice, NamedSlice, extraction methods
- `miniextendr-api/src/lib.rs` — re-exports

## Verification

1. `cargo check --workspace`
2. `cargo test -p miniextendr-api`
3. `cargo clippy --workspace`

### Compile-time assertions

- `TypedSlice<'_>: Send + Sync`
- `TypedSliceMut<'_>: Send` and `!Sync`
- `NamedSlice<'_>: Send + Sync`

## Non-Goals

- List-column (`VECSXP`) materialization
- Mutable string columns
- Wrapper types beyond the TypedSlice enum
