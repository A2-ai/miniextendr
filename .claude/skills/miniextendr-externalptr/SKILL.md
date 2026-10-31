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
- "Why does adding a `.call` argument to a sidecar wrapper break at runtime?"
- "How do I cross a crate boundary with an ExternalPtr?"
- "What is the release_any finalizer?"

## Key concepts

### Storage layout: Box<Box<dyn Any>>

`ExternalPtr<T>` stores `Box<Box<dyn Any>>` in two levels:

1. **Inner box** (`Box<RefCell<T>>` cast to `Box<dyn Any>`): owns the Rust
   value. The `Any` vtable carries `RefCell<T>`'s `drop` and `TypeId`. This is
   the fat pointer. The generated code borrows the `RefCell` for each call, so
   a callback into R that reaches the same object gets a
   `miniextendr_borrow_conflict` error instead of a second reference (#1965).
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

`ExternalPtr<T>` caches a `NonNull<RefCell<T>>` in its `cached_ptr` field, so
`borrow()` / `borrow_mut()` skip the `R_ExternalPtrAddr` FFI call. Every read
of the value is checked (#2106): the handle has no `Deref`, and `borrow()` /
`borrow_mut()` return the views `ExternalPtrRef` / `ExternalPtrMut`, raising
`miniextendr_borrow_conflict` on a conflict. The generated wrappers borrow at
the entry point, through `borrow_value` / `borrow_value_mut`, for receivers and
for `&T` / `&mut T` / view arguments (exclusive for `&mut T` / `ExternalPtrMut`,
shared otherwise), and hand the body plain references. An `ExternalPtr<T>`
argument takes no borrow: it's a handle the body keeps, consumes or reads with
`borrow()`. Only the ALTREP data1 paths read unchecked, through the hidden
`as_ref_unchecked` / `as_mut_unchecked` (#2103).

Writes go through the `RefCell`'s `UnsafeCell`, so a `cached_ptr` derived from
`&RefCell<T>` (`downcast_ref`, `NonNull::from`) may be written through. Never
cache a pointer to the `T` itself from a `&T`; that has shared-reference
provenance, and writing through it is undefined behavior under Stacked Borrows.

See `ExternalPtr::erase` in `miniextendr-api/src/externalptr.rs` for the
construction sequence.

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

Adding a `.call` argument to a sidecar R wrapper causes an "Incorrect
number of arguments" error at runtime because the C wrapper expects `numArgs`
arguments, not `numArgs + 1`. This was the root cause of a PR #344 regression
that was subsequently reverted.

Three field tiers are supported in sidecar slots:
- `Sidecar<T>` — a typed value kept in the external pointer's `prot` list
  (after the type ID and the user slot), so the pointer roots it and
  `saveRDS()` writes it. The derive generates Rust accessors with the field's
  visibility, as associated functions taking the handle (#1856):
  `#[r_data(ref)]` → `fn f(ptr: &impl SidecarOwner<Self>) -> T`,
  `#[r_data(mut)]` → `fn set_f(ptr: &mut impl SidecarOwnerMut<Self>, value: T)`,
  `#[r_data(ref, mut)]` or a bare `#[r_data]` → both. `ExternalPtr`,
  `ExternalPtrRef` and `ExternalPtrMut` are the owners. A method that needs
  the value takes a view (`self: ExternalPtrRef<'_, Self>` /
  `mut self: ExternalPtrMut<'_, Self>`) and calls `Self::f(&self)`, in an
  inherent impl or a trait (which also works over a trait-ABI object).
  The struct holds nothing that points back at its pointer, so a plain
  `&self` method (a standard trait's included) and an ALTREP data-trait
  callback can't reach the values (#1880). A value passed to
  `Sidecar::new` is pending until the struct is wrapped or the field is next
  accessed; `into_inner` reads the slots back as pending values, and
  `ExternalPtr::clone` shares the slot values as R objects. Main thread only.
  A bare `SEXP` field is a compile error: the struct can't root it (#1846).
  The name is provisional (#1857).
- Scalars (`i32`, `f64`, `bool`, `u8`) — struct fields, returned as a fresh
  length-1 vector and written with `Rf_as*`.
- Conversion types — struct fields, `IntoR` / `TryFromSexp` on every access.

Scalar and conversion values live behind the pointer's address, which
`saveRDS()` does not write; after `readRDS()` their getters error on the null
pointer while the `Sidecar<T>` R accessors still work (checked by the
`prot[0]` type ID). A type that derives `Serialize` and `Deserialize` is the
exception: its whole value comes back (#2186, below).

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

`ExternalPtr::new` must run on R's main thread: the SEXP creation and finalizer
registration go through `with_r_thread`, which panics on any other thread.

### Accessing the wrapped value

```rust
// A #[miniextendr] function borrows at the entry point:
#[miniextendr]
pub fn total(data: &MyData) -> f64 { data.value }

// A handle Rust holds reads through a checked borrow:
let mut ptr = unsafe { ExternalPtr::<MyData>::wrap_sexp(sexp) }.expect("a MyData");
let value = ptr.borrow().value;          // ExternalPtrRef<'_, MyData>
ptr.borrow_mut().value = value + 1.0;    // ExternalPtrMut<'_, MyData>
```

Both use `cached_ptr` rather than calling back into R's FFI. Type safety is
from the `Any::downcast` in `wrap_sexp`.

### Choosing the right class system for sidecar fields

The `#[externalptr(...)]` attribute selects the R class system:

| Attribute | Class system | Sidecar R accessor form |
|-----------|-------------|------------------------|
| `#[externalptr(env)]` (default) + `#[miniextendr(env(r_data_accessors))]` | Environment | `obj$field`, `obj[["field"]]` and their assignments, in `$.Class` before the methods |
| `#[externalptr(r6)]` + `#[miniextendr(r6(r_data_accessors))]` | R6 | Active bindings in R6Class |
| `#[externalptr(s3)]` + `#[miniextendr(s3(r_data_accessors))]` | S3 | `$` / `[[` / `$<-` / `[[<-` methods; other names fall through on a list or env object |
| `#[externalptr(s4)]` + `#[miniextendr(s4(r_data_accessors))]` | S4 | `setMethod("$")` / `setMethod("$<-")` |
| `#[externalptr(s7)]` + `#[miniextendr(s7(r_data_accessors))]` | S7 | Properties via `new_property()` |

On S3, S4 and env, `r_data_accessors = "get"` generates the reading methods
only (for a class with its own copy-on-modify `$<-`), and `r_data_accessors =
"read_only"` the readers and a `$<-` / `[[<-` that refuses every field with
`miniextendr_read_only_field`; the default setters write in place, so every
copy of the object sees the write. `#[r_data(read_only)]` makes one field
read-only from R in every class system (no setter; R6 bindings refuse a
value, S7 properties have no setter); `Computed` fields work the same way,
R6 and S7 included (#2268, #1888). Every class
system also gets the standalone `Type_get_field()` / `Type_set_field()`, which
take the pointer or any object carrying it; there is no `r_data_accessors!`
macro.

## Decision trees

### Type-tagged R object vs pure data?

Returning Rust data to R where R needs to hold it across `.Call` boundaries
and where the data is opaque to R:
- Use `ExternalPtr<T>`.

Returning Rust data to R where R needs to inspect, modify, or serialize it:
- Consider `#[derive(IntoList)]` (converts to a named R list) or
  `miniextendr-serde` (via serde Serialize/Deserialize).

If the object must survive `saveRDS()`/`readRDS()` or a session restart:
- Derive `Serialize` and `Deserialize` on the `ExternalPtr` type (with
  miniextendr-api's `serde` feature): the value comes back (see the pitfall
  below). Otherwise return plain data or a serde list instead.

If the data is array-like and benefits from lazy evaluation:
- Consider ALTREP (`miniextendr-altrep` skill).

### When to use sidecar fields?

Sidecar fields (`#[r_data]`) are appropriate when a subset of the struct's
fields should be readable or writable from R without hand-written getters.
Scalar and conversion fields live in the Rust struct and convert on every
access (a view: no R copy to drift); `Sidecar<T>` fields hold their values in
the pointer's `prot` list.

Use sidecar fields for:
- Values R code reads and writes as fields (R6 `obj$f`, S7 `obj@f`).
- R-owned values (data frames, closures, environments) that must live as long
  as the object: a `Sidecar<SEXP>` / `Sidecar<List>`, private if only Rust
  needs it.

Avoid sidecar fields for:
- Large data in a conversion field — every read converts the whole value.
- State that must survive `saveRDS()` in a scalar or conversion field of a
  type without serde (#1418); `Sidecar<T>` values do survive, and so does a
  serde type's whole value.

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

- **Sidecar R wrappers must not pass a `.call` argument**: sidecar
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

- **Only a serde `ExternalPtr` type survives `saveRDS()` / session
  restarts** (#2186): R saves an external pointer without its address. A
  type that derives `Serialize` and `Deserialize` (miniextendr-api's `serde`
  feature on) keeps its value in the save through an ALTREP carrier in the
  pointer's `prot` list, which R itself calls during `serialize()` and
  `readRDS()`; the value is attached at the first use. No attribute, no
  opt-out. Every `serialize()` encodes the whole value (parallel worker
  sends and `digest` hashing included), a save while a `&mut self` call
  holds the object raises `miniextendr_borrow_conflict`. The save uses the
  exact mode of the serde-to-R path (`serde/exact.rs`, #2256): the value
  comes back bit for bit, through untagged / internally tagged enums and
  `flatten` too (only `i128`/`u128` there fail: serde's `Content` has none);
  `to_r`/`from_r` keep the general, lossy mapping (#1417, #1419). A
  `serde::RObject` field carries an R object (a data frame, another saved
  object) through the save; R shares only environments, external pointers
  and symbols within one save, so anything else comes back as a copy per
  place. A save written before #2256 decodes only where the two mappings
  agree; there is no fallback. A save that fails to decode, a `version = 2` save, and any other type are
  refused at the first use with the classed `miniextendr_restored_*` errors
  (the decoding error as `e$parent`), and `readRDS()` itself succeeds. A
  save is found by crate + type name, not module path: moving a type keeps
  its saves, renaming it breaks them, and two serde types of one crate with
  the same name fail the package load.
  `docs/EXTERNALPTR.md`, "A serde type comes back after `readRDS()`".

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
