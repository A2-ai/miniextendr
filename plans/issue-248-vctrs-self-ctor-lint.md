# Issue #248 — reject Self-returning vctrs constructors

## Root cause
`miniextendr-macros/src/miniextendr_impl/vctrs_class.rs` generates an R wrapper that always passes `.val` (the C-call result) to `vctrs::new_vctr()`. When the impl method returns `Self`, `.val` is an `EXTPTRSXP` (Rust `Self` handle) — invalid for `new_vctr`, which errors `.data must be a vector type`.

## Fix (proc macro hard error)
In `ParsedImpl::parse`, after the methods loop, for `ClassSystem::Vctrs`: detect any method that would be treated as the constructor (`env == ReceiverKind::None && ident == "new"`, or explicitly marked `constructor`) whose return type is `Self`, `&Self`, `&mut Self`, the named impl type, `Box<Self>`, `Result<Self, _>`, or `Result<NamedType, _>`. Emit `syn::Error::new_spanned(ret_ty, "...").to_compile_error()` with rule name MXL120 and the canonical fix.

Constructor detection: a static method named `new` (no receiver) on a vctrs impl, or any method with `#[miniextendr(constructor)]`. The check is restricted to those.

## Tests
- New UI test: `miniextendr-macros/tests/ui_vctrs/impl_vctrs_ctor_returns_self.rs` — minimal `#[miniextendr(vctrs(...))]` impl whose `new` returns `Self`. Paired with `.stderr` snapshot.
- Existing happy paths (DerivedCurrency's `new` returning `Vec<f64>` after the fixture fix) must still compile.

## Fixture update
- `DerivedCurrency::new` currently returns `Self`. Change to return `Vec<f64>` (the `amounts` payload). The currency `symbol` is no longer carried in the Rust handle, but the `vctrs::new_vctr` wrapper only needs the vector — the class name already identifies the type. Format method becomes a free function reading the class attribute or the symbol must be passed per-call.
- Since `symbol` isn't accessible from a plain `Vec<f64>` vctrs wrapper, restructure the fixture: `new_derivedcurrency(symbol, amounts)` returns `Vec<f64>` wrapped via `new_vctr`. The `format_currency` instance method can no longer use `self.symbol` — change it to take an explicit `symbol` parameter, or remove the ExternalPtr-based format method and replace it with a simpler test.
- Update `rpkg/tests/testthat/test-vctrs-derive.R` DerivedCurrency section: replace raw `.Call` tests with wrapper-based tests using `new_derivedcurrency(...)`.
- The old `.Call` tests that expected `externalptr` are replaced by tests that `new_derivedcurrency("$", c(1.0, 2.5))` returns a proper vctrs vector.

## Acceptance
- `cargo build` rejects `Self`-returning vctrs constructors with MXL120 error.
- `Rscript -e 'library(miniextendr); new_derivedcurrency("$", c(1.0, 2.5))'` succeeds.
- `just check && just clippy && just devtools-test` clean.
- UI trybuild snapshot committed.

## Out of scope
- Mirror in `miniextendr-lint` (filed as follow-up issue).
- Auto-unwrapping Self into payload (option 2 — explicitly rejected).
- Other vctrs impls beyond `DerivedCurrency`.
