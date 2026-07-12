---
name: miniextendr-externalptr
description: Use when the user asks about ExternalPtr, how Rust structs are stored as R objects, TypedExternal, Box<Box<dyn Any>> storage, pointer provenance for cached_ptr, the release_any finalizer, sidecar field accessors, the TYPE_NAME_CSTR vs TYPE_ID_CSTR distinction, or how to pass an ExternalPtr across crate or package boundaries.
---

# miniextendr ExternalPtr

`ExternalPtr<T>` is how Rust-owned data lives in R's heap as an
`EXTPTRSXP`. Understanding its storage layout, type-safety model, pointer
provenance rules, and the sidecar accessor codegen quirks is necessary for
any work that exposes Rust structs to R code.

## When to use this skill

- "How do I expose a Rust struct to R?"
- "What is ExternalPtr and how does it work?"
- "What is TypedExternal and why do I need it?"
- "How is type safety enforced across package boundaries?"
- "What is the Box<Box<dyn Any>> layout and why?"
- "What is cached_ptr and how must I handle its provenance?"
- "How are sidecar field accessors generated?"
- "Why does adding `.call = match.call()` to a sidecar wrapper break at runtime?"
- "How do I cross a crate boundary with an ExternalPtr?"
- "What is the release_any finalizer?"

## Key concepts

### Storage layout: Box<Box<dyn Any>>

`ExternalPtr<T>` stores `Box<Box<dyn Any>>` in two levels:

1. **Inner box** (`Box<T>` cast to `Box<dyn Any>`): owns the Rust value.
   The `Any` vtable carries `T`'s `drop` and `TypeId`. This is the fat pointer.
2. **Outer box** (`Box<Box<dyn Any>>`): wraps the fat pointer in a second
   allocation. Its raw pointer (`*mut Box<dyn Any>`) is a thin pointer that
   fits in R's `R_ExternalPtrAddr` field.

The indirection exists because R's `EXTPTRSXP` stores a single `void*` pointer.
A `Box<dyn Any>` fat pointer is two words (data + vtable) and does not fit in
a `void*`. The outer box reduces it to a single thin pointer.

On finalization, `release_any` receives the SEXP, reads the thin pointer from
`R_ExternalPtrAddr`, reconstructs `Box<Box<dyn Any>>` by dropping both boxes,
and the `Any` vtable dispatches to T's `Drop` implementation. No generic
parameter on `release_any` is needed because the vtable carries it.

### cached_ptr and pointer provenance

`ExternalPtr<T>` caches a `NonNull<T>` in its `cached_ptr` field. This allows
`as_ref()` / `as_mut()` to skip the `R_ExternalPtrAddr` FFI call on every
access.

The critical constraint: **`cached_ptr` must have mutable provenance**.

When constructing `ExternalPtr::new(x)`:
1. The value is placed in a `Box<T>` via `Box::new(x)`.
2. `Box::into_raw(box_t)` yields a `*mut T` with mutable provenance.
   This raw pointer is stored as `cached_ptr`.
3. The same allocation is then re-wrapped as `Box<dyn Any>` and placed in the
   outer `Box<Box<dyn Any>>`.

The mutable-provenance constraint means you must never derive `cached_ptr` from
a `&T` reference or from `downcast_ref`. Those yield shared-reference provenance;
writing through them later is undefined behavior under the Stacked Borrows model.
The correct paths are `Box::into_raw`, `downcast_mut`, or `ptr::from_mut`.

See `miniextendr-api/src/externalptr.rs` (the `ExternalPtr::new` implementation
around line 353) for the concrete construction sequence that preserves provenance.

### TypedExternal trait

`TypedExternal` provides R-visible type identification. Two constants are required:

- `TYPE_NAME_CSTR: &'static [u8]` — short display name for the R tag slot
  (shown when you print an ExternalPtr in R). Example: `b"MyData\0"`.
- `TYPE_ID_CSTR: &'static [u8]` — namespaced type ID stored in the `prot`
  list's index-0 slot. Format: `"crate_name@crate_version::module_path::TypeName\0"`.
  The crate name and version mean that two types with the same name from
  different crates (or different versions of the same crate) are distinct
  identities and will not be confused.

`TypedExternal` is implemented manually or via `#[derive(ExternalPtr)]`. The
derive macro generates both constants from the type's name and module path using
`module_path!()` and `env!("CARGO_PKG_NAME")`.

### Type safety: Any::downcast, not R symbols

`TypedExternal` constants provide display names and error messages. They are
not the mechanism for type safety. The authoritative type check is
`Any::downcast_ref::<T>()` / `Any::downcast_mut::<T>()`, which uses Rust's
`TypeId`. An ExternalPtr wrapping `MyStruct` can only be downcast to
`MyStruct` — not to any other type with the same name or layout.

This means type safety survives across package boundaries without requiring a
shared header file or agreed-upon name string.

### IntoExternalPtr marker trait

`IntoExternalPtr: TypedExternal` is a marker trait that gives a type a blanket
`IntoR` implementation. When a type derives `#[derive(ExternalPtr)]`, it
implements both `TypedExternal` and `IntoExternalPtr`, which means it can be
returned directly from a `#[miniextendr]` function:

```rust
#[derive(ExternalPtr)]
struct MyData { value: i32 }

#[miniextendr]
fn create_data(v: i32) -> MyData {
    MyData { value: v }  // Wrapped automatically via IntoExternalPtr blanket
}
```

### The release_any finalizer

`release_any` is defined in `miniextendr-api/src/externalptr.rs` as a
non-generic `extern "C-unwind"` function registered with
`R_RegisterCFinalizerEx`. When R's GC collects an `EXTPTRSXP`:

1. `release_any` reads `R_ExternalPtrAddr(sexp)` as `*mut Box<dyn Any>`.
2. It calls `R_ClearExternalPtr(sexp)` to prevent double-finalization.
3. It reconstructs `Box<Box<dyn Any>>` from the raw pointer and drops it.
4. The drop chain runs: outer Box drops → inner Box<dyn Any> drops → T's
   `Drop` runs via the vtable.

Because `Box<dyn Any>` carries the vtable, `release_any` handles every
`ExternalPtr<T>` type with one concrete function. No generic finalizer per
type is needed.

### Sidecar field accessors

The `#[r_data]` annotation on struct fields inside a `#[derive(ExternalPtr)]`
struct enables R-side accessor generation. These accessors are called sidecar
slots and they have a fundamentally different C wrapper signature from normal
`#[miniextendr]` functions.

**Normal `#[miniextendr]` C wrappers** (via `c_wrapper_builder.rs`) prepend
`__miniextendr_call: SEXP` as the first parameter and register with `numArgs`
counting that slot.

**Sidecar accessor C wrappers** (generated in `miniextendr-macros/src/externalptr_derive.rs`)
do not have the `__miniextendr_call` slot:
- Getter: `(x: SEXP) -> SEXP`, registered with `numArgs: 1`.
- Setter: `(x: SEXP, value: SEXP) -> SEXP`, registered with `numArgs: 2`.

The R-side wrappers for sidecar accessors are generated as standalone
`Type_get_field()` / `Type_set_field()` functions (environment class system
default) or as active bindings (R6) or property accessors (S7). These wrappers
call `.Call(C_mypkg__mx_rdata_get_Type_field, self$.ptr)` directly (C symbols are crate-prefixed since #1273) — they do not pass a `.call`
argument.

Adding `.call = match.call()` to a sidecar R wrapper causes an "Incorrect
number of arguments" error at runtime because the C wrapper expects `numArgs`
arguments, not `numArgs + 1`. This was the root cause of a PR #344 regression
that was subsequently reverted.

Three field tiers are supported in sidecar slots:
- Raw SEXP — direct SEXP storage, no conversion.
- Zero-overhead scalars (`i32`, `f64`, `bool`, `u8`) — stored directly in R
  memory, read/written without copying.
- Conversion types — uses `IntoR` / `TryFromSexp` round-trip.

### Cross-package ABI

`ExternalPtr<T>` values can cross package boundaries when both packages depend
on the same `miniextendr-api` version and the same `TypedExternal` impl (i.e.,
from the same producer crate). The cross-package handoff mechanism is
`mx_abi` in `miniextendr-api/src/mx_abi.rs`. This provides `mx_wrap`,
`mx_get`, `mx_query`, and `mx_abi_register` — thin wrappers around
`R_MakeExternalPtr` / `R_ExternalPtrAddr` that are shared via
`R_GetCCallable`.

The `tests/cross-package/` directory contains `producer.pkg` and `consumer.pkg`
that exercise this path end-to-end.

## How it works

### Creating an ExternalPtr

```
ExternalPtr::new(value)
  1. Box::new(value)  →  *mut T  (mutable provenance, saved as cached_ptr)
  2. Box::from_raw(raw) as Box<dyn Any>  →  fat ptr carrying Any vtable
  3. Box::into_raw(Box::new(inner))  →  *mut Box<dyn Any>  (thin ptr)
  4. with_r_thread { R_MakeExternalPtr(thin_ptr, tag_sym, prot_list) }
  5. R_RegisterCFinalizerEx(sexp, release_any, TRUE)
  6. Return ExternalPtr { sexp, cached_ptr, _marker }
```

`ExternalPtr::new` uses `with_r_thread` so it can be called safely from the
worker thread. The SEXP creation and finalizer registration happen on R's main
thread; the Rust allocation happens on the calling thread.

### Accessing the wrapped value

```rust
// Shared access
let ptr = ExternalPtr::wrap_sexp(sexp);   // borrow the EXTPTRSXP
if let Some(data) = ptr.as_ref() {
    // data: &MyData — via downcast from Box<dyn Any>
}

// Mutable access
if let Some(data) = ptr.as_mut() {
    // data: &mut MyData
}
```

Both `as_ref()` and `as_mut()` use `cached_ptr` rather than calling back into
R's FFI. Type safety is from `Any::downcast_ref` / `Any::downcast_mut`.

### Choosing the right class system for sidecar fields

The `#[externalptr(...)]` attribute selects the R class system:

| Attribute | Class system | Sidecar R accessor form |
|-----------|-------------|------------------------|
| `#[externalptr(env)]` (default) | Environment | `Type_get_field()`, `Type_set_field()` |
| `#[externalptr(r6)]` | R6 | Active bindings in R6Class |
| `#[externalptr(s3)]` | S3 | `$.class`, `$<-.class` methods |
| `#[externalptr(s4)]` | S4 | Slot accessors |
| `#[externalptr(s7)]` | S7 | Properties via `new_property()` |

## Decision trees

### Type-tagged R object vs pure data?

Returning Rust data to R where R needs to hold it across `.Call` boundaries
and where the data is opaque to R:
- Use `ExternalPtr<T>`.

Returning Rust data to R where R needs to inspect, modify, or serialize it:
- Consider `#[derive(IntoList)]` (converts to a named R list) or
  `miniextendr-serde` (via serde Serialize/Deserialize).

If the object must survive `saveRDS()`/`readRDS()` or a session restart:
- An `ExternalPtr` cannot (see the pitfall below). Return plain data, a serde
  list, or an R closure that captures plain R data instead.

If the data is array-like and benefits from lazy evaluation:
- Consider ALTREP (`miniextendr-altrep` skill).

### When to use sidecar fields?

Sidecar fields (`#[r_data]`) are appropriate when a subset of the struct's
fields should be readable or writable from R without going through a full
round-trip of serialization. They store R SEXP values alongside the Rust
data inside the `EXTPTRSXP`.

Use sidecar fields for:
- Small scalar values that R code needs to read frequently without a Rust
  function call overhead.
- R-owned SEXP values (plots, closures, environments) that the Rust struct
  must retain a reference to without R GC-ing them.

Avoid sidecar fields for:
- Large data — each sidecar slot is a separate R allocation.
- Data that changes often and where conversion cost matters.

### Crossing a crate boundary?

If a producer package exports an `ExternalPtr<T>` and a consumer package must
receive it:
1. The consumer must have the same `TypedExternal` impl (typically by depending
   on the producer crate as a library).
2. The handoff uses `mx_abi`'s `R_GetCCallable`-based protocol
   (`miniextendr-api/src/mx_abi.rs`).
3. The `tests/cross-package/` smoke test exercises this path.

## Key files

- `miniextendr-api/src/externalptr.rs` — `ExternalPtr<T>`, `TypedExternal`,
  `IntoExternalPtr`, `ErasedExternalPtr`, `ExternalSlice<T>`, and the
  `release_any` finalizer. The `ExternalPtr::new` implementation (around line
  353) is the canonical reference for the mutable-provenance construction sequence.
- `miniextendr-macros/src/externalptr_derive.rs` — `#[derive(ExternalPtr)]`
  proc-macro. Generates `TypedExternal` impls, sidecar accessor C wrappers
  (`numArgs: 1` getter / `numArgs: 2` setter), and R wrapper fragments.
- `miniextendr-macros/src/typed_external_macro.rs` — `TYPE_NAME_CSTR` /
  `TYPE_ID_CSTR` constant generation.
- `miniextendr-api/src/mx_abi.rs` — cross-package ABI: `mx_wrap`, `mx_get`,
  `mx_query`, `mx_abi_register`.
- `tests/cross-package/` — producer/consumer smoke test exercising cross-crate
  `ExternalPtr` handoff.

## Common pitfalls

- **Deriving cached_ptr from &T or downcast_ref is UB**: the cached pointer
  must be derived from a mutable path (`Box::into_raw`, `downcast_mut`,
  `ptr::from_mut`). Writing through a `*mut T` derived from a shared reference
  is undefined behavior under the Stacked Borrows model. `ExternalPtr::new`
  preserves this invariant by capturing the `*mut T` from `Box::into_raw`
  before erasing to `dyn Any`.

- **Sidecar R wrappers must not include .call = match.call()**: sidecar
  accessors have `numArgs: 1` (getter) or `numArgs: 2` (setter) with no
  `__miniextendr_call` slot. Adding `.call` to the `.Call()` invocation adds
  an extra argument and causes "Incorrect number of arguments" at runtime.
  This is the intentional distinction from `#[miniextendr]` method wrappers.

- **TypedExternal is display, not type safety**: `TYPE_NAME_CSTR` and
  `TYPE_ID_CSTR` are stored for display and error messages. Type safety is
  enforced by `Any::downcast`, which uses Rust `TypeId`. Two types with
  identical `TYPE_ID_CSTR` strings but different `TypeId`s will downcast
  correctly (they are distinct). Two types with the same Rust `TypeId` from
  different `TypedExternal` impls are the same type — the string constants are
  irrelevant to safety.

- **release_any double-finalization guard**: `R_ClearExternalPtr` is called
  at the start of `release_any` (after the null check) to ensure that if R
  calls the finalizer twice, the second invocation sees a null pointer and
  returns early. Do not remove this guard when modifying the finalizer.

- **ExternalPtr is not an R native type**: R cannot coerce an EXTPTRSXP to
  a vector or use it in vectorized operations. This is an R limitation, not a
  miniextendr limitation. If you need R-vectorized behavior over Rust data,
  use ALTREP.

- **ExternalPtr does not survive `saveRDS()` / session restarts**: the pointer
  address cannot be serialized, so a round-tripped object keeps its R class
  but its pointer is dead — every subsequent access fails the `Any::downcast`
  with an "expected ExternalPtr<T>" error. This is inherent to `EXTPTRSXP`,
  not a miniextendr bug, and it is easy to miss because `readRDS()` itself
  succeeds. If users need a persistable interface, expose a base-R-style
  closure that captures plain R data (which serializes fine and can rebuild
  the pointer-backed object lazily), or provide getters for the plain data so
  the object can be reconstructed after reload. At minimum, document the
  limitation on the constructor.

- **Cross-package type safety requires the same crate version**: the `TypeId`
  in `Any::downcast` is per-compilation. Two separate compilations of the same
  source file (e.g., different Cargo.lock versions of the producer crate) yield
  different `TypeId`s. Pin the producer crate version in the consumer's
  `Cargo.toml` to guarantee compatibility.

## Related skills

- `miniextendr-architecture` — how distributed_slice registration works and
  where `ExternalPtr` fits in the overall crate graph.
- `miniextendr-macros` — the `#[miniextendr]` attribute on impl blocks that
  generates R class wrappers around ExternalPtr types.
- `miniextendr-conversions` — `TryFromSexp` / `IntoR` used inside sidecar
  accessor bodies for conversion-type fields.
- `miniextendr-altrep` — alternative to ExternalPtr when you need R to treat
  Rust data as a native lazy vector.
- `miniextendr-ffi` — `#[r_ffi_checked]`, `_unchecked` FFI variants, and the
  ALTREP callback context where `ExternalPtr` accessors may be called.
