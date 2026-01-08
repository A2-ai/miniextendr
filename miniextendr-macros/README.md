# miniextendr-macros

Procedural macros for exporting Rust functions and types to R.

Most users should depend on `miniextendr-api` and use its re-exports, but this
crate can be used directly if you only need the macros.

## Macros

### `#[miniextendr]`

Exports Rust items to R and generates the necessary C and R wrappers.

Applies to:
- functions
- inherent impl blocks
- trait definitions (trait ABI metadata)
- trait impls (`impl Trait for Type`) for vtables and wrappers

```rust
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Common attributes:
- `#[miniextendr(unsafe(main_thread))]` – run directly on R's main thread.
- `#[miniextendr(invisible)]` / `#[miniextendr(visible)]` – control visibility.
- `#[miniextendr(check_interrupt)]` – call `R_CheckUserInterrupt()` up front.
- `#[miniextendr(coerce)]` – enable coercion for non-native argument types.

### `miniextendr_module!`

Registers exported functions, ALTREP classes, and trait impls for a
package/module.

```rust
use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod mypkg;
    fn add;
}
```

### `#[r_ffi_checked]`

Wraps `extern "C-unwind"` blocks with wrappers that route calls to R's main
thread when invoked from a non-main thread (requires a worker context).

### Derives

- `ExternalPtr` – implement `TypedExternal` for external pointers.
- `RNativeType` – mark newtypes as R-native for vector conversions.
- ALTREP derives (e.g., `AltrepInteger`, `AltrepReal`, `AltrepString`).
- `RFactor` – enum <-> factor conversions.

### Helpers

- `typed_list` – derive typed list builders from a Rust struct.

## Notes

- Attributes like `#[miniextendr(unsafe(main_thread))]` and
  `#[miniextendr(coerce)]` control wrapper behavior and safety.
- R wrapper generation is driven by doc comments and roxygen tags.
- Impl‑block support covers S3/S4/S7/R6 methods plus env‑style dispatch.
- Trait dispatch requires `#[miniextendr]` on the trait definition and the
  trait impl, plus `impl Trait for Type;` in `miniextendr_module!`.

## Publishing to CRAN

This crate is a **build-time** dependency for R packages that use Rust. It is
not shipped to CRAN directly, but the generated wrappers and exported symbols
are part of the package.

Guidelines for CRAN-facing packages:
- Use the macros via `miniextendr-api` re-exports.
- Avoid the `nonapi` feature unless you are prepared for CRAN checks to report
  non-API symbol usage.
- Regenerate and commit the generated R wrappers (`R/miniextendr_wrappers.R`).
- Keep exported symbol names stable, and document any changes.

## Testing

This crate uses `trybuild` for compile-fail tests:

```sh
cargo test -p miniextendr-macros
```

## Maintainer

- Keep `syn`, `quote`, and `proc-macro2` versions in sync with workspace
  requirements.
- Maintain trybuild fixtures for any macro expansion changes.
- Ensure roxygen/doc extraction logic stays aligned with `miniextendr-api`
  conventions.
- When adding attributes or changing wrapper behavior, update user-facing
  examples and any tests that validate macro output.
