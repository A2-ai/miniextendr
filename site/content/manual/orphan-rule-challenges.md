+++
title = "Orphan Rule Challenges: Feature Crate Extraction"
weight = 64
description = "We explored extracting miniextendr-api's optional features (ndarray, nalgebra, serde, rayon, etc.) into separate miniextendr-<name> crates. The goal was to reduce miniextendr-api's surface area and let users depend only on what they need."
+++

## Context

We explored extracting miniextendr-api's optional features (ndarray, nalgebra, serde, rayon, etc.) into separate `miniextendr-<name>` crates. The goal was to reduce miniextendr-api's surface area and let users depend only on what they need.

## The Problem

Rust's orphan rule prevents `impl ForeignTrait for ForeignType` in a third-party crate. For a hypothetical `miniextendr-ndarray` crate:

```rust
// miniextendr-ndarray/src/lib.rs
use miniextendr_api::from_r::TryFromSexp;

impl TryFromSexp for ndarray::Array2<f64> { ... }
//   ^^^^^^^^^^^     ^^^^^^^^^^^^^^^^^^^
//   from api        from ndarray
//   (foreign)       (foreign)
//   → orphan rule violation
```

The impl must live in the crate that owns either the trait (`miniextendr-api`) or the type (`ndarray`). Since we don't control `ndarray`, the impl must stay in `miniextendr-api`.

## Approaches Considered

### 1. Bridge trait with blanket impl

Define a bridge trait in api, blanket-impl `TryFromSexp` for it, then impl the bridge trait in the feature crate.

```rust
// miniextendr-api
pub trait SexpBridge { fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError>; }
impl<T: SexpBridge> TryFromSexp for T { ... }  // fine: both traits local to api

// miniextendr-ndarray
impl SexpBridge for Array2<f64> { ... }
//   ^^^^^^^^^^     ^^^^^^^^^^
//   foreign (api)  foreign (ndarray) → still blocked
```

**Result:** Same orphan violation — `SexpBridge` is foreign to `miniextendr-ndarray`.

### 2. `TryFrom<SEXP>` (std trait)

```rust
// miniextendr-ndarray
impl TryFrom<SEXP> for Array2<f64> { ... }
//   ^^^^^^^  ^^^^    ^^^^^^^^^^
//   core     api     ndarray → all foreign, blocked
```

**Result:** `core::TryFrom` is even more foreign. Same constraint.

### 3. Derive macro on mirror traits

Have feature crates define mirror traits, with a derive macro from api that validates compatibility and generates `TryFromSexp` impls.

**Result:** No matter what macro generates the code, the impl lives in the crate where it expands. If that crate owns neither the trait nor the type, the orphan rule blocks it. Macros cannot bypass the orphan rule.

### 4. Newtype wrappers

```rust
// miniextendr-ndarray
pub struct RArray2<T>(pub ndarray::Array2<T>);  // local type
impl<T> TryFromSexp for RArray2<T> { ... }      // valid: RArray2 is local
impl<T> Deref for RArray2<T> { type Target = Array2<T>; ... }
```

**Result:** Works, but degrades ergonomics. Users write `RArray2<f64>` in `#[miniextendr]` signatures instead of `Array2<f64>`. Deref coercion helps inside function bodies but not at the API boundary.

### 5. Free functions (no traits)

Feature crate exports `pub fn array2_from_sexp(sexp: SEXP) -> Result<Array2<f64>, Error>`. Users call manually.

**Result:** Works, but loses the automatic conversion that `#[miniextendr]` provides. The macro would need per-type configuration to know which function to call.

### 6. User-crate macro expansion

Have a registration macro in the user's crate generate the impls.

**Result:** The user's crate also doesn't own `TryFromSexp` or `Array2<f64>`. Same orphan violation, regardless of where the macro is defined.

## Conclusion

There is no stable Rust mechanism to write `impl ForeignTrait for ForeignType` outside the crate that owns one of them. The only escape hatches (`#[fundamental]`, specialization) are unstable/nightly-only.

**Decision:** Optional feature support stays in `miniextendr-api` as feature-gated code. This is the same pattern used by the broader Rust ecosystem (e.g., `serde` relies on external crates opting in, which we can't do since we don't control `ndarray`, `nalgebra`, etc.).

## serde Comparison

serde solves this differently: external crates (chrono, uuid) add `serde` as an optional dependency and impl `Serialize`/`Deserialize` themselves (they own the type). We can't use this approach because we don't control the external crates.

The `serde_with` / remote derive pattern still requires the impl to live in a crate that owns either the trait or the type — it just provides ergonomic sugar for defining mirror types.
