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
| `src/externalptr.rs` | `ExternalPtr<T>` + `TypedExternal` (integrates trait ABI wrapper generation) |

### Rust (miniextendr-macros)

| File | Purpose |
|------|---------|
| `src/miniextendr_trait.rs` | `#[miniextendr]` on traits → TAG, VTable, View, shims |
| `src/miniextendr_impl_trait.rs` | `#[miniextendr]` on `impl Trait for Type` → vtable static |

### Rust (miniextendr-lint)

| Future Lints | Purpose |
|--------------|---------|
| `missing_vtable` | Trait impl without `#[miniextendr]` when type has `#[externalptr(traits = [...])]` |
| `tag_collision` | Duplicate `mx_tag` values across traits |
| `unused_trait_impl` | Vtable generated but type not exposed via ExternalPtr |

### C (rpkg)

| File | Purpose |
|------|---------|
| `inst/include/mx_abi.h` | Public C header with ABI types |
| `src/mx_abi.c` | C-callable implementations |

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

### 3. Register in Module

Register trait implementations in `miniextendr_module!` for cross-package dispatch:

```rust
#[derive(ExternalPtr)]
struct MyCounter { value: i32 }

#[miniextendr]
impl MyCounter {
    fn new(initial: i32) -> Self { Self { value: initial } }
}

miniextendr_module! {
    mod mypackage;

    impl MyCounter;                   // Register impl block methods
    impl Counter for MyCounter;       // Generate trait dispatch wrapper
}
```

The `impl Trait for Type;` line generates:
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
- [x] Implement `init_ccallables()` loader
- [x] Implement `conv.rs` conversion helpers
- [x] Implement C-callables in `mx_abi.c.in`

### M1: Code Generation (Complete)
- [x] `#[miniextendr]` on trait: generate TAG, VTable, View, shims
- [x] `#[miniextendr]` on impl: generate vtable static

### M2: Integration (Complete)
- [x] `impl Trait for Type;` syntax in `miniextendr_module!` for trait registration
- [x] `.Call` wrapper generation (via existing miniextendr_module! + impl blocks)
- [x] Panic handling in shims (catch_unwind)
- [x] Tests and examples (see `rpkg/src/rust/trait_abi_tests.rs`)

### M3: Polish
- [ ] Cross-package example
- [ ] Documentation
- [ ] Error diagnostics
- [x] miniextendr-lint: missing `impl Trait for Type;` registration detection
- [ ] miniextendr-lint: tag collision detection (future)

## Design Decisions

### Why `#[miniextendr]` instead of separate macros?

1. **Consistency**: Single attribute for all R interop
2. **Auto-detection**: Macro detects item type (fn, impl, trait, struct)
3. **Familiarity**: Users already know `#[miniextendr]`

### Why `impl Trait for Type;` in miniextendr_module?

The trait ABI wrapper generation is triggered by `impl Trait for Type;` in `miniextendr_module!` rather than a derive attribute because:
1. **Explicit registration**: Only traits listed in the module are exposed to R
2. **Single location**: All R-facing declarations in one place
3. **Lint-friendly**: Easy to verify matching `#[miniextendr]` annotations

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

### 3. Rust Side (via miniextendr-api)

```rust
// In R_init_mypackage
miniextendr_api::trait_abi::init_ccallables();
```

**Important**: The `miniextendr` package must be loaded first (via `Imports`), otherwise `R_GetCCallable` will fail.

### Version Compatibility Warning

> **NB**: This mechanism is fragile. Changes to the interface in miniextendr must be recognized by consumer packages. Either:
> - Consumer packages depend on exact miniextendr version, OR
> - Consumer packages check at runtime that the loaded version matches what they compiled against

This is why the ABI types in `abi.rs` are frozen and append-only.

## Non-Goals

- Generic trait methods (monomorphic only)
- Async trait methods
- Returning borrowed Rust references
- ABI stability across major versions

## References

- `miniextendr-api/src/abi.rs` - Type definitions
- `miniextendr-api/src/trait_abi/` - Runtime support
- `miniextendr-api/src/externalptr.rs` - ExternalPtr (integrates trait ABI wrapper generation)
- `miniextendr-macros/src/miniextendr_trait.rs` - Trait code generation
- `miniextendr-macros/src/miniextendr_impl_trait.rs` - Trait impl vtable generation
- `miniextendr-lint/` - Lints for trait ABI correctness (future)
- `rpkg/inst/include/mx_abi.h` - C header
