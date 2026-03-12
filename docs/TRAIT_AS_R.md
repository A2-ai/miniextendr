# miniextendr Trait-Based ABI Implementation Plan

This document describes the trait ABI system for cross-package trait dispatch.

## Overview

The trait ABI enables:

1. **Trait-based dispatch**: R packages can call trait methods via vtables
2. **Cross-package interop**: Objects from one package usable in another
3. **Type safety**: Runtime type checking via 128-bit tags
4. **R-native**: Everything crossing the boundary is `SEXP`

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ R Code          │     │ C-callables      │     │ Rust Runtime    │
│                 │     │ (rpkg)           │     │ (miniextendr)   │
│ .Call("method", │────►│ mx_query()       │────►│ vtable lookup   │
│       obj, ...) │     │ mx_wrap()        │     │ method shim     │
│                 │◄────│ mx_get()         │◄────│ type conversion │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

## File Structure (Scaffolded)

### Rust (miniextendr-api)

| File | Purpose |
|------|---------|
| `src/abi.rs` | Frozen ABI types (`mx_tag`, `mx_erased`, `mx_base_vtable`, `mx_meth`) |
| `src/trait_abi/mod.rs` | Module entry point, re-exports |
| `src/trait_abi/ccall.rs` | C-callable loading via `R_GetCCallable` |
| `src/trait_abi/conv.rs` | Type conversion helpers for shims |
| `src/externalptr.rs` | `ExternalPtr<T>` + `TypedExternal` |

### Rust (miniextendr-macros)

| File | Purpose |
|------|---------|
| `src/miniextendr_trait.rs` | `#[miniextendr]` on traits → TAG, VTable, View, shims |
| `src/miniextendr_impl_trait.rs` | `#[miniextendr]` on `impl Trait for Type` → vtable static |

### Rust (miniextendr-lint)

| Future Lints | Purpose |
|--------------|---------|
| `missing_vtable` | `impl Trait for Type` without `#[miniextendr]` on the impl |
| `tag_collision` | Duplicate `mx_tag` values across traits |
| `unused_trait_impl` | Vtable generated but type not exposed via ExternalPtr |

### C (rpkg)

| File | Purpose |
|------|---------|
| `inst/include/mx_abi.h` | Public C header with ABI types |
| `miniextendr-api/src/mx_abi.rs` | Rust implementation of C-callable functions |

## Usage (Future)

### 1. Define a Trait

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
}
```

Generates:

- `TAG_COUNTER: mx_tag` - Trait identifier
- `CounterVTable` - Function pointer table
- `CounterView` - Runtime wrapper (data + vtable)
- `__counter_build_vtable::<T>()` - Vtable builder

### 2. Implement the Trait

```rust
struct MyCounter { value: i32 }

#[miniextendr]
impl Counter for MyCounter {
    fn value(&self) -> i32 { self.value }
    fn increment(&mut self) { self.value += 1; }
}
```

Generates:

- `__VTABLE_COUNTER_FOR_MYCOUNTER: CounterVTable`

### 3. Registration (Automatic)

All `#[miniextendr]` items are automatically registered via linkme distributed slices:

```rust
#[derive(ExternalPtr)]
struct MyCounter { value: i32 }

#[miniextendr]
impl MyCounter {
    fn new(initial: i32) -> Self { Self { value: initial } }
}
// Registration is automatic via #[miniextendr].
```

The trait impl registration generates:

- `__MxWrapperMyCounter` - Type-erased wrapper struct
- `__MX_BASE_VTABLE_MYCOUNTER` - Base vtable with drop/query
- `__mx_wrap_mycounter()` - Constructor returning `*mut mx_erased`

## ABI Types (Frozen)

All types in `miniextendr_api::abi` are `#[repr(C)]` and append-only:

```rust
// 128-bit type tag
pub struct mx_tag { lo: u64, hi: u64 }

// Method signature: (data, argc, argv) -> SEXP
pub type mx_meth = extern "C" fn(*mut c_void, i32, *const SEXP) -> SEXP;

// Base vtable (present in all erased objects)
pub struct mx_base_vtable {
    drop: extern "C" fn(*mut mx_erased),
    concrete_tag: mx_tag,
    query: extern "C" fn(*mut mx_erased, mx_tag) -> *const c_void,
}

// Type-erased object header
pub struct mx_erased {
    base: *const mx_base_vtable,
}
```

## Implementation Milestones

### MVP (Complete)

- [x] `abi.rs` with type definitions
- [x] `trait_abi/` module structure
- [x] C header and source stubs
- [x] `#[miniextendr]` routing for traits
- [x] `#[miniextendr]` routing for trait impls
- [x] Implement `mx_tag_from_path()` hash function (FNV-1a, const-compatible)
- [x] Implement direct FFI linkage to `mx_abi.rs` functions
- [x] Implement `conv.rs` conversion helpers
- [x] Implement C-callables in `mx_abi.rs` (pure Rust, no C files)

### M1: Code Generation (Complete)

- [x] `#[miniextendr]` on trait: generate TAG, VTable, View, shims
- [x] `#[miniextendr]` on impl: generate vtable static

### M2: Integration (Complete)

- [x] Trait registration via `#[miniextendr]` (now automatic via linkme)
- [x] `.Call` wrapper generation (via `#[miniextendr]` on impl blocks)
- [x] Panic handling in shims (catch_unwind)
- [x] Tests and examples (see `rpkg/src/rust/trait_abi_tests.rs`)

### M3: Polish

- [x] Cross-package example (documented in "Cross-Package Example" section)
- [x] Documentation (TRAIT_AS_R.md updated with usage examples)
- [x] Error diagnostics (improved runtime error messages for type mismatches)
- [x] miniextendr-lint: missing `impl Trait for Type;` registration detection
- [ ] miniextendr-lint: tag collision detection (future)
- [x] R tests for trait method `.Call` wrappers (`rpkg/tests/testthat/test-trait-abi.R`)

## Design Decisions

### Why `#[miniextendr]` instead of separate macros?

1. **Consistency**: Single attribute for all R interop
2. **Auto-detection**: Macro detects item type (fn, impl, trait, struct)
3. **Familiarity**: Users already know `#[miniextendr]`

### Why C-callables instead of direct linking?

C-callables (`R_RegisterCCallable` / `R_GetCCallable`) enable:

1. Cross-package dispatch without compile-time linking
2. ABI stability across independently-compiled packages
3. R's standard mechanism for native sharing

## Consumer Package Requirements

Packages that want to use the trait ABI must:

### 1. DESCRIPTION File

Add `miniextendr` (or the base package name) to both `LinkingTo` and `Imports`:

```
Package: mypackage
LinkingTo: miniextendr
Imports: miniextendr
```

**Why both?**

- `LinkingTo`: Adds `miniextendr/inst/include` to compiler include paths, making `mx_abi.h` available
- `Imports`: Ensures miniextendr is loaded before mypackage (so C-callables are registered)

See [R-exts §5.4.3](https://cran.r-project.org/doc/manuals/r-release/R-exts.html#Linking-to-native-routines-in-other-packages) for details.

### 2. Initialization Code

Load C-callables in your package's `R_init_<pkg>()`:

```c
// In src/init.c
#include <R_ext/Rdynload.h>

// Function pointer types
typedef SEXP (*mx_wrap_fn)(mx_erased*);
typedef mx_erased* (*mx_get_fn)(SEXP);
typedef const void* (*mx_query_fn)(SEXP, mx_tag);

// Global function pointers (set at init)
static mx_wrap_fn p_mx_wrap = NULL;
static mx_get_fn p_mx_get = NULL;
static mx_query_fn p_mx_query = NULL;

void R_init_mypackage(DllInfo *dll) {
    // Load C-callables from miniextendr
    p_mx_wrap = (mx_wrap_fn) R_GetCCallable("miniextendr", "mx_wrap");
    p_mx_get = (mx_get_fn) R_GetCCallable("miniextendr", "mx_get");
    p_mx_query = (mx_query_fn) R_GetCCallable("miniextendr", "mx_query");

    // Register your own routines...
}
```

### 3. Rust Side (via mx_abi.rs)

Each package includes `mx_abi.rs` from miniextendr-api which provides `mx_wrap`/`mx_get`/`mx_query` functions.
`package_init()` (called by `miniextendr_init!`) calls `mx_abi_register()` to initialize the tag and register C-callables.
Rust code calls these directly via `extern "C"` linkage (no runtime dependency on miniextendr).

### Version Compatibility Warning

> **NB**: This mechanism is fragile. Changes to the interface in miniextendr must be recognized by consumer packages. Either:
>
> - Consumer packages depend on exact miniextendr version, OR
> - Consumer packages check at runtime that the loaded version matches what they compiled against

This is why the ABI types in `abi.rs` are frozen and append-only.

## Cross-Package Example

This example shows how package B (consumer) can use trait-based objects from package A (producer).

### Package A: Producer (defines trait and implementation)

**Rust code (`producer/src/rust/lib.rs`):**

```rust
use miniextendr_api::{miniextendr, ExternalPtr};

// Define the trait
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
}

// Implement for a concrete type
#[derive(ExternalPtr)]
pub struct SimpleCounter { value: i32 }

#[miniextendr]
impl Counter for SimpleCounter {
    fn value(&self) -> i32 { self.value }
    fn increment(&mut self) { self.value += 1; }
}

#[miniextendr]
impl SimpleCounter {
    fn new(initial: i32) -> Self { Self { value: initial } }
}
// Registration is automatic via #[miniextendr].
```

**Generated R wrappers (`producer/R/miniextendr-wrappers.R`):**

```r
# Type environment
SimpleCounter <- new.env(parent = emptyenv())
SimpleCounter$new <- function(initial) { ... }

# Trait namespace
SimpleCounter$Counter <- new.env(parent = emptyenv())
SimpleCounter$Counter$value <- function() { ... }
SimpleCounter$Counter$increment <- function() { ... }

# $ dispatch handles both inherent methods and trait namespaces
`$.SimpleCounter` <- function(self, name) {
    obj <- SimpleCounter[[name]]
    if (is.environment(obj)) {
        # Trait namespace - bind self to all methods
        bound <- new.env(parent = emptyenv())
        for (method_name in names(obj)) {
            method <- obj[[method_name]]
            if (is.function(method)) {
                environment(method) <- environment()
                bound[[method_name]] <- method
            }
        }
        bound
    } else {
        environment(obj) <- environment()
        obj
    }
}
```

### Package B: Consumer (uses producer's objects)

**DESCRIPTION:**

```yaml
Package: consumer
Imports: producer
```

**R code (`consumer/R/use_counter.R`):**

```r
#' Double a counter's value using trait methods
#' @param counter A SimpleCounter from the producer package
#' @export
double_counter <- function(counter) {
  # Access trait methods via $Counter$ namespace
  current <- counter$Counter$value()
  for (i in seq_len(current)) {
    counter$Counter$increment()
  }
  counter$Counter$value()
}
```

**Usage from R:**

```r
library(producer)
library(consumer)

# Create counter from producer
c <- SimpleCounter$new(5L)

# Use trait methods directly
c$Counter$value()      # 5
c$Counter$increment()
c$Counter$value()      # 6

# Use consumer function that calls trait methods
double_counter(c)      # 12
```

### How It Works

1. **Producer** generates `.Call` wrappers for trait methods (`C_SimpleCounter__Counter__value`, etc.)
2. **Producer** registers these in `R_init_producer_miniextendr`
3. **Consumer** imports `producer`, ensuring the DLL is loaded
4. **Consumer** calls trait methods via the `$Trait$method` syntax
5. The `$.SimpleCounter` dispatch binds `self` and returns trait methods with proper scope

### Cross-Package Vtable Dispatch (Future)

For true cross-package dispatch where consumer doesn't know the concrete type:

```r
# Future: consumer receives any object implementing Counter
increment_any_counter <- function(obj) {
  # Query for Counter vtable at runtime
  vtable <- mx_query(obj, TAG_COUNTER)
  if (!is.null(vtable)) {
    vtable$increment(obj)
  }
}
```

This requires the C-callable infrastructure (`mx_wrap`, `mx_get`, `mx_query`) which is scaffolded but not fully implemented.

## Non-Goals

- Generic trait methods (monomorphic only)
- Async trait methods
- Returning borrowed Rust references
- ABI stability across major versions

## References

- `miniextendr-api/src/abi.rs` - Type definitions
- `miniextendr-api/src/trait_abi/` - Runtime support
- `miniextendr-api/src/externalptr.rs` - ExternalPtr (`TypedExternal`)
- `miniextendr-macros/src/miniextendr_trait.rs` - Trait code generation
- `miniextendr-macros/src/miniextendr_impl_trait.rs` - Trait impl vtable generation
- `miniextendr-lint/` - Lints for trait ABI correctness (future)
- `rpkg/inst/include/mx_abi.h` - C header
