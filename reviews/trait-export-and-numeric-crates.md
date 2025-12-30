# Exporting non-owned traits + numeric crate features

Date: 2025-12-30  
Scope: clarify trait export feasibility and identify numeric crate features.

## Can we export traits we don’t own?

**Short answer:** not directly. `#[miniextendr]` must be applied to the *trait
definition*, which requires the trait to be in your crate. External traits
can’t be retroactively annotated.

### Practical pattern: adapter traits

Define a **local wrapper trait** that mirrors the subset of methods you want
to expose to R, then provide a blanket impl for types that implement the
external trait:

```rust
use num_traits::Num;

#[miniextendr]
pub trait RNum {
    fn add(&self, other: &Self) -> Self;
    fn to_string(&self) -> String;
}

impl<T: Num + Clone + ToString> RNum for T {
    fn add(&self, other: &Self) -> Self { self.clone() + other.clone() }
    fn to_string(&self) -> String { ToString::to_string(self) }
}
```

**Constraints to keep in mind (from miniextendr trait ABI):**
- Traits **cannot** have generic parameters or async methods.
- Methods **cannot** be generic.
- Method argument and return types must implement `TryFromSexp` / `IntoR`.
- Static methods are allowed but do not go through the vtable.

If the external trait has associated types or complex signatures (e.g. many of
`num-traits`), expose a **smaller adapter trait** with concrete signatures.

### Alternative: newtype wrapper

If you need total control (or the external trait has awkward signatures),
wrap the external type in a local newtype and implement a local trait on that
newtype. This avoids blanket impl pitfalls and gives you explicit conversions.

---

## Numeric crate features that make sense for miniextendr

### 1) `num-bigint` (recommended)
**Why:** Big integers don’t fit into R’s 32‑bit integers.  
**Mapping:** `BigInt`/`BigUint` ⇄ R `character` (lossless).  
**Feature name:** `num-bigint`.

### 2) `rust_decimal` (recommended)
**Why:** Decimal math is common in finance; R doubles are binary‑floating.  
**Mapping:** `Decimal` ⇄ R `character` (lossless).  
**Optional fast path:** allow `numeric` when the user opts in (with precision warning).
**Feature name:** `rust_decimal`.

### 3) `ordered-float` (recommended)
**Why:** Sorting with NaN is tricky; `OrderedFloat` gives total ordering.  
**Mapping:** `OrderedFloat<f64|f32>` ⇄ R `numeric`.
**Feature name:** `ordered-float`.

### 4) `num-traits` (internal helper)
**Why:** Useful for generic Rust implementations, but **not** a good R-facing
trait export target due to generics/associated types.  
**Recommendation:** keep as an *internal helper* feature only.

### 5) `rug` (not recommended for core)
**Why:** LGPL + system GMP dependency complicates CRAN.  
**Recommendation:** only as an advanced opt‑in with clear license notes.

---

## Suggested plan items (if you want to implement)

1) Add optional deps + features in `miniextendr-api/Cargo.toml`.
2) Create modules:
   - `num_bigint_impl.rs`, `rust_decimal_impl.rs`, `ordered_float_impl.rs`
3) Implement `TryFromSexp` / `IntoR` for each type.
4) Add feature‑gated tests under `miniextendr-api/tests/`.
5) Add short doc blocks in `lib.rs` mirroring existing feature docs.

If you want, I can draft the concrete implementation plan for these features
next, aligned with your adapter‑trait strategy.
