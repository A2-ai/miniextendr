# Plan: AltrepSexp Type for ALTREP Safety

## Problem

`DATAPTR_RO` on an ALTREP vector dispatches into R internals (C callbacks, GC toggling,
allocation). This is undefined behavior on non-R threads. Currently, ALTREP vectors are
plain SEXPs — nothing prevents sending them to rayon threads and calling `DATAPTR_RO`.

## Design

SEXP stays `Send + Sync` — it's just a pointer, and for non-ALTREP vectors the data is
stable contiguous memory. The danger is specifically ALTREP.

Introduce `AltrepSexp`: a `!Send + !Sync` wrapper for ALTREP vectors. The runtime check
`ALTREP(sexp)` gates construction. Materialization is the exit from `AltrepSexp` back to
a plain SEXP or `&[T]`.

```text
SEXP (Send + Sync)
  │
  ├── ALTREP(sexp) == false → safe to DATAPTR_RO anywhere
  │
  └── ALTREP(sexp) == true → AltrepSexp (!Send + !Sync)
                                  │
                                  └── materialize() → SEXP or &[T]
```

## Core Type

```rust
/// A SEXP known to be ALTREP. `!Send + !Sync` — must be materialized on the
/// R main thread before data can be accessed or sent to other threads.
pub struct AltrepSexp {
    sexp: SEXP,
    _not_send: PhantomData<Rc<()>>,
}

impl AltrepSexp {
    /// Wrap a SEXP that is known to be ALTREP.
    ///
    /// # Safety
    ///
    /// Caller must ensure `ALTREP(sexp)` is true.
    pub unsafe fn from_raw(sexp: SEXP) -> Self {
        debug_assert!(ffi::ALTREP(sexp) != 0);
        Self { sexp, _not_send: PhantomData }
    }

    /// Check a SEXP and wrap if ALTREP.
    pub fn try_wrap(sexp: SEXP) -> Option<Self> {
        if unsafe { ffi::ALTREP(sexp) } != 0 {
            Some(Self { sexp, _not_send: PhantomData })
        } else {
            None
        }
    }

    /// Force materialization and return the (now non-ALTREP) SEXP.
    ///
    /// Calls `DATAPTR_RO` to trigger ALTREP materialization, then returns
    /// the original SEXP (which R has now materialized in place).
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread.
    pub unsafe fn materialize(self) -> SEXP {
        // DATAPTR_RO forces materialization for contiguous types.
        // For STRSXP, iterate STRING_ELT to force element materialization.
        let typ = ffi::TYPEOF(self.sexp);
        match typ {
            STRSXP => {
                let n = ffi::Rf_xlength(self.sexp);
                for i in 0..n {
                    let _ = ffi::STRING_ELT(self.sexp, i);
                }
            }
            INTSXP | REALSXP | LGLSXP | RAWSXP | CPLXSXP => {
                let _ = ffi::DATAPTR_RO(self.sexp);
            }
            _ => {} // non-vector types, nothing to materialize
        }
        self.sexp
    }

    /// Materialize and return a typed slice.
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread. Caller must ensure `T` matches
    /// the SEXP's actual type.
    pub unsafe fn materialize_slice<'a, T: RNativeType>(&'a self) -> &'a [T] {
        let ptr = ffi::DATAPTR_RO(self.sexp) as *const T;
        let len = ffi::Rf_xlength(self.sexp) as usize;
        from_r::r_slice(ptr, len)
    }

    /// Materialize strings into owned Rust data.
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread. SEXP must be STRSXP.
    pub unsafe fn materialize_strings(&self) -> Vec<Option<String>> {
        let n = ffi::Rf_xlength(self.sexp) as usize;
        let mut out = Vec::with_capacity(n);
        for i in 0..n as isize {
            let elt = ffi::STRING_ELT(self.sexp, i as ffi::R_xlen_t);
            if elt == ffi::R_NaString {
                out.push(None);
            } else {
                let cstr = ffi::Rf_translateCharUTF8(elt);
                out.push(Some(
                    std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned()
                ));
            }
        }
        out
    }

    /// Get the inner SEXP without materializing.
    ///
    /// # Safety
    ///
    /// The returned SEXP is still ALTREP. Do not call DATAPTR_RO on it
    /// from a non-R thread.
    pub unsafe fn as_raw(&self) -> SEXP {
        self.sexp
    }
}
```

## ALTREP Creation: `into_altrep()` Returns `AltrepSexp`

Currently `into_altrep()` returns `SEXP`. Change it to return `AltrepSexp`:

```rust
// Before:
pub fn into_altrep(self) -> SEXP { ... }

// After:
pub fn into_altrep(self) -> AltrepSexp { ... }
```

This prevents accidentally treating a freshly-created ALTREP vector as a regular SEXP
and sending it to another thread. To get a SEXP (e.g., to return to R), call
`materialize()` or `as_raw()`.

The `IntoR` / `IntoRAltrep` trait impls that call `into_altrep()` internally would
use `unsafe { altrep_sexp.as_raw() }` since they're returning to R (main thread).

## Helper: `ensure_materialized`

Convenience function for the common "check and materialize if needed" pattern:

```rust
/// If `sexp` is ALTREP, materialize it in place and return the SEXP.
/// If not ALTREP, return as-is.
///
/// # Safety
///
/// Must be called on the R main thread.
pub unsafe fn ensure_materialized(sexp: SEXP) -> SEXP {
    if ffi::ALTREP(sexp) != 0 {
        AltrepSexp::from_raw(sexp).materialize()
    } else {
        sexp
    }
}
```

## Integration with DataFrameView

`typed_column()` calls `ensure_materialized` internally:

```rust
pub unsafe fn typed_column<'a>(&'a self, name: &str)
    -> Result<NamedSlice<'a>, ColumnSliceError>
{
    let sexp = self.column_raw(name)
        .ok_or_else(|| ColumnSliceError::NotFound(name.to_string()))?;

    // Materialize ALTREP if needed (R main thread)
    let sexp = ensure_materialized(sexp);

    // Now safe to get data pointer — non-ALTREP, stable memory
    let data = match ffi::TYPEOF(sexp) {
        REALSXP => {
            let ptr = ffi::DATAPTR_RO(sexp) as *const f64;
            TypedSlice::Real(from_r::r_slice(ptr, self.nrow))
        }
        // ... other types
    };
    Ok(NamedSlice { name: name.to_string(), data })
}
```

## Integration with Rayon Bridge

Existing `with_r_vec` / `with_r_matrix` don't need changes — they allocate fresh
non-ALTREP vectors. Only consumption paths (reading from R vectors) need the check.

## What Stays the Same

- `SEXP` remains `Send + Sync` — no cascading breakage across the API
- `List`, `DataFrameView`, `ExternalPtr`, etc. — unchanged
- `Sendable<T>` — unchanged (still `pub`, still used for data pointers)
- Proc-macro generated code — unchanged
- ALTREP trampolines — unchanged (main thread callbacks)
- Worker thread / `with_r_thread` — unchanged

## What Changes

| Item | Change |
|------|--------|
| `AltrepSexp` type | New — `!Send + !Sync` wrapper |
| `ensure_materialized()` | New — runtime check + materialize helper |
| `into_altrep()` return type | `SEXP` → `AltrepSexp` |
| `IntoRAltrep` trait | Update to handle `AltrepSexp` return |
| `DataFrameView::typed_column()` | Calls `ensure_materialized` internally |
| `TryFromSexp` for slice types | Could call `ensure_materialized` for safety |

## Files to Modify

| File | Change |
|------|--------|
| `miniextendr-api/src/altrep_sexp.rs` | New — `AltrepSexp`, `ensure_materialized` |
| `miniextendr-api/src/into_r.rs` | `IntoRAltrep` returns `AltrepSexp` |
| `miniextendr-api/src/dataframe.rs` | `typed_column` calls `ensure_materialized` |
| `miniextendr-api/src/lib.rs` | Re-export `AltrepSexp`, `ensure_materialized` |
| `miniextendr-api/src/from_r.rs` | Optional: `TryFromSexp` calls `ensure_materialized` |

## Verification

1. `cargo check --workspace`
2. `cargo test -p miniextendr-api`
3. `cargo test -p miniextendr-api --features rayon`
4. `cargo clippy --workspace --all-features`
5. `cargo doc --no-deps -p miniextendr-api`

### Compile-time assertions

```rust
fn assert_send<T: Send>() {}
fn assert_not_send<T>() where T: Send { panic!("should not be Send") }

// SEXP is still Send + Sync:
assert_send::<SEXP>();

// AltrepSexp is NOT Send:
// assert_send::<AltrepSexp>();  // fails to compile — good
```

## Non-Goals

- Making SEXP `!Send` (too broad — breaks everything, most SEXPs are fine)
- Deep VECSXP materialization (list columns)
- Mutable ALTREP (ALTREP vectors are logically immutable until materialized)
- Changing ALTREP callback signatures
