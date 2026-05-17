+++
title = "ExternalPtr"
weight = 6
description = "Type-safe opaque pointers with GC-integrated finalizers"
+++

`ExternalPtr<T>` is a Box-like owned pointer that wraps R's `EXTPTRSXP`. It lets you hand ownership of Rust-allocated data to R and let R's garbage collector decide when to drop it.

## Why ExternalPtr?

R has no native way to hold arbitrary Rust data. `ExternalPtr<T>` wraps R's `EXTPTRSXP` with:

- **Type-safe access** via `Any::downcast`
- **Automatic cleanup** via R GC finalizer that calls `Drop`
- **Box-like API** (`Deref`, `DerefMut`, `Clone`, `into_inner`, `into_raw`, `pin`, etc.)

## Creating an ExternalPtr

### With `#[derive(ExternalPtr)]` (recommended)

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct Database {
    connection: Connection,
}

#[miniextendr]
impl Database {
    pub fn new(path: &str) -> Self {
        Database { connection: Connection::open(path).unwrap() }
    }

    pub fn query(&self, sql: &str) -> Vec<String> {
        // ...
    }
}
```

The derive implements `TypedExternal` and `IntoExternalPtr`, so returning your struct from a `#[miniextendr]` function automatically wraps it.

## When to Use

| Strategy | Lifetime | Use Case |
|----------|----------|----------|
| `ExternalPtr` | Until R GCs | Rust data owned by R (structs returned to R) |
| `ProtectScope` | Within `.Call` | Temporary R allocations |
| Preserve list | Across `.Call`s | Long-lived R objects (not Rust values) |

## Cross-Package Dispatch

ExternalPtr objects can be passed between R packages. The `TypedExternal` trait provides R-visible type names, and `Any::downcast` provides type safety.

## Storage Model

`ExternalPtr` stores `Box<Box<dyn Any>>`:
- **Thin pointer** in `R_ExternalPtrAddr` (the outer Box)
- **Fat pointer** on the heap (carries the `Any` vtable)
- Non-generic finalizer `release_any` works for all types

## Full reference

This page is a curated entry point. See the [user manual](/manual/externalptr/) for the exhaustive treatment, edge cases, and every feature switch.
