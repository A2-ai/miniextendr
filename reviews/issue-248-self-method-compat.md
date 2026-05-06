# `&self` / `&mut self` on vctrs-tagged impls with vector-payload constructors

**Issue**: #416 (follow-up to #248 / PR #414)

## What was attempted

PR #414 added MXL120: a hard error when a `#[miniextendr(vctrs(...))]` impl's constructor
returns `Self` or the named type. Code review surfaced a gap: `&self` / `&mut self` instance
methods on such impls were still accepted by the macro even though the runtime semantics are
broken when the constructor returns a vector payload (`Vec<f64>`, etc.) instead of `Self`.

## What the macro currently emits for a `&self` method

For the snapshot fixture at `miniextendr-macros/src/miniextendr_impl/tests.rs:1917`:

```rust
impl Percent {
    pub fn new(x: f64) -> Vec<f64> { ... }
    pub fn value(&self) -> f64 { ... }
    pub fn scale(&mut self, factor: f64) { ... }
}
```

The R-side wrapper emitted by `generate_vctrs_r_wrapper` (vctrs_class.rs line 300):

```r
value.Percent <- function(x, ...) {
  .val <- .Call(C_Percent__value, .call = match.call(), x)
  ...
}
```

(See snapshot: `snapshots/miniextendr_macros__miniextendr_impl__tests__snapshot_vctrs_vctr.snap`)

Here `x` is the `Percent` vctrs vector — an S3-classed REALSXP (base vector).

The C-side wrapper emitted by `generate_method_c_wrapper` (miniextendr_impl.rs lines 2426-2471)
for `ReceiverKind::Ref` does:

```rust
let self_ptr = unsafe {
    ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(self_sexp)
};
let self_ref = self_ptr.downcast_ref::<Percent>()
    .expect("expected ExternalPtr<Percent>");
```

The `self_sexp` argument is the REALSXP — not an EXTPTRSXP. `ErasedExternalPtr::from_sexp`
reads `R_ExternalPtrAddr` from a REALSXP which produces garbage or a null pointer; the
`downcast_ref` `expect()` then panics.

## Does it compile?

**Yes, it compiles.** `Vec<f64>` is a valid constructor return type (MXL120 only checks
constructors, not instance methods). The C wrapper compiles without errors — the broken
pattern isn't detectable at compile time.

## Does it run correctly?

**No.** At runtime, calling `value(x)` where `x` is a vctrs Percent vector will:

1. R dispatches to `value.Percent(x, ...)`.
2. `.Call(C_Percent__value, .call = match.call(), x)` passes the REALSXP as `self_sexp`.
3. C wrapper calls `ErasedExternalPtr::from_sexp(self_sexp)`.
4. `downcast_ref::<Percent>()` returns `None` (REALSXP has no ExternalPtr payload).
5. `.expect("expected ExternalPtr<Percent>")` panics.
6. The framework converts the panic to an R error: `"expected ExternalPtr<Percent>"`.

This is a **loud runtime failure**, not UB or silent wrong values.

## Reproducible failure mode

A user writing:

```rust
#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "pct"))]
impl Percent {
    pub fn new(x: f64) -> Vec<f64> { vec![x] }
    pub fn value(&self) -> f64 { 0.0 }  // compiles, fails at runtime
}
```

Then in R:

```r
p <- new_percent(0.5)
value(p)  # => Error: expected ExternalPtr<Percent>
```

## Is there a salvageable design for `&self`?

For the vector-payload vctrs pattern, there is no `Self` stored anywhere in the R SEXP.
The R object is an S3-classed base vector (REALSXP, INTSXP, etc.) carrying a `"class"`
attribute, but no ExternalPtr. There is no mechanism to reconstruct a Rust `&Self` from
a bare REALSXP without additional stored state.

Potential adapter approaches are all fundamentally flawed:

- **Reconstruct Self from the REALSXP data**: requires a `TryFrom<&[f64]>` impl, but Self
  has no fields in a vector-payload pattern — the struct is a zero-sized marker.
- **Sentinel trait**: a trait that can extract Self from the base vector content. This would
  require defining what "Self" means when Self is a marker struct — it is always the zero
  value, making it useless for instance methods that would need actual state.
- **Pass a reconstructed ZST**: for pure ZSTs this is trivially correct (there's only one
  value), but then `&self` methods could never access the vector data they need (amounts, etc.)
  without receiving `x` as a separate parameter — effectively a static method.

The fundamental mismatch: `&self` in Rust means "I have a heap-allocated instance whose
address is stored in the R SEXP." For ExternalPtr-backed types this is true. For
vector-payload vctrs, the R SEXP IS the data — no heap allocation exists.

## Recommendation: Option A

Extend MXL120 to reject `&self`, `&mut self`, and `self` (by-value) receivers on
`#[miniextendr(vctrs(...))]` impls. The instance receiver semantics are fundamentally
incompatible with the vector-payload constructor pattern.

The vctrs protocol (format, arith, math, etc.) must be expressed as static methods receiving
the vector payload by parameter: `pub fn format_amounts(amounts: Vec<f64>) -> Vec<String>`.
This is already what `DerivedCurrency::format_amounts` in `rpkg/src/rust/vctrs_derive_example.rs`
does — the static pattern is the correct and working idiom.

The `ExternalPtrRef`/`ExternalPtrRefMut`/`ExternalPtrValue` receiver kinds (`self: &ExternalPtr<Self>`)
are equally broken for the same reason and should also be rejected.

The `ReceiverKind::Value` (consuming `self`) is broken for the same reason.

## Fix

- Extend MXL120 check to cover all instance-method receivers (Ref, RefMut, Value,
  ExternalPtrRef, ExternalPtrRefMut, ExternalPtrValue) on vctrs impls.
- Add UI test `impl_vctrs_self_method_rejected.rs` + `.stderr`.
- Remove `&self` methods from the `snapshot_vctrs_vctr` and `snapshot_vctrs_rcrd` fixtures,
  and from the `vctrs_protocol_method_override` test fixture.
- Update `docs/VCTRS.md` to remove the outdated `-> Self` constructor + `&self` method
  example and document the correct static-method pattern.
- Update the snapshot files to match the new fixtures.

## Files implicated

- `miniextendr-macros/src/miniextendr_impl.rs` — MXL120 check extension
- `miniextendr-macros/src/miniextendr_impl/tests.rs` — fixture cleanup
- `miniextendr-macros/src/miniextendr_impl/snapshots/` — updated snapshots
- `miniextendr-macros/tests/ui/impl_vctrs_self_method_rejected.rs` + `.stderr` — new UI test
- `docs/VCTRS.md` — doc update
