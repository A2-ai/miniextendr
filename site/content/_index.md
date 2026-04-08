+++
title = "miniextendr"
sort_by = "weight"
+++

miniextendr is a Rust-R interoperability framework for building R packages with Rust backends. It provides proc-macro-driven code generation, automatic type conversions, and first-class ALTREP support.

## Highlights

- **`#[miniextendr]` attribute** -- annotate functions and impl blocks, get R wrappers automatically
- **5 class systems** -- Env, R6, S3, S4, and S7 with a single attribute change
- **Zero-copy vectors** -- ALTREP support via derive macros for lazy/compact representations
- **CRAN-ready** -- vendored dependencies for offline builds, autoconf-based configure
- **Type-safe FFI** -- `ExternalPtr<T>` with GC-integrated finalizers and cross-package dispatch

## Quick Start

```rust
use miniextendr_api::miniextendr;

/// Add two integers.
/// @param a First number
/// @param b Second number
/// @return The sum
#[miniextendr]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

```r
library(mypackage)
add(1L, 2L)
# [1] 3
```
