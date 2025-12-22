# miniextendr-macros

Procedural macros for exporting Rust functions and types to R.

Most users should depend on `miniextendr-api` and use its re-exports, but this
crate can be used directly if you want only the macros.

## Macros

### `#[miniextendr]`

Exports a Rust function to R and generates the necessary C and R wrappers.

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

Registers exported functions and ALTREP classes for a package/module.

```rust
use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod mypkg;
    fn add;
}
```

### `#[r_ffi_checked]`

Wraps `extern "C-unwind"` blocks with thread assertions in debug builds.

### Derives

- `ExternalPtr` – implement `TypedExternal` for external pointers.
- `RNativeType` – mark newtypes as R-native for vector conversions.

## Notes

- Attributes like `#[miniextendr(unsafe(main_thread))]` and
  `#[miniextendr(coerce)]` control wrapper behavior and safety.
- R wrapper generation is driven by doc comments and roxygen tags.
- See `docs.md` and the `miniextendr-api` docs for full usage examples.

## Publishing to CRAN

This crate is a **build-time** dependency for R packages that use Rust. It is
not shipped to CRAN directly, but the generated wrappers and exported symbols
are part of the package.

Guidelines for CRAN-facing packages:
- Use the macros via `miniextendr-api` re-exports.
- Avoid the `nonapi` feature unless you are prepared for CRAN checks to report
  non-API symbol usage (see `NONAPI.md`).
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
- When adding attributes or changing wrapper behavior, update `docs.md` and
  any examples that mention macro output.
