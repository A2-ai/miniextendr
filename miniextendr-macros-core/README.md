# miniextendr-macros-core

Shared parser types for the miniextendr proc-macro and lint crates.

## What this crate provides

- **`miniextendr_module` module** -- the `syn::Parse` implementation for the
  `miniextendr_module! { ... }` macro body, including all item types (`fn`,
  `struct`, `impl`, `impl Trait for Type`, `use`, `vctrs`).
- **Naming helpers** -- `call_method_def_ident_for()` and
  `r_wrapper_const_ident_for()` which produce the deterministic identifiers
  shared between the attribute macro (definition site) and the module macro
  (reference site).

## Why it exists

`miniextendr-macros` (a proc-macro crate) and `miniextendr-lint` (a regular
library crate used from `build.rs`) both need to parse `miniextendr_module!`
invocations. Previously the parser was duplicated via a sed-based sync step.
Extracting the shared code into this crate eliminates that duplication and
the associated maintenance burden.

## Usage

This crate is an internal implementation detail. Depend on it from
`miniextendr-macros` and `miniextendr-lint`:

```toml
[dependencies]
miniextendr-macros-core = { workspace = true }
```

Then re-export or import:

```rust
// In miniextendr-macros (proc-macro crate)
pub(crate) use miniextendr_macros_core::miniextendr_module;
pub(crate) use miniextendr_macros_core::{call_method_def_ident_for, r_wrapper_const_ident_for};

// In miniextendr-lint
use miniextendr_macros_core::miniextendr_module;
```
