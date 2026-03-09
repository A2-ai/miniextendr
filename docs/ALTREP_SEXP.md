# Receiving ALTREP Vectors from R

How miniextendr handles ALTREP vectors when R passes them to Rust functions.

**See also**: [ALTREP.md](ALTREP.md) (creating ALTREP vectors), [ALTREP_QUICKREF.md](ALTREP_QUICKREF.md), [THREADS.md](THREADS.md)

## The Problem

R uses ALTREP (Alternative Representations) for many common operations. For
example, `1:10` creates a compact integer sequence — an ALTREP object — not a
regular integer vector. This is invisible to R users but matters for Rust code
because:

1. **ALTREP data pointers are unstable.** Calling `DATAPTR_RO` on an ALTREP
   vector triggers *materialization* — R allocates a new vector, runs GC, and
   fills in the data. This involves R's C runtime and must happen on the R main
   thread.

2. **Materialization is not thread-safe.** If a rayon thread calls `DATAPTR_RO`
   on an un-materialized ALTREP vector, R's internal state is corrupted.

3. **ALTREP is pervasive.** `1:N`, `seq_len(N)`, `as.character(1:N)`, and many
   other R idioms produce ALTREP vectors. Any function accepting `SEXP` from R
   may receive one.

## How miniextendr Handles It

miniextendr uses a two-type strategy:

| Parameter type | Receives ALTREP? | What happens |
|---|---|---|
| **Typed** (`Vec<i32>`, `&[f64]`, `String`, etc.) | Yes | Auto-materializes via `DATAPTR_RO` during conversion. Safe and transparent. |
| **`SEXP`** | Yes | Auto-materializes via `ensure_materialized` before the function body runs. |
| **`AltrepSexp`** | Only ALTREP | Wraps the ALTREP vector without materializing. `!Send + !Sync`. |
| **`extern "C-unwind"` with raw `SEXP`** | Yes (raw) | No conversion — receives the raw SEXP as-is, including ALTREP. |

### Typed Parameters: The Recommended Default

For most functions, use typed parameters. They handle ALTREP transparently:

```rust
#[miniextendr]
pub fn sum_integers(x: Vec<i32>) -> i64 {
    x.iter().map(|&v| v as i64).sum()
}
```

```r
sum_integers(1:1000000)  # Works — 1:N is ALTREP, auto-materialized
sum_integers(c(1L, 2L))  # Works — regular vector
```

Typed conversions (`Vec<T>`, `&[T]`, scalar types) go through their own
`TryFromSexp` implementations which call `DATAPTR_RO` internally. This triggers
materialization for ALTREP vectors as a side effect, producing a stable data
pointer.

### SEXP Parameters: Auto-Materialization

When a `#[miniextendr]` function takes `SEXP`, the generated wrapper calls
`ensure_materialized(sexp)` before passing it to your function body. This means:

```rust
#[miniextendr]
pub fn inspect_vector(x: SEXP) -> i32 {
    // By the time we get here, `x` is guaranteed materialized.
    // DATAPTR_RO is safe, data pointer is stable.
    unsafe { ffi::Rf_xlength(x) as i32 }
}
```

```r
inspect_vector(1:10)  # Works — auto-materialized before Rust sees it
inspect_vector(c(1L, 2L))  # Works — no materialization needed
```

`ensure_materialized` works by calling `DATAPTR_RO` (for contiguous types like
INTSXP, REALSXP) or iterating `STRING_ELT` (for STRSXP) to force R to
materialize the underlying data. The SEXP itself is unchanged — it still has the
ALTREP flag set — but its data pointer is now stable.

### AltrepSexp: Explicit ALTREP Handling

When you need to work with ALTREP vectors *without* materializing them (e.g., to
inspect ALTREP metadata, or to defer materialization), use `AltrepSexp`:

```rust
use miniextendr_api::AltrepSexp;
use miniextendr_api::ffi::SEXPTYPE;

#[miniextendr]
pub fn altrep_info(x: AltrepSexp) -> Vec<String> {
    vec![
        format!("type={:?}", x.sexptype()),
        format!("len={}", x.len()),
    ]
}
```

```r
altrep_info(1:10)          # Works — 1:10 is ALTREP
altrep_info(c(1L, 2L, 3L)) # Error: "expected an ALTREP vector"
```

`AltrepSexp` is `!Send + !Sync` — it cannot be sent to rayon threads or other
worker threads. This is enforced at compile time via `PhantomData<Rc<()>>`.

To extract data from an `AltrepSexp`, materialize it on the R main thread:

```rust
#[miniextendr]
pub fn materialize_ints(x: AltrepSexp) -> Vec<i32> {
    assert_eq!(x.sexptype(), SEXPTYPE::INTSXP);
    let slice: &[i32] = unsafe { x.materialize_integer() };
    slice.to_vec()
}
```

#### AltrepSexp Methods

| Method | Returns | Description |
|---|---|---|
| `try_wrap(sexp)` | `Option<AltrepSexp>` | Wrap if ALTREP, `None` otherwise |
| `from_raw(sexp)` | `AltrepSexp` | Unsafe wrap (caller asserts ALTREP) |
| `materialize(self)` | `SEXP` | Force materialization, consume self |
| `materialize_integer(&self)` | `&[i32]` | Materialize INTSXP to slice |
| `materialize_real(&self)` | `&[f64]` | Materialize REALSXP to slice |
| `materialize_logical(&self)` | `&[i32]` | Materialize LGLSXP to slice |
| `materialize_raw(&self)` | `&[u8]` | Materialize RAWSXP to slice |
| `materialize_complex(&self)` | `&[Rcomplex]` | Materialize CPLXSXP to slice |
| `materialize_strings(&self)` | `Vec<Option<String>>` | Materialize STRSXP (NA-aware) |
| `as_raw(&self)` | `SEXP` | Get inner SEXP without materializing (unsafe) |
| `sexptype()` | `SEXPTYPE` | Get the R type |
| `len()` | `usize` | Get the length |
| `is_empty()` | `bool` | Check if empty |

### extern "C-unwind": Raw SEXP Access

For functions that must receive the *exact* SEXP from R without any conversion
(e.g., inspecting ALTREP state before materialization), use `extern "C-unwind"`:

```rust
use miniextendr_api::ffi::{SEXP, ALTREP};
use miniextendr_api::IntoR;

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_is_materialized(x: SEXP) -> SEXP {
    let is_altrep = unsafe { ALTREP(x) } != 0;
    is_altrep.into_sexp()
}
```

`extern "C-unwind"` functions bypass `TryFromSexp` entirely — they receive the
raw `SEXP` directly from R's `.Call()`. This is necessary when you need to
inspect ALTREP state without triggering materialization.

**Requirements for `extern "C-unwind"` functions:**
- Must have `#[unsafe(no_mangle)]` and `#[allow(non_snake_case)]`
- Function name must start with `C_` (convention)
- Must return `SEXP` (not typed returns)
- Must manually call `.into_sexp()` on return values
- Registration is automatic via `#[miniextendr]`
- R wrapper will be `unsafe_C_name()` (prefixed with `unsafe_`)

## Thread Safety Model

```
┌─────────────────────────────────────────────────────────┐
│  R Main Thread                                           │
│                                                          │
│  user calls: my_func(1:1000000)                         │
│       │                                                  │
│       ▼                                                  │
│  TryFromSexp for SEXP                                    │
│       │                                                  │
│       ├─ calls ensure_materialized(sexp)                │
│       │   └─ DATAPTR_RO triggers materialization        │
│       │       └─ R allocates, fills data, runs GC       │
│       │                                                  │
│       ▼                                                  │
│  SEXP now has stable data pointer                        │
│  (still ALTREP-flagged, but data is materialized)       │
│       │                                                  │
│       ├─ Safe to send to rayon threads                  │
│       ├─ Safe to call DATAPTR_RO again (no-op)         │
│       └─ Safe to create &[T] slices                     │
└─────────────────────────────────────────────────────────┘
```

Key invariant: **all materialization happens on the R main thread**, before any
SEXP crosses a thread boundary. After materialization, the data pointer is stable
and can be safely accessed from any thread.

### AltrepSexp Prevents Thread Crossing

`AltrepSexp` is `!Send + !Sync`. If you try to send it to a rayon thread:

```rust
// This will NOT compile:
#[miniextendr]
pub fn bad_parallel(x: AltrepSexp) {
    rayon::spawn(move || {
        let _ = x.len();  // ❌ compile error: AltrepSexp is !Send
    });
}
```

This is the key safety property: un-materialized ALTREP vectors cannot
accidentally reach threads where `DATAPTR_RO` would invoke R internals.

### SEXP Is Send + Sync

Plain `SEXP` is `Send + Sync` because after auto-materialization, the data
pointer is stable. This is safe for the same reason `&[i32]` is safe to share:
the underlying memory won't move or be freed while the SEXP is protected.

## Decision Guide

```
What parameter type should I use?

Need the actual data? (sum, transform, filter)
├─ Yes → Use typed parameter: Vec<i32>, &[f64], etc.
│        ✅ Simplest. Handles ALTREP transparently.
│
└─ No, need the SEXP handle
   │
   ├─ Will you inspect ALTREP metadata?
   │  (class info, materialization state)
   │  └─ Yes → Use extern "C-unwind" with raw SEXP
   │           ✅ Gets exact SEXP, no conversion.
   │
   ├─ Want to accept only ALTREP vectors?
   │  └─ Yes → Use AltrepSexp parameter
   │           ✅ Compile-time !Send safety.
   │
   └─ Just passing SEXP to another R/C API?
      └─ Use SEXP parameter
         ✅ Auto-materializes. Safe for downstream use.
```

## Common ALTREP Sources in R

These R expressions produce ALTREP vectors:

| Expression | ALTREP class | TYPEOF |
|---|---|---|
| `1:10` | compact_intseq | INTSXP |
| `seq_len(10)` | compact_intseq | INTSXP |
| `seq.int(1, 10, 2)` | compact_intseq | INTSXP |
| `1.0:10.0` | compact_realseq | REALSXP |
| `as.character(1:10)` | deferred_string | STRSXP |
| `rep_len(0L, 1e6)` | compact_intseq | INTSXP |

Note: `c(1:10)`, `(1:10)[]`, and `1:10 + 0L` all force materialization in R,
producing a regular (non-ALTREP) vector. This can be useful when calling
`extern "C-unwind"` functions from R where you don't want the overhead
of the wrapper, but typically you should just use typed parameters.

## Examples

### Consuming ALTREP Data (Recommended)

```rust
/// Process integers — works with any R integer vector, ALTREP or not.
#[miniextendr]
pub fn double_values(x: Vec<i32>) -> Vec<i32> {
    x.iter().map(|&v| v.wrapping_mul(2)).collect()
}
```

### Inspecting ALTREP State

```rust
/// Check if a vector is ALTREP (without materializing it).
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_is_altrep(x: SEXP) -> SEXP {
    use miniextendr_api::AltrepSexp;
    AltrepSexp::try_wrap(x).is_some().into_sexp()
}
```

### Explicit Materialization

```rust
use miniextendr_api::AltrepSexp;
use miniextendr_api::ffi::SEXPTYPE;

/// Accept only ALTREP, materialize, and return the data.
#[miniextendr]
pub fn materialize_altrep(x: AltrepSexp) -> Vec<i32> {
    assert_eq!(x.sexptype(), SEXPTYPE::INTSXP);
    unsafe { x.materialize_integer() }.to_vec()
}
```

### Using ensure_materialized in extern "C-unwind"

```rust
use miniextendr_api::altrep_sexp::ensure_materialized;
use miniextendr_api::ffi::SEXP;
use miniextendr_api::IntoR;

/// Accept any SEXP (ALTREP or not), materialize, then convert.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_safe_extract(x: SEXP) -> SEXP {
    let materialized = unsafe { ensure_materialized(x) };
    let data: Vec<i32> =
        miniextendr_api::from_r::TryFromSexp::try_from_sexp(materialized).unwrap();
    data.into_sexp()
}
```

## API Reference

### Types

- **`AltrepSexp`** — `!Send + !Sync` wrapper for ALTREP vectors. Use as a
  `#[miniextendr]` parameter to accept only ALTREP input.
  ([source](../miniextendr-api/src/altrep_sexp.rs))

### Functions

- **`ensure_materialized(sexp: SEXP) -> SEXP`** — If ALTREP, materialize and
  return. If not ALTREP, return as-is. Must be called on the R main thread.
  ([source](../miniextendr-api/src/altrep_sexp.rs))

### TryFromSexp Implementations

- **`TryFromSexp for SEXP`** — Auto-materializes ALTREP via `ensure_materialized`.
  ([source](../miniextendr-api/src/from_r.rs))

- **`TryFromSexp for AltrepSexp`** — Accepts only ALTREP vectors, errors on
  non-ALTREP input.
  ([source](../miniextendr-api/src/altrep_sexp.rs))
