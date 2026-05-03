+++
title = "Issue #112 — FFI encapsulation: scalar constructors (cat 4) + ALTREP data slots (cat 6)"
+++

# FFI encapsulation: scalar constructors + ALTREP data slots

Closes #112 categories 4 and 6. Documents keep-as-pub(crate) decisions for cats 2/3/7/8/9/10.
Follow-up issues for cats 1 (Rf_xlength) and 5 (data pointers).

## Category 4 — Scalar constructors

`Rf_ScalarComplex`, `Rf_ScalarInteger`, `Rf_ScalarLogical`, `Rf_ScalarRaw`, `Rf_ScalarReal`,
`Rf_ScalarString` — currently `pub(crate)` because `into_r.rs`'s `impl_scalar_into_r!` macro
needs both the checked and unchecked variant name.

**Fix**: add `SEXP::scalar_*_unchecked()` methods in the scalar construction region of `ffi.rs`
(mirroring the existing `SEXP::scalar_*()` checked wrappers). Migrate all callers:
- `impl_scalar_into_r!` macro in `into_r.rs` → use `SEXP::scalar_*(self)` / `SEXP::scalar_*_unchecked(self)`
- Direct `crate::ffi::Rf_Scalar*_unchecked(...)` calls in `into_r/large_integers.rs` → `SEXP::scalar_*_unchecked`
- Option<Rcomplex> impl in `into_r.rs` → `SEXP::scalar_complex_unchecked`

Drop `pub(crate)` from all six `Rf_Scalar*` lines.

## Category 6 — ALTREP data slots

`ALTREP_CLASS`, `R_altrep_data1`, `R_altrep_data2`, `R_set_altrep_data1`, `R_set_altrep_data2`
— currently `pub(crate)`.

**Fix**: extend `AltrepSexpExt` trait with raw-SEXP-returning methods:
- `altrep_data1_raw(&self) -> SEXP` — checked, wraps `R_altrep_data1`
- `altrep_data1_raw_unchecked(&self) -> SEXP` — unchecked, wraps `R_altrep_data1_unchecked`
- `set_altrep_data1(&self, v: SEXP)` — checked, wraps `R_set_altrep_data1`
- `altrep_data2_raw_unchecked(&self) -> SEXP` — unchecked, wraps `R_altrep_data2_unchecked`
- `set_altrep_data2_unchecked(&self, v: SEXP)` — unchecked, wraps `R_set_altrep_data2_unchecked`
- `altrep_class(&self) -> SEXP` — checked, wraps `ALTREP_CLASS` (no callers yet but documents intent)

Migrate `externalptr/altrep_helpers.rs` to use trait methods. `altrep_ext.rs` already uses
the trait for checked calls; migrate to use `altrep_data1_raw` for consistency.

Drop `pub(crate)` from all five ALTREP data items.

## Category 2/3/7/8/9/10 — kept pub(crate) comments

Add one-line `// Issue #112 cat. N: kept pub(crate) — <reason>` comments at the top of
each grouping. These are grep-friendly decision records.

- **Cat 2** (ExternalPtr fns): kept pub(crate) — ExternalPtr<T> encapsulates for users; raw access needed within externalptr.rs
- **Cat 3** (unwind protect): kept pub(crate) — only used from unwind_protect.rs; behind safe wrapper for users
- **Cat 7** (registration): kept pub(crate) — only used from init.rs; not worth a wrapper type
- **Cat 8** (connections): kept pub(crate) — feature-gated; behind Connection type
- **Cat 9** (evaluation): kept pub(crate) — only used from expression.rs / s4_helpers.rs
- **Cat 10** (misc): kept pub(crate) — single-caller utilities; wrapping adds no value

## Follow-up issues

- **Cat 1** (`Rf_xlength`): mechanical migration of ~60 callers to `SexpExt::xlength()` — track as issue
- **Cat 5** (data pointers): partial migration to `SexpExt::as_mut_slice()`; keep pub(crate) for raw-pointer sites — track as issue
