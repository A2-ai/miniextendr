---
name: miniextendr-conversions
description: Use when the user asks about converting between R and Rust types, how TryFromSexp or IntoR work, what NA handling looks like, how strict mode differs from normal coercion, why Vec<i32> from an empty R vector panics, or how bool, Option<T>, or large integer types behave across the R-Rust boundary.
---

# miniextendr Type Conversions

The conversion layer is the interface between R's SEXP values and Rust types.
Every argument a `#[miniextendr]` function receives goes through `TryFromSexp`;
every return value goes through `IntoR`. Getting these right is prerequisite
knowledge for working on any function that crosses the R-Rust boundary.

## When to use this skill

- "How do I receive a `Vec<i32>` from R?"
- "What happens when R passes NA to a `i32` argument?"
- "My `i64` return value is becoming a double in R — why?"
- "What is strict mode and when do I use it?"
- "Why does converting from an empty R vector panic?"
- "How do I handle logical NA (three-state true/false/NA)?"
- "What types does `Option<T>` accept?"
- "How do I add a new conversion type?"
- "Why can't I use `bool` where R uses logical vectors?"
- "What does `Coerce` vs `TryCoerce` mean?"

## Key concepts

### TryFromSexp — R to Rust

`TryFromSexp` is the trait that converts an incoming SEXP to a Rust type.
It lives in `miniextendr-api/src/from_r.rs`. The generated C wrapper calls
`try_from_sexp` for each argument before invoking the user's function.

The trait has two methods:
- `try_from_sexp(sexp: SEXP)` — checked path, runs debug thread assertions.
- `try_from_sexp_unchecked(sexp: SEXP)` — unchecked path for ALTREP callbacks
  and other contexts where the thread is already known.

### IntoR — Rust to R

`IntoR` converts a Rust value into a SEXP after the user's function returns.
It lives in `miniextendr-api/src/into_r.rs`. The trait is infallible by
convention for scalar types (using `std::convert::Infallible` as the error
type), but fallible for types that can fail (such as strings exceeding R's
i32 length limit — `IntoRError` is the error type for those).

### Coerce and TryCoerce — R-side type coercion

`Coerce<R>` (infallible) and `TryCoerce<R>` (fallible) sit between R native
types and Rust non-native types. They live in `miniextendr-api/src/coerce.rs`.
These are not exposed directly to function authors; `TryFromSexp` impls use
them internally for the "coerced scalar" types (see below).

### Three conversion modes

The full matrix is in `docs/CONVERSION_MATRIX.md`. The three modes are:

**Normal mode** — each Rust type accepts exactly one R type. `i32` only
accepts `INTSXP`; `f64` only accepts `REALSXP`. A type mismatch produces a
`SexpTypeError`.

**Coerce mode** — sub-integer types (`i8`, `i16`, `u16`, `u32`, `f32`) and
large integer types (`i64`, `u64`, `isize`, `usize`) accept multiple R types:
`INTSXP`, `REALSXP`, `RAWSXP`, and `LGLSXP`. The value is extracted as the
R native type then converted to the Rust type via `TryCoerce`. This is the
default for these types — no attribute is needed.

**Strict mode** — activated via `#[miniextendr(strict)]` on the function.
Large integer types (`i64`, `u64`, `isize`, `usize`) are restricted to
`INTSXP` and `REALSXP` only; `RAWSXP` and `LGLSXP` are rejected. On output,
values that do not fit in i32 cause a panic (which becomes an R error via the
framework's error transport) instead of silently widening to `REALSXP`. Strict
mode is implemented in `miniextendr-api/src/strict.rs`.

### NA handling

R NA values are type-specific sentinel values:

- `NA_integer_` = `i32::MIN`. This value is excluded from the valid range for
  `i32` — receiving it via `TryFromSexp` returns `SexpError::Na`. If you need
  NA-aware integer handling, use `Option<i32>`.
- `NA_real_` is a specific NaN bit pattern (not the same as `f64::is_nan()`).
  `f64` receives it as the NA_real_ NaN; `Option<f64>` maps it to `None`.
- `NA_logical` is a third state for logicals — neither true nor false.
  Use `RLogical` (not `bool`) to represent it. `bool` treats NA as an error.
- `NA_character_` causes an error for `String` / `&str`. Use
  `Option<String>` / `Option<&str>` to accept it as `None`.

`Option<T>` maps both NA and NULL to `None` for all wrapped types.

On output, `Option<T>` produces NA (not NULL) for `None` when T is a scalar
type. `Option<Vec<T>>` produces NULL.

### bool is not RNativeType

R represents logical values as i32 (three-state: 0=FALSE, 1=TRUE, `NA_integer_`=NA).
Rust `bool` is two-state and has no `IntoR`/`TryFromSexp` implementation via
the `RNativeType` blanket. Instead, `bool` has separate explicit impls:
- `TryFromSexp for bool` — accepts LGLSXP, fails on NA.
- `IntoR for bool` — emits `1_i32` or `0_i32` as LGLSXP.

If you need NA-aware logical handling, use `RLogical` (which carries
`RLogical::True`, `RLogical::False`, `RLogical::Na`).

### Empty-vector pointer trap

R returns a sentinel pointer (`0x1`, not null) for the data pointer of
zero-length vectors (e.g., `LOGICAL(integer(0))`). Rust 1.93+ validates
pointer alignment even for `len == 0` in `slice::from_raw_parts`, so passing
R's sentinel directly causes a precondition-check abort.

All slice construction inside miniextendr goes through `r_slice()` and
`r_slice_mut()`, defined in `miniextendr-api/src/from_r.rs`. These helpers
return `&[]` / `&mut []` when `len == 0` without dereferencing the pointer.
If you add a custom `TryFromSexp` impl that calls `slice::from_raw_parts`
directly, you must handle this case. Never call `from_raw_parts` with R's
raw data pointer without first checking `len > 0`.

### Large integer types: smart widening vs strict

By default, `i64`, `u64`, `isize`, and `usize` use smart widening on output:
- Value fits in `(i32::MIN, i32::MAX]` → `INTSXP`
- Value outside that range → `REALSXP` (may lose precision above 2^53)
- `i32::MIN` is always excluded from `INTSXP` because it is `NA_integer_`

With `#[miniextendr(strict)]`, out-of-range values panic instead of widening
to `REALSXP`. The strict helpers are in `miniextendr-api/src/strict.rs`:
`checked_into_sexp_i64`, `checked_into_sexp_u64`, and the vector variants.

## How it works

### Normal call path

1. `#[miniextendr]` generates a C wrapper. For each argument, it calls
   `TryFromSexp::try_from_sexp(sexp)` (or the `_unchecked` variant in
   appropriate contexts).
2. Conversion failures become panics, which the `with_r_unwind_protect`
   boundary converts into R errors via the tagged-SEXP transport in
   `miniextendr-api/src/error_value.rs`.
3. The user's function runs with native Rust types.
4. The return value is converted via `IntoR::into_sexp()` and returned
   to R.

For the macro code that generates this glue, see:
- `miniextendr-macros/src/rust_conversion_builder.rs` — argument conversion
  code generation.
- `miniextendr-macros/src/return_type_analysis.rs` — return type analysis
  (strict-aware).

### Adding a new conversion type

The six-step checklist from `miniextendr-api/CLAUDE.md`:
1. `from_r.rs` — implement `TryFromSexp`.
2. `into_r.rs` — implement `IntoR`.
3. `coerce.rs` — implement `Coerce<R>` or `TryCoerce<R>` if needed.
4. Update serde docs (if the type participates in serde).
5. Add an rpkg fixture function so the type is exercised under
   `gctorture(TRUE)`.
6. Run `just vendor-sync-check` to confirm vendored copies are in sync.

`bool` is not `RNativeType` (R uses i32 for logicals), so it needs separate
impls. The proc-macro handles `Box<[T]>` generically — no macro changes are
needed when adding a new element type.

## Decision trees

### Which type do I use for a function argument?

```
Do you need NA-awareness?
  Yes → Option<T>: maps NA and NULL to None.
  No:
    Is the value a scalar?
      Yes — what R type?
        INTSXP → i32 (exact) or i64/u64 (coerced)
        REALSXP → f64 (exact) or f32/i64/u64 (coerced)
        LGLSXP → bool (error on NA) or RLogical (NA-aware)
        STRSXP → &str (borrowed, no alloc) or String (owned)
        RAWSXP → u8
      No (vector)?
        Vec<i32>, Vec<f64>, Vec<u8>, Vec<bool>, Vec<String>, Vec<RLogical>
        Vec<Option<T>> for NA-aware variants
        &[T] for borrowed read-only slices (no copying)
Do you want multi-source coercion (INTSXP or REALSXP or RAWSXP)?
  Yes → use i8, i16, u16, u32, f32, i64, u64, isize, usize (coerce mode)
```

### Strict mode or not?

Use `#[miniextendr(strict)]` when:
- The function explicitly contracts that all values fit in R's integer range.
- You want a user-facing error rather than silent precision loss for large i64.

Avoid strict mode when:
- The function handles large integers that may legitimately exceed i32::MAX.
- You want R users to pass either integer or numeric vectors without a strict
  type contract.

### What does Option do?

- `Option<i32>` input: accepts `NA_integer_` → `None`; accepts non-NA integer
  → `Some(val)`.
- `Option<i32>` output: `None` → `NA_integer_`; `Some(v)` → integer scalar.
- `Option<Vec<i32>>` output: `None` → NULL (`R_NilValue`).

## Key files

- `miniextendr-api/src/from_r.rs` — `TryFromSexp` trait and all built-in
  implementations. Also contains `r_slice` / `r_slice_mut` (the empty-vector
  pointer helpers), `SexpError`, `SexpTypeError`, `SexpLengthError`,
  `SexpNaError`.
- `miniextendr-api/src/into_r.rs` — `IntoR` trait and all built-in
  implementations. Submodules for large integers, collections, Result.
- `miniextendr-api/src/coerce.rs` — `Coerce<R>` and `TryCoerce<R>` traits
  used internally by the coerce-mode `TryFromSexp` impls.
- `miniextendr-api/src/strict.rs` — `checked_into_sexp_i64`,
  `checked_into_sexp_u64`, `checked_into_sexp_isize`, `checked_into_sexp_usize`,
  and their vector variants.
- `docs/CONVERSION_MATRIX.md` — the authoritative R type × Rust type
  reference table covering all three modes in both directions.
- `miniextendr-macros/src/rust_conversion_builder.rs` — proc-macro code that
  generates the `TryFromSexp` call sites in C wrappers.
- `miniextendr-macros/src/return_type_analysis.rs` — proc-macro code that
  selects normal vs strict `IntoR` call sites based on the function attribute.

## Common pitfalls

- **i32::MIN is NA_integer_**: passing `i32::MIN` from R as an `i32` argument
  returns `SexpError::Na`, not a large negative number. Use `Option<i32>` to
  accept it explicitly as `None`. On output, never return `i32::MIN` from a
  function expecting a valid integer — R will display it as `NA`.

- **bool does not have a blanket RNativeType impl**: if you write a generic
  function over `T: RNativeType`, it will not cover `bool`. That is intentional
  — R's logical type is three-state (TRUE/FALSE/NA). Use `RLogical` for the
  full three-state representation.

- **Coerce-mode types silently accept LGLSXP**: `i64` from an LGLSXP is legal
  in normal (non-strict) mode. If you want to enforce that only numeric R values
  are accepted, add `#[miniextendr(strict)]`.

- **Empty vector sentinel causes precondition abort in Rust 1.93+**: never call
  `slice::from_raw_parts` with a raw R data pointer without checking `len > 0`
  first. Use `r_slice()` / `r_slice_mut()` from `from_r.rs` instead.

- **NA_real_ is a specific NaN bit pattern**: `f64::is_nan()` returns `true`
  for all NaN values including `0.0/0.0`. Use `is_na_real()` from `from_r.rs`
  to distinguish R's NA from regular NaN.

- **Option<Vec<T>> returns NULL, not NA**: on output, `None` for a vector
  `Option<Vec<T>>` produces `R_NilValue` (R's NULL), not a length-1 NA vector.
  Scalar `Option<i32>` produces `NA_integer_`. The distinction matters for
  downstream R code that uses `is.null()` vs `is.na()`.

- **R arrays are column-major; conversion never reorders them**: a `Vec<f64>`
  or `&[f64]` received from an R `matrix`/`array` is the flat column-major
  (Fortran-order) buffer as R stores it — `dim` is just an attribute, and
  `TryFromSexp` does not permute data. Rust numeric crates typically expect
  row-major / C-order flat buffers, so a wrapper must transpose explicitly.
  Cheapest is on the R side before the `.Call`:
  `as.double(aperm(x, rev(seq_along(dim(x)))))`, and the inverse on return:
  `aperm(array(v, rev(dims)), rev(seq_along(dims)))`. For 1-D vectors the two
  orders coincide and no transform is needed.

## Related skills

- `miniextendr-getting-started` — how to write and register a first function.
- `miniextendr-macros` — how `#[miniextendr]` attribute parsing selects strict
  vs normal mode and how `TryFromSexp` call sites are generated.
- `miniextendr-serde` — alternative path for complex types via serde's
  Serialize/Deserialize instead of hand-rolled TryFromSexp/IntoR.
- `miniextendr-ffi` — thread safety, `_unchecked` FFI variants, and the
  MXL300/MXL301 lint rules that govern when unchecked conversions are legal.
- `miniextendr-altrep` — ALTREP vectors, which use `r_slice` / `r_slice_mut`
  directly inside ALTREP callbacks.
