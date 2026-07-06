# miniextendr_api v0.1.0

miniextendr-api: core runtime for Rust <-> R interop.

This crate provides the FFI surface, safety wrappers, and macro re-exports
used by most miniextendr users. It is the primary dependency for building
Rust-powered R packages and exposing Rust types to R.

At a glance:
- FFI bindings + checked wrappers for R's C API (`sys`, `r_ffi_checked`).
- Conversions between Rust and R types (`IntoR`, `TryFromSexp`, `Coerce`).
- ALTREP traits, registration helpers, and iterator-backed ALTREP data types.
- Wrapper generation from Rust signatures (`#[miniextendr]`, automatic registration via linkme).
- Worker-thread pattern for panic isolation and `Drop` safety (`worker`).
- Class system support (S3, S4, S7, R6, env-style impl blocks).
- Cross-package trait ABI for type-erased dispatch (`trait_abi`).

Most users should depend on this crate directly. For embedding R in
standalone binaries or integration tests, see `miniextendr-engine`.

## Quick start

```ignore
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

That's it — `#[miniextendr]` handles everything. Items self-register
at link time; `miniextendr_init!` generates the `R_init_*` function
that calls `package_init()` to register all routines with R.
Wrapper R code is produced from Rust doc comments (roxygen tags are
extracted) by the cdylib-based wrapper generator and committed into
`R/miniextendr_wrappers.R` so CRAN builds do not require codegen.

## Choosing the right API

miniextendr has several places where two or more APIs reach the same
goal with different tradeoffs — a stricter / safer / more validated
option, and a looser / easier / less protective one. The most common
pairs are:

| I'm reaching for... | Consider also | Why |
|---|---|---|
| default `IntoR` for `i64` / `u64` / `isize` / `usize` (silently widens to `REALSXP` on overflow) | `#[miniextendr(strict)]` → [`crate::strict`] helpers (panic on overflow) | strict catches the truncation bugs caused by R having no native 64-bit integer type |
| [`Coerce`] (infallible widening) | [`TryCoerce`] (fallible) | the source range can exceed the target type |
| `Rf_*_unchecked` FFI | checked variants | unchecked is only safe inside ALTREP callbacks, `with_r_unwind_protect`, or `with_r_thread` — MXL301 lint enforces |
| `panic!(msg)` | `miniextendr_api::error!("msg", class = "...")` | typed conditions let R-side `tryCatch` handlers route by class |
| raw `_dots: &Dots` | `#[miniextendr(dots = typed_list!(...))]` | validation moves from runtime to macro call site |
| `#[derive(AltrepInteger)]` field-based | `#[altrep(manual)]` + handwritten traits | when custom storage or computed-on-access can't fit the derive |
| hand-rolled [`TryFromSexp`] + [`IntoR`] | `#[derive(RSerializeNative)]` (serde feature) | serde is ergonomic for nested structs; hand-rolled is zero-overhead and fully controlled |

Project-wide defaults are controlled by mutually-exclusive cargo
features — see the "Project-wide Defaults" feature table below.

### Default opinion

When in doubt, pick the **stricter** path. The framework's default
stance is "fail loudly, leave a trail." The looser variants exist for
cases where the cost is measured or the looser semantics are correct
for your data — they are not the default.

## GC protection and ownership

R's garbage collector can reclaim any SEXP that isn't protected. miniextendr
provides three complementary protection mechanisms:

| Strategy | Module | Lifetime | Release Order | Use Case |
|----------|--------|----------|---------------|----------|
| **PROTECT stack** | [`gc_protect`] | Within `.Call` | LIFO (stack) | Temporary allocations |
| **VECSXP pool** | [`protect_pool`] | Across `.Call`s | Any order | Long-lived R objects |
| **R ownership** | [`ExternalPtr`](struct@ExternalPtr) | Until R GCs | R decides | Rust data owned by R |

Quick guide:

**Temporary allocations during computation** -> [`ProtectScope`]
```ignore
unsafe fn compute(x: SEXP) -> SEXP {
    let scope = ProtectScope::new();
    let temp = scope.protect(Rf_allocVector(REALSXP, 100));
    // ... work with temp ...
    result.into_raw()
} // UNPROTECT(n) called automatically
```

**R objects surviving across `.Call`s** -> [`ProtectPool`] or `R_PreserveObject`
```ignore
// ProtectPool: O(1) insert/release with generational keys
let mut pool = unsafe { ProtectPool::new(16) };
let key = unsafe { pool.insert(backing_vec) };
// ... use across multiple .Calls ...
unsafe { pool.release(key) };
```

**Rust data owned by R** -> [`ExternalPtr`](struct@ExternalPtr)
```ignore
#[miniextendr]
fn create_model() -> ExternalPtr<MyModel> {
    ExternalPtr::new(MyModel::new())
} // R owns it; Drop runs when R GCs
```

Note: ALTREP trait methods receive raw SEXP pointers from R's runtime.
These are safe to dereference because R guarantees valid SEXPs in ALTREP callbacks.

## Threading and safety

R uses `longjmp` for errors, which can bypass Rust destructors. The default
pattern is to run Rust logic on a worker thread and marshal R API calls back
to the main R thread via `with_r_thread`. Most FFI wrappers are
main-thread routed via `#[r_ffi_checked]`. Use unchecked variants only when
you have arranged a safe context.

With the `nonapi` feature, miniextendr can disable R's stack checking to allow
calls from other threads. R is still not thread-safe; serialize all R API use.

## Feature Flags

### Core Features

| Feature | Description |
|---------|-------------|
| `nonapi` | Non-API R symbols (stack controls, mutable `DATAPTR`). May break with R updates. |
| `rayon` | Parallel iterators via Rayon. Adds `RParallelIterator`, `RParallelExtend`. |
| `connections` | Experimental R connection framework. **Unstable R API.** |
| `indicatif` | Progress bars routed through R connections. Requires `nonapi` + `connections`. |
| `vctrs` | vctrs class construction (`new_vctr`, `new_rcrd`, `new_list_of`) and `#[derive(Vctrs)]`. |
| `worker-thread` | Worker thread for panic isolation and `Drop` safety. Without it, stubs run inline. |

### Type Conversions (Scalars & Vectors)

| Feature | Rust Type | R Type | Notes |
|---------|-----------|--------|-------|
| `either` | `Either<L, R>` | Tries L then R | Union-like dispatch |
| `uuid` | `Uuid`, `Vec<Uuid>` | `character` | UUID ↔ string |
| `regex` | `Regex` | `character(1)` | Compiles pattern from R |
| `url` | `Url`, `Vec<Url>` | `character` | Validated URLs |
| `time` | `OffsetDateTime`, `Date` | `POSIXct`, `Date` | Date/time conversions |
| `ordered-float` | `OrderedFloat<f64>` | `numeric` | NaN-orderable floats |
| `num-bigint` | `BigInt`, `BigUint` | `character` | Arbitrary precision via strings |
| `rust_decimal` | `Decimal` | `character` | Fixed-point decimals |
| `num-complex` | `Complex<f64>` | `complex` | Native R complex support |
| `indexmap` | `IndexMap<String, T>` | named `list` | Preserves insertion order |
| `bitflags` | `RFlags<T>` | `integer` | Bitflags ↔ integer |
| `bitvec` | `RBitVec` | `logical` | Bit vectors ↔ logical |
| `tinyvec` | `TinyVec<[T; N]>`, `ArrayVec<[T; N]>` | vectors | Small-vector optimization |

### Matrix & Array Libraries

| Feature | Types | Conversions |
|---------|-------|-------------|
| `ndarray` | `Array1`–`Array6`, `ArrayD`, views | R vectors/matrices ↔ ndarray |
| `nalgebra` | `DVector`, `DMatrix` | R vectors/matrices ↔ nalgebra |

### Serialization

| Feature | Traits/Modules | Description |
|---------|----------------|-------------|
| `serde` | `RSerializeNative`, `RDeserializeNative` | Direct Rust ↔ R native serialization |
| `serde_json` | `RSerialize`, `RDeserialize` | JSON string serialization (includes `serde`) |
| `borsh` | `Borsh<T>` | Binary serialization ↔ raw vectors via Borsh |

### Adapter Traits (Generic Operations)

| Feature | Traits | Use Case |
|---------|--------|----------|
| `num-traits` | `RNum`, `RSigned`, `RFloat` | Generic numeric operations |
| `bytes` | `RBuf`, `RBufMut` | Byte buffer operations |

### Text & Data Processing

| Feature | Types/Functions | Description |
|---------|-----------------|-------------|
| `aho-corasick` | `AhoCorasick`, `aho_compile` | Fast multi-pattern string search |
| `toml` | `TomlValue`, `toml_from_str` | TOML parsing and serialization |
| `tabled` | `table_to_string` | ASCII/Unicode table formatting |
| `sha2` | `sha256_str`, `sha512_bytes` | Cryptographic hashing |

### Random Number Generation

| Feature | Types | Description |
|---------|-------|-------------|
| `rand` | `RRng`, `RDistributions` | Wraps R's RNG with `rand` traits |
| `rand_distr` | Re-exports `rand_distr` | Additional distributions (Normal, Exp, etc.) |

### Binary Data

| Feature | Types | Description |
|---------|-------|-------------|
| `raw_conversions` | `Raw<T>`, `RawSlice<T>` | POD types ↔ raw vectors via bytemuck |

### Project-wide Defaults (mutually exclusive where noted)

| Feature | Description |
|---------|-------------|
| `r6-default` | Default class system: R6 (mutually exclusive with `s7-default`) |
| `s7-default` | Default class system: S7 (mutually exclusive with `r6-default`) |
| `worker-default` | Default to worker thread dispatch (implies `worker-thread`) |
| `strict-default` | Default to strict mode for lossy integer conversions |
| `coerce-default` | Default to coerce mode for type conversions |

### Development / Diagnostics

| Feature | Description |
|---------|-------------|
| `doc-lint` | Warn on roxygen doc comment mismatches (enabled by default) |
| `macro-coverage` | Expose macro coverage test module for `cargo expand` auditing |
| `growth-debug` | Track and report collection growth events (zero-cost when off) |

---

## Structs

### `abi::mx_base_vtable`

Base vtable present in all erased objects.

This vtable provides the minimal operations needed for any erased object:
- Destructor for cleanup when R garbage collects the wrapper
- Concrete type tag for type-safe downcasts
- Query function to retrieve interface vtables

#### Layout Guarantee

This type is `#[repr(C)]` and its layout is frozen. Fields will never
be reordered, and new fields will only be appended at the end.

#### Generated By

`#[derive(ExternalPtr)]` emits a static instance of this vtable for each
wrapped type.

**Fields:**

- `drop`: `{'function_pointer': {'sig': {'inputs': [['ptr', {'raw_pointer': {'is_mutable': True, 'type': {'resolved_path': {'path': 'mx_erased', 'id': 5200, 'args': None}}}}]], 'output': None, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': False}}}}}`
  - Destructor called when the R external pointer is garbage collected.
- `concrete_tag`: `mx_tag`
  - Tag identifying the concrete type wrapped by this object.
- `query`: `{'function_pointer': {'sig': {'inputs': [['ptr', {'raw_pointer': {'is_mutable': True, 'type': {'resolved_path': {'path': 'mx_erased', 'id': 5200, 'args': None}}}}], ['trait_tag', {'resolved_path': {'path': 'mx_tag', 'id': 5146, 'args': None}}]], 'output': {'raw_pointer': {'is_mutable': False, 'type': {'resolved_path': {'path': 'std::os::raw::c_void', 'id': 4026, 'args': None}}}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': False}}}}}`
  - Query function to retrieve interface vtables.
- `data_offset`: `usize`
  - Byte offset from the start of the wrapper struct to the `data` field.

### `abi::mx_erased`

Type-erased object header.

This is the common prefix of all erased objects, providing access to
the base vtable. The actual data follows this header in memory.

#### Memory Layout

```text
┌─────────────────────────────────────┐
│ mx_erased                           │
│   base: *const mx_base_vtable ──────┼──► static vtable
├─────────────────────────────────────┤
│ (type-specific data follows...)     │
│   data: T                           │
│   interface_views: [...]            │
└─────────────────────────────────────┘
```

#### Layout Guarantee

This type is `#[repr(C)]` and its layout is frozen. The `base` field
will always be at offset 0, and new fields will only be appended.

#### Generated By

`#[derive(ExternalPtr)]` generates wrapper structs that place `mx_erased`
as the first field for proper layout.

**Fields:**

- `base`: `*const mx_base_vtable`
  - Pointer to the base vtable.

### `abi::mx_tag`

Type tag for runtime type identification.

A 128-bit identifier split into two 64-bit halves for C compatibility.
Used to identify concrete types and trait interfaces at runtime.

#### Generation

Tags should be generated as compile-time constants, typically using
a hash of the fully-qualified type/trait path. The `#[miniextendr]`
attribute macro handles this automatically.

#### Comparison

Tags are compared by value equality of both `lo` and `hi` fields.

#### Layout Guarantee

This type is `#[repr(C)]` and its layout is frozen. Fields will never
be reordered, and new fields will only be appended.

**Fields:**

- `lo`: `u64`
  - Lower 64 bits of the type tag.
- `hi`: `u64`
  - Upper 64 bits of the type tag.

**Methods:**

#### `new`

```rust
const new(lo: u64, hi: u64) -> Self
```

Create a new type tag from two 64-bit values.

##### Arguments

* `lo` - Lower 64 bits
* `hi` - Upper 64 bits

##### Example

```ignore
const MY_TAG: mx_tag = mx_tag::new(0x1234_5678_9abc_def0, 0xfed_cba9_8765_4321);
```

### `adapter_traits::RCloneView`

Runtime view for objects implementing `RClone`.
Generated from source location line 368, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RCloneVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::RCopyView`

Runtime view for objects implementing `RCopy`.
Generated from source location line 454, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RCopyVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `is_copy`

```rust
is_copy(self: &Self) -> bool
```

Call `is_copy` through the vtable.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::RDebugView`

Runtime view for objects implementing `RDebug`.
Generated from source location line 55, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RDebugVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `debug_str`

```rust
debug_str(self: &Self) -> String
```

Call `debug_str` through the vtable.

#### `debug_str_pretty`

```rust
debug_str_pretty(self: &Self) -> String
```

Call `debug_str_pretty` through the vtable.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::RDefaultView`

Runtime view for objects implementing `RDefault`.
Generated from source location line 407, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RDefaultVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::RDisplayView`

Runtime view for objects implementing `RDisplay`.
Generated from source location line 97, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RDisplayVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `as_r_string`

```rust
as_r_string(self: &Self) -> String
```

Call `as_r_string` through the vtable.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::RErrorView`

Runtime view for objects implementing `RError`.
Generated from source location line 264, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RErrorVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `error_chain`

```rust
error_chain(self: &Self) -> Vec<String>
```

Call `error_chain` through the vtable.

#### `error_chain_length`

```rust
error_chain_length(self: &Self) -> i32
```

Call `error_chain_length` through the vtable.

#### `error_message`

```rust
error_message(self: &Self) -> String
```

Call `error_message` through the vtable.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::RFromStrView`

Runtime view for objects implementing `RFromStr`.
Generated from source location line 328, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RFromStrVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::RHashView`

Runtime view for objects implementing `RHash`.
Generated from source location line 132, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RHashVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `hash`

```rust
hash(self: &Self) -> i64
```

Call `hash` through the vtable.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::RIteratorView`

Runtime view for objects implementing `RIterator`.
Generated from source location line 540, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RIteratorVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `count`

```rust
count(self: &Self) -> i64
```

Call `count` through the vtable.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `skip`

```rust
skip(self: &Self, n: i32) -> i32
```

Call `skip` through the vtable.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::ROrdView`

Runtime view for objects implementing `ROrd`.
Generated from source location line 164, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const ROrdVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `adapter_traits::RPartialOrdView`

Runtime view for objects implementing `RPartialOrd`.
Generated from source location line 204, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

 Combines a data pointer with a vtable pointer for method dispatch.
 Use `try_from_sexp` to create a view from an R external pointer.

**Fields:**

- `data`: `*mut ::std::os::raw::c_void`
  - Pointer to the concrete object data.
- `vtable`: `*const RPartialOrdVTable`
  - Pointer to the vtable for this trait.

**Methods:**

#### `from_sexp`

```rust
unsafe from_sexp(sexp: ::miniextendr_api::SEXP) -> Self
```

Try to create a view, panicking with error message on failure.

##### Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

##### Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `allocator::RAllocator`

R-backed global allocator.

All allocations are backed by R RAWSXP objects and protected from
garbage collection. The allocator stores metadata before the returned
pointer to enable proper deallocation.

**Note:** This should NOT be used as `#[global_allocator]` in R package
library crates, as it would be invoked during compilation/build time when
R isn't available. Instead, use it explicitly in standalone binaries that
embed R, or use arena-style allocation APIs.

#### Thread Safety

This allocator is safe to use from any thread. R API calls are automatically
routed to the main thread via `with_r_thread_or_inline`.

### `altrep_data::iter::coerce::IterComplexData`

Iterator-backed complex number vector.

Wraps an iterator producing `Rcomplex` values and exposes it as an ALTREP complex vector.

#### Example

```ignore
use miniextendr_api::altrep_data::IterComplexData;
use miniextendr_api::Rcomplex;

let iter = (0..5).map(|x| Rcomplex { r: x as f64, i: (x * 2) as f64 });
let data = IterComplexData::from_iter(iter, 5);
```

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::coerce::IterIntCoerceData`

Iterator-backed integer vector with coercion from any integer-like type.

Wraps an iterator producing values that coerce to `i32` (e.g., `u16`, `i8`, etc.).

#### Example

```ignore
use miniextendr_api::altrep_data::IterIntCoerceData;

// Create from an iterator of u16 values
let iter = (0..10u16).map(|x| x * 100);
let data = IterIntCoerceData::from_iter(iter, 10);
// Values are coerced from u16 to i32 when accessed
```

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::coerce::IterIntFromBoolData`

Iterator-backed integer vector with coercion from bool.

Wraps an iterator producing `bool` values that coerce to `i32`.
Useful for converting boolean iterators to integer vectors.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::coerce::IterListData`

Iterator-backed list vector.

Wraps an iterator producing R `SEXP` values and exposes it as an ALTREP list.

#### Safety

The iterator must produce valid, protected SEXP values. Each SEXP must remain
protected for the lifetime of the ALTREP object.

#### Example

```ignore
use miniextendr_api::altrep_data::IterListData;
use miniextendr_api::IntoR;

let iter = (0..5).map(|x| vec![x, x+1, x+2].into_sexp());
let data = IterListData::from_iter(iter, 5);
```

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

##### Safety

The iterator must produce valid, protected SEXP values.

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

##### Safety

The iterator must produce valid, protected SEXP values.

### `altrep_data::iter::coerce::IterRealCoerceData`

Iterator-backed real vector with coercion from any float-like type.

Wraps an iterator producing values that coerce to `f64` (e.g., `f32`, integer types).

#### Example

```ignore
use miniextendr_api::altrep_data::IterRealCoerceData;

// Create from an iterator of f32 values
let iter = (0..5).map(|x| x as f32 * 1.5);
let data = IterRealCoerceData::from_iter(iter, 5);
// Values are coerced from f32 to f64 when accessed
```

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::coerce::IterStringData`

Iterator-backed string vector.

Wraps an iterator producing `String` values and exposes it as an ALTREP character vector.

#### Performance Warning

Unlike other `Iter*Data` types, **accessing ANY element forces full materialization
of the entire iterator**. This is because R's `AltStringData::elt()` returns a borrowed
`&str`, which requires stable storage. The internal `RefCell` cannot provide the required
lifetime, so all strings must be materialized upfront.

This means:
- `elt(0)` on a million-element iterator will generate ALL million strings
- There is no lazy evaluation benefit for string iterators
- Memory usage equals the full vector regardless of access patterns

For truly lazy string ALTREP, consider implementing a custom type that stores
strings in a way that allows borrowing without full materialization (e.g., arena
allocation or caching generated strings incrementally).

#### Example

```ignore
use miniextendr_api::altrep_data::IterStringData;

let iter = (0..5).map(|x| format!("item_{}", x));
let data = IterStringData::from_iter(iter, 5);
// First access to ANY element will materialize all 5 strings
```

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::sparse::SparseIterComplexData`

Sparse iterator-backed complex number vector.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::sparse::SparseIterIntData`

Sparse iterator-backed integer vector data.

Uses `Iterator::nth()` to skip directly to requested indices.
Only accessed elements are cached; skipped elements return `NA_INTEGER`.

#### Example

```ignore
use miniextendr_api::altrep_data::SparseIterIntData;

// Access only specific elements from a large range
let data = SparseIterIntData::from_iter(0..1_000_000, 1_000_000);
let elem = data.elt(500_000);  // Skips 0..499_999
```

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::sparse::SparseIterLogicalData`

Sparse iterator-backed logical vector data.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::sparse::SparseIterRawData`

Sparse iterator-backed raw (u8) vector data.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::sparse::SparseIterRealData`

Sparse iterator-backed real (f64) vector data.

Uses `Iterator::nth()` to skip directly to requested indices.
Only accessed elements are cached; skipped elements return `NaN`.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::sparse::SparseIterState`

Core state for sparse iterator-backed ALTREP vectors.

Unlike [`super::IterState`], this variant uses `Iterator::nth()` to skip elements
efficiently, only caching the elements that are actually accessed.

#### Type Parameters

- `I`: The iterator type
- `T`: The element type produced by the iterator

#### Design

- **Sparse:** Only accessed elements are cached (uses `BTreeMap`)
- **Skipping:** Uses `nth()` to skip directly to requested indices
- **Trade-off:** Skipped elements are gone forever (iterator is consumed)
- **Best for:** Large iterators where only a small subset of elements are accessed

#### Comparison with `IterState`

| Feature | `IterState` | `SparseIterState` |
|---------|-------------|-------------------|
| Cache storage | Contiguous `Vec<T>` | Sparse `BTreeMap<usize, T>` |
| Access pattern | Prefix (0..=i) cached | Only accessed indices cached |
| Skipped elements | All cached | Gone forever (return NA) |
| Memory for sparse access | O(max_index) | O(num_accessed) |
| `as_slice()` support | Yes (after full materialization) | No (sparse) |

#### Example

```ignore
use miniextendr_api::altrep_data::SparseIterIntData;

// Create from an infinite-ish iterator
let data = SparseIterIntData::from_iter((0..).map(|x| x * 2), 1_000_000);

// Access only element 999_999 - skips directly there
let last = data.elt(999_999);  // Only this element is generated

// Element 0 was skipped and is now inaccessible
let first = data.elt(0);  // Returns NA_INTEGER
```

**Methods:**

#### `cached_count`

```rust
cached_count(self: &Self) -> usize
```

Get the number of cached elements.

#### `from_exact_size`

```rust
from_exact_size(iter: I) -> Self
```

Create a new sparse iterator state from an `ExactSizeIterator`.

#### `get_element`

```rust
get_element(self: &Self, i: usize) -> Option<T>
```

Get an element, skipping intermediate elements if needed.

Uses `Iterator::nth()` to skip efficiently. Skipped elements are
consumed from the iterator and cannot be retrieved later.

##### Returns

- `Some(T)` if element exists and is accessible
- `None` if:
  - Index is out of bounds
  - Element was already skipped (iterator advanced past it)
  - Iterator exhausted before reaching the index

#### `is_cached`

```rust
is_cached(self: &Self, i: usize) -> bool
```

Check if an element has been cached.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if the vector is empty.

#### `iterator_position`

```rust
iterator_position(self: &Self) -> Option<usize>
```

Get the current iterator position (next index to be produced).

Returns `None` if the iterator has been exhausted.

#### `len`

```rust
len(self: &Self) -> usize
```

Get the current length.

#### `new`

```rust
new(iter: I, len: usize) -> Self
```

Create a new sparse iterator state with an explicit length.

##### Arguments

- `iter`: The iterator to wrap
- `len`: The expected number of elements

### `altrep_data::iter::state::IterIntData`

Iterator-backed integer vector data.

Wraps an iterator producing `i32` values and exposes it as an ALTREP integer vector.

#### Example

```ignore
use miniextendr_api::altrep_data::IterIntData;

// Create from an iterator
let data = IterIntData::from_iter((1..=10).map(|x| x * 2), 10);
```

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::state::IterLogicalData`

Iterator-backed logical vector data.

Wraps an iterator producing `bool` values and exposes it as an ALTREP logical vector.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::state::IterRawData`

Iterator-backed raw (u8) vector data.

Wraps an iterator producing `u8` values and exposes it as an ALTREP raw vector.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::state::IterRealData`

Iterator-backed real (f64) vector data.

Wraps an iterator producing `f64` values and exposes it as an ALTREP real vector.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I) -> Self
```

Create from an ExactSizeIterator (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

### `altrep_data::iter::state::IterState`

Core state for iterator-backed ALTREP vectors.

Provides lazy element generation with caching for random-access semantics.
Iterator elements are cached as they're accessed, enabling repeatable reads.

#### Type Parameters

- `I`: The iterator type (must be `ExactSizeIterator` or provide explicit length)
- `T`: The element type produced by the iterator

#### Design

- **Lazy:** Elements generated on-demand via `elt(i)`
- **Cached:** Once generated, elements stored in cache for repeat access
- **Materializable:** Can be fully materialized for `Dataptr` or serialization
- **Safe:** Uses `RefCell` for interior mutability, protected by R's GC

**Methods:**

#### `as_materialized`

```rust
as_materialized(self: &Self) -> Option<&[T]>
```

Get the materialized vector if all elements have been generated.

Returns `None` if not yet fully materialized.

#### `from_exact_size`

```rust
from_exact_size(iter: I) -> Self
```

Create a new iterator state from an `ExactSizeIterator`.

The length is automatically determined from `iter.len()`.

#### `get_element`

```rust
get_element(self: &Self, i: usize) -> Option<T>
```

Ensure the element at index `i` is in the cache and return it by value.

Advances the iterator as needed. Only works for `Copy` types.

##### Returns

- `Some(T)` if element exists and has been generated
- `None` if index is out of bounds or iterator exhausted before reaching index `i`

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if the vector is empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Get the current length.

#### `materialize_all`

```rust
materialize_all(self: &Self) -> &[T]
```

Materialize all remaining elements from the iterator.

After this call, all elements are guaranteed to be in memory and
`as_materialized()` will return `Some`.

##### Length Mismatch Handling

If the iterator produces fewer elements than declared `len`, the missing
elements are left uninitialized in the cache (callers should handle this
via bounds checking). If the iterator produces more elements than declared,
extra elements are silently ignored (truncated to `len`).

A warning is printed to stderr if a length mismatch is detected.

#### `new`

```rust
new(iter: I, len: usize) -> Self
```

Create a new iterator state with an explicit length.

##### Arguments

- `iter`: The iterator to wrap
- `len`: The expected number of elements

##### Length Mismatch

If the iterator produces a different number of elements than `len`:
- Fewer elements: Missing indices return `None`/NA/default values
- More elements: Extra elements are ignored (truncated to `len`)

A warning is printed to stderr when a mismatch is detected.

### `altrep_data::iter::windowed::WindowedIterIntData`

Windowed iterator-backed integer vector data.

Like [`super::IterIntData`], but only keeps a sliding window of elements in memory.
Sequential forward access within the window is O(1). Access outside the
window triggers full materialization.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I, window_size: usize) -> Self
```

Create from an ExactSizeIterator with window size (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize, window_size: usize) -> Self
```

Create from an iterator with explicit length and window size.

### `altrep_data::iter::windowed::WindowedIterRealData`

Windowed iterator-backed real (f64) vector data.

Like [`super::IterRealData`], but only keeps a sliding window of elements in memory.
Sequential forward access within the window is O(1). Access outside the
window triggers full materialization.

**Methods:**

#### `from_exact_iter`

```rust
from_exact_iter(iter: I, window_size: usize) -> Self
```

Create from an ExactSizeIterator with window size (length auto-detected).

#### `from_iter`

```rust
from_iter(iter: I, len: usize, window_size: usize) -> Self
```

Create from an iterator with explicit length and window size.

### `altrep_data::iter::windowed::WindowedIterState`

Core state for windowed iterator-backed ALTREP vectors.

Like [`super::IterState`], but only keeps a sliding window of elements in memory.
Sequential access within the window is O(1). Access outside the window
materializes the entire vector (falling back to full caching).

This is useful for large iterators where only a small region is accessed
at a time (e.g., streaming data processed in order).

#### Type Parameters

- `I`: The iterator type
- `T`: The element type produced by the iterator

**Methods:**

#### `as_materialized`

```rust
as_materialized(self: &Self) -> Option<&[T]>
```

Get materialized slice if available.

#### `from_exact_size`

```rust
from_exact_size(iter: I, window_size: usize) -> Self
```

Create from an ExactSizeIterator.

#### `get_element`

```rust
get_element(self: &Self, i: usize) -> Option<T>
```

Get element at index `i`.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Get the length.

#### `materialize_all`

```rust
materialize_all(self: &Self) -> &[T]
```

Materialize all elements.

#### `new`

```rust
new(iter: I, len: usize, window_size: usize) -> Self
```

Create a new windowed iterator state.

### `altrep_data::stream::StreamingIntData`

Streaming ALTREP for integer (i32) vectors.

Elements are loaded on-demand via a reader closure in fixed-size chunks.
Chunks are cached in a `BTreeMap` for repeated access.

#### Reader Contract

The reader `F(start, buf) -> count` fills `buf` with elements starting
at index `start` and returns the number of elements actually written.

#### Example

```ignore
use miniextendr_api::altrep_data::StreamingIntData;

let data = StreamingIntData::new(1000, 64, |start, buf| {
    let count = buf.len().min(1000 - start);
    for (i, slot) in buf[..count].iter_mut().enumerate() {
        *slot = (start + i) as i32;
    }
    count
});
```

**Methods:**

#### `new`

```rust
new(len: usize, chunk_size: usize, reader: F) -> Self
```

Create a new streaming integer data source.

- `len`: total number of elements
- `chunk_size`: number of elements per cache chunk
- `reader`: closure that fills a buffer starting at a given index

### `altrep_data::stream::StreamingRealData`

Streaming ALTREP for real (f64) vectors.

Elements are loaded on-demand via a reader closure in fixed-size chunks.
Chunks are cached in a `BTreeMap` for repeated access.

#### Reader Contract

The reader `F(start, buf) -> count` fills `buf` with elements starting
at index `start` and returns the number of elements actually written.

#### Example

```ignore
use miniextendr_api::altrep_data::StreamingRealData;

let data = StreamingRealData::new(1000, 64, |start, buf| {
    let count = buf.len().min(1000 - start);
    for (i, slot) in buf[..count].iter_mut().enumerate() {
        *slot = (start + i) as f64 * 0.1;
    }
    count
});
```

**Methods:**

#### `new`

```rust
new(len: usize, chunk_size: usize, reader: F) -> Self
```

Create a new streaming real data source.

- `len`: total number of elements
- `chunk_size`: number of elements per cache chunk
- `reader`: closure that fills a buffer starting at a given index

### `altrep_sexp::AltrepSexp`

A SEXP known to be ALTREP. `!Send + !Sync` — must be materialized on the
R main thread before data can be accessed or sent to other threads.

This type prevents ALTREP vectors from being accidentally sent to rayon
or other worker threads where `DATAPTR_RO` would invoke R internals
(undefined behavior).

#### As a `#[miniextendr]` parameter

`AltrepSexp` implements [`TryFromSexp`](crate::from_r::TryFromSexp), so it
can be used directly as a function parameter. It **only accepts ALTREP
vectors** — non-ALTREP input produces an error.

```ignore
#[miniextendr]
pub fn altrep_info(x: AltrepSexp) -> String {
    format!("{:?}, len={}", x.sexptype(), x.len())
}
```

```r
altrep_info(1:10)          # OK — 1:10 is ALTREP
altrep_info(c(1L, 2L, 3L)) # Error: "expected an ALTREP vector"
```

#### Construction

- [`AltrepSexp::try_wrap`] — runtime check, returns `None` if not ALTREP
- [`AltrepSexp::from_raw`] — unsafe, caller asserts `ALTREP(sexp) != 0`

#### Materialization

All materialization methods must be called on the R main thread.

- [`AltrepSexp::materialize`] — forces R to materialize, returns plain SEXP
- [`AltrepSexp::materialize_integer`] — materialize INTSXP and return `&[i32]`
- [`AltrepSexp::materialize_real`] — materialize REALSXP and return `&[f64]`
- [`AltrepSexp::materialize_logical`] — materialize LGLSXP and return `&[i32]`
- [`AltrepSexp::materialize_raw`] — materialize RAWSXP and return `&[u8]`
- [`AltrepSexp::materialize_complex`] — materialize CPLXSXP and return `&[Rcomplex]`
- [`AltrepSexp::materialize_strings`] — materialize STRSXP to `Vec<Option<String>>`

#### Thread safety

`AltrepSexp` is `!Send + !Sync` (via `PhantomData<Rc<()>>`). This is a
compile-time guarantee: you cannot send an un-materialized ALTREP vector
to another thread. Call one of the `materialize_*` methods first to get
a `Send + Sync` slice or SEXP.

**Methods:**

#### `as_raw`

```rust
unsafe as_raw(self: &Self) -> SEXP
```

Get the inner SEXP without materializing.

##### Safety

The returned SEXP is still ALTREP. Do not call `DATAPTR_RO` on it
from a non-R thread.

#### `from_raw`

```rust
unsafe from_raw(sexp: SEXP) -> Self
```

Wrap a SEXP that is known to be ALTREP.

##### Safety

Caller must ensure `ALTREP(sexp)` is true (non-zero).

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if the underlying vector is empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Get the length of the underlying vector.

#### `materialize`

```rust
unsafe materialize(self: Self) -> SEXP
```

Force materialization and return the (now materialized) SEXP.

For contiguous types (INTSXP, REALSXP, LGLSXP, RAWSXP, CPLXSXP),
calls `DATAPTR_RO` to trigger ALTREP materialization.
For STRSXP, iterates `STRING_ELT` to force element materialization.

After this call, the SEXP's data pointer is stable and can be safely
accessed from any thread (the SEXP itself is still `Send + Sync`).

##### Safety

Must be called on the R main thread.

#### `materialize_complex`

```rust
unsafe materialize_complex(self: &Self) -> &[Rcomplex]
```

Materialize and return a typed slice of `Rcomplex` (CPLXSXP).

##### Safety

Must be called on the R main thread. The SEXP must be CPLXSXP.

#### `materialize_integer`

```rust
unsafe materialize_integer(self: &Self) -> &[i32]
```

Materialize and return a typed slice of `i32` (INTSXP).

##### Safety

Must be called on the R main thread. The SEXP must be INTSXP.

#### `materialize_logical`

```rust
unsafe materialize_logical(self: &Self) -> &[i32]
```

Materialize and return a typed slice of `i32` (LGLSXP, R's internal logical storage).

##### Safety

Must be called on the R main thread. The SEXP must be LGLSXP.

#### `materialize_raw`

```rust
unsafe materialize_raw(self: &Self) -> &[u8]
```

Materialize and return a typed slice of `u8` (RAWSXP).

##### Safety

Must be called on the R main thread. The SEXP must be RAWSXP.

#### `materialize_real`

```rust
unsafe materialize_real(self: &Self) -> &[f64]
```

Materialize and return a typed slice of `f64` (REALSXP).

##### Safety

Must be called on the R main thread. The SEXP must be REALSXP.

#### `materialize_strings`

```rust
unsafe materialize_strings(self: &Self) -> Vec<Option<String>>
```

Materialize strings into owned Rust data.

Each element is `None` for `NA_character_`, or `Some(String)` otherwise.

##### Safety

Must be called on the R main thread. The SEXP must be STRSXP.

#### `sexptype`

```rust
sexptype(self: &Self) -> SEXPTYPE
```

Get the SEXPTYPE of the underlying vector.

#### `try_wrap`

```rust
try_wrap(sexp: SEXP) -> Option<Self>
```

Check a SEXP and wrap if ALTREP. Returns `None` if not ALTREP.

### `coerce::Coerced`

Wrapper for values coerced from an R native type during conversion.

This enables using non-native Rust types in collections read from R:

```ignore
// Read a Vec of i64 from R integers (i32)
let vec: Vec<Coerced<i64, i32>> = TryFromSexp::try_from_sexp(sexp)?;

// Extract the values
let i64_vec: Vec<i64> = vec.into_iter().map(Coerced::into_inner).collect();
```

The type parameters are:
- `T`: The target Rust type you want
- `R`: The R-native type to read and coerce from

**Methods:**

#### `as_inner`

```rust
const as_inner(self: &Self) -> &T
```

Get a reference to the inner value.

#### `as_inner_mut`

```rust
as_inner_mut(self: &mut Self) -> &mut T
```

Get a mutable reference to the inner value.

#### `into_inner`

```rust
into_inner(self: Self) -> T
```

Extract the inner value.

#### `new`

```rust
const new(value: T) -> Self
```

Create a new Coerced wrapper.

### `condition::AsRError`

Structured error wrapper that preserves the `std::error::Error` cause chain.

When displayed, formats the error message with its full source chain:
```text
top-level message
  caused by: middle error
  caused by: root cause
```

Implements `From<E>` so it works with `?` and `.map_err(AsRError)`.

#### Example

```ignore
use miniextendr_api::condition::AsRError;
use std::num::ParseIntError;

#[miniextendr]
fn parse_number(s: &str) -> Result<i32, AsRError<ParseIntError>> {
    s.parse::<i32>().map_err(AsRError)
}
```

**Methods:**

#### `cause_chain`

```rust
cause_chain(self: &Self) -> Vec<String>
```

Collect the full cause chain as a `Vec<String>`.

#### `into_inner`

```rust
into_inner(self: Self) -> E
```

Get the inner error.

#### `rust_type_name`

```rust
rust_type_name(self: &Self) -> &'static str
```

Get the Rust type name of the wrapped error (for programmatic matching).

### `convert::AsDataFrame`

Wrap a value and convert it to an R `data.frame` via [`IntoDataFrame`](crate::dataframe::IntoDataFrame) when returned.

Use this at a call site to force a single return value into a data.frame without making
that the type's default representation (for the always-a-data.frame default, use
`#[derive(PreferDataFrame)]` / `#[miniextendr(dataframe)]`). The inner `T` is typically a
`Vec<Row>` where `Row` derives [`DataFrameRow`](crate::markers::DataFrameRow).

A failed conversion ([`DataFrameError`](crate::dataframe::DataFrameError)) surfaces in R as
an error condition.

#### Example

```ignore
#[derive(DataFrameRow)]
struct Point { x: f64, y: f64 }

#[miniextendr]
fn grid() -> AsDataFrame<Vec<Point>> {
    AsDataFrame(vec![Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }])
}
// In R: grid() returns a data.frame with columns x, y
```

### `convert::AsDisplay`

Wrap a `T: Display` and convert it to an R character scalar.

Any type implementing `std::fmt::Display` can be returned to R as a string
without implementing miniextendr traits.

#### Example

```ignore
use std::net::IpAddr;

#[miniextendr]
fn format_ip(ip: &str) -> AsDisplay<IpAddr> {
    AsDisplay(ip.parse().unwrap())
}
// R gets: "192.168.1.1"
```

### `convert::AsDisplayVec`

Wrap a `Vec<T: Display>` and convert it to an R character vector.

#### Example

```ignore
#[miniextendr]
fn format_errors(errors: Vec<std::io::Error>) -> AsDisplayVec<std::io::Error> {
    AsDisplayVec(errors)
}
```

### `convert::AsExternalPtr`

Wrap a value and convert it to an R external pointer when returned from Rust.

Use this wrapper when you want to return a Rust value as an opaque pointer
that R code can pass back to Rust functions later.

#### Example

```ignore
struct Connection { handle: u64 }

impl IntoExternalPtr for Connection { /* ... */ }

#[miniextendr]
fn open_connection(path: &str) -> AsExternalPtr<Connection> {
    AsExternalPtr(Connection { handle: 42 })
}
// In R: open_connection("foo") returns an external pointer
```

### `convert::AsFromStr`

Wrap a parsed `T: FromStr` from an R character scalar.

Pass an R character scalar and it will be parsed into `T` via `str::parse()`.

#### Example

```ignore
use std::net::IpAddr;

#[miniextendr]
fn check_ip(addr: AsFromStr<IpAddr>) -> bool {
    addr.0.is_loopback()
}
// R: check_ip("127.0.0.1") → TRUE
```

### `convert::AsFromStrVec`

Wrap a `Vec<T: FromStr>` parsed from an R character vector.

Each element of the R character vector is parsed into `T`.
All parse errors are collected with their indices.

#### Example

```ignore
use std::net::IpAddr;

#[miniextendr]
fn parse_ips(addrs: AsFromStrVec<IpAddr>) -> Vec<bool> {
    addrs.0.into_iter().map(|ip| ip.is_loopback()).collect()
}
// R: parse_ips(c("127.0.0.1", "8.8.8.8")) → c(TRUE, FALSE)
```

### `convert::AsList`

Wrap a value and convert it to an R list via [`IntoList`] when returned from Rust.

Use this wrapper when you want to convert a single value to an R list without
making that the default behavior for the type.

#### Example

```ignore
#[derive(IntoList)]
struct Point { x: f64, y: f64 }

#[miniextendr]
fn make_point() -> AsList<Point> {
    AsList(Point { x: 1.0, y: 2.0 })
}
// In R: make_point() returns list(x = 1.0, y = 2.0)
```

### `convert::AsNamedList`

Wrap a tuple pair collection and convert it to a **named R list** (VECSXP).

Preserves insertion order and allows duplicate names (sequence semantics).

#### Supported input types

| Input | Bounds |
|-------|--------|
| `Vec<(K, V)>` | `K: AsRef<str>`, `V: IntoR` |
| `[(K, V); N]` | `K: AsRef<str>`, `V: IntoR` |
| `&[(K, V)]` | `K: AsRef<str>`, `V: Clone + IntoR` |

#### Example

```ignore
#[miniextendr]
fn make_config() -> AsNamedList<Vec<(String, i32)>> {
    AsNamedList(vec![
        ("width".into(), 100),
        ("height".into(), 200),
    ])
}
// In R: make_config() returns list(width = 100L, height = 200L)
```

### `convert::AsNamedVector`

Wrap a tuple pair collection and convert it to a **named atomic R vector**
(INTSXP, REALSXP, LGLSXP, RAWSXP, or STRSXP).

Preserves insertion order and allows duplicate names (sequence semantics).
Values must be homogeneous and implement [`AtomicElement`].

#### Supported input types

| Input | Bounds |
|-------|--------|
| `Vec<(K, V)>` | `K: AsRef<str>`, `V: AtomicElement` |
| `[(K, V); N]` | `K: AsRef<str>`, `V: AtomicElement` |
| `&[(K, V)]` | `K: AsRef<str>`, `V: Clone + AtomicElement` |

#### Example

```ignore
#[miniextendr]
fn make_scores() -> AsNamedVector<Vec<(&str, f64)>> {
    AsNamedVector(vec![("alice", 95.0), ("bob", 87.5)])
}
// In R: make_scores() returns c(alice = 95.0, bob = 87.5)
```

### `convert::AsRNative`

Wrap a scalar [`RNativeType`] and force native R vector conversion.

This creates a length-1 R vector containing the scalar value. Use this when
you want to ensure a value is converted to its native R representation (e.g.,
`i32` → integer vector, `f64` → numeric vector) rather than another path
like `IntoExternalPtr`.

#### Example

```ignore
#[derive(Clone, Copy, RNativeType)]
struct Meters(f64);

#[miniextendr]
fn distance() -> AsRNative<Meters> {
    AsRNative(Meters(42.5))
}
// In R: distance() returns 42.5 (numeric vector of length 1)
```

#### Performance

This wrapper directly allocates an R vector and writes the value,
avoiding intermediate Rust allocations.

### `convert::AsVctrs`

Wrap a value and convert it to a **vctrs** S3 vector via [`IntoVctrs`](crate::vctrs::IntoVctrs)
when returned.

Use this at a call site to return a `#[derive(Vctrs)]` type as its R vctrs object without the
manual `value.into_vctrs().map_err(...)` boilerplate. For a type that should *always* convert
this way, use `#[derive(Vctrs, PreferVctrs)]` instead.

A failed build ([`VctrsBuildError`](crate::vctrs::VctrsBuildError)) surfaces in R as an error
condition.

#### Example

```ignore
#[derive(Vctrs)]
#[vctrs(class = "percent", base = "double")]
struct Percent { #[vctrs(data)] values: Vec<f64> }

#[miniextendr]
fn percent(x: Vec<f64>) -> AsVctrs<Percent> {
    AsVctrs(Percent { values: x })
}
```

### `convert::Collect`

Write an `ExactSizeIterator` of native R types directly into an R vector.

Skips the intermediate `Vec` allocation — the R vector is allocated once
and the iterator writes directly into it.

Requires `ExactSizeIterator` because R vectors must know their length
at allocation time.

#### Naming

`Collect` is in the representation-forcing wrapper family but does not take the
`As*` prefix used by [`AsList`] / [`AsExternalPtr`] / [`AsRNative`]: those wrap a
finished value `T`, whereas `Collect` wraps an *iterator* and materializes it into
an R vector. The divergence is intentional — see the module docs and #871.

#### Example

```ignore
#[miniextendr]
fn sines(n: i32) -> Collect<impl ExactSizeIterator<Item = f64>> {
    Collect((0..n).map(|i| (i as f64).sin()))
}
```

### `convert::CollectNA`

Write an `ExactSizeIterator` of `Option<T>` directly into an R vector with NA support.

`None` values become `NA` in R. Works for `f64` and `i32`.

Like [`Collect`], this is an iterator adapter and is exempt from the `As*`
naming convention (see #871).

#### Example

```ignore
#[miniextendr]
fn with_gaps(n: i32) -> CollectNA<impl ExactSizeIterator<Item = Option<f64>>> {
    CollectNA((0..n).map(|i| if i % 3 == 0 { None } else { Some(i as f64) }))
}
```

### `convert::CollectNAInt`

Write an `ExactSizeIterator` of `Option<i32>` directly into an R integer vector with NA.

Like [`Collect`], this is an iterator adapter and is exempt from the `As*`
naming convention (see #871).

### `convert::CollectStrings`

Write an `ExactSizeIterator` of `String` directly into an R character vector.

Strings require per-element CHARSXP allocation (no bulk `copy_from_slice`),
so this is a separate type from [`Collect`]. Like [`Collect`], it is an
iterator adapter and is exempt from the `As*` naming convention (see #871).

#### Example

```ignore
#[miniextendr]
fn upper(words: Vec<String>) -> CollectStrings<impl ExactSizeIterator<Item = String>> {
    CollectStrings(words.into_iter().map(|w| w.to_uppercase()))
}
```

### `dataframe::DataFrame`

An owned, validated R `data.frame`. **The** data-frame type.

Wraps a built VECSXP carrying the `data.frame` class + `row.names`. A single coherent
type for building (Rust → R), reading (R → Rust), and post-assembly editing — replacing
the historical row-buffer / built-SEXP / read-wrapper trio with one coherent type.

#### Building

Prefer the [`IntoDataFrame`] trait on your data:

```ignore
let df: DataFrame = rows.into_dataframe()?;
```

or the closure-fill [`DataFrame::builder`] for heterogeneous parallel column fill
(`feature = "rayon"`).

#### Reading

Wrap an incoming SEXP with [`DataFrame::from_sexp`] (or accept `DataFrame` directly as a
`#[miniextendr]` argument), then pull typed columns with [`DataFrame::column`], or
deserialize whole rows with [`FromDataFrame`].

**Methods:**

#### `as_list`

```rust
as_list(self: &Self) -> List
```

Get the underlying [`List`].

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `builder`

```rust
builder(nrow: usize) -> crate::rayon_bridge::RDataFrameBuilder
```

Start a closure-per-column parallel-fill builder yielding a [`DataFrame`].

The heterogeneous-column analogue of `with_r_matrix`: each column buffer is R memory
filled in parallel. Only available with `feature = "rayon"`.

```ignore
let df = DataFrame::builder(1000)
    .column::<f64>("x", |chunk, off| for (i, v) in chunk.iter_mut().enumerate() { *v = (off + i) as f64 })
    .column_str("label", |i| Some(format!("row{i}")))
    .build();
```

#### `column`

```rust
column<T>(self: &Self, name: &str) -> Option<T>
```

Get a column by name, converting each element to type `T`.

Returns `None` if the column name is not found or conversion fails.

#### `column_index`

```rust
column_index<T>(self: &Self, idx: usize) -> Option<T>
```

Get a column by 0-based index, converting to type `T`.

#### `column_raw`

```rust
column_raw(self: &Self, name: &str) -> Option<SEXP>
```

Get the raw SEXP for a column by name.

#### `contains_column`

```rust
contains_column(self: &Self, name: &str) -> bool
```

Check whether a column name exists.

#### `drop`

```rust
drop(self: Self, col: &str) -> Self
```

Remove a column by name. No-op if the column doesn't exist.

#### `from_built_sexp`

```rust
unsafe from_built_sexp(sexp: SEXP) -> Self
```

Wrap an already-built `data.frame` SEXP without re-validation.

Used by the column assemblers, which produce a well-formed `data.frame` by
construction.

##### Safety

`sexp` must be a VECSXP with the `data.frame` class and consistent `row.names`.

#### `from_sexp`

```rust
from_sexp(sexp: SEXP) -> Result<Self, DataFrameError>
```

Wrap an existing R `data.frame` SEXP, validating it.

Validates that the object:
1. Is a VECSXP (list)
2. Inherits from `"data.frame"`
3. Has a `names` attribute
4. Has extractable `row.names` for nrow

##### Errors

Returns [`DataFrameError`] if validation fails.

#### `names`

```rust
names(self: &Self) -> Vec<String>
```

Collect column names in column order.

#### `ncol`

```rust
ncol(self: &Self) -> usize
```

Number of columns.

#### `nrow`

```rust
nrow(self: &Self) -> usize
```

Number of rows.

#### `prepend_column`

```rust
prepend_column(self: Self, name: &str, column: SEXP) -> Self
```

Insert a column at index 0 (leftmost), removing any same-named column first.

#### `rename`

```rust
rename(self: Self, from: &str, to: &str) -> Self
```

Rename a column. No-op if `from` doesn't match any column name.

#### `select`

```rust
select(self: Self, cols: &[&str]) -> Self
```

Keep only the named columns, in the order given. Unknown names are skipped.

#### `select_rows`

```rust
select_rows(self: &Self, idx: &[usize]) -> Self
```

Keep only the rows at the given 0-based indices, in order.

Subsets every column (each a vector or list-column) to the specified rows
and rebuilds compact integer `row.names`. Used by the enum reader to
densify a flattened sub-frame before recursing into the inner type's reader.

##### PROTECT discipline

Allocates one new column vector per column — `OwnedProtect`s the output list
across the loop so previously-built column SEXPs survive subsequent allocations.

#### `strip_prefix`

```rust
strip_prefix(self: Self, prefix: &str) -> Self
```

Strip a prefix from all column names that start with it.

#### `validate`

```rust
validate(self: &Self, spec: &TypedListSpec) -> Result<TypedList, TypedListError>
```

Validate the data frame's column types against a [`TypedListSpec`].

#### `with_column`

```rust
with_column(self: Self, name: &str, column: SEXP) -> Self
```

Upsert a column: replace the column named `name` if it exists, else append.

### `dots::Dots`

Rust type representing R's `...` (variadic arguments).

The generated R wrapper captures `...` as `list(...)` and passes it to Rust,
so `Dots` holds a list SEXP. Use [`as_list`](Dots::as_list) or
[`try_list`](Dots::try_list) to access elements by name or position.

Declare as the last parameter: `fn foo(x: i32, _dots: &Dots)`.
Use `name @ ...` syntax for a custom parameter name.

**Fields:**

- `inner`: `crate::SEXP`
  - Raw list backing this `...` capture.

**Methods:**

#### `as_list`

```rust
as_list(self: &Self) -> List
```

Convert to a [`List`] without additional validation.

This is a zero-cost conversion since the R wrapper already passes
`list(...)` to us. Use this when you trust the input or want
maximum performance.

##### Safety Note

This is safe because the miniextendr macro always wraps `...` in
`list(...)` on the R side. However, if you're receiving a SEXP
from another source, use [`try_list`](Dots::try_list) instead.

##### Example
```ignore
#[miniextendr]
pub fn process_dots(dots: ...) -> i32 {
    let list = dots.as_list();
    list.len() as i32
}
```

#### `empty`

```rust
empty() -> Self
```

Create an empty Dots (equivalent to no `...` arguments).

This is useful when calling R functions from Rust that expect
dots arguments but you want to pass nothing.

##### Example
```ignore
#[miniextendr]
pub fn my_constructor(x: Doubles, dots: ...) -> Robj {
    // ...
}

// Call from Rust with empty dots:
let result = my_constructor(data, Dots::empty());
```

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Returns true if no arguments were passed to `...`.

#### `len`

```rust
len(self: &Self) -> isize
```

Get the number of elements in the dots list.

This is equivalent to `dots.as_list().len()` but avoids
creating an intermediate List wrapper.

#### `try_list`

```rust
try_list(self: &Self) -> Result<List, ListFromSexpError>
```

Try to convert to a [`List`] with full validation.

This validates that the underlying SEXP is actually a list and
checks for duplicate names. Use this when you want strict validation
or are working with untrusted input.

##### Errors

Returns [`ListFromSexpError`] if:
- The SEXP is not a list type (VECSXP or pairlist)
- The list contains duplicate non-NA names

##### Example
```ignore
#[miniextendr]
pub fn safe_process_dots(dots: ...) -> Result<i32, String> {
    let list = dots.try_list().map_err(|e| e.to_string())?;
    Ok(list.len() as i32)
}
```

#### `typed`

```rust
typed(self: &Self, spec: TypedListSpec) -> Result<TypedList, TypedListError>
```

Validate the dots against a typed list specification.

This provides structured validation with clear error messages for
functions that expect specific named arguments via `...`.

##### Example

```ignore
use miniextendr_api::typed_list::{TypedListSpec, TypedEntry, TypeSpec};

#[miniextendr]
pub fn process_args(dots: ...) -> Result<i32, String> {
    let spec = TypedListSpec::new(vec![
        TypedEntry::required("alpha", TypeSpec::Numeric(Some(4))),
        TypedEntry::optional("beta", TypeSpec::List(None)),
    ]);

    let validated = dots.typed(spec).map_err(|e| e.to_string())?;
    let alpha: Vec<f64> = validated.get("alpha").map_err(|e| e.to_string())?;
    Ok(alpha.len() as i32)
}
```

##### Errors

Returns [`TypedListError`] if:
- The dots are not a valid list
- A required field is missing
- A field has the wrong type or length
- Extra fields exist when `allow_extra = false`

### `encoding::REncodingInfo`

Cached snapshot of R's encoding / locale state at init time.

### `expression::RCall`

Builder for constructing and evaluating R function calls.

`RCall` constructs a LANGSXP (R language object) from a function name or
SEXP and a sequence of arguments (optionally named). It handles GC
protection during construction and evaluation.

#### Example

```ignore
use miniextendr_api::expression::RCall;
use miniextendr_api::sys;

unsafe {
    // seq_len(10)
    let result = RCall::new("seq_len")
        .arg(SEXP::scalar_integer(10))
        .eval_base()?;

    // paste(x, collapse = ", ")
    let result = RCall::new("paste")
        .arg(some_sexp)
        .named_arg("collapse", sys::Rf_mkString(c", ".as_ptr()))
        .eval_base()?;
}
```

**Methods:**

#### `arg`

```rust
arg(self: Self, value: SEXP) -> Self
```

Add a positional argument.

#### `build`

```rust
unsafe build(self: &Self) -> SEXP
```

Build the LANGSXP without evaluating it.

The returned SEXP is **unprotected**. The caller must protect it if
further allocations will occur before use.

##### Safety

Must be called from the R main thread. All argument SEXPs must still
be valid (protected or otherwise reachable by R's GC).

#### `eval`

```rust
unsafe eval(self: &Self, env: SEXP) -> Result<SEXP, String>
```

Evaluate the call in the given environment.

Uses `R_tryEvalSilent` so that R errors are captured as `Err(String)`
rather than causing a longjmp through Rust frames.

##### Safety

- Must be called from the R main thread.
- `env` must be a valid ENVSXP.
- All argument SEXPs must still be valid.

##### Returns

- `Ok(SEXP)` with the result (unprotected — caller should protect if needed)
- `Err(String)` with the R error message on failure

#### `eval_base`

```rust
unsafe eval_base(self: &Self) -> Result<SEXP, String>
```

Evaluate in `R_BaseEnv`.

##### Safety

Same as [`eval`](Self::eval).

#### `from_cstr`

```rust
unsafe from_cstr(fun_name: &CStr) -> Self
```

Start building a call to a function given as a C string literal.

More efficient than [`new`](Self::new) when a `&CStr` is available.

##### Safety

Must be called from the R main thread.

#### `from_sexp`

```rust
unsafe from_sexp(fun: SEXP) -> Self
```

Start building a call with a function SEXP (closure, builtin, etc.).

##### Safety

`fun` must be a valid SEXP representing a callable R object.

#### `named_arg`

```rust
named_arg(self: Self, name: &str, value: SEXP) -> Self
```

Add a named argument.

##### Panics

Panics if `name` contains a null byte.

#### `new`

```rust
unsafe new(fun_name: &str) -> Self
```

Start building a call to a named R function.

The function is looked up via `Rf_install`, which returns an interned symbol.

##### Safety

Must be called from the R main thread.

##### Panics

Panics if `fun_name` contains a null byte.

### `expression::REnv`

Handle to a well-known R environment.

Provides access to R's standard environments without raw FFI calls.

**Methods:**

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `base`

```rust
unsafe base() -> Self
```

The base environment (`R_BaseEnv`).

##### Safety

Must be called from the R main thread.

#### `base_namespace`

```rust
base_namespace() -> Self
```

The base namespace (`SEXP::base_namespace()`).

Unlike [`base()`](Self::base) which is the base *environment* (exported
functions visible to users), this is the base *namespace* (includes
internal helpers). Rarely needed — prefer [`base()`](Self::base) unless
you specifically need unexported base internals.

##### Safety

Must be called from the R main thread.

#### `caller`

```rust
unsafe caller() -> Self
```

The current execution environment.

Returns the environment of the innermost active closure on R's call
stack, or the global environment if no closure is active.

Useful when you need to evaluate an expression in the caller's context
rather than a fixed well-known environment.

##### Safety

Must be called from the R main thread.

#### `empty`

```rust
unsafe empty() -> Self
```

The empty environment (`R_EmptyEnv`).

##### Safety

Must be called from the R main thread.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: SEXP) -> Self
```

Wrap an arbitrary environment SEXP.

##### Safety

`sexp` must be a valid ENVSXP.

#### `global`

```rust
unsafe global() -> Self
```

The global environment (`R_GlobalEnv`).

##### Safety

Must be called from the R main thread.

#### `package_namespace`

```rust
unsafe package_namespace(name: &str) -> Result<Self, String>
```

A package's namespace environment.

Finds the namespace for a loaded package. Use this to evaluate functions
that live in a specific package (e.g., `slot()` from `methods`).

This is a safe wrapper around `R_FindNamespace` — it uses
`R_tryEvalSilent` internally so that a missing namespace returns
`Err` instead of longjmping through Rust frames.

##### Safety

Must be called from the R main thread.

##### Errors

Returns `Err` if the package namespace is not found (package not loaded).

### `expression::RSymbol`

A safe wrapper around R symbols (SYMSXP).

R symbols are interned strings used as variable and function names.
They are never garbage collected, so `RSymbol` does not need GC protection.

#### Example

```ignore
let sym = RSymbol::new("paste0");
// sym.as_sexp() is a SYMSXP that can be used in call construction
```

**Methods:**

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `from_cstr`

```rust
unsafe from_cstr(name: &CStr) -> Self
```

Create a symbol from a C string literal.

This avoids the allocation needed by [`new`](Self::new) when you have
a `&CStr` available (e.g., from `c"name"` literals).

##### Safety

Must be called from the R main thread.

#### `new`

```rust
unsafe new(name: &str) -> Self
```

Create or retrieve an interned R symbol.

##### Safety

Must be called from the R main thread.

##### Panics

Panics if `name` contains a null byte.

### `externalptr::ExternalPtr`

An owned pointer stored in R's external pointer SEXP.

This is conceptually similar to `Box<T>`, but with the following differences:
- Memory is freed by R's GC via a registered finalizer (non-deterministic)
- The underlying SEXP is Copy, so aliasing must be manually prevented
- Type checking happens at runtime via `Any::downcast` (Rust `TypeId`)

#### Thread Safety

`ExternalPtr` is `Send` to allow returning from worker thread functions.
However, **concurrent access is not allowed** - R's runtime is single-threaded.
All R API calls are serialized through the main thread via `with_r_thread`.

#### Safety

The ExternalPtr assumes exclusive ownership of the underlying data.
Cloning the raw SEXP without proper handling will lead to double-free.

#### Examples

```no_run
use miniextendr_api::externalptr::{ExternalPtr, TypedExternal};

struct MyData { value: f64 }
impl TypedExternal for MyData {
    const TYPE_NAME: &'static str = "MyData";
    const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"my_crate::MyData\0";
}

let ptr = ExternalPtr::new(MyData { value: 3.14 });
assert_eq!(ptr.as_ref().unwrap().value, 3.14);
```

**Methods:**

#### `as_mut`

```rust
as_mut(self: &mut Self) -> Option<&mut T>
```

Returns a mutable reference to the underlying value.

Uses the cached pointer set at construction time, avoiding the
`R_ExternalPtrAddr` FFI call on every access.

#### `as_mut_ptr`

```rust
as_mut_ptr(self: &mut Self) -> *mut T
```

Returns the raw mutable pointer without consuming the ExternalPtr.

#### `as_ptr`

```rust
as_ptr(self: &Self) -> *const T
```

Returns the raw pointer without consuming the ExternalPtr.

#### `as_ref`

```rust
as_ref(self: &Self) -> Option<&T>
```

Returns a reference to the underlying value.

Uses the cached pointer set at construction time, avoiding the
`R_ExternalPtrAddr` FFI call on every access.

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Returns the underlying SEXP.

##### Warning

The returned SEXP must not be duplicated or the finalizer will double-free.

#### `assume_init`

```rust
assume_init(self: Self) -> ExternalPtr<T>
```

Converts to `ExternalPtr<T>`.

##### Safety

The value must have been initialized.

##### Implementation Note

This method creates a *new* SEXP with `T`'s type information, leaving
the original `MaybeUninit<T>` SEXP as an orphaned empty shell in R's heap.
This is necessary because the type ID stored in the prot slot must match
the actual type. The orphaned SEXP will be cleaned up by R's GC eventually.

If you need to avoid this overhead, consider using `ExternalPtr<T>::new`
directly and initializing in place via `as_mut`.

Equivalent to `Box::assume_init`.

#### `collect_into_r_list`

```rust
collect_into_r_list<I>(items: I) -> SEXP
```

Collect an iterator of values into a protected R list (`VECSXP`) holding
one fresh external pointer per item, rooting each via the destination
list instead of the [`ProtectPool`](crate::protect_pool).

This is the GC-safe, allocation-lean way to hand many Rust values to R at
once — e.g. converting a `Vec<T>` into an R `list()` of external pointers.
Each `EXTPTRSXP` is created and **immediately** stored into the
already-protected result list, so the list roots it the instant it
exists: there is no unprotected window between element allocations, and
**no per-element pool traffic**.

Contrast the naive `items.map(ExternalPtr::new).collect::<Vec<_>>()`,
which roots every handle in the process-wide pool (keeping the `Vec`
GC-safe while held — #836) only to release every root again when the `Vec`
drops, then still needs a second pass to copy the handles into a list.
Here the list *is* the root, so both the pool round-trip and the copy
pass are skipped. The whole batch also crosses to R's main thread in a
single [`with_r_thread`](crate::worker::with_r_thread) hop rather than one
per element.

The returned `VECSXP` is **not** protected: the caller must protect it or
return it to R immediately, exactly like any other freshly built SEXP
(e.g. an [`IntoR`](crate::IntoR) result).

#### `downcast_mut`

```rust
downcast_mut<T>(self: &mut Self) -> Option<&mut T>
```

Downcast to a mutable reference of the stored type if it matches `T`.

Uses `Any::downcast_mut` for authoritative runtime type checking.

#### `downcast_ref`

```rust
downcast_ref<T>(self: &Self) -> Option<&T>
```

Downcast to an immutable reference of the stored type if it matches `T`.

Uses `Any::downcast_ref` for authoritative runtime type checking.

#### `from_raw`

```rust
unsafe from_raw(raw: *mut T) -> Self
```

Constructs an ExternalPtr from a raw pointer.

Re-wraps the `*mut T` in `Box<dyn Any>` for the new storage format.

##### Safety

- `raw` must have been allocated via `Box::into_raw` or equivalent
- `raw` must not be null
- Caller transfers ownership to the ExternalPtr
- Must be called from R's main thread

Equivalent to `Box::from_raw`.

#### `from_raw_unchecked`

```rust
unsafe from_raw_unchecked(raw: *mut T) -> Self
```

Constructs an ExternalPtr from a raw pointer, without thread checks.

##### Safety

- `raw` must have been allocated via `Box::into_raw` or equivalent
- `raw` must not be null
- Caller transfers ownership to the ExternalPtr
- Must be called from R's main thread (no debug assertions)

#### `from_sexp`

```rust
unsafe from_sexp(sexp: SEXP) -> Self
```

Create a type-erased ExternalPtr from an EXTPTRSXP without checking the stored type.

##### Safety

- `sexp` must be a valid EXTPTRSXP
- Caller must ensure exclusive ownership semantics are upheld

#### `from_sexp_unchecked`

```rust
unsafe from_sexp_unchecked(sexp: SEXP) -> Self
```

Create an ExternalPtr from an SEXP without type checking.

##### Safety

- `sexp` must be a valid EXTPTRSXP containing a `*mut Box<dyn Any>`
  wrapping a value of type `T`
- The caller must ensure exclusive ownership

#### `into_inner`

```rust
into_inner(this: Self) -> T
```

Consumes the ExternalPtr, returning the wrapped value.

Uses `Box<dyn Any>::downcast` to recover the concrete `Box<T>`,
then moves the value out.

Equivalent to `*boxed` (deref move) or `Box::into_inner`.

#### `into_non_null`

```rust
into_non_null(this: Self) -> NonNull<T>
```

Consumes the ExternalPtr, returning a `NonNull` pointer.

Equivalent to `Box::into_non_null`.

#### `into_pin`

```rust
into_pin(this: Self) -> Pin<Self>
```

Converts a `ExternalPtr<T>` into a `Pin<ExternalPtr<T>>`.

Equivalent to `Box::into_pin`.

#### `into_raw`

```rust
into_raw(this: Self) -> *mut T
```

Consumes the ExternalPtr, returning a raw pointer.

The caller is responsible for the memory, and the finalizer is
effectively orphaned (will do nothing since we clear the pointer).

Equivalent to `Box::into_raw`.

#### `is`

```rust
is<T>(self: &Self) -> bool
```

Check whether the stored `Box<dyn Any>` contains a `T`.

Uses `Any::is` for authoritative runtime type checking.

#### `is_null`

```rust
is_null(self: &Self) -> bool
```

Checks if the internal pointer is null (already finalized or cleared).

#### `leak`

```rust
leak<'a>(this: Self) -> &'a mut T
```

Consumes and leaks the ExternalPtr, returning a mutable reference.

The memory will never be freed (from Rust's perspective; R's GC
finalizer is neutralized).

Equivalent to `Box::leak`.

#### `new`

```rust
new(x: T) -> Self
```

Allocates memory on the heap and places `x` into it.

Internally stores a `Box<Box<dyn Any>>` — a thin pointer (fits in R's
`R_ExternalPtrAddr`) pointing to a fat pointer (carries the `Any` vtable
for runtime type checking via `downcast`).

This function can be called from any thread:
- If called from R's main thread, creates the ExternalPtr directly
- If called from the worker thread (during `run_on_worker`), automatically
  sends the R API calls to the main thread via [`with_r_thread`]

##### Panics

Panics if called from a non-main thread outside of a `run_on_worker` context.

Equivalent to `Box::new`.

[`with_r_thread`]: crate::worker::with_r_thread

#### `new_unchecked`

```rust
unsafe new_unchecked(x: T) -> Self
```

Allocates memory on the heap and places `x` into it, without thread checks.

##### Safety

Must be called from R's main thread. Calling from another thread
is undefined behavior (R APIs are not thread-safe).

#### `new_uninit`

```rust
new_uninit() -> ExternalPtr<MaybeUninit<T>>
```

Constructs a new `ExternalPtr` with uninitialized contents.

Equivalent to `Box::new_uninit`.

#### `new_zeroed`

```rust
new_zeroed() -> ExternalPtr<MaybeUninit<T>>
```

Constructs a new `ExternalPtr` with zeroed contents.

Equivalent to `Box::new_zeroed`.

#### `pin`

```rust
pin(x: T) -> Pin<Self>
```

Constructs a new `Pin<ExternalPtr<T>>`.

Equivalent to `Box::pin`.

##### Note

Unlike `Box::pin`, this requires `T: Unpin` because `ExternalPtr`
implements `DerefMut` unconditionally. For `!Unpin` types, use
`ExternalPtr::new` and manage pinning guarantees manually.

#### `pin_unchecked`

```rust
pin_unchecked(x: T) -> Pin<Self>
```

Constructs a new `Pin<ExternalPtr<T>>` without requiring `Unpin`.

##### Safety

The caller must ensure that the pinning invariants are upheld:
- The data will not be moved out of the `ExternalPtr`
- The data will not be accessed mutably in ways that would move it

Since `ExternalPtr` implements `DerefMut`, using this with `!Unpin`
types requires careful handling to avoid moving the inner value.

#### `prot_raw`

```rust
prot_raw(self: &Self) -> SEXP
```

Returns the raw prot VECSXP (contains both type ID and user protected).

Prefer using `protected()` for user data and `stored_type_id()` for type info.

#### `protected`

```rust
protected(self: &Self) -> SEXP
```

Returns the protected SEXP slot (user-protected objects).

This returns the user-protected object stored in the prot VECSXP,
not the VECSXP itself.

#### `protected_unchecked`

```rust
unsafe protected_unchecked(self: &Self) -> SEXP
```

Returns the protected SEXP slot (unchecked version).

Skips thread safety checks for performance-critical paths.

##### Safety

Must be called from the R main thread. Only use in ALTREP callbacks
or other contexts where you're certain you're on the main thread.

#### `ptr_eq`

```rust
ptr_eq(this: &Self, other: &Self) -> bool
```

Checks whether two `ExternalPtr`s refer to the same allocation (pointer identity).

This ignores the pointee values. Use this when you need alias detection;
prefer `PartialEq`/`PartialOrd` or `as_ref()` for value comparisons.

#### `reborrow`

```rust
reborrow(self: &Self) -> Self
```

Create a lightweight alias of this ExternalPtr sharing the same R object.

The returned `ExternalPtr` points to the **same** underlying EXTPTRSXP.
No data is copied and no new R object is allocated -- both the original
and the alias refer to the same R-level external pointer.

This is the correct way to return "self" from a method that takes
`self: &ExternalPtr<Self>`, preserving R object identity:

```ignore
#[miniextendr(env)]
impl MyType {
    pub fn identity(self: &ExternalPtr<Self>) -> ExternalPtr<Self> {
        self.reborrow()
    }
}
```

##### Safety note

The caller must not use the original and the alias to create overlapping
mutable references (`as_mut`). In typical use (returning from a method),
the borrow of the original ends when the method returns, so this is safe.

#### `set_protected`

```rust
unsafe set_protected(self: &Self, user_prot: SEXP) -> bool
```

Sets the user-protected SEXP slot.

Use this to prevent R objects from being GC'd while this ExternalPtr exists.
The type ID stored in prot slot 0 is preserved.

Returns `false` if the prot structure is malformed (should not happen
for ExternalPtrs created by this library).

##### Safety

- `user_prot` must be a valid SEXP or R_NilValue
- Must be called from the R main thread

#### `stored_type_name`

```rust
stored_type_name(self: &Self) -> Option<&'static str>
```

Returns the type name stored in this ExternalPtr's prot slot.

Returns `None` if the prot slot doesn't contain a valid type symbol.

#### `tag`

```rust
tag(self: &Self) -> SEXP
```

Returns the tag SEXP (type identifier symbol).

#### `tag_unchecked`

```rust
unsafe tag_unchecked(self: &Self) -> SEXP
```

Returns the tag SEXP (unchecked version).

Skips thread safety checks for performance-critical paths.

##### Safety

Must be called from the R main thread. Only use in ALTREP callbacks
or other contexts where you're certain you're on the main thread.

#### `type_name`

```rust
type_name() -> &'static str
```

Returns the type name for type T.

#### `wrap_sexp`

```rust
unsafe wrap_sexp(sexp: SEXP) -> Option<Self>
```

Attempt to wrap a SEXP as an ExternalPtr with type checking.

Uses `Any::downcast_ref` for authoritative type checking (Rust `TypeId`).
Falls back to R symbol comparison for type-erased `ExternalPtr<()>`.

Returns `None` if:
- The internal pointer is null
- The stored `Box<dyn Any>` does not contain a `T`

##### Safety

- `sexp` must be a valid EXTPTRSXP created by this library
- The caller must ensure no other ExternalPtr owns this SEXP

#### `wrap_sexp_unchecked`

```rust
unsafe wrap_sexp_unchecked(sexp: SEXP) -> Option<Self>
```

Attempt to wrap a SEXP as an ExternalPtr (unchecked version).

Skips thread safety checks for performance-critical paths like ALTREP callbacks.

##### Safety

- `sexp` must be a valid EXTPTRSXP created by this library
- The caller must ensure exclusive ownership
- Must be called from the R main thread (guaranteed in ALTREP callbacks)

#### `wrap_sexp_with_error`

```rust
unsafe wrap_sexp_with_error(sexp: SEXP) -> Result<Self, TypeMismatchError>
```

Attempt to wrap a SEXP as an ExternalPtr, returning an error with type info on mismatch.

This is used by the [`TryFromSexp`] trait implementation.

##### Safety

Same as [`wrap_sexp`](Self::wrap_sexp).

[`TryFromSexp`]: crate::TryFromSexp

#### `write`

```rust
write(self: Self, value: T) -> ExternalPtr<T>
```

Writes a value and converts to initialized.

Creates a new SEXP with `T`'s type information (the original
`MaybeUninit<T>` SEXP becomes an orphaned shell, cleaned up by GC).

### `externalptr::ExternalSlice`

A slice stored as a standalone struct, suitable for wrapping in ExternalPtr.

This is analogous to the data inside a `Box<[T]>`, but stores capacity
for proper deallocation when created from a `Vec`.

#### Usage

To use with `ExternalPtr`, implement `TypedExternal` for your specific
`ExternalSlice<YourType>`:

```ignore
impl_typed_external!(ExternalSlice<MyElement>);
let ptr = ExternalPtr::new(ExternalSlice::new(vec![1, 2, 3]));
```

**Methods:**

#### `as_mut_slice`

```rust
as_mut_slice(self: &mut Self) -> &mut [T]
```

Borrow the contents as a mutable slice.

#### `as_slice`

```rust
as_slice(self: &Self) -> &[T]
```

Borrow the contents as a shared slice.

#### `capacity`

```rust
capacity(self: &Self) -> usize
```

Capacity of the underlying allocation.

#### `from_boxed`

```rust
from_boxed(boxed: Box<[T]>) -> Self
```

Create from a boxed slice (capacity == len).

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Returns true if the slice is empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Number of elements in the slice.

#### `new`

```rust
new(slice: Vec<T>) -> Self
```

Create an external slice from a `Vec`, preserving its allocation.

### `externalptr::altrep_helpers::RSidecar`

Marker type for enabling R sidecar accessors in an `ExternalPtr` struct.

When used with `#[derive(ExternalPtr)]` and `#[r_data]`, this field acts as
a selector that enables R-facing accessors for sibling `#[r_data]` fields.

#### Supported Field Types

- **`SEXP`** - Raw SEXP access, no conversion
- **`i32`, `f64`, `bool`, `u8`** - Zero-overhead scalars (stored directly in R)
- **Any `IntoR` type** - Automatic conversion (e.g., `String`, `Vec<T>`)

#### Example

```ignore
use miniextendr_api::SEXP;

#[derive(ExternalPtr)]
pub struct MyType {
    pub x: i32,

    /// Selector field - enables R wrapper generation
    #[r_data]
    r: RSidecar,

    /// Raw SEXP slot - MyType_get_raw() / MyType_set_raw()
    #[r_data]
    pub raw: SEXP,

    /// Zero-overhead scalar - MyType_get_count() / MyType_set_count()
    #[r_data]
    pub count: i32,

    /// Conversion type - MyType_get_name() / MyType_set_name()
    #[r_data]
    pub name: String,
}
```

#### Design Notes

- `RSidecar` is a ZST (zero-sized type) - no runtime cost
- Only `pub` `#[r_data]` fields get R wrapper functions generated
- Multiple `RSidecar` fields in one struct is a compile error

### `factor::Factor`

A borrowed view into an R factor's integer indices.

Provides `Deref` to `&[i32]` for direct slice access to the factor's
underlying integer data. The indices are 1-based (matching R's convention)
with `NA_INTEGER` for missing values.

#### Example

```ignore
let factor = Factor::try_new(sexp)?;
for &idx in factor.iter() {
    if idx == NA_INTEGER {
        println!("NA");
    } else {
        println!("level index: {}", idx);
    }
}
```

**Methods:**

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Whether the factor is empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Number of elements in the factor.

#### `level`

```rust
level(self: &Self, idx: usize) -> &'a str
```

Get level string at 0-based index.

#### `levels_sexp`

```rust
levels_sexp(self: &Self) -> SEXP
```

The levels STRSXP.

#### `n_levels`

```rust
n_levels(self: &Self) -> usize
```

Number of levels.

#### `try_new`

```rust
try_new(sexp: SEXP) -> Result<Self, SexpError>
```

Create a Factor from a factor SEXP.

Returns an error if the SEXP is not a factor.

### `factor::FactorMut`

A mutable borrowed view into an R factor's integer indices.

Provides `DerefMut` to `&mut [i32]` for direct mutable slice access.
The indices are 1-based (matching R's convention) with `NA_INTEGER` for NA.

#### Example

```ignore
let mut factor_mut = FactorMut::try_new(sexp)?;
// Set all values to level 1
for idx in factor_mut.iter_mut() {
    *idx = 1;
}
```

**Methods:**

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Whether the factor is empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Number of elements in the factor.

#### `level`

```rust
level(self: &Self, idx: usize) -> &'a str
```

Get level string at 0-based index.

#### `levels_sexp`

```rust
levels_sexp(self: &Self) -> SEXP
```

The levels STRSXP.

#### `n_levels`

```rust
n_levels(self: &Self) -> usize
```

Number of levels.

#### `try_new`

```rust
try_new(sexp: SEXP) -> Result<Self, SexpError>
```

Create a FactorMut from a factor SEXP.

Returns an error if the SEXP is not a factor.

### `factor::FactorOptionVec`

Wrapper for `Vec<Option<T: RFactor>>` with NA support.

**Methods:**

#### `into_inner`

```rust
into_inner(self: Self) -> Vec<Option<T>>
```

Extract the inner vector.

#### `new`

```rust
new(vec: Vec<Option<T>>) -> Self
```

Wrap a `Vec<Option<T>>` so it can be converted to and from R factors with NA support.

### `factor::FactorVec`

Wrapper for `Vec<T: RFactor>` enabling `IntoR`/`TryFromSexp`.

**Methods:**

#### `into_inner`

```rust
into_inner(self: Self) -> Vec<T>
```

Extract the inner vector.

#### `new`

```rust
new(vec: Vec<T>) -> Self
```

Wrap a `Vec<T>` so it can be converted to and from R factors.

### `from_r::SexpLengthError`

Error describing an unexpected R object length.

**Fields:**

- `expected`: `usize`
  - Required length.
- `actual`: `usize`
  - Actual length encountered.

### `from_r::SexpNaError`

Error for NA values in conversions that require non-missing values.

**Fields:**

- `sexp_type`: `crate::SEXPTYPE`
  - R type where an NA was found.

### `from_r::SexpTypeError`

Error describing an unexpected R `SEXPTYPE`.

**Fields:**

- `expected`: `crate::SEXPTYPE`
  - Expected R type.
- `actual`: `crate::SEXPTYPE`
  - Actual R type encountered.

### `gc_protect::OwnedProtect`

A single-object RAII guard: `PROTECT` on create, `UNPROTECT(1)` on drop.

Use this for simple cases where you're protecting a single value and
don't need the batching benefits of [`ProtectScope`].

#### Example

```ignore
unsafe fn allocate_and_fill() -> SEXP {
    let guard = OwnedProtect::new(Rf_allocVector(REALSXP, 10));
    fill_vector(guard.get());
    // Return the SEXP - guard drops and unprotects on this line.
    // This is safe because no GC can occur between unprotect and return.
    guard.get()
}
```

#### Warning: Stack Ordering

`OwnedProtect` uses `UNPROTECT(1)`, which removes the **top** of the protection
stack. If you have nested protections from other sources, the drop order matters!

For complex scenarios, prefer [`ProtectScope`] which unprotects all its values
at once when dropped.

**Methods:**

#### `forget`

```rust
unsafe forget(self: Self)
```

Escape hatch: do not `UNPROTECT(1)` on drop.

##### Safety

Leaks one protection entry unless unprotected elsewhere.

#### `get`

```rust
get(self: &Self) -> SEXP
```

Get the protected SEXP.

#### `new`

```rust
unsafe new(x: SEXP) -> Self
```

Create a new protection guard for `x`.

Calls `Rf_protect(x)` immediately.

##### Safety

- Must be called from the R main thread
- `x` must be a valid SEXP

### `gc_protect::ProtectScope`

A scope that automatically balances `UNPROTECT(n)` on drop.

This is the primary tool for managing GC protection in batch operations.
Each call to [`protect`][Self::protect] or [`protect_with_index`][Self::protect_with_index]
increments an internal counter; when the scope is dropped, `UNPROTECT(n)` is called.

#### Example

```ignore
unsafe fn my_call(x: SEXP, y: SEXP) -> SEXP {
    let scope = ProtectScope::new();
    let x = scope.protect(x);
    let y = scope.protect(y);

    // Both x and y are protected until scope drops
    let result = scope.protect(some_operation(x.get(), y.get()));
    result.get()
} // UNPROTECT(3)
```

#### Nested Scopes

Scopes can be nested. Each scope tracks only its own protections:

```ignore
unsafe fn outer(x: SEXP) -> SEXP {
    let scope = ProtectScope::new();
    let x = scope.protect(x);

    let result = helper(&scope, x.get());
    scope.protect(result).get()
} // UNPROTECT(2)

unsafe fn helper(_parent: &ProtectScope, x: SEXP) -> SEXP {
    let scope = ProtectScope::new();
    let temp = scope.protect(allocate_something());
    combine(x, temp.get())
} // UNPROTECT(1) - only this scope's protections
```

**Methods:**

#### `alloc_character`

```rust
unsafe alloc_character<'a>(self: &'a Self, n: usize) -> Root<'a>
```

Allocate a character vector (STRSXP), protected.

##### Safety

Must be called from the R main thread.

#### `alloc_complex`

```rust
unsafe alloc_complex<'a>(self: &'a Self, n: usize) -> Root<'a>
```

Allocate a complex vector (CPLXSXP), protected.

##### Safety

Must be called from the R main thread.

#### `alloc_integer`

```rust
unsafe alloc_integer<'a>(self: &'a Self, n: usize) -> Root<'a>
```

Allocate an integer vector (INTSXP), protected.

##### Safety

Must be called from the R main thread.

#### `alloc_list`

```rust
unsafe alloc_list<'a>(self: &'a Self, n: i32) -> Root<'a>
```

Allocate a list (VECSXP) of the given length and immediately protect it.

##### Safety

Same as [`alloc_vector`][Self::alloc_vector].

#### `alloc_logical`

```rust
unsafe alloc_logical<'a>(self: &'a Self, n: usize) -> Root<'a>
```

Allocate a logical vector (LGLSXP), protected.

##### Safety

Must be called from the R main thread.

#### `alloc_matrix`

```rust
unsafe alloc_matrix<'a>(self: &'a Self, ty: SEXPTYPE, nrow: i32, ncol: i32) -> Root<'a>
```

Allocate a matrix of the given type and dimensions, and immediately protect it.

##### Safety

Same as [`alloc_vector`][Self::alloc_vector].

#### `alloc_raw`

```rust
unsafe alloc_raw<'a>(self: &'a Self, n: usize) -> Root<'a>
```

Allocate a raw vector (RAWSXP), protected.

##### Safety

Must be called from the R main thread.

#### `alloc_real`

```rust
unsafe alloc_real<'a>(self: &'a Self, n: usize) -> Root<'a>
```

Allocate a real vector (REALSXP), protected.

##### Safety

Must be called from the R main thread.

#### `alloc_strsxp`

```rust
unsafe alloc_strsxp<'a>(self: &'a Self, n: usize) -> Root<'a>
```

Allocate a STRSXP (character vector) of the given length and immediately protect it.

##### Safety

Same as [`alloc_vector`][Self::alloc_vector].

#### `alloc_vecsxp`

```rust
unsafe alloc_vecsxp<'a>(self: &'a Self, n: usize) -> Root<'a>
```

Allocate a VECSXP (generic list) of the given length and immediately protect it.

##### Safety

Same as [`alloc_vector`][Self::alloc_vector].

#### `alloc_vector`

```rust
unsafe alloc_vector<'a>(self: &'a Self, ty: SEXPTYPE, n: R_xlen_t) -> Root<'a>
```

Allocate a vector of the given type and length, and immediately protect it.

This combines allocation and protection in a single step, eliminating the
GC gap that exists when you separately allocate and then protect.

##### Safety

- Must be called from the R main thread
- Only protects the newly allocated object; does not protect other live
  unprotected objects during allocation

##### Example

```ignore
unsafe fn make_ints(n: R_xlen_t) -> SEXP {
    let scope = ProtectScope::new();
    let vec = scope.alloc_vector(SEXPTYPE::INTSXP, n);
    // fill via INTEGER(vec.get()) ...
    vec.get()
}
```

#### `coerce`

```rust
unsafe coerce<'a>(self: &'a Self, x: SEXP, target: SEXPTYPE) -> Root<'a>
```

Coerce a SEXP to a different type, protected.

##### Safety

Must be called from the R main thread. `x` must be a valid SEXP.

#### `collect`

```rust
unsafe collect<'a, T, I>(self: &'a Self, iter: I) -> Root<'a>
```

Collect an iterator into a typed R vector.

This allocates once, protects, and fills directly - the most efficient pattern
for typed vectors. The element type `T` determines the R vector type via
the [`RNativeType`] trait.

##### Type Mapping

| Rust Type | R Vector Type |
|-----------|---------------|
| `i32` | `INTSXP` |
| `f64` | `REALSXP` |
| `u8` | `RAWSXP` |
| [`RLogical`](crate::RLogical) | `LGLSXP` |
| [`Rcomplex`](crate::Rcomplex) | `CPLXSXP` |

##### Safety

Must be called from the R main thread.

##### Example

```ignore
unsafe fn squares(n: usize) -> SEXP {
    let scope = ProtectScope::new();
    // Type inferred from iterator
    scope.collect((0..n).map(|i| (i * i) as i32)).get()
}
```

##### Unknown Length

For iterators without exact size (e.g., `filter`), collect to `Vec` first:

```ignore
let evens: Vec<i32> = data.iter().filter(|x| *x % 2 == 0).copied().collect();
scope.collect(evens)
```

#### `count`

```rust
count(self: &Self) -> i32
```

Return the current protection count.

#### `disarm`

```rust
unsafe disarm(self: &Self)
```

Escape hatch: disable `UNPROTECT` on drop.

After calling this, the scope will **not** unprotect its values when dropped.
You become responsible for ensuring correct unprotection.

##### Safety

You must ensure the protects performed in this scope are correctly
unprotected elsewhere, or you will leak protect stack entries.

#### `duplicate`

```rust
unsafe duplicate<'a>(self: &'a Self, x: SEXP) -> Root<'a>
```

Deep-duplicate a SEXP, protected.

##### Safety

Must be called from the R main thread. `x` must be a valid SEXP.

#### `mkchar`

```rust
unsafe mkchar<'a>(self: &'a Self, s: &str) -> Root<'a>
```

Create a CHARSXP from a Rust `&str`, protected.

##### Safety

Must be called from the R main thread.

#### `new`

```rust
unsafe new() -> Self
```

Create a new protection scope.

##### Safety

Must be called from the R main thread.

#### `new_env`

```rust
unsafe new_env<'a>(self: &'a Self, parent: SEXP, hash: bool, size: i32) -> Root<'a>
```

Create a new environment, protected.

##### Safety

Must be called from the R main thread.

#### `protect`

```rust
unsafe protect<'a>(self: &'a Self, x: SEXP) -> Root<'a>
```

Protect `x` and return a rooted handle tied to this scope.

This always calls `Rf_protect`. The protection is released when
the scope is dropped (along with all other protections in this scope).

##### Safety

- Must be called from the R main thread
- `x` must be a valid SEXP

#### `protect2`

```rust
unsafe protect2<'a>(self: &'a Self, a: SEXP, b: SEXP) -> (Root<'a>, Root<'a>)
```

Protect two values at once (convenience method).

##### Safety

Same as [`protect`][Self::protect].

#### `protect3`

```rust
unsafe protect3<'a>(self: &'a Self, a: SEXP, b: SEXP, c: SEXP) -> (Root<'a>, Root<'a>, Root<'a>)
```

Protect three values at once (convenience method).

##### Safety

Same as [`protect`][Self::protect].

#### `protect_raw`

```rust
unsafe protect_raw(self: &Self, x: SEXP) -> SEXP
```

Protect and return the raw `SEXP` (sometimes more convenient).

##### Safety

Same as [`protect`][Self::protect].

#### `protect_with_index`

```rust
unsafe protect_with_index<'a>(self: &'a Self, x: SEXP) -> ReprotectSlot<'a>
```

Protect `x` with an index slot so it can be replaced later via [`R_Reprotect`].

Use this when you need to update a protected value in-place without
growing the protection stack.

##### Safety

- Must be called from the R main thread
- `x` must be a valid SEXP

##### Example

```ignore
unsafe fn accumulate(values: &[SEXP]) -> SEXP {
    let scope = ProtectScope::new();
    let slot = scope.protect_with_index(values[0]);

    for &v in &values[1..] {
        let combined = combine(slot.get(), v);
        slot.set(combined);  // Reprotect without growing stack
    }

    slot.get()
}
```

#### `rearm`

```rust
unsafe rearm(self: &Self)
```

Re-arm a previously disarmed scope.

##### Safety

Only call if you know the scope was disarmed and you want to restore
automatic unprotection. Be careful not to double-unprotect.

#### `rooted`

```rust
unsafe rooted<'a>(self: &'a Self, sexp: SEXP) -> Root<'a>
```

Create a `Root<'a>` for an already-protected SEXP without adding protection.

This is useful when you have a SEXP that is already protected by some other
mechanism (e.g., a `ReprotectSlot`) and want to return it as a `Root` tied
to this scope's lifetime for API consistency.

##### Safety

- The caller must ensure `sexp` is already protected and will remain
  protected for at least the lifetime of this scope
- Must be called from the R main thread

#### `scalar_complex`

```rust
unsafe scalar_complex<'a>(self: &'a Self, x: crate::Rcomplex) -> Root<'a>
```

Create a scalar complex (length-1 CPLXSXP), protected.

##### Safety

Must be called from the R main thread.

#### `scalar_integer`

```rust
unsafe scalar_integer<'a>(self: &'a Self, x: i32) -> Root<'a>
```

Create a scalar integer (length-1 INTSXP), protected.

##### Safety

Must be called from the R main thread.

#### `scalar_logical`

```rust
unsafe scalar_logical<'a>(self: &'a Self, x: bool) -> Root<'a>
```

Create a scalar logical (length-1 LGLSXP), protected.

##### Safety

Must be called from the R main thread.

#### `scalar_raw`

```rust
unsafe scalar_raw<'a>(self: &'a Self, x: u8) -> Root<'a>
```

Create a scalar raw (length-1 RAWSXP), protected.

##### Safety

Must be called from the R main thread.

#### `scalar_real`

```rust
unsafe scalar_real<'a>(self: &'a Self, x: f64) -> Root<'a>
```

Create a scalar real (length-1 REALSXP), protected.

##### Safety

Must be called from the R main thread.

#### `scalar_string`

```rust
unsafe scalar_string<'a>(self: &'a Self, s: &str) -> Root<'a>
```

Create a scalar string (length-1 STRSXP) from a Rust `&str`, protected.

##### Safety

Must be called from the R main thread.

#### `shallow_duplicate`

```rust
unsafe shallow_duplicate<'a>(self: &'a Self, x: SEXP) -> Root<'a>
```

Shallow-duplicate a SEXP, protected.

##### Safety

Must be called from the R main thread. `x` must be a valid SEXP.

### `gc_protect::Protected`

A Rust value (`T`) bundled with an [`OwnedProtect`] guard on an SEXP
the value borrows from. The protect releases on drop; the lifetime
ties any borrows inside `T` to `&self`, so `T`'s SEXP-internal
references can't outlive the protection.

#### When to use `Protected<'a, T>` vs the alternatives

| Pattern | Use | Why |
|---------|-----|-----|
| [`OwnedProtect`] | raw SEXP, no Rust view | one-shot protect/unprotect on a single SEXP |
| [`ProtectScope`] + [`Root`] | several SEXPs in one function body | batched UNPROTECT, no Rust view |
| `Protected<'a, T>` | SEXP + Rust view of its data | hand the bundle to callers; borrows in `T` tied to `&self` |

#### Notes on Send/Sync

When constructed via [`Protected::new`], the inner [`OwnedProtect`] carries
`!Send + !Sync` (via `NoSendSync`). When constructed via
[`Protected::from_trusted`], the `_protect` field is `None` and the type
becomes auto-`Send`/`Sync` — the same behaviour as
[`ProtectedStrVec`](crate::strvec::ProtectedStrVec) today.

**Methods:**

#### `from_trusted`

```rust
unsafe from_trusted(_sexp: SEXP, inner: T) -> Self
```

Create a protected bundle without adding to the protect stack.

Use when `sexp` is already protected by R (a `.Call` argument,
a [`ProtectScope`] slot, an enclosing [`OwnedProtect`]) to avoid
double-protecting. The lifetime contract is unchanged — `'a`
still ties any borrows inside `inner` to `&self`.

##### Safety

- Must be called from the R main thread.
- `sexp` must be a valid SEXP.
- `sexp` must remain GC-protected for the lifetime of the
  returned `Protected`.
- Lifetime contract same as [`Protected::new`].

#### `get`

```rust
get(self: &Self) -> &T
```

Borrow the inner view.

#### `into_inner`

```rust
into_inner(self: Self) -> T
```

Consume the bundle and return the inner view.

The [`OwnedProtect`] guard drops here, releasing the SEXP from the protect
stack. Any owned data extracted from `T` must not retain SEXP references
after this point.

#### `new`

```rust
unsafe new(sexp: SEXP, inner: T) -> Self
```

Create a protected bundle. Calls `Rf_protect` on `sexp`.

`inner` may borrow from `sexp`; the lifetime `'a` is tied to
`&self` thereafter, so any borrow inside `inner` cannot outlive
this `Protected`.

##### Safety

- Must be called from the R main thread.
- `sexp` must be a valid SEXP.
- If `inner` borrows from `sexp`, its lifetime parameter must
  match `'a`.

##### Example

```ignore
use miniextendr_api::{Protected, OwnedProtect};
use miniextendr_api::prelude::SEXP;

unsafe fn wrap_view(sexp: SEXP, view: MyView<'_>) -> Protected<'_, MyView<'_>> {
    // Protect the SEXP and bundle it with the view.
    // The view's borrow is tied to the returned Protected.
    Protected::new(sexp, view)
}
```

### `gc_protect::ReprotectSlot`

A protected slot created with `R_ProtectWithIndex` and updated with `R_Reprotect`.

This allows updating a protected value in-place without growing the protection
stack. Useful for loops that repeatedly allocate and update a value.

The slot is valid only while the creating [`ProtectScope`] is alive.

#### When to Use `ReprotectSlot`

Use `ReprotectSlot` when you need to **reassign a protected value** multiple times:

| Pattern | Use | Why |
|---------|-----|-----|
| Accumulator loop | `ReprotectSlot` | Repeatedly replace result without stack growth |
| Single allocation | `ProtectScope::protect` | Simpler, no reassignment needed |
| Child insertion | `List::set_elt` | Container handles child protection |

#### Warning: RAII Assignment Pitfall

R's PROTECT stack is LIFO. Rust's RAII drop order can cause problems:

```ignore
// WRONG - can unprotect the new value instead of the old!
let mut guard = OwnedProtect::new(old_value);
guard = OwnedProtect::new(new_value);  // Old guard drops AFTER new is assigned
```

`ReprotectSlot` avoids this by using `R_Reprotect` which replaces in-place:

```ignore
// CORRECT - always keeps exactly one slot protected
let slot = scope.protect_with_index(old_value);
slot.set(new_value);  // R_Reprotect, no stack change
```

#### Examples

##### Accumulator Pattern

```ignore
unsafe fn sum_allocated_vectors(n: i32) -> SEXP {
    let scope = ProtectScope::new();

    // Initial allocation
    let slot = scope.protect_with_index(Rf_allocVector(REALSXP, 10));

    for i in 0..n {
        // Each iteration allocates a new vector
        let new_vec = compute_step(slot.get(), i);
        slot.set(new_vec);  // Replace without growing protect stack
    }

    slot.get()
}
```

##### Starting with Empty Slot

```ignore
unsafe fn build_result(items: &[Input]) -> SEXP {
    let scope = ProtectScope::new();

    // Start with R_NilValue, replace with first real result
    let slot = scope.protect_with_index(R_NilValue);

    for (i, item) in items.iter().enumerate() {
        let result = process_item(item, slot.get());
        slot.set(result);
    }

    slot.get()
}
```

##### Multiple Slots

```ignore
unsafe fn merge_sorted(a: SEXP, b: SEXP) -> SEXP {
    let scope = ProtectScope::new();

    let slot_a = scope.protect_with_index(a);
    let slot_b = scope.protect_with_index(b);
    let result = scope.protect_with_index(R_NilValue);

    // Process both inputs, updating result
    while !is_empty(slot_a.get()) && !is_empty(slot_b.get()) {
        let merged = merge_next(slot_a.get(), slot_b.get());
        result.set(merged);
        // ... update slot_a and slot_b as needed
    }

    result.get()
}
```

**Methods:**

#### `clear`

```rust
unsafe clear(self: &Self)
```

Clear the slot by setting it to `R_NilValue`.

The slot remains allocated (protect stack depth unchanged), but releases
its reference to the previous value. The previous value may still be
rooted elsewhere.

##### Safety

Must be called from the R main thread.

#### `get`

```rust
get(self: &Self) -> SEXP
```

Get the currently protected SEXP.

#### `is_nil`

```rust
unsafe is_nil(self: &Self) -> bool
```

Check if the slot is currently cleared (holds `R_NilValue`).

##### Safety

Must be called from the R main thread (accesses R's `R_NilValue`).

#### `replace`

```rust
unsafe replace(self: &Self, x: SEXP) -> SEXP
```

Replace the slot's value with `x` and return the old value.

This provides `Option::replace`-like semantics. The slot now protects
`x`, and the old value is returned **unprotected**.

##### Safety

- Must be called from the R main thread
- `x` must be a valid SEXP
- The returned SEXP is **unprotected**. If it needs to survive further
  allocations, you must protect it explicitly.

##### Example

```ignore
let slot = scope.protect_with_index(initial);
let old = slot.replace(new_value);
// old is unprotected, slot now protects new_value
```

#### `set`

```rust
unsafe set(self: &Self, x: SEXP) -> SEXP
```

Replace the protected value in-place using `R_Reprotect`.

The new value `x` becomes protected in this slot, and the old value
is no longer protected (but may still be rooted elsewhere).

Returns the raw SEXP for convenience. Note that this SEXP is only
protected until the next call to `set()` on this slot - if you need
to hold multiple protected values simultaneously, use separate
protection slots or `OwnedProtect`.

##### Safety

- Must be called from the R main thread
- `x` must be a valid SEXP

#### `set_with`

```rust
unsafe set_with<F>(self: &Self, f: F) -> SEXP
```

Allocate a new value via the closure and replace this slot's value safely.

This method encodes the safe pattern for replacing a protected slot with
a newly allocated value. It:

1. Calls the closure `f()` to allocate a new SEXP
2. Temporarily protects the new value (to close the GC gap)
3. Calls `R_Reprotect` to replace this slot's value
4. Unprotects the temporary protection

This prevents the GC gap that would exist if you called `f()` and then
`set()` separately - during that window, the newly allocated value would
be unprotected.

##### Safety

- Must be called from the R main thread
- The closure must return a valid SEXP

##### Example

```ignore
unsafe fn grow_list(scope: &ProtectScope, old_list: SEXP) -> SEXP {
    let slot = scope.protect_with_index(old_list);

    // Safely grow the list without GC gap
    slot.set_with(|| {
        let new_list = Rf_allocVector(VECSXP, new_size);
        // copy elements from old_list to new_list...
        new_list
    });

    slot.get()
}
```

#### `take`

```rust
unsafe take(self: &Self) -> SEXP
```

Take the current value and clear the slot to `R_NilValue`.

This provides `Option::take`-like semantics. The slot remains allocated
(protect stack depth unchanged), but now holds `R_NilValue` (immortal).

##### Safety

- Must be called from the R main thread
- The returned SEXP is **unprotected**. If it needs to survive further
  allocations, you must protect it explicitly.

##### Example

```ignore
let slot = scope.protect_with_index(some_value);
// ... work with slot.get() ...
let old = slot.take();  // slot now holds R_NilValue
// old is unprotected - protect it if needed
let guard = OwnedProtect::new(old);
```

### `gc_protect::Root`

A rooted SEXP tied to the lifetime of a [`ProtectScope`].

This type has **no `Drop`**. The scope owns unprotection responsibility.
This makes `Root` cheap to move and copy (it's just a pointer + lifetime).

#### Lifetime

The `'a` lifetime ties the root to its creating scope. The compiler ensures
you cannot use the root after the scope has been dropped.

**Methods:**

#### `get`

```rust
get(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `into_raw`

```rust
into_raw(self: Self) -> SEXP
```

Consume the root and return the underlying SEXP.

The SEXP remains protected until the scope drops.

### `gc_protect::WorkerUnprotectGuard`

A `Send`-safe guard that calls `Rf_unprotect(n)` on drop via `with_r_thread`.

Use this when you `Rf_protect` on the R main thread, then need the unprotect
to happen when a guard drops on a **worker thread** (e.g., rayon parallel code).

[`OwnedProtect`] and [`ProtectScope`] are `!Send` — they can only be used on
the R main thread. `WorkerUnprotectGuard` fills the gap for cross-thread patterns
where allocation + protect happen on the R thread but the guard lives on a worker.

#### Example

```ignore
use miniextendr_api::gc_protect::WorkerUnprotectGuard;

let sexp = with_r_thread(|| unsafe {
    let sexp = Rf_allocVector(REALSXP, n);
    Rf_protect(sexp);
    sexp
});
let _guard = WorkerUnprotectGuard::new(1);

// ... parallel work on sexp's data ...
// _guard drops here, dispatching Rf_unprotect(1) back to R thread
```

**Methods:**

#### `new`

```rust
new(n: i32) -> Self
```

Create a guard that will unprotect `n` entries on drop.

### `gc_protect::tls::TlsRoot`

A rooted SEXP from TLS protection.

This is similar to [`super::Root`] but without a compile-time lifetime tie to
the scope. The protection is valid as long as the enclosing
[`with_protect_scope`] block hasn't exited.

#### Warning

Using a `TlsRoot` after its scope has exited is undefined behavior.
The compile-time lifetime checking of [`super::Root`] is safer; use TLS
convenience only when necessary.

**Methods:**

#### `get`

```rust
get(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `into_raw`

```rust
into_raw(self: Self) -> SEXP
```

Consume and return the underlying SEXP.

### `into_r::altrep::Altrep`

**Methods:**

#### `into_altrep_sexp`

```rust
into_altrep_sexp(self: Self) -> crate::altrep_sexp::AltrepSexp
```

Convert to R ALTREP and wrap in [`AltrepSexp`](crate::altrep_sexp::AltrepSexp) (`!Send + !Sync`).

This creates the ALTREP SEXP and wraps it in an `AltrepSexp` that
prevents the result from being sent to non-R threads. Use this when
you need to keep the ALTREP vector in Rust code and want compile-time
thread safety guarantees.

For returning directly to R from `#[miniextendr]` functions, use
`Altrep<T>` as the return type (which implements `IntoR`) or call
`.into_sexp()` / `.into_sexp_altrep()` instead.

#### `into_inner`

```rust
into_inner(self: Self) -> T
```

Unwrap and return the inner value.

#### `new`

```rust
new(value: T) -> Self
```

Create a new ALTREP marker wrapper.

### `into_r::result::NullOnErr`

Marker type for `Result<T, ()>` that converts `Err(())` to NULL.

This type is used internally by the `#[miniextendr]` macro when handling
`Result<T, ()>` return types. When the error type is `()`, there's no
error message to report, so we return NULL instead of raising an error.

#### Usage

You typically don't use this directly. When you write:

```ignore
#[miniextendr]
fn maybe_value(x: i32) -> Result<i32, ()> {
    if x > 0 { Ok(x) } else { Err(()) }
}
```

The macro generates code that converts `Err(())` to `Err(NullOnErr)` and
returns `NULL` in R.

#### Note

`NullOnErr` intentionally does NOT implement `Display` to avoid conflicting
with the generic `IntoR for Result<T, E: Display>` impl. It has its own
specialized `IntoR` impl that returns NULL on error.

### `list::DuplicateNameError`

Error when a list has duplicate non-NA names.

**Fields:**

- `name`: `String`
  - The duplicate name that was found.

### `list::List`

Owned handle to an R list (`VECSXP`).

#### Examples

```no_run
use miniextendr_api::list::List;

let list = List::from_values(vec![1i32, 2, 3]);
assert_eq!(list.len(), 3);
let first: Option<i32> = list.get_index(0);
```

**Methods:**

#### `as_data_frame`

```rust
as_data_frame(self: &Self) -> Result<DataFrame, DataFrameError>
```

Promote this named list to a [`DataFrame`].

##### Errors

Returns [`DataFrameError`] if the list has no names or columns differ in length.

#### `as_sexp`

```rust
const as_sexp(self: Self) -> SEXP
```

Get the underlying `SEXP`.

#### `from_pairs`

```rust
from_pairs<N, T>(pairs: Vec<(N, T)>) -> Self
```

Build a list from `(name, value)` pairs, setting `names` in one pass.

#### `from_raw`

```rust
const unsafe from_raw(sexp: SEXP) -> Self
```

Wrap an existing `VECSXP` without additional checks.

##### Safety

Caller must ensure `sexp` is a valid list object (typically a `VECSXP` or
a pairlist coerced to `VECSXP`) whose lifetime remains managed by R.

#### `from_raw_pairs`

```rust
from_raw_pairs<N>(pairs: Vec<(N, SEXP)>) -> Self
```

Build a list from `(name, SEXP)` pairs (heterogeneous-friendly).

##### Safety Note

The input SEXPs should already be protected or be children of protected
containers. This function protects the list and names vector during
construction.

#### `from_raw_pairs_empty`

```rust
from_raw_pairs_empty() -> Self
```

Build an empty named-list SEXP (zero elements, `names` attribute set).

Equivalent to [`Self::from_raw_pairs`]`(vec![])`, but avoids the
`Vec<(&str, SEXP)>` type annotation that Rust requires at empty-vector
callsites where type inference cannot resolve the element type.

Codegen paths that emit an empty `from_raw_pairs` call (e.g. unit-variant
partitions in `#[derive(DataFrameRow)]`) use this helper so that a future
signature change to `from_raw_pairs` only needs to be updated in one
place.

#### `from_raw_values`

```rust
from_raw_values(values: Vec<SEXP>) -> Self
```

Build an unnamed list from pre-converted SEXPs.

##### Safety Note

The input SEXPs should already be protected or be children of protected
containers. This function protects the list during construction.

#### `from_scalars_or_list`

```rust
from_scalars_or_list(elements: &[SEXP]) -> Self
```

Build an atomic vector from homogeneous length-1 scalar SEXPs.

If all elements are length-1 scalars of the same coalesceable type
(INTSXP, REALSXP, LGLSXP, STRSXP), returns that atomic vector.
Otherwise returns a VECSXP (generic list).

This is the canonical entry point for both `DataFrame::into_data_frame`
(column building) and `SeqSerializer::end` (sequence coalescing).

##### Safety Note

The input SEXPs should already be protected or be children of protected
containers.

#### `from_values`

```rust
from_values<T>(values: Vec<T>) -> Self
```

Build an unnamed list from values.

Use this for tuple-like structures where positional access is more natural.

##### Example

```ignore
let list = List::from_values(vec![1i32, 2i32, 3i32]);
// R: list(1L, 2L, 3L) - accessed as [[1]], [[2]], [[3]]
```

#### `get`

```rust
get(self: Self, idx: isize) -> Option<SEXP>
```

Get raw SEXP element at 0-based index. Returns `None` if out of bounds.

#### `get_class`

```rust
get_class(self: Self) -> Option<SEXP>
```

Get the `class` attribute if present.

#### `get_colnames`

```rust
get_colnames(self: Self) -> Option<SEXP>
```

Get column names from the `dimnames` attribute.

#### `get_dim`

```rust
get_dim(self: Self) -> Option<SEXP>
```

Get the `dim` attribute if present.

#### `get_dimnames`

```rust
get_dimnames(self: Self) -> Option<SEXP>
```

Get the `dimnames` attribute if present.

#### `get_index`

```rust
get_index<T>(self: Self, idx: isize) -> Option<T>
```

Get element at 0-based index and convert to type `T`.

Returns `None` if index is out of bounds or conversion fails.

The conversion error is discarded, so `T`'s `TryFromSexp::Error` is
unconstrained — any element type works, not only those whose error is
`SexpError`. Callers that need the error (e.g. to distinguish "missing"
from "wrong type") should use [`get`](Self::get) and convert directly.

#### `get_levels`

```rust
get_levels(self: Self) -> Option<SEXP>
```

Get the `levels` attribute if present (for factors).

#### `get_named`

```rust
get_named<T>(self: Self, name: &str) -> Option<T>
```

Get element by name and convert to type `T`.

Returns `None` if name not found or conversion fails.

The conversion error is discarded, so `T`'s `TryFromSexp::Error` is
unconstrained. Use [`get_named_sexp`](Self::get_named_sexp) and convert
directly when you need to inspect the conversion failure.

#### `get_named_sexp`

```rust
get_named_sexp(self: Self, name: &str) -> Option<SEXP>
```

Get the raw element `SEXP` associated with `name`, without conversion.

Returns the element exactly as stored so callers can convert it with any
[`TryFromSexp`] error type — not only those whose error is `SexpError`.
Returns `None` when the list has no `names` attribute or no name matches.

#### `get_rownames`

```rust
get_rownames(self: Self) -> Option<SEXP>
```

Get row names from the `dimnames` attribute.

#### `get_tsp`

```rust
get_tsp(self: Self) -> Option<SEXP>
```

Get the `tsp` attribute if present (for time series).

#### `is_empty`

```rust
is_empty(self: Self) -> bool
```

Returns true if the list is empty.

#### `is_list`

```rust
is_list(self: Self) -> bool
```

Return true if the underlying SEXP is a list (VECSXP) according to R.

Uses `SexpExt::is_list` (VECSXP check) — **not** `is_pair_list` (LISTSXP).

#### `len`

```rust
len(self: Self) -> isize
```

Length of the list (number of elements).

#### `names`

```rust
names(self: Self) -> Option<SEXP>
```

Get the `names` attribute if present.

#### `set_class`

```rust
set_class(self: Self, class: SEXP) -> Self
```

Set the `class` attribute; returns the same list for chaining.

Equivalent to R's `SET_CLASS(x, n)`.

#### `set_class_str`

```rust
set_class_str(self: Self, classes: &[&str]) -> Self
```

Set the `class` attribute from a slice of class names.

This is a convenience wrapper that creates a character vector from the
provided strings and sets it as the class attribute.

##### Example

```ignore
let list = List::from_pairs(vec![("x", vec![1, 2, 3])]);
let df = list.set_class_str(&["data.frame"]);
```

#### `set_data_frame_class`

```rust
set_data_frame_class(self: Self) -> Self
```

Set class = `"data.frame"` using a cached class STRSXP.

Equivalent to `set_class_str(&["data.frame"])` but avoids allocation.

#### `set_dim`

```rust
set_dim(self: Self, dim: SEXP) -> Self
```

Set the `dim` attribute; returns the same list for chaining.

Equivalent to R's `SET_DIM(x, n)`.

#### `set_dimnames`

```rust
set_dimnames(self: Self, dimnames: SEXP) -> Self
```

Set the `dimnames` attribute; returns the same list for chaining.

Equivalent to R's `SET_DIMNAMES(x, n)`.

#### `set_elt`

```rust
unsafe set_elt(self: Self, idx: isize, child: SEXP)
```

Set an element at the given index, protecting the child during insertion.

This is the safe way to insert a freshly allocated SEXP into a list.
The child is protected for the duration of the `SET_VECTOR_ELT` call,
ensuring it cannot be garbage collected.

##### Safety

- Must be called from the R main thread
- `child` must be a valid SEXP
- `self` must be a valid, protected VECSXP

##### Panics

Panics if `idx` is out of bounds.

##### Example

```ignore
let scope = ProtectScope::new();
let list = List::from_raw(scope.alloc_vecsxp(n).into_raw());

for i in 0..n {
    let child = Rf_allocVector(REALSXP, 10);  // unprotected!
    list.set_elt(i, child);  // safe: protects child during insertion
}
```

#### `set_elt_unchecked`

```rust
unsafe set_elt_unchecked(self: Self, idx: isize, child: SEXP)
```

Set an element without protecting the child.

##### Safety

In addition to the safety requirements of [`set_elt`](Self::set_elt):
- The caller must ensure `child` is already protected or that no GC
  can occur between child allocation and this call.

Use this for performance when you know the child is already protected
(e.g., it's a child of another protected container, or you have an
`OwnedProtect` guard for it).

#### `set_elt_with`

```rust
unsafe set_elt_with<F>(self: Self, idx: isize, f: F)
```

Set an element using a callback that produces the child.

The callback is executed within a protection scope, so any allocations
it performs are protected until insertion completes.

##### Safety

- Must be called from the R main thread
- `self` must be a valid, protected VECSXP

##### Example

```ignore
let list = List::from_raw(scope.alloc_vecsxp(n).into_raw());

for i in 0..n {
    list.set_elt_with(i, || {
        let vec = Rf_allocVector(REALSXP, 10);
        fill_vector(vec);  // can allocate internally
        vec
    });
}
```

#### `set_levels`

```rust
set_levels(self: Self, levels: SEXP) -> Self
```

Set the `levels` attribute; returns the same list for chaining.

Equivalent to R's `SET_LEVELS(x, l)`.

#### `set_names`

```rust
set_names(self: Self, names: SEXP) -> Self
```

Set the `names` attribute; returns the same list for chaining.

Equivalent to R's `SET_NAMES(x, n)`.

#### `set_names_str`

```rust
set_names_str(self: Self, names: &[&str]) -> Self
```

Set the `names` attribute from a slice of strings.

This is a convenience wrapper that creates a character vector from the
provided strings and sets it as the names attribute.

##### Example

```ignore
let list = List::from_values(vec![1, 2, 3]);
let named = list.set_names_str(&["a", "b", "c"]);
```

#### `set_row_names_int`

```rust
set_row_names_int(self: Self, n: usize) -> Self
```

Set `row.names` for a data.frame using compact integer form.

R internally represents row.names as a compact integer vector
`c(NA_integer_, -n)` when the row names are just `1:n`. This is more
memory-efficient than storing n strings.

##### Example

```ignore
let list = List::from_pairs(vec![
    ("x", vec![1, 2, 3]),
    ("y", vec![4, 5, 6]),
])
.set_class_str(&["data.frame"])
.set_row_names_int(3);  // Row names: "1", "2", "3"
```

#### `set_row_names_str`

```rust
set_row_names_str(self: Self, row_names: &[&str]) -> Self
```

Set `row.names` from a vector of strings.

Use this when you need custom row names. For simple sequential row names
(1, 2, 3, ...), use [`set_row_names_int`](Self::set_row_names_int) instead.

##### Example

```ignore
let list = List::from_pairs(vec![
    ("x", vec![1, 2, 3]),
])
.set_class_str(&["data.frame"])
.set_row_names_str(&["row_a", "row_b", "row_c"]);
```

### `list::ListBuilder`

Builder for constructing lists with efficient protection management.

`ListBuilder` holds a reference to a [`ProtectScope`], allowing multiple
elements to be inserted without repeatedly protecting/unprotecting each one.
This is more efficient than using [`List::set_elt`] in a loop.

#### Example

```ignore
unsafe fn build_list(n: isize) -> SEXP {
    let scope = ProtectScope::new();
    let builder = ListBuilder::new(&scope, n);

    for i in 0..n {
        // Allocations inside the loop are protected by the scope
        let child = scope.alloc_real(10).into_raw();
        builder.set(i, child);
    }

    builder.into_sexp()
}
```

**Methods:**

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Get the underlying list SEXP.

#### `from_protected`

```rust
unsafe from_protected(scope: &'a ProtectScope, list: SEXP) -> Self
```

Create a builder wrapping an existing protected list.

##### Safety

- Must be called from the R main thread
- `list` must be a valid, protected VECSXP

#### `into_list`

```rust
into_list(self: Self) -> List
```

Convert to a `List` wrapper.

#### `into_sexp`

```rust
into_sexp(self: Self) -> SEXP
```

Convert to the underlying SEXP.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if the list is empty.

#### `len`

```rust
len(self: &Self) -> isize
```

Get the length of the list.

#### `new`

```rust
unsafe new(scope: &'a ProtectScope, len: usize) -> Self
```

Create a new list builder with the given length.

The list is allocated and protected using the provided scope.

##### Safety

Must be called from the R main thread.

#### `set`

```rust
unsafe set(self: &Self, idx: isize, child: SEXP)
```

Set an element at the given index.

The `child` should be protected by the same scope (or a parent scope).
Use `scope.protect_raw(...)` before calling this method.

##### Safety

- `child` must be a valid SEXP
- `child` should be protected (typically via the same scope)

#### `set_protected`

```rust
unsafe set_protected(self: &Self, idx: isize, child: SEXP)
```

Set an element, protecting the child within the builder's scope.

This is a convenience method that protects the child and then inserts it.

##### Safety

- `child` must be a valid SEXP

### `list::ListMut`

Mutable view of an R list (`VECSXP`).

This is a wrapper type instead of `&mut [SEXP]` to avoid exposing a raw slice
that could become invalid if list elements are replaced with `NULL`.

**Methods:**

#### `as_sexp`

```rust
const as_sexp(self: &Self) -> SEXP
```

Get the underlying `SEXP`.

#### `from_raw`

```rust
const unsafe from_raw(sexp: SEXP) -> Self
```

Wrap an existing `VECSXP` without additional checks.

##### Safety

Caller must ensure `sexp` is a valid `VECSXP` and remains managed by R.

#### `get`

```rust
get(self: &Self, idx: isize) -> Option<SEXP>
```

Get raw SEXP element at 0-based index. Returns `None` if out of bounds.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Returns true if the list is empty.

#### `len`

```rust
len(self: &Self) -> isize
```

Length of the list (number of elements).

#### `set`

```rust
set(self: &mut Self, idx: isize, value: SEXP) -> Result<(), SexpError>
```

Set raw SEXP element at 0-based index.

### `list::accumulator::ListAccumulator`

Accumulator for building lists when the length is unknown upfront.

Unlike [`super::ListBuilder`] which requires knowing the size at construction,
`ListAccumulator` supports dynamic growth via [`push`](Self::push). It uses
[`ReprotectSlot`] internally to maintain **O(1) protect stack usage** regardless
of how many elements are pushed.

#### When to Use

| Scenario | Recommended Type |
|----------|-----------------|
| Known size | [`super::ListBuilder`] - more efficient, no reallocation |
| Unknown size | `ListAccumulator` - bounded stack, dynamic growth |
| Streaming/iterators | `ListAccumulator` or [`collect_list`] |

#### Growth Strategy

The internal list grows exponentially (2x) when capacity is exceeded,
achieving amortized O(1) push. Elements are copied during growth.

#### Example

```ignore
unsafe fn collect_filtered(items: &[i32]) -> SEXP {
    let scope = ProtectScope::new();
    let mut acc = ListAccumulator::new(&scope, 4); // initial capacity hint

    for &item in items {
        if item > 0 {
            acc.push(item);  // auto-converts via IntoR
        }
    }

    acc.into_root().get()
}
```

**Methods:**

#### `capacity`

```rust
capacity(self: &Self) -> usize
```

Get the current capacity.

#### `extend_from`

```rust
unsafe extend_from<I, T>(self: &mut Self, iter: I)
```

Push all items from an iterator.

##### Safety

Must be called from the R main thread.

#### `into_root`

```rust
unsafe into_root(self: Self) -> Root<'a>
```

Finalize the accumulator and return a `Root` pointing to the list.

The returned list is truncated to the actual length (if smaller than capacity).

##### Safety

Must be called from the R main thread.

#### `into_sexp`

```rust
unsafe into_sexp(self: Self) -> SEXP
```

Finalize and return the raw SEXP.

##### Safety

Must be called from the R main thread.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if the accumulator is empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Get the current number of elements.

#### `new`

```rust
unsafe new(scope: &'a ProtectScope, initial_cap: usize) -> Self
```

Create a new accumulator with the given initial capacity.

A capacity of 0 is valid; the list will grow on first push.

##### Safety

Must be called from the R main thread.

#### `push`

```rust
unsafe push<T>(self: &mut Self, value: T)
```

Push a value onto the accumulator.

The value is converted to a SEXP via [`IntoR`] and inserted.
If the internal list is full, it grows automatically.

##### Safety

Must be called from the R main thread.

#### `push_if`

```rust
unsafe push_if<T>(self: &mut Self, condition: bool, value: T)
```

Push a value only if the condition is true.

##### Safety

Must be called from the R main thread.

#### `push_if_with`

```rust
unsafe push_if_with<T>(self: &mut Self, condition: bool, f: impl FnOnce)
```

Push a lazily-evaluated value only if the condition is true.

The closure is only called if `condition` is true.

##### Safety

Must be called from the R main thread.

#### `push_named`

```rust
unsafe push_named<T>(self: &mut Self, name: &str, value: T)
```

Push a named value onto the accumulator.

##### Safety

Must be called from the R main thread.

#### `push_sexp`

```rust
unsafe push_sexp(self: &mut Self, sexp: SEXP)
```

Push a raw SEXP onto the accumulator.

##### Safety

- Must be called from the R main thread
- `sexp` must be a valid SEXP (it will be temporarily protected)

### `list::named::NamedList`

A named list with O(1) name-based element lookup.

Wraps a [`List`] and builds a `HashMap<String, usize>` index of element names
on construction. Use this when you need to access multiple elements by name
from the same list — each lookup is O(1) instead of O(n).

#### When to Use

| Pattern | Type |
|---------|------|
| Single named lookup | [`List::get_named`] is fine |
| Multiple named lookups | `NamedList` (O(n) build + O(1) per lookup) |
| Positional access only | [`List`] — no indexing overhead |

#### Name Handling

- `NA` and empty-string names are excluded from the index
- If duplicate names exist, the **last** occurrence wins
- Positional access via [`get_index`](Self::get_index) is always available

**Methods:**

#### `as_data_frame`

```rust
as_data_frame(self: &Self) -> Result<DataFrame, DataFrameError>
```

Promote this named list to a [`DataFrame`].

Validates equal column lengths, sets the `data.frame` class, and adds compact integer
`row.names`.

##### Errors

Returns [`DataFrameError::UnequalLengths`] if columns differ in length.

#### `as_list`

```rust
as_list(self: &Self) -> List
```

Get the underlying `List`.

#### `contains`

```rust
contains(self: &Self, name: &str) -> bool
```

Check if a name exists in the index.

#### `entries`

```rust
entries(self: &Self) -> impl Iterator
```

Iterate over `(name, position)` pairs (unordered).

#### `get`

```rust
get<T>(self: &Self, name: &str) -> Option<T>
```

Get an element by name with O(1) lookup, converting to type `T`.

Returns `None` if the name is not found or conversion fails. The
conversion error is discarded, so `T`'s `TryFromSexp::Error` is
unconstrained; use [`get_raw`](Self::get_raw) when you need the error.

#### `get_index`

```rust
get_index<T>(self: &Self, idx: isize) -> Option<T>
```

Get element at 0-based index and convert to type `T`.

Falls through to [`List::get_index`] — no name lookup involved.

#### `get_raw`

```rust
get_raw(self: &Self, name: &str) -> Option<SEXP>
```

Get a raw SEXP element by name with O(1) lookup.

#### `into_list`

```rust
into_list(self: Self) -> List
```

Consume and return the underlying `List`.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Returns `true` if the list is empty.

#### `len`

```rust
len(self: &Self) -> isize
```

Number of elements in the list (including unnamed ones).

#### `named_len`

```rust
named_len(self: &Self) -> usize
```

Number of indexed (named) elements.

#### `names`

```rust
names(self: &Self) -> impl Iterator
```

Iterate over indexed names (unordered).

#### `new`

```rust
new(list: List) -> Option<Self>
```

Build a `NamedList` from a `List`, indexing all non-empty, non-NA names.

Returns `None` if the list has no `names` attribute.

### `named_vector::NamedVector`

Wrapper that converts a map to/from a **named atomic R vector** instead of a
named list.

The inner map must have `String` keys and values that implement [`AtomicElement`].

#### Supported value types

| Rust type | R SEXPTYPE |
|-----------|-----------|
| `i32` | INTSXP |
| `f64` | REALSXP |
| `u8` | RAWSXP |
| `bool` | LGLSXP |
| `String` | STRSXP |
| `Option<i32>` | INTSXP (NA = NA_INTEGER) |
| `Option<f64>` | REALSXP (NA = NA_REAL) |
| `Option<bool>` | LGLSXP (NA = NA_LOGICAL) |
| `Option<String>` | STRSXP (NA = NA_character_) |

**Methods:**

#### `into_inner`

```rust
into_inner(self: Self) -> M
```

Unwrap, returning the inner map.

### `optionals::arrow_impl::RPrimitive`

A primitive Arrow array that may be backed by R memory.

`RPrimitive<T>` wraps a [`PrimitiveArray<T>`](arrow_array::PrimitiveArray) and optionally carries the
source R SEXP. When the array came from R (via `TryFromSexp`), converting
back to R is zero-copy — the original SEXP is returned directly.

All Arrow APIs work transparently via `Deref<Target = PrimitiveArray<T>>`.

#### Example

```ignore
use miniextendr_api::optionals::arrow_impl::{RPrimitive, Float64Type};

#[miniextendr]
pub fn passthrough(x: RPrimitive<Float64Type>) -> RPrimitive<Float64Type> {
    x  // zero-copy round-trip: R REALSXP → Arrow → same REALSXP
}

#[miniextendr]
pub fn doubled(x: RPrimitive<Float64Type>) -> RPrimitive<Float64Type> {
    let result: Float64Array = x.iter().map(|v| v.map(|f| f * 2.0)).collect();
    RPrimitive::from_arrow(result)  // no R source → copies on IntoR
}
```

**Methods:**

#### `from_arrow`

```rust
from_arrow(array: arrow_array::PrimitiveArray<T>) -> Self
```

Wrap a computed Arrow array (no R source). IntoR will copy.

#### `from_r`

```rust
unsafe from_r(array: arrow_array::PrimitiveArray<T>, sexp: SEXP) -> Self
```

Wrap an Arrow array with a known R source SEXP.

##### Safety

The caller must ensure `sexp` is a valid R vector whose data buffer
backs `array` (i.e., the array was created via `sexp_to_arrow_buffer`).

#### `into_inner`

```rust
into_inner(self: Self) -> arrow_array::PrimitiveArray<T>
```

Get the inner Arrow array, discarding provenance.

### `optionals::arrow_impl::RStringArray`

A string Arrow array that may be backed by an R STRSXP.

R's STRSXP and Arrow's StringArray have incompatible memory layouts,
so R→Arrow always copies string data. However, for unmodified round-trips,
the original STRSXP is returned on IntoR without rebuilding it.

**Methods:**

#### `from_arrow`

```rust
from_arrow(array: StringArray) -> Self
```

Wrap a computed StringArray (no R source).

#### `from_r`

```rust
unsafe from_r(array: StringArray, sexp: SEXP) -> Self
```

Wrap a StringArray with a known R source STRSXP.

##### Safety

The caller must ensure `sexp` is a valid STRSXP that was the source
for the StringArray's data (i.e., the array was built from this STRSXP).

#### `into_inner`

```rust
into_inner(self: Self) -> StringArray
```

Get the inner StringArray, discarding provenance.

### `optionals::bitflags_impl::RFlags`

Wrapper for bitflags types that implements R conversions.

`RFlags<T>` wraps any type `T` that implements `bitflags::Flags` and provides
`TryFromSexp` and `IntoR` implementations for R interop.

#### Example

```ignore
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Options: u8 {
        const VERBOSE = 0b0001;
        const DEBUG = 0b0010;
    }
}

#[miniextendr]
fn process(opts: RFlags<Options>) -> String {
    if opts.contains(Options::VERBOSE) {
        "verbose mode".to_string()
    } else {
        "quiet mode".to_string()
    }
}
```

**Methods:**

#### `into_inner`

```rust
into_inner(self: Self) -> T
```

Get the wrapped flags.

#### `new`

```rust
new(flags: T) -> Self
```

Create a new `RFlags` wrapper.

### `optionals::borsh_impl::Borsh`

Wrapper for borsh-serializable types.

Converts between R raw vectors (RAWSXP) and borsh binary format.
Use `Borsh(value)` to wrap a value for conversion.

### `optionals::jiff_impl::JiffTimestampVec`

ALTREP-backed lazy vector of `Timestamp`s.

Materialized on element access as seconds-since-epoch f64. Registered as a
`REALSXP` ALTREP vector with class `"JiffTimestampVec"`.

**Fields:**

- `data`: `std::sync::Arc<Vec<Timestamp>>`
  - Shared ownership of the timestamps.

**Methods:**

#### `new`

```rust
new(data: Vec<Timestamp>) -> Self
```

Create a new ALTREP-backed timestamp vector.

### `optionals::jiff_impl::JiffTimestampVecMut`

Mutable reference wrapper for [`JiffTimestampVec`] ALTREP data. Implements `TryFromSexp`, `Deref`, and `DerefMut`.

### `optionals::jiff_impl::JiffTimestampVecRef`

Immutable reference wrapper for [`JiffTimestampVec`] ALTREP data. Implements `TryFromSexp` and `Deref<Target = JiffTimestampVec>`.

### `optionals::jiff_impl::JiffZonedVec`

ALTREP-backed lazy vector of `Zoned` datetimes, single-timezone strict.

All elements must share the same IANA timezone (verified at construction
time). Elements are materialized on access as seconds-since-epoch f64.
Registered as a `REALSXP` ALTREP vector with class `"JiffZonedVec"`.

The `tzone` attribute on the resulting SEXP carries the canonical IANA name
(e.g. `"America/New_York"`), matching R's POSIXct convention.

**Fields:**

- `data`: `std::sync::Arc<Vec<Zoned>>`
  - Shared ownership of the zoned datetimes.
- `tzone`: `String`
  - Canonical IANA timezone name shared by every element.

**Methods:**

#### `into_posixct_sexp`

```rust
into_posixct_sexp(self: Self) -> crate::SEXP
```

Convert to a `POSIXct` ALTREP SEXP with the correct `class` and `tzone`
attributes.

This is the primary conversion path. The derive-generated [`IntoR`] impl
produces a raw ALTREP without class/tzone; use this method instead when
you want a fully-formed R POSIXct.

#### `new`

```rust
new(data: Vec<Zoned>) -> Result<Self, String>
```

Construct a `JiffZonedVec`, enforcing single-timezone invariant.

Returns an error if any element's IANA timezone name differs from
`data[0]`'s timezone. An empty vector succeeds with `tzone = "UTC"`.

### `optionals::jiff_impl::JiffZonedVecMut`

Mutable reference wrapper for [`JiffZonedVec`] ALTREP data. Implements `TryFromSexp`, `Deref`, and `DerefMut`.

### `optionals::jiff_impl::JiffZonedVecRef`

Immutable reference wrapper for [`JiffZonedVec`] ALTREP data. Implements `TryFromSexp` and `Deref<Target = JiffZonedVec>`.

### `optionals::nalgebra_impl::RVecStorage`

Column-major R-backed storage for nalgebra matrices.

This type wraps an R SEXP (REALSXP, INTSXP, or RAWSXP) and implements
nalgebra's storage traits. The underlying data is R-allocated memory,
protected from garbage collection via `R_PreserveObject`.

#### Zero-Copy Guarantee

- **Input**: `TryFromSexp` wraps the R vector directly (no copy)
- **Output**: `IntoR` returns the underlying SEXP (no copy)
- **Compute**: all nalgebra operations read/write R memory directly

#### Thread Safety

`RVecStorage` is `!Send` and `!Sync` because accessing R memory requires
being on R's main thread. Functions using `RDVector`/`RDMatrix` automatically
route to the main thread via the `#[miniextendr]` macro.

#### ALTREP

If the input R vector is an ALTREP object, accessing its data pointer will
trigger materialization. This is unavoidable for contiguous memory access.

**Methods:**

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `from_sexp_matrix`

```rust
unsafe from_sexp_matrix(sexp: SEXP, nrows: usize, ncols: usize) -> Result<Self, SexpError>
```

Wrap an existing R SEXP as matrix storage (zero-copy).

##### Safety

Must be called on R's main thread.

#### `from_sexp_vector`

```rust
unsafe from_sexp_vector(sexp: SEXP) -> Result<Self, SexpError>
```

Wrap an existing R SEXP as a column vector storage (zero-copy).

##### Safety

Must be called on R's main thread.

#### `into_sexp`

```rust
into_sexp(self: Self, scope: &crate::gc_protect::ProtectScope) -> SEXP
```

Consume, transfer GC protection to the protect stack, and return the SEXP.

#### `into_sexp_unprotected`

```rust
unsafe into_sexp_unprotected(self: Self) -> SEXP
```

Consume, release GC protection, and return the raw SEXP.

##### Safety

The returned SEXP is **unprotected**. The caller must either return it
directly to R (`.Call` return) or protect it immediately.

#### `new_matrix`

```rust
unsafe new_matrix(nrows: usize, ncols: usize, init: impl FnOnce) -> Self
```

Allocate a new R matrix and initialize it.

##### Safety

Must be called on R's main thread.

#### `new_vector`

```rust
unsafe new_vector(len: usize, init: impl FnOnce) -> Self
```

Allocate a new R vector and initialize it.

##### Safety

Must be called on R's main thread.

### `optionals::ndarray_impl::RndMat`

An R matrix with zero-copy ndarray 2D view access.

`RndMat<T>` wraps an R matrix SEXP and provides `.view()` / `.view_mut()`
methods returning Fortran-order (column-major) `ArrayView2` / `ArrayViewMut2`.

#### Example

```rust,ignore
#[miniextendr]
fn matrix_trace(m: RndMat<f64>) -> f64 {
    m.view().diag().sum()  // zero-copy
}
```

**Methods:**

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: SEXP) -> Result<Self, SexpError>
```

Wrap an existing R matrix SEXP (zero-copy).

##### Safety

Must be called on R's main thread.

#### `into_sexp`

```rust
into_sexp(self: Self, scope: &crate::gc_protect::ProtectScope) -> SEXP
```

Consume, transfer GC protection to the protect stack, and return the SEXP.

#### `into_sexp_unprotected`

```rust
unsafe into_sexp_unprotected(self: Self) -> SEXP
```

Consume, release GC protection, and return the raw SEXP.

##### Safety

The returned SEXP is **unprotected**. See [`RndVec::into_sexp_unprotected`].

#### `ncol`

```rust
ncol(self: &Self) -> usize
```

Number of columns.

#### `new`

```rust
unsafe new(nrow: usize, ncol: usize, init: impl FnOnce) -> Self
```

Allocate a new R matrix and initialize it.

##### Safety

Must be called on R's main thread.

#### `nrow`

```rust
nrow(self: &Self) -> usize
```

Number of rows.

#### `view`

```rust
view(self: &Self) -> ArrayView2<'_, T>
```

Zero-copy Fortran-order (column-major) 2D view.

#### `view_mut`

```rust
view_mut(self: &mut Self) -> ArrayViewMut2<'_, T>
```

Zero-copy mutable Fortran-order 2D view.

#### `zeros`

```rust
unsafe zeros(nrow: usize, ncol: usize) -> Self
```

Allocate a zero-filled R matrix.

##### Safety

Must be called on R's main thread.

### `optionals::ndarray_impl::RndVec`

An R vector with zero-copy ndarray view access.

`RndVec<T>` wraps an R SEXP and provides `.view()` / `.view_mut()` methods
that return ndarray `ArrayView1` / `ArrayViewMut1` views directly over
R's memory. No data is copied on input or output.

#### GC Protection

Uses `R_PreserveObject` for GC protection — safe across `.Call` boundaries.

#### Thread Safety

`!Send` and `!Sync` — must be used on R's main thread.

#### Example

```rust,ignore
#[miniextendr]
fn double_vector(v: RndVec<f64>) -> RndVec<f64> {
    let input = v.view();
    let mut result = unsafe { RndVec::<f64>::new(input.len(), |s| s.fill(0.0)) };
    result.view_mut().assign(&(&input * 2.0));
    result  // zero-copy return
}
```

**Methods:**

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: SEXP) -> Result<Self, SexpError>
```

Wrap an existing R SEXP as an ndarray-compatible vector (zero-copy).

##### Safety

Must be called on R's main thread.

#### `into_sexp`

```rust
into_sexp(self: Self, scope: &crate::gc_protect::ProtectScope) -> SEXP
```

Consume, transfer GC protection to the protect stack, and return the SEXP.

The returned SEXP is protected on R's protect stack via the scope.
It remains protected until the scope is dropped.

#### `into_sexp_unprotected`

```rust
unsafe into_sexp_unprotected(self: Self) -> SEXP
```

Consume, release GC protection, and return the raw SEXP.

##### Safety

The returned SEXP is **unprotected**. The caller must either:
- Return it directly to R (R protects on receipt via `.Call`)
- Protect it immediately via `Rf_protect` or a `ProtectScope`

Any R allocation between this call and protection could trigger GC
and collect the returned SEXP.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Number of elements.

#### `new`

```rust
unsafe new(len: usize, init: impl FnOnce) -> Self
```

Allocate a new R vector and initialize it.

##### Safety

Must be called on R's main thread.

#### `view`

```rust
view(self: &Self) -> ArrayView1<'_, T>
```

Zero-copy read-only ndarray view.

#### `view_mut`

```rust
view_mut(self: &mut Self) -> ArrayViewMut1<'_, T>
```

Zero-copy mutable ndarray view.

#### `zeros`

```rust
unsafe zeros(len: usize) -> Self
```

Allocate a zero-filled R vector.

##### Safety

Must be called on R's main thread.

### `optionals::rand_impl::RRng`

A wrapper around R's random number generator that implements `rand::Rng`.

This allows using R's RNG with any `rand`-compatible code, ensuring
reproducibility when seeds are set via `set.seed()` in R.

#### Requirements

R's RNG state must be initialized before using this type. Either:
- Use `#[miniextendr(rng)]` attribute on the function
- Create an [`RngGuard`][crate::RngGuard] before using `RRng`

#### Example

```ignore
use miniextendr_api::rand_impl::RRng;
use rand::RngExt;

#[miniextendr(rng)]
fn random_sample(n: i32) -> Vec<f64> {
    let mut rng = RRng::new();
    // Generate n random f64 values in [0, 1)
    (0..n).map(|_| rng.random()).collect()
}
```

**Methods:**

#### `new`

```rust
new() -> Self
```

Create a new R RNG wrapper.

##### Safety Requirements

R's RNG state must have been initialized via `GetRNGstate()` before
calling any methods on this type. Use `#[miniextendr(rng)]` or
[`RngGuard`][crate::RngGuard] to ensure this.

### `optionals::rayon_bridge::RDataFrameBuilder`

Builder for assembling an R `data.frame` whose columns are filled in parallel.

This is the heterogeneous-column analogue of [`with_r_matrix`]: instead of one
homogeneous matrix, you declare a set of typed columns (each with its own
element type and fill closure) and the builder fills them all in **one flat
parallel pass** over `(column, row-range)` work-items.

#### Two axes of parallelism, one work-stealing pass

There are two ways to parallelise a column fill:

- **Column-granular** — one task per column. Fan-out width equals the column
  count, so a 3-column × 10M-row frame only ever uses 3 threads.
- **Row-slice-granular** — split *one* column into contiguous row ranges
  (what [`with_r_vec`] does internally). Great for one long column, but on
  its own it serialises across columns.

`RDataFrameBuilder` does **not** choose. [`build`][RDataFrameBuilder::build]
flattens the entire job into a single work-list of `(column_index, row-range)`
items — each native/character column is split into
`chunk_size = max(1, nrow / (current_num_threads() * 4))`-row chunks (the same
heuristic as [`with_r_vec`], with 4× oversubscription) — then runs **one**
`par_iter` over that flat list. Rayon's work-stealing balances both axes
automatically:

- **wide** (100 cols × short) → ~100+ items, column-dominated.
- **tall** (3 cols × 10M rows) → each column shatters into `~nthreads*4`
  chunks → hundreds of items, saturated even with 3 columns.
- **skewed** (1 huge col + many tiny) → the huge column's chunks get stolen
  by threads idle after finishing the tiny columns.

This also avoids the per-column barrier and repeated pool spin-up that the
naive "fill each column, each internally parallel" (nested `par_iter`) shape
would cause.

#### Phases

1. Allocate each column's backing storage **serially on the R/worker thread**
   (native columns get a protected R vector; character columns get an owned
   `Vec<Option<String>>`). Strict PROTECT discipline — the dangerous part.
2. Fill all columns in **one flat parallel pass**. No R API calls happen
   inside the parallel region.
3. Set character `CHARSXP`s serially on the R thread (CHARSXP allocation is
   forbidden on rayon threads), then assemble the `VECSXP`, `names`, compact
   `row.names` (`c(NA_integer_, -nrow)`), and `class = "data.frame"`.

#### Column kinds

- [`column::<T>`][RDataFrameBuilder::column] — a native-typed column
  (`f64`/`i32`/`RLogical`/`u8`/`Rcomplex`). The fill closure receives a
  mutable chunk and its offset, exactly like [`with_r_vec`]. The buffer is R
  memory, filled directly with zero intermediate allocation.
- [`column_str`][RDataFrameBuilder::column_str] — a character (`STRSXP`)
  column. The per-row `Option<String>` values are computed **in parallel**
  (contributing chunks to the same flat work-list as native columns), but the
  `CHARSXP`s are set **serially** afterward. `None` becomes `NA_character_`.

#### Example

```ignore
use miniextendr_api::rayon_bridge::RDataFrameBuilder;

let df: SEXP = RDataFrameBuilder::new(1000)
    .column::<f64>("x", |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = ((offset + i) as f64).sqrt();
        }
    })
    .column::<i32>("y", |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = (offset + i) as i32;
        }
    })
    .column_str("label", |i| Some(format!("row_{i}")))
    .build();
```

#### Safety argument (disjoint mutation, no aliasing)

The flat work-list never produces two items that overlap:

- Different columns address **different** backing buffers (distinct R vectors
  / distinct `Vec`s), so cross-column items are trivially disjoint.
- Within a column, the row ranges are a partition of `[0, nrow)` produced by
  chunking `nrow` into fixed-size, non-overlapping spans. Each `(offset, len)`
  item therefore owns a unique slice of that column's buffer.

Each `RangeFiller` reconstitutes its slice via
`slice::from_raw_parts_mut(base.add(offset), len)` and writes only that span.
Because the spans are disjoint, no two threads ever form overlapping `&mut`
references — there is no aliasing UB even though the work-list shares the raw
base pointers (`ColPtr`, `Send + Sync`).

#### Protection

Every native column SEXP is PROTECTed from allocation through insertion into
the `VECSXP`; the `names` / `row.names` / class transients are likewise
protected across each subsequent allocation. After
[`build`][RDataFrameBuilder::build] returns, the resulting data.frame SEXP is
unprotected and becomes the caller's responsibility (return it from a
`#[miniextendr]` fn, or PROTECT it).

**Methods:**

#### `build`

```rust
build(self: Self) -> crate::dataframe::DataFrame
```

Allocate, fill, and assemble the [`DataFrame`](crate::dataframe::DataFrame).

Flattens every column into a single `(column_index, row-range)` work-list
and runs one parallel pass over it (see the type-level docs for the
scheduling argument), then assembles the `data.frame` on the R thread.

#### `column`

```rust
column<T>(self: Self, name: impl Into<String>, f: impl Fn + Send + Sync) -> Self
```

Add a native-typed column (`f64`/`i32`/`RLogical`/`u8`/`Rcomplex`).

The fill closure `f(chunk, offset)` is dispatched in parallel over chunks
of the (already-allocated) R column buffer, identical in shape to
[`with_r_vec`]. Chunk boundaries are deterministic for a given `nrow` and
thread count.

#### `column_str`

```rust
column_str(self: Self, name: impl Into<String>, f: impl Fn + Send + Sync) -> Self
```

Add a character (`STRSXP`) column.

The fill closure `f(i)` returns the value for row `i` as `Option<String>`,
where `None` maps to `NA_character_`. Values are computed in parallel
(contributing chunks to the same flat work-list as native columns), then
set into the R `STRSXP` serially on the R thread (CHARSXP allocation
cannot happen on Rayon threads).

#### `new`

```rust
new(nrow: usize) -> Self
```

Start building a data.frame with `nrow` rows.

### `optionals::regex_impl::CaptureGroups`

Wrapper for regex capture groups.

This type wraps `regex::Captures` for access from R.
It holds owned copies of capture group strings for safe access.

**Methods:**

#### `capture`

```rust
capture(re: &Regex, text: &str) -> Option<Self>
```

Create capture groups from a regex and text.

### `optionals::serde_impl::JsonOptions`

Options for converting R objects to JSON.

#### Example

```rust,ignore
use miniextendr_api::serde_impl::{JsonOptions, NaHandling, SpecialFloatHandling};

let opts = JsonOptions::default()
    .na(NaHandling::String("NA".into()))
    .nan(SpecialFloatHandling::Null)
    .inf(SpecialFloatHandling::String);

let json = json_from_sexp_with(sexp, &opts)?;
```

**Fields:**

- `na`: `NaHandling`
  - How to handle NA values.
- `nan`: `SpecialFloatHandling`
  - How to handle NaN values.
- `inf`: `SpecialFloatHandling`
  - How to handle Inf/-Inf values.
- `factor`: `FactorHandling`
  - How to serialize factors.

**Methods:**

#### `factor`

```rust
factor(self: Self, handling: FactorHandling) -> Self
```

Set factor handling.

#### `inf`

```rust
inf(self: Self, handling: SpecialFloatHandling) -> Self
```

Set Inf handling.

#### `na`

```rust
na(self: Self, handling: NaHandling) -> Self
```

Set NA handling.

#### `nan`

```rust
nan(self: Self, handling: SpecialFloatHandling) -> Self
```

Set NaN handling.

#### `new`

```rust
new() -> Self
```

Create new options with defaults (NA→null, NaN/Inf→error, factors→labels).

#### `permissive`

```rust
permissive() -> Self
```

Create permissive options (all special values become null).

#### `strict`

```rust
strict() -> Self
```

Create strict options (all special values cause errors).

### `panic_telemetry::PanicReport`

A structured panic report passed to the telemetry hook.

**Fields:**

- `message`: `&'a str`
  - The panic message (extracted from the panic payload).
- `source`: `PanicSource`
  - Which panic→R-error boundary caught this panic.

### `protect_pool::ProtectKey`

Generational key for a slot in a [`ProtectPool`].

Contains a slot index and a generation counter. If a slot is released and
reused, the old key's generation won't match and operations will safely
return `None` or no-op.

8 bytes: 4-byte slot index + 4-byte generation.

### `protect_pool::ProtectPool`

A VECSXP-backed pool for GC protection with generational keys.

#### Example

```ignore
let mut pool = unsafe { ProtectPool::new(16) };

let key = unsafe { pool.insert(some_sexp) };
// SEXP is now protected from GC

let sexp = pool.get(key).unwrap();
// Use the SEXP...

unsafe { pool.release(key) };
// SEXP is no longer protected (eligible for GC)
```

**Methods:**

#### `capacity`

```rust
capacity(self: &Self) -> usize
```

Current capacity of the backing VECSXP.

#### `contains_key`

```rust
contains_key(self: &Self, key: ProtectKey) -> bool
```

Check if a key is currently valid (not stale).

#### `get`

```rust
get(self: &Self, key: ProtectKey) -> Option<SEXP>
```

Get the SEXP for a key, or `None` if the key is stale.

#### `insert`

```rust
unsafe insert(self: &mut Self, sexp: SEXP) -> ProtectKey
```

Protect a SEXP, returning a generational key.

The SEXP will be protected from GC until [`release`](Self::release) is called
with the returned key. If the key is dropped without calling `release`, the
SEXP remains protected (leak, not crash).

##### Safety

Must be called from the R main thread. `sexp` must be a valid SEXP.

##### Panics

Panics if the pool has grown beyond `u32::MAX` slots.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Whether the pool is empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Number of currently protected objects.

#### `new`

```rust
unsafe new(capacity: usize) -> Self
```

Create a new pool with the given initial capacity.

##### Safety

Must be called from the R main thread.

#### `release`

```rust
unsafe release(self: &mut Self, key: ProtectKey)
```

Release a previously protected SEXP.

If the key is stale (already released, or from a different pool), this is a no-op.

##### Safety

Must be called from the R main thread.

#### `replace`

```rust
unsafe replace(self: &mut Self, key: ProtectKey, sexp: SEXP) -> bool
```

Overwrite the SEXP at an existing key without releasing/reinserting.

Returns `true` if the key was valid and the value was replaced.
Returns `false` if the key was stale (no-op).

This is the pool equivalent of `R_Reprotect` — O(1), no allocation.

##### Safety

Must be called from the R main thread. `sexp` must be a valid SEXP.

#### `with_capacity`

```rust
unsafe with_capacity(capacity: usize) -> Self
```

Create a new pool with a specific initial capacity.

##### Safety

Must be called from the R main thread.

##### Panics

Panics if `capacity` exceeds `R_xlen_t::MAX` or `u32::MAX`.

### `pump::WorkerPump`

Runs a worker thread in parallel with a main-thread pump loop.

See [the module documentation][self] for the longjmp-safety contract and a
usage example.

**Methods:**

#### `channel_capacity`

```rust
channel_capacity(self: Self, n: usize) -> Self
```

Set the capacity of the bounded MPSC channel.

The default is 64.  A larger capacity allows the worker to get further
ahead of the pump; a capacity of 0 makes every send synchronous
(rendezvous channel).

When the channel is full the worker blocks on `tx.send` until the pump
drains a slot.  If the pump panics or a longjmp fires, `rx` is dropped
as part of scope unwinding, which unblocks `tx.send` with an `Err` and
lets the worker exit cleanly.

#### `drain_logs_each_tick`

```rust
drain_logs_each_tick(self: Self, on: bool) -> Self
```

Control whether the cross-thread log queue is drained on every pump tick.

Default: `true`.  Set to `false` if the consumer manages its own log
drain cadence (e.g. it calls `drain_log_queue()` explicitly at
coarser granularity).

Has no effect when the `log` feature is disabled.

#### `new`

```rust
new() -> Self
```

Create a new `WorkerPump` with default settings.

Defaults:
- channel capacity: 64
- `drain_logs_each_tick`: `true`

#### `run`

```rust
run<R, W, P>(self: Self, worker: W, pump: P) -> Result<R, WorkerError>
```

Run the worker/pump pair and return the worker's result.

- `worker` runs on a scoped background thread.  It receives a
  [`SyncSender<T>`] and sends messages to the pump.  When `worker`
  returns (success or error) it should drop `tx`; the pump's receive
  loop then terminates naturally.
- `pump` is called on the **current (main R) thread** for every message
  the worker sends.

`run` returns `Ok(R)` on success, or `Err` if the worker returned an
error or panicked.

##### Panics

If the worker thread panics, `run` returns
`Err("WorkerPump worker panicked")`.

If the pump closure panics, the panic propagates normally through
`thread::scope`'s `Drop` (which joins the worker), and then out of
`run`.  When called from inside an `#[miniextendr]` body the outer
`R_UnwindProtect` catches it and converts it to an R error.

### `rarray::RArray`

An N-dimensional R array.

This type wraps an R array SEXP. The dimension count `NDIM` is tracked
at compile time, but dimension sizes are read from the R object.

#### Type Parameters

- `T`: The element type, must implement [`RNativeType`]
- `NDIM`: The number of dimensions (compile-time constant)

#### Thread Safety

This type is `!Send` and `!Sync` because its methods require access to
R APIs that must run on the R main thread.

**Methods:**

#### `as_sexp`

```rust
const as_sexp(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `as_slice`

```rust
unsafe as_slice(self: &Self) -> &[T]
```

Get the data as a slice (column-major order).

##### Safety

The SEXP must be protected and valid.

#### `as_slice_mut`

```rust
unsafe as_slice_mut(self: &mut Self) -> &mut [T]
```

Get the data as a mutable slice (column-major order).

##### Safety

- The SEXP must be protected and valid
- No other references to the data may exist

#### `column`

```rust
unsafe column(self: &Self, col: usize) -> &[T]
```

Get a column as a slice.

##### Safety

The SEXP must be protected and valid.

#### `column_mut`

```rust
unsafe column_mut(self: &mut Self, col: usize) -> &mut [T]
```

Get a mutable column as a slice.

Columns are contiguous in R's column-major layout, so this returns
a proper `&mut [T]` without any striding.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if `col >= ncol`.

#### `dim`

```rust
unsafe dim(self: &Self, dim: usize) -> usize
```

Get a specific dimension size.

##### Safety

The SEXP must be valid.

##### Panics

Panics if `dim >= NDIM`.

#### `dims`

```rust
unsafe dims(self: &Self) -> [usize; NDIM]
```

Get the dimensions as an array.

##### Safety

The SEXP must be valid.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: SEXP) -> Result<Self, SexpError>
```

Create an RArray from a SEXP, validating type and dimensions.

##### Safety

The SEXP must be protected from GC for the lifetime of the returned RArray.

##### Errors

Returns an error if:
- The SEXP type doesn't match `T::SEXP_TYPE`
- The dim attribute has wrong number of dimensions

#### `from_sexp_unchecked`

```rust
const unsafe from_sexp_unchecked(sexp: SEXP) -> Self
```

Create an RArray from a SEXP without validation.

##### Safety

- The SEXP must be protected from GC
- The SEXP must have the correct type for `T`
- The SEXP must have exactly `NDIM` dimensions

#### `get`

```rust
unsafe get(self: &Self, indices: [usize; NDIM]) -> T
```

Get an element by N-dimensional indices.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any index is out of bounds.

#### `get_class`

```rust
unsafe get_class(self: &Self) -> Option<SEXP>
```

Get the `class` attribute if present.

Equivalent to R's `GET_CLASS(x)`.

##### Safety

The SEXP must be valid.

#### `get_colnames`

```rust
unsafe get_colnames(self: &Self) -> Option<SEXP>
```

Get column names from the `dimnames` attribute.

Equivalent to R's `GET_COLNAMES(x)` / `Rf_GetColNames(x)`.

##### Safety

The SEXP must be valid.

#### `get_dimnames`

```rust
unsafe get_dimnames(self: &Self) -> Option<SEXP>
```

Get the `dimnames` attribute if present.

Equivalent to R's `GET_DIMNAMES(x)`.

##### Safety

The SEXP must be valid.

#### `get_names`

```rust
unsafe get_names(self: &Self) -> Option<SEXP>
```

Get the `names` attribute if present.

Equivalent to R's `GET_NAMES(x)`.

##### Safety

The SEXP must be valid.

#### `get_rc`

```rust
unsafe get_rc(self: &Self, row: usize, col: usize) -> T
```

Get an element by row and column.

##### Safety

The SEXP must be protected and valid.

#### `get_rownames`

```rust
unsafe get_rownames(self: &Self) -> Option<SEXP>
```

Get row names from the `dimnames` attribute.

Equivalent to R's `GET_ROWNAMES(x)` / `Rf_GetRowNames(x)`.

##### Safety

The SEXP must be valid.

#### `into_inner`

```rust
into_inner(self: Self) -> SEXP
```

Consume and return the underlying SEXP.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if the array is empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Get the total number of elements.

#### `linear_index`

```rust
unsafe linear_index(self: &Self, indices: [usize; NDIM]) -> usize
```

Convert N-dimensional indices to linear index (column-major).

##### Safety

The SEXP must be valid (needed to read dims).

##### Panics

Panics if any index is out of bounds.

#### `ncol`

```rust
unsafe ncol(self: &Self) -> usize
```

Get the number of columns.

##### Safety

The SEXP must be valid.

#### `new`

```rust
unsafe new<F>(dims: [usize; NDIM], init: F) -> Self
```

Allocate a new R array with the given dimensions.

The array is allocated. The closure receives a mutable slice to
initialize the data.

##### Safety

Must be called from the R main thread (or via routed FFI).
The returned RArray holds an unprotected SEXP - caller must protect.

##### Example

```ignore
let matrix = unsafe {
    RMatrix::<f64>::new([3, 4], |slice| {
        for (i, v) in slice.iter_mut().enumerate() {
            *v = i as f64;
        }
    })
};
```

#### `nrow`

```rust
unsafe nrow(self: &Self) -> usize
```

Get the number of rows.

##### Safety

The SEXP must be valid.

#### `set`

```rust
unsafe set(self: &mut Self, indices: [usize; NDIM], value: T)
```

Set an element by N-dimensional indices.

##### Safety

- The SEXP must be protected and valid
- No other references to the data may exist

##### Panics

Panics if any index is out of bounds.

#### `set_class`

```rust
unsafe set_class(self: &mut Self, class: SEXP)
```

Set the `class` attribute.

Equivalent to R's `SET_CLASS(x, n)`.

##### Safety

The SEXP must be valid and not shared.

#### `set_dimnames`

```rust
unsafe set_dimnames(self: &mut Self, dimnames: SEXP)
```

Set the `dimnames` attribute.

Equivalent to R's `SET_DIMNAMES(x, n)`.

##### Safety

The SEXP must be valid and not shared.

#### `set_names`

```rust
unsafe set_names(self: &mut Self, names: SEXP)
```

Set an arbitrary attribute by symbol (unchecked internal helper).

##### Safety

Set the `names` attribute.

Equivalent to R's `SET_NAMES(x, n)`.

##### Safety

The SEXP must be valid and not shared.

#### `set_rc`

```rust
unsafe set_rc(self: &mut Self, row: usize, col: usize, value: T)
```

Set an element by row and column.

##### Safety

- The SEXP must be protected and valid
- No other references to the data may exist

#### `to_vec`

```rust
unsafe to_vec(self: &Self) -> Vec<T>
```

Copy array data to an owned `Vec<T>`.

This method copies the data, making it safe to use in worker threads
or pass to parallel computation. The copy is performed on the current
thread (which must be the R main thread).

##### Safety

The SEXP must be protected and valid.

##### Example

```ignore
use miniextendr_api::rarray::RMatrix;

#[miniextendr]
fn process_matrix(m: RMatrix<f64>) -> f64 {
    // Copy data - Vec<f64> is Send and can be used in worker threads
    let data: Vec<f64> = unsafe { m.to_vec() };
    // Now data can be passed to parallel computation
    data.iter().sum()
}
```

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<i8>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<i16>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<i64>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<isize>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<u16>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<u32>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<u64>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<usize>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<f32>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<bool>
```

Copy array data to an owned `Vec`, coercing from the R native type.

##### Safety

The SEXP must be protected and valid.

##### Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `zeros`

```rust
unsafe zeros(dims: [usize; NDIM]) -> Self
```

Allocate a new R array filled with zeros.

##### Safety

Must be called from the R main thread (or via routed FFI).
The returned RArray holds an unprotected SEXP - caller must protect.

### `raw_conversions::Raw`

Wrapper for a single POD value (headerless, native layout).

Use this for fast serialization when portability is not needed.

**Methods:**

#### `inner`

```rust
inner(self: &Self) -> &T
```

Get a reference to the inner value.

#### `into_inner`

```rust
into_inner(self: Self) -> T
```

Unwrap the inner value.

### `raw_conversions::RawHeader`

Header for tagged raw format.

Layout: magic (4 bytes) + version (4 bytes) + elem_size (4 bytes) + elem_count (4 bytes)

**Fields:**

- `magic`: `[u8; 4]`
  - Magic bytes: "MXRB"
- `version`: `u32`
  - Format version (currently 1)
- `elem_size`: `u32`
  - Size of each element in bytes
- `elem_count`: `u32`
  - Number of elements

**Methods:**

#### `new_single`

```rust
new_single<T>() -> Self
```

Create a new header for a single element.

#### `new_slice`

```rust
new_slice<T>(count: usize) -> Self
```

Create a new header for a slice.

#### `validate`

```rust
validate<T>(self: &Self, expected_count: Option<usize>) -> Result<(), RawError>
```

Validate header.

### `raw_conversions::RawSlice`

Wrapper for a slice of POD values (headerless, native layout).

Use this for fast serialization when portability is not needed.

**Methods:**

#### `inner`

```rust
inner(self: &Self) -> &[T]
```

Get a reference to the inner vector.

#### `into_inner`

```rust
into_inner(self: Self) -> Vec<T>
```

Unwrap the inner vector.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Get the number of elements.

### `raw_conversions::RawSliceTagged`

Wrapper for a slice of POD values with header metadata.

The tagged format includes a header with magic bytes, version, and size info
for safer decoding across sessions.

**Methods:**

#### `inner`

```rust
inner(self: &Self) -> &[T]
```

Get a reference to the inner vector.

#### `into_inner`

```rust
into_inner(self: Self) -> Vec<T>
```

Unwrap the inner vector.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if empty.

#### `len`

```rust
len(self: &Self) -> usize
```

Get the number of elements.

### `raw_conversions::RawTagged`

Wrapper for a single POD value with header metadata.

The tagged format includes a header with magic bytes, version, and size info
for safer decoding across sessions.

**Methods:**

#### `inner`

```rust
inner(self: &Self) -> &T
```

Get a reference to the inner value.

#### `into_inner`

```rust
into_inner(self: Self) -> T
```

Unwrap the inner value.

### `rcow::RBorrow`

Borrowed arm of [`RCow`]: a whole-vector view that remembers its source SEXP.

Fields are private by design — the only constructor is
[`RCow::try_from_sexp`], so a borrowed view can never be a sub-slice. That
invariant is what lets [`IntoR`] return the source SEXP zero-copy without the
provenance-free pointer probe that `Cow<[T]>` required (#880).

**Methods:**

#### `as_slice`

```rust
as_slice(self: &Self) -> &[T]
```

The borrowed view (the whole source vector).

#### `source_sexp`

```rust
source_sexp(self: &Self) -> SEXP
```

The source R vector this view borrows from.

### `refcount_protect::Arena`

A reference-counted arena for GC protection, generic over map type.

This provides an alternative to R's PROTECT stack that:
- Uses reference counting for each SEXP
- Allows releasing protections in any order
- Has no stack size limit (uses heap allocation)

#### Type Aliases

- [`RefCountedArena`] = `Arena<BTreeMap<...>>` (ordered, good for ref counting)
- [`HashMapArena`] = `Arena<HashMap<...>>` (faster for large collections)

**Methods:**

#### `capacity`

```rust
capacity(self: &Self) -> usize
```

Get the current capacity.

#### `clear`

```rust
unsafe clear(self: &Self)
```

Clear all protections.

##### Safety

Must be called from the R main thread.

#### `guard`

```rust
unsafe guard(self: &Self, x: SEXP) -> ArenaGuard<'_, M>
```

Protect a SEXP and return an RAII guard.

##### Safety

Must be called from the R main thread.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if the arena is empty.

#### `is_protected`

```rust
is_protected(self: &Self, x: SEXP) -> bool
```

Check if a SEXP is currently protected by this arena.

#### `len`

```rust
len(self: &Self) -> usize
```

Get the number of distinct SEXPs currently protected.

#### `new`

```rust
unsafe new() -> Self
```

Create a new arena with default capacity (16 slots).

For workloads protecting many distinct SEXPs (e.g., ppsize-scale loops),
prefer [`with_capacity`](Self::with_capacity) to avoid backing VECSXP
growth and map rehashing during operation.

##### Safety

Must be called from the R main thread.

#### `protect`

```rust
unsafe protect(self: &Self, x: SEXP) -> SEXP
```

Protect a SEXP, incrementing its reference count.

##### Safety

Must be called from the R main thread.

#### `ref_count`

```rust
ref_count(self: &Self, x: SEXP) -> usize
```

Get the reference count for a SEXP (0 if not protected).

#### `try_unprotect`

```rust
unsafe try_unprotect(self: &Self, x: SEXP) -> bool
```

Try to unprotect a SEXP, returning `true` if it was protected.

##### Safety

Must be called from the R main thread.

#### `unprotect`

```rust
unsafe unprotect(self: &Self, x: SEXP)
```

Unprotect a SEXP, decrementing its reference count.

##### Safety

Must be called from the R main thread.

##### Panics

Panics if `x` was not protected by this arena.

#### `with_capacity`

```rust
unsafe with_capacity(capacity: usize) -> Self
```

Create a new arena with specific initial capacity.

Pre-sizing the arena avoids growth of the backing VECSXP and rehashing
of the internal map. Use this when the expected number of distinct
protected values is known or can be estimated.

##### Safety

Must be called from the R main thread.

### `refcount_protect::ArenaGuard`

An RAII guard that unprotects a SEXP when dropped.

**Methods:**

#### `get`

```rust
get(self: &Self) -> SEXP
```

Returns the protected SEXP.

#### `new`

```rust
unsafe new(arena: &'a Arena<M>, sexp: SEXP) -> Self
```

Create a new guard that protects the SEXP and unprotects on drop.

##### Safety

Must be called from the R main thread. The SEXP must be valid.

### `refcount_protect::ThreadLocalArena`

Thread-local BTreeMap-based arena.

This provides the lowest overhead for protection operations by
eliminating RefCell borrow checking.

### `refcount_protect::ThreadLocalHashArena`

Thread-local HashMap-based arena.

Combines HashMap's performance for large collections with
thread-local storage's low overhead.

### `registry::AltrepRegistration`

ALTREP class registration entry: fn pointer + `#[no_mangle]` symbol name.

See [`MX_ALTREP_REGISTRATIONS`] for context.

**Fields:**

- `register`: `{'function_pointer': {'sig': {'inputs': [], 'output': None, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': False, 'is_async': False, 'abi': {'C': {'unwind': False}}}}}`
  - Registration function called once at `R_init_*`.
- `symbol`: `&'static str`
  - Symbol name of `register` (e.g. `"__mx_altrep_reg_MyType"`). Consumed by

### `registry::ClassNameEntry`

Entry mapping a Rust type name to its R-visible class name and class system.

Emitted by every `#[miniextendr(env|r6|s3|s4|s7|vctrs)]` impl block.
Used by the resolver in `write_r_wrappers_to_file` to replace
`.__MX_CLASS_REF_<RustName>__` placeholders with the actual R class name.

**Fields:**

- `rust_type`: `&'static str`
  - Rust type identifier, e.g. `"S7Shape"`.
- `r_class_name`: `&'static str`
  - R-visible class name. Equals `rust_type` unless `class = "Override"` was
- `class_system`: `&'static str`
  - Class system tag: `"env"` | `"r6"` | `"s3"` | `"s4"` | `"s7"` | `"vctrs"`.

### `registry::MatchArgChoicesEntry`

Entry for replacing match_arg placeholder defaults with actual choices.

**Fields:**

- `placeholder`: `&'static str`
  - Placeholder string in the R formal default, e.g. `".__MX_MATCH_ARG_CHOICES_mode__"`.
- `choices_str`: `{'function_pointer': {'sig': {'inputs': [], 'output': {'resolved_path': {'path': 'String', 'id': 26, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': False, 'is_async': False, 'abi': 'Rust'}}}`
  - Function that returns the choices as a comma-separated quoted string,
- `preferred_default`: `&'static str`
  - User-supplied `default = "..."` value (unquoted, e.g. `"zstd"`), or `""`

### `registry::MatchArgParamDocEntry`

Entry for replacing match_arg `@param` doc placeholders with human-readable
choice descriptions.

**Fields:**

- `placeholder`: `&'static str`
  - Placeholder string in the `@param` roxygen tag, e.g.
- `several_ok`: `bool`
  - `true` for `several_ok` params (emits "One or more of …");
- `choices_str`: `{'function_pointer': {'sig': {'inputs': [], 'output': {'resolved_path': {'path': 'String', 'id': 26, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': False, 'is_async': False, 'abi': 'Rust'}}}`
  - Function that returns the choices as a comma-separated quoted string,

### `registry::RWrapperEntry`

R wrapper code with priority for ordering.

**Fields:**

- `priority`: `RWrapperPriority`
  - Ordering priority (lower = earlier in output file).
- `content`: `&'static str`
  - R source code fragment.
- `source_file`: `&'static str`
  - Source file path (from `file!()`). Used to derive a default `@rdname`

### `registry::SidecarPropEntry`

Entry documenting a sidecar (`#[r_data]`) property on an S7 ExternalPtr type.

Emitted by `#[derive(ExternalPtr)] #[externalptr(s7)]` for each public `#[r_data]`
field. Used by `write_r_wrappers_to_file` to substitute the
`.__MX_S7_SIDECAR_PROP_DOCS_<TypeName>__` placeholder with `#' @prop` lines.

**Fields:**

- `rust_type`: `&'static str`
  - Rust type name, e.g. `"SidecarS7"`.
- `field_name`: `&'static str`
  - Field name, e.g. `"prop_int"`.
- `prop_doc`: `&'static str`
  - Documentation string for this property.

### `registry::TraitDispatchEntry`

Trait dispatch entry mapping (concrete_tag, trait_tag) → vtable.

**Fields:**

- `concrete_tag`: `crate::abi::mx_tag`
  - Tag identifying the concrete type.
- `trait_tag`: `crate::abi::mx_tag`
  - Tag identifying the trait interface.
- `vtable`: `*const std::os::raw::c_void`
  - Pointer to the trait's vtable (cast from `&'static SomeVTable`).
- `vtable_symbol`: `&'static str`
  - Symbol name of the `#[no_mangle]` vtable static

### `rng::RngGuard`

RAII guard for R's RNG state.

Calls `GetRNGstate()` on creation and `PutRNGstate()` on drop.
This ensures RNG state is properly saved even if the function panics
or returns early.

#### Example

```ignore
use miniextendr_api::rng::RngGuard;
use miniextendr_api::sys::unif_rand;

fn generate_uniform() -> f64 {
    let _guard = RngGuard::new();
    unsafe { unif_rand() }
}
```

#### Warning: R Longjumps

This guard relies on Rust's drop semantics. If R triggers a longjmp
(via `Rf_error` etc.), the destructor will NOT run unless the code
is wrapped in `with_r_unwind_protect`. For functions exposed to R,
prefer using `#[miniextendr(rng)]` which handles this correctly.

#### Safety

Must be used on R's main thread. The guard assumes it has exclusive
access to R's RNG state while alive.

**Methods:**

#### `new`

```rust
new() -> Self
```

Create a new RNG guard, loading the current RNG state.

Calls `GetRNGstate()` to load R's `.Random.seed` into the RNG.

##### Safety

Must be called from R's main thread.

### `serde::columnar::DispatchNames`

Custom slot names for [`dispatch_to_dataframes`]'s output list.

Defaults to `ok = "ok"`, `err = "err"`. Override either or both via
`DispatchNames { ok: "results".into(), err: "errors".into() }`.

**Fields:**

- `ok`: `String`
- `err`: `String`

### `serde::columnar::NamedDataFrameListBuilder`

Assemble a named list whose elements are [`DataFrame`]s,
without per-result `OwnedProtect` bookkeeping.

#### Why this is distinct from [`DataFrame::builder`]

[`DataFrame::builder`](crate::dataframe::DataFrame::builder) and
[`SerdeRowBuilder`] both produce a *single* [`DataFrame`]. This builder
produces a different shape — a named *list of* data.frames, e.g.
`list(results = df, error = df)` — so it deliberately keeps its own name
rather than folding into the `DataFrame::builder` vocabulary. Its inputs
are [`DataFrame`]s (typically from [`vec_to_dataframe`]); its output is a
[`List`](crate::list::List).

Each [`push`](NamedDataFrameListBuilder::push) protects the input
data.frame's SEXP via an internal [`ProtectScope`](crate::ProtectScope);
[`build`](NamedDataFrameListBuilder::build) consumes the builder and emits
a named list via [`List::from_raw_pairs`](crate::list::List::from_raw_pairs).
The scope drops at the end of `build`, releasing the per-input protects —
by which point the children are reachable from the assembled list.

#### Example

```ignore
let result = NamedDataFrameListBuilder::new()
    .push("results", vec_to_dataframe(&oks)?)
    .push("error",   vec_to_dataframe(&errs)?)
    .build();
```

**Methods:**

#### `build`

```rust
build(self: Self) -> crate::list::List
```

Consume the builder and return the assembled named [`List`](crate::list::List).

The returned `List`'s SEXP is *not* separately protected on return — the
caller takes responsibility for protection (typically by immediately
handing it back to R via the `.Call` return path). This matches the
contract of [`List::from_raw_pairs`](crate::list::List::from_raw_pairs).

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Whether no entries have been pushed yet.

#### `len`

```rust
len(self: &Self) -> usize
```

Number of entries pushed so far.

#### `new`

```rust
new() -> Self
```

Create an empty builder.

##### Safety (caller)

Must be called from the R main thread. The internal
[`ProtectScope`](crate::ProtectScope) carries `!Send + !Sync`
so the builder cannot be moved to another thread.

#### `push`

```rust
push<S>(self: Self, name: S, df: DataFrame) -> Self
```

Append a named data.frame. The input's SEXP is protected
internally for the lifetime of the builder.

#### `with_capacity`

```rust
with_capacity(n: usize) -> Self
```

Create a builder pre-allocated for `n` entries.

Equivalent to [`new`](Self::new) but avoids repeated re-allocations
when the number of partitions is known up front.

### `serde::columnar::SerdeRowBuilder`

Builder for incremental data.frame assembly.

Three schema modes:

1. **Default** ([`SerdeRowBuilder::new`]) — schema discovered from the
   first [`push`](Self::push); subsequent rows that introduce new fields
   are rejected.
2. **Pre-declared** ([`SerdeRowBuilder::with_schema`]) — schema fixed at
   construction; first push skips discovery; later pushes must conform.
3. **Growing** ([`SerdeRowBuilder::grow_schema`]) — new fields seen in
   later rows are added on-the-fly and back-filled with NA on prior rows.
   Composes with [`with_schema`](Self::with_schema) to start from a
   declared partial schema.

Call [`finish`](Self::finish) to produce the [`DataFrame`].

Use [`iter_to_dataframe`] when an iterator suffices; reach for this when
you need explicit control over push points (conditional skipping,
streaming from multiple sources, custom NA strategies).

#### Examples

```rust,ignore
# use miniextendr_api::serde::{SerdeRowBuilder, TypeSpec};
# use serde::Serialize;
#[derive(Serialize)]
struct Row { id: i32, label: Option<String> }

// Pre-declared schema. Optional(Character) keeps the column character-typed
// even if the first row's label is None.
let mut b = SerdeRowBuilder::<Row>::with_schema(
    [
        ("id", TypeSpec::Integer),
        ("label", TypeSpec::Optional(Box::new(TypeSpec::Character))),
    ],
    None,
);
b.push(Row { id: 1, label: None }).unwrap();
b.push(Row { id: 2, label: Some("two".into()) }).unwrap();
let df = b.finish().unwrap();
```

**Methods:**

#### `finish`

```rust
finish(self: Self) -> Result<DataFrame, RSerdeError>
```

Consume the builder and produce the data.frame.

An empty builder produces an empty 0-row 0-column data.frame
(matching `vec_to_dataframe(&[])`).

#### `grow_schema`

```rust
grow_schema(self: Self) -> Self
```

Enable growing-schema mode: new fields discovered in later rows are
added on the fly and back-filled with NA on prior rows.

Composes with [`with_schema`](Self::with_schema) — call
`SerdeRowBuilder::with_schema(...).grow_schema()` to start with a
declared partial schema and let new fields appear as rows arrive.

Cost: O(new_fields × existing_nrow) on each push that introduces a
new field. For row-by-row growing types this is amortised
O(nrow × ncols) — the same shape as `vec_to_dataframe` today.

**Type clashes**: a later row writing a `String` to a column whose
first-seen value was an `Integer` follows today's union-path
behaviour — the value is coerced or NA-filled by
`ColumnBuffer::push_value`. No new error is raised. If your data
is genuinely heterogeneous, declare the column as
`TypeSpec::Generic` to get a list-column.

##### Examples

```rust,ignore
# use miniextendr_api::serde::SerdeRowBuilder;
# use std::collections::BTreeMap;
// Heterogeneous rows: each row is a map; later rows introduce new keys.
let mut b = SerdeRowBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();

let r1: BTreeMap<String, i32> = [("a".into(), 1)].into_iter().collect();
let r2: BTreeMap<String, i32> = [("a".into(), 2), ("b".into(), 3)].into_iter().collect();
b.push(r1).unwrap();
b.push(r2).unwrap();  // adds column "b", back-fills NA on row 0
let df = b.finish().unwrap();
```

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Whether no rows have been pushed yet.

#### `len`

```rust
len(self: &Self) -> usize
```

Number of rows pushed so far.

#### `new`

```rust
new(nrow_hint: Option<usize>) -> Self
```

Create a new builder with schema discovered on first [`push`](Self::push).

`nrow_hint` pre-sizes column buffers; `None` is acceptable.

#### `push`

```rust
push(self: &mut Self, row: T) -> Result<(), RSerdeError>
```

Append a row.

In default mode the first call discovers the schema. In
[`with_schema`](Self::with_schema) mode the schema is fixed at
construction. In [`grow_schema`](Self::grow_schema) mode each push
also runs a discovery pass and absorbs any new fields, back-filling
NA on prior rows.

#### `with_schema`

```rust
with_schema<S, I>(schema: I, nrow_hint: Option<usize>) -> Self
```

Create a builder with a pre-declared flat schema.

Skips the first-row discovery pass. All later pushes are validated
against this schema by the strict `ColumnFiller`; fields not in
the schema produce an error (unless [`grow_schema`](Self::grow_schema)
is chained, in which case new fields are added on the fly).

`schema` is an iterable of `(name, TypeSpec)` pairs. Order is
preserved in the resulting data.frame's column layout.

**Limitation**: this constructor takes a flat schema only — nested
struct flattening (`parent_child` columns) is not supported here.
Callers who need flattened nested structs either let default
discovery handle it, or pre-flatten the names themselves
(`"parent_child"` strings).

##### Examples

```rust,ignore
# use miniextendr_api::serde::{SerdeRowBuilder, TypeSpec};
# use serde::Serialize;
#[derive(Serialize)]
struct R { id: i32, name: String }

let mut b = SerdeRowBuilder::<R>::with_schema(
    [("id", TypeSpec::Integer), ("name", TypeSpec::Character)],
    Some(100),
);
for i in 0..100 {
    b.push(R { id: i, name: format!("row_{i}") }).unwrap();
}
let df = b.finish().unwrap();
```

### `serde::dataframe_de::SerdeRows`

Wrapper that converts `Vec<T: Serialize>` into a [`DataFrame`]
through the two-phase columnar serializer (schema discovery + column fill), the richer
serde build path than the per-row `IntoList` transposition.

```ignore
use miniextendr_api::dataframe::IntoDataFrame;
use miniextendr_api::serde::SerdeRows;

let df = SerdeRows(people).into_dataframe()?;
```

**Methods:**

#### `into_inner`

```rust
into_inner(self: Self) -> Vec<T>
```

Unwrap the inner `Vec<T>`.

### `serde::de::RDeserializer`

Deserializer that converts R SEXP to Rust values.

#### Type Mappings

| R Type | Rust Type |
|--------|-----------|
| `logical(1)` | `bool` |
| `integer(1)` | `i32` |
| `numeric(1)` | `f64` |
| `character(1)` | `String` |
| NA values | `Option<T>::None` |
| atomic vectors | `Vec<primitive>` or `Box<[primitive]>` |
| lists | `Vec<T>` or struct |
| named lists | struct or `HashMap<String, T>` |
| NULL | `()` or `Option<T>::None` |

#### Example

```ignore
use miniextendr_api::serde_r::RDeserializer;
use serde::Deserialize;

#[derive(Deserialize)]
struct Point { x: f64, y: f64 }

// Given list(x = 1.0, y = 2.0) from R:
let point: Point = RDeserializer::from_sexp(sexp).unwrap();
```

**Methods:**

#### `from_sexp`

```rust
from_sexp(sexp: SEXP) -> Self
```

Create a new deserializer from an R SEXP.

#### `from_sexp_to`

```rust
from_sexp_to<T>(sexp: SEXP) -> Result<T, RSerdeError>
```

Deserialize an R SEXP to a Rust value.

### `serde::json_string::AsJson`

Serialize `T` to a compact JSON string, return as R character scalar.

#### Example

```ignore
use serde::Serialize;

#[derive(Serialize)]
struct Response { status: i32, body: String }

#[miniextendr]
fn api_response() -> AsJson<Response> {
    AsJson(Response { status: 200, body: "ok".into() })
}
// R gets: '{"status":200,"body":"ok"}'
```

### `serde::json_string::AsJsonPretty`

Serialize `T` to a pretty-printed JSON string, return as R character scalar.

Same as [`AsJson`] but with indentation for human readability.

### `serde::json_string::AsJsonVec`

Serialize each element of a `Vec<T>` to a JSON string, return as R character vector.

#### Example

```ignore
#[miniextendr]
fn serialize_points(points: Vec<Point>) -> AsJsonVec<Point> {
    AsJsonVec(points)
}
// R gets: c('{"x":1,"y":2}', '{"x":3,"y":4}')
```

### `serde::json_string::FromJson`

Parse an R character scalar as JSON into `T: Deserialize`.

#### Example

```ignore
use serde::Deserialize;

#[derive(Deserialize)]
struct Config { max_threads: i32 }

#[miniextendr]
fn parse_config(json: FromJson<Config>) -> i32 {
    json.0.max_threads
}
// R: parse_config('{"max_threads": 4}')
```

### `serde::ser::MapSerializer`

Serializer for maps (HashMap, BTreeMap).

### `serde::ser::RSerializer`

Serializer that converts Rust values directly to R SEXP.

#### Type Mappings

| Rust Type | R Type |
|-----------|--------|
| `bool` | `logical(1)` |
| `i8/i16/i32` | `integer(1)` |
| `i64/u64/f32/f64` | `numeric(1)` |
| `String/&str` | `character(1)` |
| `Option<T>::None` | NA of appropriate type |
| `Vec<primitive>` | atomic vector |
| `Vec<struct>` | list of lists |
| `HashMap<String, T>` | named list |
| `struct` | named list |

#### Example

```ignore
use miniextendr_api::serde_r::RSerializer;
use serde::Serialize;

#[derive(Serialize)]
struct Point { x: f64, y: f64 }

let p = Point { x: 1.0, y: 2.0 };
let sexp = RSerializer::to_sexp(&p).unwrap();
// Result: list(x = 1.0, y = 2.0)
```

**Methods:**

#### `to_sexp`

```rust
to_sexp<T>(value: &T) -> Result<SEXP, RSerdeError>
```

Serialize a Rust value to an R SEXP.

### `serde::ser::SeqSerializer`

Serializer for sequences (Vec, tuples).

Uses smart dispatch: if all elements are homogeneous scalars of the same
primitive type, coalesces into an R atomic vector. Otherwise creates a list.

### `serde::ser::StructSerializer`

Serializer for structs.

### `serde::ser::StructVariantSerializer`

Serializer for struct variants: `Enum::Variant { a, b }` -> `list(Variant = list(a=..., b=...))`

### `serde::ser::TupleVariantSerializer`

Serializer for tuple variants: `Enum::Variant(a, b)` -> `list(Variant = list(a, b))`

### `serde::traits::AsSerialize`

Wrapper that converts any `Serialize` type to R via serde_r.

This is the serde analog to `AsList<T: IntoList>`. Use it when you want to
return a `Serialize` type from a `#[miniextendr]` function and have it
automatically converted to an R list.

#### Example

```rust,ignore
use miniextendr_api::serde_r::AsSerialize;
use serde::Serialize;

#[derive(Serialize)]
struct Point { x: f64, y: f64 }

#[miniextendr]
fn make_point(x: f64, y: f64) -> AsSerialize<Point> {
    AsSerialize(Point { x, y })
}
// Returns list(x = 1.0, y = 2.0) in R

#[derive(Serialize)]
struct Result { success: bool, message: String }

#[miniextendr]
fn process() -> AsSerialize<Vec<Result>> {
    AsSerialize(vec![
        Result { success: true, message: "ok".into() },
        Result { success: false, message: "error".into() },
    ])
}
// Returns list of lists in R
```

#### Extracting the inner value

```rust,ignore
let wrapped = AsSerialize(my_value);
let inner = wrapped.into_inner();  // Get T back
let inner_ref = wrapped.as_ref();  // Get &T
```

**Methods:**

#### `into_inner`

```rust
into_inner(self: Self) -> T
```

Extract the inner value.

#### `new`

```rust
new(value: T) -> Self
```

Create a new `AsSerialize` wrapper.

### `sexp::SEXP`

R's pointer type for S-expressions.

This is a newtype wrapper around `*mut SEXPREC` that implements Send and Sync.
SEXP is just a handle (pointer) - the actual data it points to is managed by R's
garbage collector and should only be accessed on R's main thread.

#### Safety

While SEXP is Send+Sync (allowing it to be passed between threads), the data
it points to must only be accessed on R's main thread. The miniextendr runtime
enforces this through the worker thread pattern.

#### Equality Semantics

IMPORTANT: The derived `PartialEq` compares **pointer equality**, not semantic equality.
For proper R semantics (comparing object contents), use `R_compute_identical`.

```ignore
// Pointer equality (fast, often wrong for R semantics)
if sexp1 == sexp2 { ... }  // Only true if same pointer

// Semantic equality (correct R semantics)
if R_compute_identical(sexp1, sexp2, 16) != 0 { ... }
```

**Hash trait removed**: SEXP no longer implements `Hash` because proper hashing
would require deep content inspection via `R_compute_identical`, which is too
expensive for general use. If you need SEXP as a HashMap key, use pointer identity:

```ignore
// Store by pointer identity (common pattern for R symbol lookups)
let mut map: HashMap<*mut SEXPREC, Value> = HashMap::new();
map.insert(sexp.as_ptr(), value);
```

**Methods:**

#### `alloc`

```rust
unsafe alloc(ty: SEXPTYPE, n: R_xlen_t) -> SEXP
```

Allocate a fresh R vector of the given type and length.

Direct wrapper over `Rf_allocVector`. For typed allocations, prefer
helpers like [`SEXP::alloc_list`], [`SEXP::alloc_strsxp`], or wrap the
result in [`OwnedProtect`](crate::gc_protect::OwnedProtect) immediately
— the returned SEXP is unprotected.

##### Safety

Must be called from the R main thread. The returned SEXP is unprotected;
any subsequent allocation may collect it.

#### `alloc_list`

```rust
unsafe alloc_list(n: R_xlen_t) -> SEXP
```

Allocate an R list (VECSXP) of length `n`. Unprotected.

Equivalent to `Rf_allocVector(VECSXP, n)`. Elements are initialised to `R_NilValue`.

##### Safety

Must be called from the R main thread. The returned SEXP is unprotected —
wrap it in [`OwnedProtect`](crate::gc_protect::OwnedProtect) before any
other allocation that could trigger GC.

#### `alloc_strsxp`

```rust
unsafe alloc_strsxp(n: R_xlen_t) -> SEXP
```

Allocate an R character vector (STRSXP) of length `n`. Unprotected.

Equivalent to `Rf_allocVector(STRSXP, n)`. Elements are initialised to `R_BlankString`.

##### Safety

Must be called from the R main thread. The returned SEXP is unprotected —
wrap it in [`OwnedProtect`](crate::gc_protect::OwnedProtect) before any
other allocation that could trigger GC.

#### `altrep_data1_raw`

```rust
unsafe altrep_data1_raw(self: Self) -> SEXP
```

Get the raw SEXP in the ALTREP data1 slot.

##### Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `altrep_data1_raw_unchecked`

```rust
unsafe altrep_data1_raw_unchecked(self: Self) -> SEXP
```

Get the raw SEXP in the ALTREP data1 slot (unchecked — no thread routing).

##### Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `altrep_data2_raw`

```rust
unsafe altrep_data2_raw(self: Self) -> SEXP
```

Get the raw SEXP in the ALTREP data2 slot.

##### Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `altrep_data2_raw_unchecked`

```rust
unsafe altrep_data2_raw_unchecked(self: Self) -> SEXP
```

Get the raw SEXP in the ALTREP data2 slot (unchecked — no thread routing).

##### Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `as_ptr`

```rust
const as_ptr(self: Self) -> *mut SEXPREC
```

Get the raw pointer.

#### `base_namespace`

```rust
base_namespace() -> SEXP
```

R's base namespace environment.

#### `blank_string`

```rust
blank_string() -> SEXP
```

R's empty string `""` singleton.

#### `charsxp`

```rust
charsxp(s: &str) -> SEXP
```

Create a CHARSXP from a Rust `&str` (UTF-8).

#### `class_symbol`

```rust
class_symbol() -> SEXP
```

R's `class` attribute symbol.

#### `dim_symbol`

```rust
dim_symbol() -> SEXP
```

R's `dim` attribute symbol.

#### `dimnames_symbol`

```rust
dimnames_symbol() -> SEXP
```

R's `dimnames` attribute symbol.

#### `from_ptr`

```rust
const from_ptr(ptr: *mut SEXPREC) -> Self
```

Create from a raw pointer.

#### `install_char`

```rust
install_char(charsxp: SEXP) -> SEXP
```

Create an R symbol (SYMSXP) from a CHARSXP.

Equivalent to `Rf_installChar(charsxp)`. The symbol is interned
in R's global symbol table and never garbage collected.

#### `is_null`

```rust
const is_null(self: Self) -> bool
```

Check if this SEXP is a C null pointer (0x0).

To check if an SEXP is R's `NULL` (`R_NilValue`), use
[`crate::SexpExt::is_nil()`] instead.

See also: [`crate::SexpExt::is_nil()`], [`crate::SexpExt::is_null_or_nil()`]

#### `levels_symbol`

```rust
levels_symbol() -> SEXP
```

R's `levels` attribute symbol (factors).

#### `missing_arg`

```rust
missing_arg() -> SEXP
```

R's missing argument sentinel.

#### `na_string`

```rust
na_string() -> SEXP
```

R's `NA_character_` singleton.

#### `names_symbol`

```rust
names_symbol() -> SEXP
```

R's `names` attribute symbol.

#### `nil`

```rust
nil() -> Self
```

Return R's `NULL` singleton (`R_NilValue`).

This is **not** a C null pointer — it points to R's actual nil object
on the heap. Use this for `.Call()` return values, SEXP arguments to
R API functions, and any slot in R data structures.

See also: [`SEXP::null()`], [`crate::SexpExt::is_nil()`], [`SEXP::is_null()`]

#### `null`

```rust
const null() -> Self
```

Create a C null pointer SEXP (0x0).

This is **not** R's `NULL` value (`R_NilValue`). R's `NULL` is a real
heap-allocated singleton; a C null pointer is just address zero. Passing
`SEXP::null()` where R expects `R_NilValue` will corrupt R's GC state
and likely segfault.

Use [`SEXP::nil()`] for R's `NULL`. Only use `null()` for low-level
pointer initialization, ALTREP Sum/Min/Max "can't compute" returns
(R checks `!= NULL`, not `!= R_NilValue`), or comparison against
uninitialized pointers.

See also: [`SEXP::nil()`], [`SEXP::is_null()`], [`crate::SexpExt::is_nil()`]

#### `scalar_complex`

```rust
scalar_complex(x: Rcomplex) -> SEXP
```

Create a length-1 complex vector.

#### `scalar_complex_unchecked`

```rust
unsafe scalar_complex_unchecked(x: Rcomplex) -> SEXP
```

Create a length-1 complex vector (unchecked — no thread routing).

##### Safety

Must be called from the R main thread.

#### `scalar_integer`

```rust
scalar_integer(x: i32) -> SEXP
```

Create a length-1 integer vector.

#### `scalar_integer_unchecked`

```rust
unsafe scalar_integer_unchecked(x: i32) -> SEXP
```

Create a length-1 integer vector (unchecked — no thread routing).

##### Safety

Must be called from the R main thread.

#### `scalar_logical`

```rust
scalar_logical(x: bool) -> SEXP
```

Create a length-1 logical vector.

#### `scalar_logical_raw`

```rust
scalar_logical_raw(x: i32) -> SEXP
```

Create a length-1 logical vector from raw i32 (0=FALSE, 1=TRUE, NA_LOGICAL=NA).
Accepts 0 (FALSE), 1 (TRUE), or `NA_LOGICAL` (`i32::MIN`) for NA.
Prefer [`scalar_logical`](Self::scalar_logical) for non-NA values.

#### `scalar_logical_raw_unchecked`

```rust
unsafe scalar_logical_raw_unchecked(x: i32) -> SEXP
```

Create a length-1 logical vector from raw i32 (unchecked — no thread routing).

Accepts 0 (FALSE), 1 (TRUE), or `NA_LOGICAL` (`i32::MIN`) for NA.

##### Safety

Must be called from the R main thread.

#### `scalar_raw`

```rust
scalar_raw(x: u8) -> SEXP
```

Create a length-1 raw vector.

#### `scalar_raw_unchecked`

```rust
unsafe scalar_raw_unchecked(x: u8) -> SEXP
```

Create a length-1 raw vector (unchecked — no thread routing).

##### Safety

Must be called from the R main thread.

#### `scalar_real`

```rust
scalar_real(x: f64) -> SEXP
```

Create a length-1 real vector.

#### `scalar_real_unchecked`

```rust
unsafe scalar_real_unchecked(x: f64) -> SEXP
```

Create a length-1 real vector (unchecked — no thread routing).

##### Safety

Must be called from the R main thread.

#### `scalar_string`

```rust
scalar_string(charsxp: SEXP) -> SEXP
```

Create a length-1 character vector from a CHARSXP.

#### `scalar_string_from_str`

```rust
scalar_string_from_str(s: &str) -> SEXP
```

Create a length-1 character vector from a Rust `&str`.

#### `scalar_string_unchecked`

```rust
unsafe scalar_string_unchecked(charsxp: SEXP) -> SEXP
```

Create a length-1 character vector from a CHARSXP (unchecked — no thread routing).

##### Safety

Must be called from the R main thread.

#### `set_altrep_data1`

```rust
unsafe set_altrep_data1(self: Self, v: SEXP)
```

Set the ALTREP data1 slot.

##### Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `set_altrep_data2`

```rust
unsafe set_altrep_data2(self: Self, v: SEXP)
```

Set the ALTREP data2 slot.

##### Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `set_altrep_data2_unchecked`

```rust
unsafe set_altrep_data2_unchecked(self: Self, v: SEXP)
```

Set the ALTREP data2 slot (unchecked — no thread routing).

##### Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `symbol`

```rust
symbol(name: &str) -> SEXP
```

Create an R symbol (SYMSXP) from a Rust `&str`.

Combines `SEXP::charsxp()` + `Rf_installChar` into one call.
The symbol is interned and never garbage collected.

#### `tsp_symbol`

```rust
tsp_symbol() -> SEXP
```

R's `tsp` attribute symbol (time series).

### `sexp::SEXPREC`

Opaque underlying S-expression header type.

### `sexp_types::RLogical`

R's logical element type (the contents of a `LGLSXP` vector).

In R, logical vectors are stored as `int` with possible values:
- `0` for FALSE
- `1` for TRUE
- `NA_LOGICAL` (typically `INT_MIN`) for NA

**Important:** R may also contain other non-zero values in logical vectors
(e.g., from low-level code). Those should be interpreted as TRUE.

This type is `repr(transparent)` over `i32` so *any* raw value is valid,
avoiding UB when viewing `LGLSXP` data as a slice.

**Methods:**

#### `from_i32`

```rust
const from_i32(raw: i32) -> Self
```

Construct directly from raw R logical storage.

#### `is_na`

```rust
const is_na(self: Self) -> bool
```

Returns whether the value is `NA_LOGICAL`.

#### `to_i32`

```rust
const to_i32(self: Self) -> i32
```

Get raw R logical storage value.

#### `to_option_bool`

```rust
const to_option_bool(self: Self) -> Option<bool>
```

Convert to Rust `Option<bool>` (`None` for `NA`).

### `sexp_types::Rcomplex`

R's complex scalar layout (`Rcomplex`).

**Fields:**

- `r`: `f64`
  - Real part.
- `i`: `f64`
  - Imaginary part.

### `strvec::ProtectedStrVec`

GC-protected view over an R character vector (`STRSXP`).

Unlike [`StrVec`] (which is `Copy` and trusts the caller for GC protection),
`ProtectedStrVec` wraps a [`Protected<'static, StrVec>`](crate::gc_protect::Protected) that keeps the
STRSXP alive. All borrowed data (`&str`, iterators) has its lifetime tied to `&self`,
not `'static` — preventing use-after-GC bugs at compile time.

#### When to use

- **`StrVec`**: for SEXP arguments to `.Call` (R protects them), or when you
  manage protection yourself. Lightweight, `Copy`.
- **`ProtectedStrVec`**: when you allocate or receive an STRSXP and need to
  keep it alive beyond the immediate scope. Not `Copy`.

#### Example

```ignore
#[miniextendr]
pub fn count_unique(strings: ProtectedStrVec) -> i32 {
    let unique: HashSet<&str> = strings.iter()
        .filter_map(|s| s)
        .collect();
    unique.len() as i32
}
```

**Methods:**

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Get the underlying SEXP (still protected by this handle).

#### `as_strvec`

```rust
as_strvec(self: &Self) -> StrVec
```

Get the inner `StrVec` (unprotected copy — caller assumes protection responsibility).

#### `from_sexp_trusted`

```rust
unsafe from_sexp_trusted(sexp: SEXP) -> Self
```

Create a view without adding GC protection.

Use this when the SEXP is already protected by R (e.g., a `.Call`
argument, or in a `ProtectScope`). Avoids the redundant
`Rf_protect`/`Rf_unprotect` pair.

The lifetime-bound `&str` borrows are still enforced — this only
skips the protect stack push, not the safety guarantees.

##### Safety

- `sexp` must be a valid STRSXP.
- `sexp` must remain GC-protected for the lifetime of this struct.
- Must be called from the R main thread.

#### `get_cow`

```rust
get_cow(self: &Self, idx: isize) -> Option<Cow<'_, str>>
```

Get the string at index as `Cow<str>` (encoding-safe, lifetime tied to `&self`).

#### `get_str`

```rust
get_str(self: &Self, idx: isize) -> Option<&str>
```

Get the string at index (zero-copy, lifetime tied to `&self`).

Returns `None` for out-of-bounds or `NA_character_`.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Whether the vector is empty.

#### `iter`

```rust
iter(self: &Self) -> ProtectedStrVecIter<'_>
```

Iterate over elements as `Option<&str>` (lifetime tied to `&self`).

#### `iter_cow`

```rust
iter_cow(self: &Self) -> ProtectedStrVecCowIter<'_>
```

Iterate over elements as `Option<Cow<str>>` (encoding-safe).

#### `len`

```rust
len(self: &Self) -> isize
```

Number of elements.

#### `new`

```rust
unsafe new(sexp: SEXP) -> Self
```

Create a protected view over an STRSXP.

Calls `Rf_protect` on the SEXP. Use [`from_sexp_trusted`](Self::from_sexp_trusted)
when the SEXP is already protected (e.g., `.Call` arguments) to avoid
double-protecting.

##### Safety

- `sexp` must be a valid STRSXP.
- Must be called from the R main thread.

### `strvec::ProtectedStrVecCowIter`

Encoding-safe iterator over `ProtectedStrVec`.

### `strvec::ProtectedStrVecIter`

Iterator over `ProtectedStrVec` with lifetime tied to the protection guard.

### `strvec::StrVec`

Owned handle to an R character vector (`STRSXP`).

This wrapper provides safe methods for building character vectors
element-by-element with proper GC protection.

**Methods:**

#### `as_sexp`

```rust
const as_sexp(self: Self) -> SEXP
```

Get the underlying `SEXP`.

#### `from_raw`

```rust
const unsafe from_raw(sexp: SEXP) -> Self
```

Wrap an existing `STRSXP` without additional checks.

##### Safety

Caller must ensure `sexp` is a valid character vector (`STRSXP`)
whose lifetime remains managed by R.

#### `get_charsxp`

```rust
get_charsxp(self: Self, idx: isize) -> Option<SEXP>
```

Get the CHARSXP at the given index.

Returns `None` if out of bounds.

#### `get_cow`

```rust
get_cow(self: Self, idx: isize) -> Option<Cow<'static, str>>
```

Get the string at the given index as `Cow<str>` (encoding-safe).

Returns `Cow::Borrowed` for UTF-8 strings (zero-copy), `Cow::Owned` for
non-UTF-8 strings (translated via `Rf_translateCharUTF8`).
Returns `None` if out of bounds or `NA_character_`.

#### `get_str`

```rust
get_str(self: Self, idx: isize) -> Option<&'static str>
```

Get the string at the given index (zero-copy).

Returns `None` if out of bounds or if the element is `NA_character_`.
Panics if the CHARSXP is not valid UTF-8 (should not happen in a UTF-8 locale).

#### `is_empty`

```rust
is_empty(self: Self) -> bool
```

Returns true if the vector is empty.

#### `iter`

```rust
iter(self: Self) -> StrVecIter
```

Iterate over elements as `Option<&str>`.

`NA_character_` elements yield `None`, valid strings yield `Some(&str)`.
Zero-copy — each `&str` borrows directly from R's CHARSXP.

#### `iter_cow`

```rust
iter_cow(self: Self) -> StrVecCowIter
```

Iterate over elements as `Option<Cow<str>>` (encoding-safe).

Like [`iter`](Self::iter) but handles non-UTF-8 CHARSXPs gracefully.

#### `len`

```rust
len(self: Self) -> isize
```

Length of the character vector (number of elements).

#### `set_charsxp`

```rust
unsafe set_charsxp(self: Self, idx: isize, charsxp: SEXP)
```

Set a CHARSXP at the given index, protecting it during insertion.

This is the safe way to insert a freshly allocated CHARSXP into a string vector.

##### Safety

- Must be called from the R main thread
- `charsxp` must be a valid CHARSXP (from `Rf_mkChar*` or `STRING_ELT`)
- `self` must be a valid, protected STRSXP

##### Panics

Panics if `idx` is out of bounds.

#### `set_charsxp_unchecked`

```rust
unsafe set_charsxp_unchecked(self: Self, idx: isize, charsxp: SEXP)
```

Set a CHARSXP without protecting it.

##### Safety

In addition to the safety requirements of [`set_charsxp`](Self::set_charsxp):
- The caller must ensure `charsxp` is already protected or from the
  global CHARSXP cache.

#### `set_na`

```rust
unsafe set_na(self: Self, idx: isize)
```

Set an element to `NA_character_`.

##### Safety

- Must be called from the R main thread
- `self` must be a valid, protected STRSXP

##### Panics

Panics if `idx` is out of bounds.

#### `set_opt_str`

```rust
unsafe set_opt_str(self: Self, idx: isize, s: Option<&str>)
```

Set an element from an optional string.

`None` becomes `NA_character_`.

##### Safety

- Must be called from the R main thread
- `self` must be a valid, protected STRSXP

##### Panics

Panics if `idx` is out of bounds.

#### `set_str`

```rust
unsafe set_str(self: Self, idx: isize, s: &str)
```

Set an element from a Rust string.

Creates a CHARSXP from the string and inserts it safely.

##### Safety

- Must be called from the R main thread
- `self` must be a valid, protected STRSXP

##### Panics

Panics if `idx` is out of bounds.

### `strvec::StrVecBuilder`

Builder for constructing string vectors with efficient protection management.

#### Example

```ignore
unsafe fn build_strvec(strings: &[&str]) -> SEXP {
    let scope = ProtectScope::new();
    let builder = StrVecBuilder::new(&scope, strings.len() as isize);

    for (i, s) in strings.iter().enumerate() {
        builder.set_str(i as isize, s);
    }

    builder.into_sexp()
}
```

**Methods:**

#### `as_sexp`

```rust
as_sexp(self: &Self) -> SEXP
```

Get the underlying SEXP.

#### `into_sexp`

```rust
into_sexp(self: Self) -> SEXP
```

Convert to the underlying SEXP.

#### `into_strvec`

```rust
into_strvec(self: Self) -> StrVec
```

Convert to a `StrVec` wrapper.

#### `is_empty`

```rust
is_empty(self: &Self) -> bool
```

Check if empty.

#### `len`

```rust
len(self: &Self) -> isize
```

Get the length.

#### `new`

```rust
unsafe new(scope: &'a ProtectScope, len: usize) -> Self
```

Create a new string vector builder with the given length.

##### Safety

Must be called from the R main thread.

#### `set_na`

```rust
unsafe set_na(self: &Self, idx: isize)
```

Set an element to `NA_character_`.

##### Safety

Must be called from the R main thread.

#### `set_opt_str`

```rust
unsafe set_opt_str(self: &Self, idx: isize, s: Option<&str>)
```

Set an element from an optional string.

##### Safety

Must be called from the R main thread.

#### `set_str`

```rust
unsafe set_str(self: &Self, idx: isize, s: &str)
```

Set an element from a Rust string.

##### Safety

Must be called from the R main thread.

### `strvec::StrVecCowIter`

Iterator over `StrVec` elements as `Option<Cow<'static, str>>`.

Like [`StrVecIter`] but handles non-UTF-8 CHARSXPs via `Rf_translateCharUTF8`.

### `strvec::StrVecIter`

Iterator over `StrVec` elements as `Option<&str>`.

Yields `None` for `NA_character_`, `Some(&str)` for valid strings.
Zero-copy — each `&str` borrows directly from R's CHARSXP.

### `sys::DllInfo`

Opaque dynamic library descriptor from R.

### `sys::R_CMethodDef`

Method definition for .C interface routines.

Used to register C functions callable via `.C()` from R.

**Fields:**

- `name`: `*const ::std::os::raw::c_char`
  - Exported symbol name.
- `fun`: `DL_FUNC`
  - Function pointer implementing the routine.
- `numArgs`: `::std::os::raw::c_int`
  - Declared arity.
- `types`: `*const R_NativePrimitiveArgType`
  - Optional array of argument types for type checking. May be null.

### `sys::R_CallMethodDef`

Method definition for .Call interface routines.

Used to register C functions callable via `.Call()` from R.
Unlike `.C()` routines, `.Call()` functions receive and return SEXP values directly.

**Fields:**

- `name`: `*const ::std::os::raw::c_char`
  - Exported symbol name.
- `fun`: `DL_FUNC`
  - Function pointer implementing the routine.
- `numArgs`: `::std::os::raw::c_int`
  - Declared arity.

### `sys::altrep::R_altrep_class_t`

Opaque ALTREP class handle.

**Fields:**

- `ptr`: `crate::SEXP`
  - Underlying class object SEXP.

**Methods:**

#### `as_sexp`

```rust
as_sexp(self: Self) -> SEXP
```

Get the underlying SEXP.

Rust equivalent of C macro `R_SEXP(x)`.

#### `from_sexp`

```rust
const from_sexp(ptr: SEXP) -> Self
```

Create from a raw SEXP pointer.

Rust equivalent of C macro `R_SUBTYPE_INIT(x)`.

#### `inherits`

```rust
unsafe inherits(self: Self, x: SEXP) -> bool
```

Check if `x` is an instance of this ALTREP class.

##### Safety
Must be called on R's main thread. `x` must be a valid SEXP.

#### `new_altrep`

```rust
unsafe new_altrep(self: Self, data1: SEXP, data2: SEXP) -> SEXP
```

Create a new ALTREP instance with data1 and data2 slots.

##### Safety
Must be called on R's main thread. `data1` and `data2` must be valid SEXPs.

#### `new_altrep_unchecked`

```rust
unsafe new_altrep_unchecked(self: Self, data1: SEXP, data2: SEXP) -> SEXP
```

Create a new ALTREP instance (no thread check).

##### Safety
Must be called on R's main thread.

#### `set_coerce_method`

```rust
unsafe set_coerce_method(self: Self, fun: R_altrep_Coerce_method_t)
```

Set the Coerce method.
##### Safety
Must be called during R initialization.

#### `set_complex_elt_method`

```rust
unsafe set_complex_elt_method(self: Self, fun: R_altcomplex_Elt_method_t)
```

Set the complex Elt method.
##### Safety
Must be called during R initialization.

#### `set_complex_get_region_method`

```rust
unsafe set_complex_get_region_method(self: Self, fun: R_altcomplex_Get_region_method_t)
```

Set the complex Get_region method.
##### Safety
Must be called during R initialization.

#### `set_dataptr_method`

```rust
unsafe set_dataptr_method(self: Self, fun: R_altvec_Dataptr_method_t)
```

Set the Dataptr method.
##### Safety
Must be called during R initialization.

#### `set_dataptr_or_null_method`

```rust
unsafe set_dataptr_or_null_method(self: Self, fun: R_altvec_Dataptr_or_null_method_t)
```

Set the Dataptr_or_null method.
##### Safety
Must be called during R initialization.

#### `set_duplicate_ex_method`

```rust
unsafe set_duplicate_ex_method(self: Self, fun: R_altrep_DuplicateEX_method_t)
```

Set the DuplicateEX method.
##### Safety
Must be called during R initialization.

#### `set_duplicate_method`

```rust
unsafe set_duplicate_method(self: Self, fun: R_altrep_Duplicate_method_t)
```

Set the Duplicate method.
##### Safety
Must be called during R initialization.

#### `set_extract_subset_method`

```rust
unsafe set_extract_subset_method(self: Self, fun: R_altvec_Extract_subset_method_t)
```

Set the Extract_subset method.
##### Safety
Must be called during R initialization.

#### `set_inspect_method`

```rust
unsafe set_inspect_method(self: Self, fun: R_altrep_Inspect_method_t)
```

Set the Inspect method.
##### Safety
Must be called during R initialization.

#### `set_integer_elt_method`

```rust
unsafe set_integer_elt_method(self: Self, fun: R_altinteger_Elt_method_t)
```

Set the integer Elt method.
##### Safety
Must be called during R initialization.

#### `set_integer_get_region_method`

```rust
unsafe set_integer_get_region_method(self: Self, fun: R_altinteger_Get_region_method_t)
```

Set the integer Get_region method.
##### Safety
Must be called during R initialization.

#### `set_integer_is_sorted_method`

```rust
unsafe set_integer_is_sorted_method(self: Self, fun: R_altinteger_Is_sorted_method_t)
```

Set the integer Is_sorted method.
##### Safety
Must be called during R initialization.

#### `set_integer_max_method`

```rust
unsafe set_integer_max_method(self: Self, fun: R_altinteger_Max_method_t)
```

Set the integer Max method.
##### Safety
Must be called during R initialization.

#### `set_integer_min_method`

```rust
unsafe set_integer_min_method(self: Self, fun: R_altinteger_Min_method_t)
```

Set the integer Min method.
##### Safety
Must be called during R initialization.

#### `set_integer_no_na_method`

```rust
unsafe set_integer_no_na_method(self: Self, fun: R_altinteger_No_NA_method_t)
```

Set the integer No_NA method.
##### Safety
Must be called during R initialization.

#### `set_integer_sum_method`

```rust
unsafe set_integer_sum_method(self: Self, fun: R_altinteger_Sum_method_t)
```

Set the integer Sum method.
##### Safety
Must be called during R initialization.

#### `set_length_method`

```rust
unsafe set_length_method(self: Self, fun: R_altrep_Length_method_t)
```

Set the Length method.
##### Safety
Must be called during R initialization.

#### `set_list_elt_method`

```rust
unsafe set_list_elt_method(self: Self, fun: R_altlist_Elt_method_t)
```

Set the list Elt method.
##### Safety
Must be called during R initialization.

#### `set_list_set_elt_method`

```rust
unsafe set_list_set_elt_method(self: Self, fun: R_altlist_Set_elt_method_t)
```

Set the list Set_elt method.
##### Safety
Must be called during R initialization.

#### `set_logical_elt_method`

```rust
unsafe set_logical_elt_method(self: Self, fun: R_altlogical_Elt_method_t)
```

Set the logical Elt method.
##### Safety
Must be called during R initialization.

#### `set_logical_get_region_method`

```rust
unsafe set_logical_get_region_method(self: Self, fun: R_altlogical_Get_region_method_t)
```

Set the logical Get_region method.
##### Safety
Must be called during R initialization.

#### `set_logical_is_sorted_method`

```rust
unsafe set_logical_is_sorted_method(self: Self, fun: R_altlogical_Is_sorted_method_t)
```

Set the logical Is_sorted method.
##### Safety
Must be called during R initialization.

#### `set_logical_no_na_method`

```rust
unsafe set_logical_no_na_method(self: Self, fun: R_altlogical_No_NA_method_t)
```

Set the logical No_NA method.
##### Safety
Must be called during R initialization.

#### `set_logical_sum_method`

```rust
unsafe set_logical_sum_method(self: Self, fun: R_altlogical_Sum_method_t)
```

Set the logical Sum method.
##### Safety
Must be called during R initialization.

#### `set_raw_elt_method`

```rust
unsafe set_raw_elt_method(self: Self, fun: R_altraw_Elt_method_t)
```

Set the raw Elt method.
##### Safety
Must be called during R initialization.

#### `set_raw_get_region_method`

```rust
unsafe set_raw_get_region_method(self: Self, fun: R_altraw_Get_region_method_t)
```

Set the raw Get_region method.
##### Safety
Must be called during R initialization.

#### `set_real_elt_method`

```rust
unsafe set_real_elt_method(self: Self, fun: R_altreal_Elt_method_t)
```

Set the real Elt method.
##### Safety
Must be called during R initialization.

#### `set_real_get_region_method`

```rust
unsafe set_real_get_region_method(self: Self, fun: R_altreal_Get_region_method_t)
```

Set the real Get_region method.
##### Safety
Must be called during R initialization.

#### `set_real_is_sorted_method`

```rust
unsafe set_real_is_sorted_method(self: Self, fun: R_altreal_Is_sorted_method_t)
```

Set the real Is_sorted method.
##### Safety
Must be called during R initialization.

#### `set_real_max_method`

```rust
unsafe set_real_max_method(self: Self, fun: R_altreal_Max_method_t)
```

Set the real Max method.
##### Safety
Must be called during R initialization.

#### `set_real_min_method`

```rust
unsafe set_real_min_method(self: Self, fun: R_altreal_Min_method_t)
```

Set the real Min method.
##### Safety
Must be called during R initialization.

#### `set_real_no_na_method`

```rust
unsafe set_real_no_na_method(self: Self, fun: R_altreal_No_NA_method_t)
```

Set the real No_NA method.
##### Safety
Must be called during R initialization.

#### `set_real_sum_method`

```rust
unsafe set_real_sum_method(self: Self, fun: R_altreal_Sum_method_t)
```

Set the real Sum method.
##### Safety
Must be called during R initialization.

#### `set_serialized_state_method`

```rust
unsafe set_serialized_state_method(self: Self, fun: R_altrep_Serialized_state_method_t)
```

Set the Serialized_state method.
##### Safety
Must be called during R initialization.

#### `set_string_elt_method`

```rust
unsafe set_string_elt_method(self: Self, fun: R_altstring_Elt_method_t)
```

Set the string Elt method.
##### Safety
Must be called during R initialization.

#### `set_string_is_sorted_method`

```rust
unsafe set_string_is_sorted_method(self: Self, fun: R_altstring_Is_sorted_method_t)
```

Set the string Is_sorted method.
##### Safety
Must be called during R initialization.

#### `set_string_no_na_method`

```rust
unsafe set_string_no_na_method(self: Self, fun: R_altstring_No_NA_method_t)
```

Set the string No_NA method.
##### Safety
Must be called during R initialization.

#### `set_string_set_elt_method`

```rust
unsafe set_string_set_elt_method(self: Self, fun: R_altstring_Set_elt_method_t)
```

Set the string Set_elt method.
##### Safety
Must be called during R initialization.

#### `set_unserialize_ex_method`

```rust
unsafe set_unserialize_ex_method(self: Self, fun: R_altrep_UnserializeEX_method_t)
```

Set the UnserializeEX method.
##### Safety
Must be called during R initialization.

#### `set_unserialize_method`

```rust
unsafe set_unserialize_method(self: Self, fun: R_altrep_Unserialize_method_t)
```

Set the Unserialize method.
##### Safety
Must be called during R initialization.

### `thread::RThreadBuilder`

Builder for spawning threads with R-appropriate stack sizes.

This builder is always available and configures threads with stack sizes
suitable for R workloads (8 MiB default, vs Rust's 2 MiB default).

When the `nonapi` feature is enabled, spawned threads also automatically
disable R's stack checking via `StackCheckGuard`, allowing R API calls
from the thread.

#### Example

```ignore
use miniextendr_api::thread::RThreadBuilder;

let handle = RThreadBuilder::new()
    .stack_size(16 * 1024 * 1024)  // 16 MiB
    .name("r-worker".to_string())
    .spawn(|| {
        // With `nonapi`: R API calls safe here
        // Without `nonapi`: Just a thread with correct stack size
    })?;
```

**Methods:**

#### `name`

```rust
name(self: Self, name: String) -> Self
```

Set the name for the thread (for debugging).

#### `new`

```rust
new() -> Self
```

Create a new builder with default settings.

Default stack size is [`DEFAULT_R_STACK_SIZE`] (8 MiB).

#### `spawn`

```rust
spawn<F, T>(self: Self, f: F) -> std::io::Result<std::thread::JoinHandle<T>>
```

Spawn the thread with the configured settings.

With `nonapi` feature: automatically disables R's stack checking.
Without `nonapi` feature: just spawns with the configured stack size.

#### `spawn_join`

```rust
spawn_join<F, T>(self: Self, f: F) -> std::thread::Result<T>
```

Spawn and immediately join, returning the result.

Convenience method for synchronous R calls on a separate thread.

##### Example

```ignore
let result = RThreadBuilder::new()
    .spawn_join(|| unsafe { miniextendr_api::SEXP::scalar_integer_unchecked(42) })
    .unwrap();
```

#### `stack_size`

```rust
stack_size(self: Self, size: usize) -> Self
```

Set the stack size for the thread.

R typically requires more stack space than Rust's default 2 MiB.
The default is 8 MiB to match typical R installations.

### `typed_list::TypedEntry`

A single entry specification in a typed list.

**Fields:**

- `name`: `&'static str`
  - The expected name of this entry.
- `spec`: `TypeSpec`
  - The expected type of this entry.
- `optional`: `bool`
  - If `true`, the entry is optional (missing allowed).

**Methods:**

#### `any`

```rust
const any(name: &'static str) -> Self
```

Create a required entry that accepts any type.

#### `any_optional`

```rust
const any_optional(name: &'static str) -> Self
```

Create an optional entry that accepts any type.

#### `optional`

```rust
const optional(name: &'static str, spec: TypeSpec) -> Self
```

Create an optional entry with the given name and type.

#### `required`

```rust
const required(name: &'static str, spec: TypeSpec) -> Self
```

Create a required entry with the given name and type.

### `typed_list::TypedList`

A validated list that matches a [`TypedListSpec`].

Provides typed accessors for list elements with good error messages.

**Methods:**

#### `as_list`

```rust
as_list(self: &Self) -> List
```

Get the underlying [`List`].

#### `get`

```rust
get<T>(self: &Self, name: &str) -> Result<T, TypedListError>
```

Get an element by name and convert to type `T`.

Returns [`TypedListError::Missing`] if the field doesn't exist.
Returns [`TypedListError::WrongType`] if conversion fails.

#### `get_opt`

```rust
get_opt<T>(self: &Self, name: &str) -> Result<Option<T>, TypedListError>
```

Get an optional element by name and convert to type `T`.

Returns `Ok(None)` if the field doesn't exist.
Returns [`TypedListError::WrongType`] if the field exists but conversion fails.

#### `get_raw`

```rust
get_raw(self: &Self, name: &str) -> Result<SEXP, TypedListError>
```

Get the raw SEXP for a named element.

#### `spec`

```rust
spec(self: &Self) -> &TypedListSpec
```

Get the specification this list was validated against.

### `typed_list::TypedListSpec`

Specification for validating a typed list.

Describes the expected structure of an R list, including required and
optional named entries with their type constraints.

**Fields:**

- `entries`: `Vec<TypedEntry>`
  - Expected entries in the list.
- `allow_extra`: `bool`
  - If `false`, reject lists with named entries not in the spec.

**Methods:**

#### `new`

```rust
new(entries: Vec<TypedEntry>) -> Self
```

Create a new spec that allows extra fields.

#### `strict`

```rust
strict(entries: Vec<TypedEntry>) -> Self
```

Create a strict spec that rejects extra named fields.

### `wasm_registry_writer::AltrepRegRow`

Pre-extracted view of one `MX_ALTREP_REGISTRATIONS` entry.

**Fields:**

- `symbol`: `String`

### `wasm_registry_writer::CallDefRow`

Pre-extracted, cdylib-side view of one `R_CallMethodDef`.

`R_CallMethodDef` carries `name` as a raw `*const c_char`; safely walking
it requires `unsafe`. The formatter takes already-extracted, owned values
so it can be unit-tested without globals.

**Fields:**

- `name`: `String`
- `num_args`: `i32`

### `wasm_registry_writer::TraitDispatchRow`

Pre-extracted view of one `MX_TRAIT_DISPATCH` entry.

**Fields:**

- `concrete_tag`: `crate::abi::mx_tag`
- `trait_tag`: `crate::abi::mx_tag`
- `vtable_symbol`: `String`

---

## Enums

### `altrep::RBase`

Base type for ALTREP vectors.

**Variants:**

- `Int`
  - Integer vectors (`INTSXP`).
- `Real`
  - Double vectors (`REALSXP`).
- `Logical`
  - Logical vectors (`LGLSXP`).
- `Raw`
  - Raw byte vectors (`RAWSXP`).
- `String`
  - Character vectors (`STRSXP`).
- `List`
  - Generic list vectors (`VECSXP`).
- `Complex`
  - Complex vectors (`CPLXSXP`).

**Methods:**

#### `sexptype`

```rust
const sexptype(self: Self) -> crate::SEXPTYPE
```

The [`SEXPTYPE`](crate::SEXPTYPE) an ALTREP vector of this base
presents to R.

### `altrep_data::core::Logical`

Logical value: TRUE, FALSE, or NA.

**Variants:**

- `False`
  - Logical false.
- `True`
  - Logical true.
- `Na`
  - Missing logical value.

**Methods:**

#### `from_bool`

```rust
from_bool(b: bool) -> Self
```

Convert from Rust bool (no NA representation).

#### `from_r_int`

```rust
from_r_int(i: i32) -> Self
```

Convert from R's integer representation.

#### `to_r_int`

```rust
to_r_int(self: Self) -> i32
```

Convert to R's integer representation.

### `altrep_data::core::Sortedness`

Sortedness hint for ALTREP vectors.

**Variants:**

- `Unknown`
  - Unknown sortedness.
- `KnownUnsorted`
  - Known to be unsorted.
- `Increasing`
  - Sorted in increasing order (may have ties).
- `Decreasing`
  - Sorted in decreasing order (may have ties).
- `IncreasingNaFirst`
  - Sorted in increasing order, with NAs first.
- `DecreasingNaFirst`
  - Sorted in decreasing order, with NAs first.

**Methods:**

#### `from_r_int`

```rust
from_r_int(i: i32) -> Self
```

Convert from R's integer representation.

#### `to_r_int`

```rust
to_r_int(self: Self) -> i32
```

Convert to R's integer representation.

### `altrep_traits::AltrepGuard`

Controls the panic/error guard used around ALTREP trampoline callbacks.

Each mode trades off safety vs performance:

- [`Unsafe`](AltrepGuard::Unsafe): No protection. If the callback panics,
  behavior is undefined (unwinding through C frames). Use only for trivial
  callbacks that cannot panic.

- [`RustUnwind`](AltrepGuard::RustUnwind): Wraps in `catch_unwind`, converting
  Rust panics to R errors. This is the **default** and safe for all pure-Rust
  callbacks. Overhead: ~1-2ns per call.

- [`RUnwind`](AltrepGuard::RUnwind): Wraps in `R_UnwindProtect`, catching both
  Rust panics and R `longjmp` errors. Use when ALTREP callbacks invoke R API
  functions that might error (e.g., `Rf_allocVector`, `Rf_eval`).

The guard is selected via the `const GUARD` associated constant on the [`Altrep`]
trait. Since it is a const, the compiler eliminates dead branches at
monomorphization time — zero runtime overhead for the chosen mode.

**Variants:**

- `Unsafe`
  - No protection. Fastest, but if the callback panics, behavior is undefined.
- `RustUnwind`
  - `catch_unwind` — catches Rust panics, converts to R errors. Default.
- `RUnwind`
  - `with_r_unwind_protect` — catches both Rust panics and R longjmps.

### `coerce::CoerceError`

Error type for coercion failures.

**Variants:**

- `Overflow`
  - The value cannot fit in the destination range.
- `PrecisionLoss`
  - The destination type cannot represent this value exactly.
- `NaN`
  - The input was NaN and destination disallows it.
- `Zero`
  - Zero is not allowed by the conversion rule.

### `coerce::LogicalCoerceError`

Error type for logical coercion failures.

**Variants:**

- `NAValue`
  - R's NA_LOGICAL cannot be represented as Rust bool
- `InvalidValue(i32)`
  - Value is not 0 or 1

### `dataframe::DataFrameError`

Error returned by any [`DataFrame`] construction, read, or conversion path.

This is the single data-frame error contract: the row-buffer build path, the serde
columnar path, the parallel R→Rust reader, and validation all surface a `DataFrameError`.

**Variants:**

- `NotList(String)`
  - The SEXP is not a VECSXP.
- `NotDataFrame`
  - The object does not inherit from `data.frame`.
- `NoNames`
  - The list has no `names` attribute (columns must be named).
- `BadRowNames(String)`
  - Could not extract `nrow` from `row.names` attribute.
- `UnequalLengths { ... }`
  - Columns have unequal lengths (when promoting from NamedList).
- `UnnamedColumns`
  - A row could not be turned into named columns (e.g. unnamed list elements
- `Conversion(String)`
  - A serde-driven schema/serialize/deserialize failure (the bridged

### `externalptr::TypeMismatchError`

Error returned when type checking fails in `try_from_sexp_with_error`.

The `found` field in `Mismatch` contains a `&'static str` from R's
interned symbol table, which persists for the R session lifetime.

**Variants:**

- `NullPointer`
  - The external pointer's address was null.
- `InvalidTypeId`
  - The prot slot didn't contain a valid type symbol.
- `Mismatch { ... }`
  - The stored type doesn't match the expected type.

### `ffi_guard::GuardMode`

FFI guard mode controlling how panics are caught at Rust-R boundaries.

**Variants:**

- `CatchUnwind`
  - `catch_unwind` only. On panic: fire telemetry, then `Rf_error` (diverges).
- `RUnwind`
  - `R_UnwindProtect`. Catches both Rust panics and R longjmps.

### `from_r::SexpError`

Unified conversion error when decoding an R `SEXP`.

**Variants:**

- `Type(SexpTypeError)`
  - `SEXPTYPE` did not match the expected one.
- `Length(SexpLengthError)`
  - Length did not match the expected one.
- `Na(SexpNaError)`
  - Missing value encountered where disallowed.
- `InvalidValue(String)`
  - Value is syntactically valid but semantically invalid (e.g. parse error).
- `MissingField(String)`
  - A required field was missing from a named list.
- `DuplicateName(String)`
  - A named list has duplicate non-empty names.
- `EitherConversion { ... }`
  - Failed to convert to `Either<L, R>` - both branches failed.

### `into_r_as::StorageCoerceError`

Error type for storage-directed conversion failures.

**Variants:**

- `Unsupported { ... }`
  - Conversion between these types is not supported.
- `OutOfRange { ... }`
  - Value is out of range for the target type.
- `NonFinite { ... }`
  - Value is non-finite (NaN or Inf) but target requires finite.
- `PrecisionLoss { ... }`
  - Conversion would lose precision.
- `NotIntegral { ... }`
  - Float value is not integral but target is integer type.
- `MissingValue { ... }`
  - Missing value (NA) cannot be represented in target type.
- `InvalidUtf8 { ... }`
  - Invalid UTF-8 in string conversion.

**Methods:**

#### `at_index`

```rust
at_index(self: Self, idx: usize) -> Self
```

Add index information to the error.

### `into_r_error::IntoRError`

Error returned by [`IntoR::try_into_sexp`](crate::into_r::IntoR::try_into_sexp)
for types whose conversion to R can fail.

#### Variants

- `StringTooLong` — a Rust string exceeds R's `i32` length limit (~2 GB)
- `LengthOverflow` — a collection length exceeds R's `R_xlen_t` capacity
- `Inner` — a sub-conversion failed (wraps the inner error message)

**Variants:**

- `StringTooLong { ... }`
  - A string's byte length exceeds `i32::MAX`.
- `LengthOverflow { ... }`
  - A collection's element count exceeds the target R vector capacity.
- `Inner(String)`
  - A nested conversion failed.

### `list::ListFromSexpError`

Error when converting SEXP to List fails.

**Variants:**

- `Type(crate::from_r::SexpTypeError)`
  - Wrong SEXP type.
- `DuplicateName(DuplicateNameError)`
  - Duplicate non-NA name found.

### `match_arg::MatchArgError`

Error type for `MatchArg` conversion failures.

**Variants:**

- `InvalidType(crate::SEXPTYPE)`
  - The SEXP was not a character or factor type.
- `InvalidLength(usize)`
  - The input had length != 1.
- `IsNa`
  - The input was NA.
- `NoMatch { ... }`
  - No choice matched the input.

### `missing::Missing`

Wrapper type that detects if an R argument was not passed (missing).

This corresponds to R's `missing()` function. When a function parameter
has type `Missing<T>`, it will be `Missing::Absent` if the caller didn't
provide that argument, or `Missing::Present(value)` if they did.

#### Example

```ignore
use miniextendr_api::{miniextendr, Missing};

#[miniextendr]
fn maybe_square(x: Missing<f64>) -> f64 {
    match x {
        Missing::Present(val) => val * val,
        Missing::Absent => 0.0,
    }
}
```

In R:
```r
maybe_square(5)  # 25
maybe_square()   # 0
```

**Variants:**

- `Present(T)`
  - The argument was provided.
- `Absent`
  - The argument was not provided (missing in R).

**Methods:**

#### `as_mut`

```rust
as_mut(self: &mut Self) -> Missing<&mut T>
```

Get a mutable reference to the value if present.

#### `as_ref`

```rust
as_ref(self: &Self) -> Missing<&T>
```

Get a reference to the value if present.

#### `expect`

```rust
expect(self: Self, msg: &str) -> T
```

Returns the contained value, panicking with a custom message if absent.

##### Panics

Panics with the provided message if the value is `Absent`.

#### `into_option`

```rust
into_option(self: Self) -> Option<T>
```

Convert to `Option<T>`, returning `None` if missing.

#### `is_missing`

```rust
is_missing(self: &Self) -> bool
```

Returns `true` if the argument was not provided.

Named to match R's `missing()` function.

#### `is_present`

```rust
is_present(self: &Self) -> bool
```

Returns `true` if the argument was provided.

#### `map`

```rust
map<U, F>(self: Self, f: F) -> Missing<U>
```

Maps `Missing<T>` to `Missing<U>` by applying a function.

#### `unwrap`

```rust
unwrap(self: Self) -> T
```

Returns the contained value, panicking if absent.

##### Panics

Panics if the value is `Absent`.

#### `unwrap_or`

```rust
unwrap_or(self: Self, default: T) -> T
```

Returns the contained value or a default.

#### `unwrap_or_default`

```rust
unwrap_or_default(self: Self) -> T
```

Returns the contained value or the default for that type.

#### `unwrap_or_else`

```rust
unwrap_or_else<F>(self: Self, f: F) -> T
```

Returns the contained value or computes it from a closure.

### `optionals::serde_impl::FactorHandling`

How to serialize R factors to JSON.

**Variants:**

- `Label`
  - Use the factor level label as a string (default).
- `Index`
  - Use the factor level index as an integer (1-based, matching R).

### `optionals::serde_impl::NaHandling`

How to handle NA values when converting R to JSON.

**Variants:**

- `Null`
  - Convert NA to JSON null (default).
- `Error`
  - Return an error when NA is encountered.
- `String(String)`
  - Convert NA to a custom string value.

### `optionals::serde_impl::SpecialFloatHandling`

How to handle special float values (NaN, Inf) when converting R to JSON.

**Variants:**

- `Error`
  - Return an error (default) - JSON has no representation for these.
- `Null`
  - Convert to JSON null.
- `String`
  - Convert to a string representation ("NaN", "Infinity", "-Infinity").

### `panic_telemetry::PanicSource`

Describes where a panic originated before being converted to an R error.

**Variants:**

- `Worker`
  - Panic on the worker thread (caught by `run_on_worker`).
- `Altrep`
  - Panic inside an ALTREP trampoline (caught by `catch_altrep_panic`).
- `UnwindProtect`
  - Panic inside `with_r_unwind_protect` (caught by `with_r_unwind_protect_sourced`).
- `Connection`
  - Panic inside a connection callback trampoline.

### `r_coerce::RCoerceError`

Error type for `as.<class>()` coercion failures.

This error type provides structured information about why a coercion failed,
allowing for meaningful error messages in R.

**Variants:**

- `NotSupported { ... }`
  - The conversion is not supported for this type combination.
- `InvalidData { ... }`
  - The conversion failed due to invalid or malformed data.
- `PrecisionLoss { ... }`
  - The conversion would result in unacceptable precision loss.
- `Custom(String)`
  - A custom error message.

### `raw_conversions::RawError`

Errors that can occur during raw conversion.

**Variants:**

- `LengthMismatch { ... }`
  - Length mismatch during conversion.
- `AlignmentMismatch { ... }`
  - Alignment mismatch (internal - we handle this by copying).
- `InvalidHeader(String)`
  - Invalid header in tagged format.
- `TypeMismatch { ... }`
  - Type name mismatch.

### `rcow::RCow`

An R-aware copy-on-write slice — the safe, zero-copy-round-trip alternative
to [`std::borrow::Cow<[T]>`](std::borrow::Cow).

See the [module docs](self) for why this exists and how it closes the #880
hazard. In brief: the [`Borrowed`](RCow::Borrowed) arm carries its source
SEXP, so returning it to R is a direct hand-back rather than a speculative
pointer recovery.

#### Example

```ignore
// Zero-copy in *and* out: the returned SEXP is the original R vector.
#[miniextendr]
pub fn passthrough(x: RCow<'static, f64>) -> RCow<'static, f64> {
    x
}

// Mutating forces a copy (copy-on-write), then materializes a fresh vector.
#[miniextendr]
pub fn doubled(mut x: RCow<'static, f64>) -> RCow<'static, f64> {
    for v in x.to_mut() {
        *v *= 2.0;
    }
    x
}
```

**Variants:**

- `Borrowed(RBorrow<'a, T>)`
  - Zero-copy view of a whole R vector, carrying its source SEXP.
- `Owned(Vec<T>)`
  - Owned data; materializes a fresh R vector on [`IntoR`].

**Methods:**

#### `into_owned`

```rust
into_owned(self: Self) -> Vec<T>
```

Consume into an owned [`Vec<T>`], cloning out of R if borrowed.

#### `is_borrowed`

```rust
is_borrowed(self: &Self) -> bool
```

`true` if this is a borrowed (zero-copy) view of an R vector.

#### `is_owned`

```rust
is_owned(self: &Self) -> bool
```

`true` if this owns its data.

#### `to_mut`

```rust
to_mut(self: &mut Self) -> &mut Vec<T>
```

Acquire a mutable reference to the owned data, cloning out of R first if
borrowed (copy-on-write). After this the `RCow` is always
[`Owned`](RCow::Owned).

### `registry::RWrapperPriority`

Ordering priority for R wrapper code fragments.

Variant declaration order = output order. The order matters because
R evaluates the wrapper file top-to-bottom, so dependencies must come first:
sidecar accessors before class definitions, classes before functions, etc.

**Variants:**

- `Sidecar`
  - `#[r_data]` getters/setters — must come before class definitions.
- `Class`
  - Class definitions (impl blocks: env/R6/S3/S4/S7).
- `Function`
  - Standalone `#[miniextendr]` functions.
- `TraitImpl`
  - Trait impl wrappers (`impl Trait for Type`).
- `Vctrs`
  - Vctrs S3 method wrappers (`#[derive(Vctrs)]`).

### `serde::columnar::DataFrameShape`

Categorical return shape for the dataframe-helpers family
([`vec_to_dataframe_split`] / [`result_to_dataframe`]).

Carries enough type information that downstream Rust code can `match`
on the variant without dispatching on SEXP type. Convert to a SEXP at
the `#[miniextendr]` function boundary via the [`crate::IntoR`] impl,
which collapses every variant to the equivalent R value (bare
data.frame / named list of data.frames / `list(results=, error=)`).

**Variants:**

- `Bare(crate::dataframe::DataFrame)`
  - Single data.frame.
- `Split { ... }`
  - `list(results = <df | sentinel>, error = df)`.
- `PerVariantList(Vec<(String, crate::dataframe::DataFrame)>)`
  - `list(VariantA = df, VariantB = df, …)`.

### `serde::columnar::ResultShape`

Shape selector for [`result_to_dataframe`].

Configures whether the helper returns a bare data.frame, a split
`list(results=, error=)`, or a collated single-data.frame with an
`is_error` column and the union of Ok and Err fields.

**Variants:**

- `Auto { ... }`
  - All-Ok input → bare data.frame; otherwise → `list(results=, error=)`.
- `Collated`
  - Single collated data.frame: every row, with an `is_error` LGLSXP
- `Split { ... }`
  - Always `list(results=, error=)`, even when all rows are `Ok` (in

### `serde::columnar::SplitResults`

Result partition for [`DataFrameShape::Split`].

Used to distinguish "no Ok rows at all" (which lets the caller supply
a sentinel value such as `NULL`, `NA`, `FALSE`, …) from a real
zero-row data.frame.

**Variants:**

- `Some(crate::dataframe::DataFrame)`
  - At least one `Ok` row — partition has a concrete data.frame.
- `None(crate::SEXP)`
  - No `Ok` rows — sentinel SEXP supplied by the caller via

### `serde::columnar::SplitShape`

Output-shape selector for [`vec_to_dataframe_split`].

Configures whether per-variant data.frames carry an explicit variant-tag
column, and whether the result is one list per variant or a single
collated data.frame with the variant name on every row.

The variant name on the R side is whatever serde emits (PascalCase by
default). Override with `#[serde(rename_all = "snake_case")]` (or
similar) on the enum definition.

**Variants:**

- `PerVariantList`
  - `list(VariantA = df, VariantB = df, …)` — historical behaviour.
- `PerVariantListWithTag { ... }`
  - Same shape as [`PerVariantList`](Self::PerVariantList) but each
- `Collated { ... }`
  - Single collated data.frame containing the union of every variant's

### `serde::columnar::TypeSpec`

User-facing column type descriptor for [`SerdeRowBuilder::with_schema`].

Maps onto the internal `ColumnType` and unlocks an NA-tolerance hint via
`Optional(_)`. The wrapper does **not** change the underlying column type —
`Optional(Integer)` produces an integer column where `None` lands as
`NA_INTEGER`. Without the hint, an all-`None` column discovered from the
first row would otherwise degrade to a logical-NA column (see
`vec_to_dataframe` doc).

**Variants:**

- `Logical`
  - R `logical` column (`bool`).
- `Integer`
  - R `integer` column (`i8`/`i16`/`i32`).
- `Real`
  - R `numeric` column (`f32`/`f64`/`i64`/`u64`).
- `Character`
  - R `character` column (`String`/`&str`).
- `Generic`
  - R generic list column (per-element SEXP fallback).
- `Optional(Box<TypeSpec>)`
  - NA-tolerance hint wrapping a base type. `Optional(Integer)` is an

### `serde::error::RSerdeError`

Error type for R serialization/deserialization.

This error type implements both `serde::ser::Error` and `serde::de::Error`,
allowing it to be used in both serialization and deserialization contexts.

**Variants:**

- `Message(String)`
  - Generic message error (from serde's `Error::custom`).
- `TypeMismatch { ... }`
  - Type mismatch during deserialization.
- `MissingField(String)`
  - Missing field in struct deserialization.
- `InvalidVariant { ... }`
  - Invalid enum variant during deserialization.
- `LengthMismatch { ... }`
  - Length mismatch (e.g., tuple deserialization).
- `UnexpectedNa`
  - NA value encountered where not allowed.
- `Overflow { ... }`
  - Value overflow during numeric conversion.
- `InvalidUtf8`
  - Invalid UTF-8 in R string.
- `NonStringKey`
  - Key was not a string (required for R named lists).
- `UnsupportedType { ... }`
  - Unsupported R type for deserialization.

### `sexp_types::Rboolean`

Binary boolean used by many R C APIs.

**Variants:**

- `FALSE`
  - False.
- `TRUE`
  - True.

### `sexp_types::SEXPTYPE`

R S-expression tag values (`SEXPTYPE`).

**Variants:**

- `NILSXP`
  - nil = NULL
- `SYMSXP`
  - symbols
- `LISTSXP`
  - lists of dotted pairs
- `CLOSXP`
  - closures
- `ENVSXP`
  - environments
- `PROMSXP`
  - promises: \[un\]evaluated closure arguments
- `LANGSXP`
  - language constructs (special lists)
- `SPECIALSXP`
  - special forms
- `BUILTINSXP`
  - builtin non-special forms
- `CHARSXP`
  - "scalar" string type (internal only)
- `LGLSXP`
  - logical vectors
- `INTSXP`
  - integer vectors
- `REALSXP`
  - real variables
- `CPLXSXP`
  - complex variables
- `STRSXP`
  - string vectors
- `DOTSXP`
  - dot-dot-dot object
- `ANYSXP`
  - make "any" args work
- `VECSXP`
  - generic vectors
- `EXPRSXP`
  - expressions vectors
- `BCODESXP`
  - byte code
- `EXTPTRSXP`
  - external pointer
- `WEAKREFSXP`
  - weak reference
- `RAWSXP`
  - raw bytes
- `S4SXP`
  - S4 non-vector
- `NEWSXP`
  - fresh node created in new page
- `FREESXP`
  - node released by GC
- `FUNSXP`
  - Closure or Builtin

**Methods:**

#### `type_name`

```rust
type_name(self: Self) -> &'static str
```

Get R's name for this SEXPTYPE (e.g. `"double"`, `"integer"`, `"list"`).

Returns the same string as R's `typeof()` function.

### `sexp_types::cetype_t`

Character encoding tag used by CHARSXP constructors.

**Variants:**

- `CE_NATIVE`
  - Native locale encoding.
- `CE_UTF8`
  - UTF-8 encoding.
- `CE_LATIN1`
  - Latin-1 encoding.
- `CE_BYTES`
  - Raw bytes encoding.
- `CE_SYMBOL`
  - Symbol encoding marker.
- `CE_ANY`
  - Any encoding accepted.

### `sys::N01type`

Normal distribution generator type enum from R_ext/Random.h

**Variants:**

- `BUGGY_KINDERMAN_RAMAGE`
  - Legacy buggy Kinderman-Ramage method.
- `AHRENS_DIETER`
  - Ahrens-Dieter method.
- `BOX_MULLER`
  - Box-Muller transform.
- `USER_NORM`
  - User-supplied normal generator.
- `INVERSION`
  - Inversion method.
- `KINDERMAN_RAMAGE`
  - Fixed Kinderman-Ramage method.

### `sys::ParseStatus`

Outcome of [`R_ParseVector`] (from `R_ext/Parse.h`).

`PARSE_NULL` is never returned by `R_ParseVector`; the meaningful success
value is [`ParseStatus::PARSE_OK`]. The remaining variants indicate parse
failures (`PARSE_ERROR`), incomplete input (`PARSE_INCOMPLETE`), or
end-of-input (`PARSE_EOF`).

**Variants:**

- `PARSE_NULL`
  - Never returned by `R_ParseVector`; the default-initialized sentinel.
- `PARSE_OK`
  - Parse succeeded.
- `PARSE_INCOMPLETE`
  - Input ended mid-expression (e.g. an unbalanced delimiter).
- `PARSE_ERROR`
  - A syntax error was encountered.
- `PARSE_EOF`
  - End of input reached with no further expressions.

### `sys::RNGtype`

RNG type enum from R_ext/Random.h

**Variants:**

- `WICHMANN_HILL`
  - Wichmann-Hill generator.
- `MARSAGLIA_MULTICARRY`
  - Marsaglia-Multicarry generator.
- `SUPER_DUPER`
  - Super-Duper generator.
- `MERSENNE_TWISTER`
  - Mersenne Twister generator.
- `KNUTH_TAOCP`
  - Knuth TAOCP generator.
- `USER_UNIF`
  - User-supplied uniform generator.
- `KNUTH_TAOCP2`
  - Knuth TAOCP 2002 variant.
- `LECUYER_CMRG`
  - L'Ecuyer-CMRG generator.

### `sys::Sampletype`

Discrete uniform sample method enum from R_ext/Random.h

**Variants:**

- `ROUNDING`
  - Rounding method for integer sampling.
- `REJECTION`
  - Rejection sampling method.

### `typed_list::TypeSpec`

Type specification for a single list element.

The optional `usize` parameter specifies an exact length constraint.
`None` means any length is accepted.

**Variants:**

- `Any`
  - Accept any type.
- `Numeric(Option<usize>)`
  - Numeric (real/double) vector. `REALSXP` only.
- `Integer(Option<usize>)`
  - Integer vector. `INTSXP` only.
- `Logical(Option<usize>)`
  - Logical vector.
- `Character(Option<usize>)`
  - Character vector.
- `Raw(Option<usize>)`
  - Raw vector.
- `Complex(Option<usize>)`
  - Complex vector.
- `List(Option<usize>)`
  - List (VECSXP or pairlist).
- `Class(&'static str)`
  - Object inheriting from a specific class.
- `DataFrame`
  - Data frame (inherits `data.frame`).
- `Factor`
  - Factor (`SEXP::is_factor`).
- `Matrix`
  - Matrix (`SEXP::is_matrix`).
- `Array`
  - Array (`SEXP::is_array`).
- `Function`
  - Function (`SEXP::is_function`).
- `Environment`
  - Environment (`SEXP::is_environment`).
- `Null`
  - NULL only (`SEXP::is_nil`).

**Methods:**

#### `type_name`

```rust
type_name(self: &Self) -> String
```

Get a human-readable name for this type specification.

### `typed_list::TypedListError`

Error returned when list validation fails.

**Variants:**

- `NotList(crate::list::ListFromSexpError)`
  - The input was not a list.
- `Missing { ... }`
  - A required field is missing.
- `WrongType { ... }`
  - A field has the wrong type.
- `WrongLen { ... }`
  - A field has the wrong length.
- `ExtraFields { ... }`
  - Extra named fields found when `allow_extra = false`.
- `DuplicateNames { ... }`
  - Duplicate non-empty names in the list.

### `vctrs::VctrsBuildError`

Error type for vctrs object construction.

#### Examples

```ignore
use miniextendr_api::vctrs::{new_vctr, VctrsBuildError};

match new_vctr(data, &["my_class"], &[], None) {
    Ok(sexp) => { /* use the vctrs object */ }
    Err(VctrsBuildError::NotAVector) => {
        eprintln!("Data is not a vector");
    }
    Err(e) => eprintln!("Build error: {}", e),
}
```

**Variants:**

- `NotAVector`
  - The data is not a vector type (atomic, list, or expression).
- `ListRequiresInheritBaseType`
  - List data requires `inherit_base_type = true`.
- `FieldLengthMismatch { ... }`
  - Record fields must all have the same length.
- `EmptyRecord`
  - Record must have at least one field.
- `DuplicateFieldName { ... }`
  - Record field names must be unique.
- `UnnamedFields`
  - Record fields must be named.
- `MissingPtypeOrSize`
  - list_of requires at least one of ptype or size.
- `InvalidSize { ... }`
  - Invalid size (must be non-negative).
- `EmptyClass`
  - Class vector must not be empty.

### `vctrs::VctrsKind`

The kind of vctrs class being created.

This corresponds to the different vctrs constructors:
- [`Vctr`](VctrsKind::Vctr): Simple vector backed by a base type (`vctrs::new_vctr`)
- [`Rcrd`](VctrsKind::Rcrd): Record type with named fields (`vctrs::new_rcrd`)
- [`ListOf`](VctrsKind::ListOf): Homogeneous list with prototype (`vctrs::new_list_of`)

#### Examples

```ignore
use miniextendr_api::vctrs::VctrsKind;

// VctrsKind defaults to Vctr
let kind = VctrsKind::default();
assert_eq!(kind, VctrsKind::Vctr);

// Use in a VctrsClass implementation to select the constructor
const KIND: VctrsKind = VctrsKind::Rcrd;
```

**Variants:**

- `Vctr`
  - Simple vctr backed by a base vector (double, integer, character, etc.).
- `Rcrd`
  - Record type with named fields of equal length.
- `ListOf`
  - Homogeneous list where all elements share a common prototype.

---

## Traits

### `adapter_traits::RClone`

Adapter trait for [`std::clone::Clone`].

Provides explicit deep copying for R. This is useful when R users need
to create independent copies of Rust objects (which normally use reference
semantics via `ExternalPtr`).

#### Methods

- `clone()` - Create a deep copy of this value

#### Example

```rust,ignore
#[derive(Clone, ExternalPtr)]
struct Buffer { data: Vec<u8> }

#[miniextendr]
impl RClone for Buffer {}
```

In R:
```r
buf1 <- Buffer$new(...)
buf2 <- buf1$clone()  # Independent copy
```

**Required methods:**

- `clone(self: &Self) -> Self`
  - Create a deep copy of this value.

### `adapter_traits::RCopy`

Adapter trait for [`std::marker::Copy`].

Indicates that a type can be cheaply copied (bitwise copy, no heap allocation).
This is useful for R users to know that copying is O(1) and doesn't involve
deep cloning of heap data.

#### Methods

- `copy()` - Create a bitwise copy of this value
- `is_copy()` - Returns true (useful for runtime type checking in R)

#### Difference from RClone

Both `RCopy` and `RClone` create copies, but:
- `RCopy`: Only for types where copying is cheap (stack-only, no heap)
- `RClone`: For any clonable type (may involve heap allocation)

If a type implements both, prefer `copy()` when you know copies are frequent.

#### Example

```rust,ignore
#[derive(Copy, Clone, ExternalPtr)]
struct Point { x: f64, y: f64 }

#[miniextendr]
impl RCopy for Point {}
```

In R:
```r
p1 <- Point$new(1.0, 2.0)
p2 <- p1$copy()  # Cheap bitwise copy
p1$is_copy()       # TRUE
```

**Required methods:**

- `copy(self: &Self) -> Self`
  - Create a bitwise copy of this value.
- `is_copy(self: &Self) -> bool`
  - Check if this type implements Copy.

### `adapter_traits::RDebug`

Adapter trait for [`std::fmt::Debug`].

Provides string representations for debugging and inspection in R.
Automatically implemented for any type that implements `Debug`.

#### Methods

- `debug_str()` - Returns compact debug string (`:?` format)
- `debug_str_pretty()` - Returns pretty-printed debug string (`:#?` format)

#### Example

```rust,ignore
#[derive(Debug, ExternalPtr)]
struct Config { name: String, value: i32 }

#[miniextendr]
impl RDebug for Config {}
```

**Required methods:**

- `debug_str(self: &Self) -> String`
  - Get a compact debug string representation.
- `debug_str_pretty(self: &Self) -> String`
  - Get a pretty-printed debug string with indentation.

### `adapter_traits::RDefault`

Adapter trait for [`std::default::Default`].

Provides default value construction for R. This allows R users to create
instances with default values without needing to specify all parameters.

#### Methods

- `default()` - Create a new instance with default values

#### Example

```rust,ignore
#[derive(Default, ExternalPtr)]
struct Config {
    timeout: u32,     // defaults to 0
    retries: u32,     // defaults to 0
    verbose: bool,    // defaults to false
}

#[miniextendr]
impl RDefault for Config {}
```

In R:
```r
config <- Config$default()  # All fields have default values
```

**Required methods:**

- `default() -> Self`
  - Create a new instance with default values.

### `adapter_traits::RDisplay`

Adapter trait for [`std::fmt::Display`].

Provides user-friendly string conversion for R.
Automatically implemented for any type that implements `Display`.

#### Methods

- `as_r_string()` - Returns the Display representation

#### Example

```rust,ignore
struct Version(u32, u32, u32);

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

#[miniextendr]
impl RDisplay for Version {}
```

**Required methods:**

- `as_r_string(self: &Self) -> String`
  - Convert to a user-friendly string.

### `adapter_traits::RError`

Adapter trait for [`std::error::Error`].

Provides error message extraction and error chain walking for R.
Automatically implemented for any type that implements `Error`.

#### Methods

- `error_message()` - Returns the error's display message
- `error_chain()` - Returns all messages in the error chain

#### Example

```rust,ignore
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct MyError { msg: String, source: Option<Box<dyn Error + Send + Sync>> }

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

// Wrap in ExternalPtr for R access
#[derive(ExternalPtr)]
struct MyErrorWrapper(MyError);

#[miniextendr]
impl RError for MyErrorWrapper {}
```

**Required methods:**

- `error_message(self: &Self) -> String`
  - Get the error message (Display representation).
- `error_chain(self: &Self) -> Vec<String>`
  - Get all error messages in the chain, from outermost to innermost.
- `error_chain_length(self: &Self) -> i32`
  - Get the number of errors in the chain.

### `adapter_traits::RExtend`

Adapter trait for [`std::iter::Extend`].

Provides collection extension operations for R, allowing Rust collections
to be extended with R vectors. Since extension requires mutation, the
wrapper type should use interior mutability (e.g., `RefCell`).

#### Methods

- `extend_from_vec(items)` - Extend the collection with items from a vector
- `extend_from_slice(items)` - Extend from a slice (for Clone items)

#### Example

```rust,ignore
use std::cell::RefCell;

#[derive(ExternalPtr)]
struct MyVec(RefCell<Vec<i32>>);

impl MyVec {
    fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }
}

impl RExtend<i32> for MyVec {
    fn extend_from_vec(&self, items: Vec<i32>) {
        self.0.borrow_mut().extend(items);
    }
}

#[miniextendr]
impl RExtend<i32> for MyVec {}
```

In R:
```r
v <- MyVec$new()
v$extend_from_vec(c(1L, 2L, 3L))  # Add items
v$extend_from_vec(c(4L, 5L))      # Add more items
```

#### Design Note

Like `RIterator`, `RExtend` does NOT have a blanket impl because `Extend::extend()`
requires `&mut self`, but R's ExternalPtr pattern provides `&self`. Users must
implement this trait manually using interior mutability (RefCell, Mutex, etc.).

**Required methods:**

- `extend_from_vec(self: &Self, items: Vec<T>)`
  - Extend the collection with items from a vector.

**Provided methods:**

- `extend_from_slice(self: &Self, items: &[T])`
  - Extend the collection with cloned items from a slice.
- `len(self: &Self) -> i64`
  - Get the current length of the collection.
- `is_empty(self: &Self) -> bool`
  - Check if the collection is empty.

### `adapter_traits::RFromIter`

Adapter trait for [`std::iter::FromIterator`].

Provides collection construction from iterators/vectors for R.
Unlike `RExtend`, this creates a new collection from items.

#### Methods

- `from_vec(items)` - Create a new collection from a vector

#### Example

```rust,ignore
#[derive(ExternalPtr)]
struct MySet(std::collections::HashSet<i32>);

impl RFromIter<i32> for MySet {
    fn from_vec(items: Vec<i32>) -> Self {
        Self(items.into_iter().collect())
    }
}

#[miniextendr]
impl RFromIter<i32> for MySet {}
```

In R:
```r
set <- MySet$from_vec(c(1L, 2L, 2L, 3L))  # Creates {1, 2, 3}
```

**Required methods:**

- `from_vec(items: Vec<T>) -> Self`
  - Create a new collection from a vector of items.

### `adapter_traits::RFromStr`

Adapter trait for [`std::str::FromStr`].

Provides string parsing for R, allowing R strings to be parsed into Rust types.
Automatically implemented for any type that implements `FromStr`.

#### Methods

- `from_str(s: &str)` - Parse a string into this type, returning None on failure

#### Example

```rust,ignore
use std::net::IpAddr;

// IpAddr implements FromStr
#[derive(ExternalPtr)]
struct IpAddress(IpAddr);

#[miniextendr]
impl RFromStr for IpAddress {}
```

In R:
```r
ip <- IpAddress$from_str("192.168.1.1")
```

**Required methods:**

- `from_str(s: &str) -> Option<Self>`
  - Parse a string into this type.

### `adapter_traits::RHash`

Adapter trait for [`std::hash::Hash`].

Provides hashing for deduplication and environment keys in R.
Automatically implemented for any type that implements `Hash`.

#### Methods

- `hash()` - Returns a 64-bit hash as i64

#### Note

Hash values are deterministic within a single R session but may vary
between sessions due to Rust's hasher implementation.

#### Example

```rust,ignore
#[derive(Hash, ExternalPtr)]
struct Record { id: String, value: i64 }

#[miniextendr]
impl RHash for Record {}
```

**Required methods:**

- `hash(self: &Self) -> i64`
  - Compute a hash of this value.

### `adapter_traits::RIterator`

Adapter trait for [`std::iter::Iterator`].

Provides iterator operations for R, allowing Rust iterators to be consumed
element-by-element from R code. Since iterators are stateful, the wrapper
type should use interior mutability (e.g., `RefCell`).

#### Methods

- `next()` - Get the next element, or None if exhausted
- `size_hint()` - Get estimated remaining elements as `c(lower, upper)`
- `count()` - Consume and count remaining elements
- `collect_n(n)` - Collect up to n elements into a vector
- `skip(n)` - Skip n elements
- `nth(n)` - Get the nth element (0-indexed)

#### Example

```rust,ignore
use std::cell::RefCell;

#[derive(ExternalPtr)]
struct MyIter(RefCell<std::vec::IntoIter<i32>>);

impl MyIter {
    fn new(data: Vec<i32>) -> Self {
        Self(RefCell::new(data.into_iter()))
    }
}

impl RIterator for MyIter {
    type Item = i32;

    fn next(&self) -> Option<Self::Item> {
        self.0.borrow_mut().next()
    }

    fn size_hint(&self) -> (i64, Option<i64>) {
        let (lo, hi) = self.0.borrow().size_hint();
        (lo as i64, hi.map(|h| h as i64))
    }
}

#[miniextendr]
impl RIterator for MyIter {}
```

In R (note: `next` is a reserved word, so expose as `next_item` or similar):
```r
it <- MyIter$new(c(1L, 2L, 3L))
it$next_item()   # 1L
it$next_item()   # 2L
it$size_hint()   # c(1, 1) - one element remaining
it$next_item()   # 3L
it$next_item()   # NULL (exhausted)
```

#### Design Note

Unlike other adapter traits, `RIterator` does NOT have a blanket impl
because iterators require `&mut self` for `next()`, but R's ExternalPtr
pattern typically provides `&self`. Users must implement this trait
manually using interior mutability (RefCell, Mutex, etc.).

**Required methods:**

- `next(self: &Self) -> Option<<Self as >::Item>`
  - Get the next element from the iterator.
- `size_hint(self: &Self) -> (i64, Option<i64>)`
  - Get the estimated number of remaining elements.

**Provided methods:**

- `count(self: &Self) -> i64`
  - Consume the iterator and count remaining elements.
- `collect_n(self: &Self, n: i32) -> Vec<<Self as >::Item>`
  - Collect up to `n` elements into a vector.
- `skip(self: &Self, n: i32) -> i32`
  - Skip `n` elements from the iterator.
- `nth(self: &Self, n: i32) -> Option<<Self as >::Item>`
  - Get the `n`th element (0-indexed), consuming elements up to and including it.

**Associated items:**

- associated type `Item`

### `adapter_traits::RMakeIter`

Adapter trait for creating iterator wrappers from collections.

This trait provides a way to create an [`RIterator`] wrapper from a collection.
Since `ExternalPtr` methods receive `&self`, this trait clones the underlying
data to create an independent iterator.

#### Type Parameters

- `T`: The element type yielded by the iterator
- `I`: The iterator type returned (must implement [`RIterator`])

#### Design Note

The returned iterator is independent from the source collection. Modifications
to the original collection after calling `make_iter()` won't affect the
iterator's output.

#### Example

```rust,ignore
use std::cell::RefCell;

#[derive(ExternalPtr)]
struct MyVec(Vec<i32>);

#[derive(ExternalPtr)]
struct MyVecIter(RefCell<std::vec::IntoIter<i32>>);

impl RIterator for MyVecIter {
    type Item = i32;
    fn next(&self) -> Option<i32> {
        self.0.borrow_mut().next()
    }
    fn size_hint(&self) -> (i64, Option<i64>) {
        let (lo, hi) = self.0.borrow().size_hint();
        (lo as i64, hi.map(|h| h as i64))
    }
}

impl RMakeIter<i32, MyVecIter> for MyVec {
    fn make_iter(&self) -> MyVecIter {
        MyVecIter(RefCell::new(self.0.clone().into_iter()))
    }
}

#[miniextendr]
impl RMakeIter<i32, MyVecIter> for MyVec {}
```

In R (note: expose `next` as `next_item` since `next` is reserved):
```r
v <- MyVec$new(c(1L, 2L, 3L))
it <- v$make_iter()   # Create iterator
it$next_item()        # 1L
it$next_item()        # 2L
v$to_vec()            # c(1L, 2L, 3L) - original unchanged
```

**Required methods:**

- `make_iter(self: &Self) -> I`
  - Create a new iterator wrapper.

### `adapter_traits::ROrd`

Adapter trait for [`std::cmp::Ord`].

Provides total ordering comparison for R sorting operations.
Automatically implemented for any type that implements `Ord`.

#### Methods

- `cmp(&self, other: &Self)` - Returns -1, 0, or 1

#### Example

```rust,ignore
#[derive(Ord, PartialOrd, Eq, PartialEq, ExternalPtr)]
struct Priority(u32);

#[miniextendr]
impl ROrd for Priority {}
```

**Required methods:**

- `cmp(self: &Self, other: &Self) -> i32`
  - Compare with another value.

### `adapter_traits::RPartialOrd`

Adapter trait for [`std::cmp::PartialOrd`].

Provides partial ordering comparison for R, handling incomparable values.
Automatically implemented for any type that implements `PartialOrd`.

#### Methods

- `partial_cmp(&self, other: &Self)` - Returns Some(-1/0/1) or None

#### Example

```rust,ignore
// f64 has partial ordering (NaN is not comparable)
#[derive(PartialOrd, PartialEq, ExternalPtr)]
struct MyFloat(f64);

#[miniextendr]
impl RPartialOrd for MyFloat {}
```

**Required methods:**

- `partial_cmp(self: &Self, other: &Self) -> Option<i32>`
  - Compare with another value, returning None if incomparable.

### `adapter_traits::RToVec`

Adapter trait for collections that can be converted to vectors.

This is the complement to [`RFromIter`]: while `RFromIter` creates collections
from vectors, `RToVec` extracts vectors from collections.

#### Methods

- `to_vec()` - Collect all elements into a vector (cloning elements)
- `len()` - Get the number of elements
- `is_empty()` - Check if the collection is empty

#### Design Note

Unlike Rust's `IntoIterator::into_iter()` which consumes the collection,
this trait borrows the collection and clones elements. This is necessary
because R's `ExternalPtr` pattern provides `&self`, not owned `self`.

For consuming iteration, use [`RIterator`] with interior mutability.

#### Example

```rust,ignore
use std::collections::HashSet;

#[derive(ExternalPtr)]
struct MySet(HashSet<i32>);

// RToVec is automatically available via blanket impl
#[miniextendr]
impl RToVec<i32> for MySet {}
```

In R:
```r
set <- MySet$new(...)
vec <- set$to_vec()    # Get all elements as vector
set$len()              # Number of elements
set$is_empty()         # Check if empty
```

**Required methods:**

- `to_vec(self: &Self) -> Vec<T>`
  - Collect all elements into a vector.
- `len(self: &Self) -> i64`
  - Get the number of elements in the collection.

**Provided methods:**

- `is_empty(self: &Self) -> bool`
  - Check if the collection is empty.

### `altrep::AltrepClass`

Trait implemented by ALTREP classes via `#[miniextendr]`.

This trait is automatically implemented when using the proc-macro with
ALTREP attributes (class, pkg, base).

**Associated items:**

- associated const `CLASS_NAME: &'static std::ffi::CStr`
- associated const `BASE: RBase`

### `altrep::RegisterAltrep`

Registration trait: implemented per type by the macro on struct items.

The `get_or_init_class` method returns the ALTREP class handle, initializing
it on first call and returning the cached handle on subsequent calls.

This trait combines class creation and method installation into a single
`get_or_init_class` call that caches the result.

**Required methods:**

- `get_or_init_class() -> R_altrep_class_t`
  - Get the ALTREP class handle, initializing it if this is the first call.

### `altrep_data::core::AltrepDataptr`

Trait for ALTREP types that can expose a data pointer.

#### Writability contract

When `writable = true`, R **will** write through the returned pointer
(e.g., `x[i] <- val`). The implementation must ensure:

1. The returned pointer is safe to write to (not read-only memory).
2. Writes are visible to subsequent `Elt`/`Get_region` calls (no stale cache).

For owned containers (`Vec<T>`, `Box<[T]>`), this is automatic because
DATAPTR and Elt both access the same allocation (data1).

For copy-on-write types (`Cow<'static, [T]>`), `writable = true` should
trigger the copy so writes go to owned memory. When `writable = false`,
the borrowed pointer can be returned directly.

For immutable data (`&'static [T]`), `writable = true` should panic or
return `None` since the data cannot be modified.

The `__impl_altvec_dataptr` macro uses `dataptr_or_null` for read-only
access and only calls `dataptr(&mut self, true)` when R requests a
writable pointer.

**Required methods:**

- `dataptr(self: &mut Self, writable: bool) -> Option<*mut T>`
  - Get a pointer to the underlying data, possibly triggering materialization.

**Provided methods:**

- `dataptr_or_null(self: &Self) -> Option<*const T>`
  - Get a read-only pointer without forcing materialization.

### `altrep_data::core::AltrepExtract`

How to extract a reference to `Self` from an ALTREP SEXP's data1 slot.

The default implementation (for types that implement `TypedExternal`) extracts
via `ExternalPtr<T>` downcast from data1. Power users who want native SEXP
storage can implement this trait manually.

#### Safety

Implementations must ensure that the returned references are valid for the
duration of the ALTREP callback (i.e., the SEXP is protected by R's GC).

#### Panics

The blanket implementation panics if ExternalPtr extraction fails (type
mismatch, null pointer, etc.). This is a programmer error, not a runtime
condition. Callers must ensure the ALTREP SEXP was created with the correct
data type. The panic is caught by the ALTREP guard (`RustUnwind` or `RUnwind`)
and converted to an R error. Using `AltrepGuard::Unsafe` with a type that
can fail extraction is unsound.

**Required methods:**

- `unsafe altrep_extract_ref(x: crate::SEXP) -> &'static Self`
  - Extract a shared reference from the ALTREP data1 slot.
- `unsafe altrep_extract_mut(x: crate::SEXP) -> &'static mut Self`
  - Extract a mutable reference from the ALTREP data1 slot.

### `altrep_data::core::AltrepExtractSubset`

Trait for ALTREP types that can provide optimized subsetting.

**Required methods:**

- `extract_subset(self: &Self, indices: &[i32]) -> Option<SEXP>`
  - Extract a subset of this ALTREP.

### `altrep_data::core::AltrepLen`

Base trait for ALTREP data types. All ALTREP types must provide length.

**Required methods:**

- `len(self: &Self) -> usize`
  - Returns the length of this ALTREP vector.

**Provided methods:**

- `is_empty(self: &Self) -> bool`
  - Returns true if the vector is empty.

### `altrep_data::core::AltrepSerialize`

Trait for ALTREP types that support serialization.

**Required methods:**

- `serialized_state(self: &Self) -> SEXP`
  - Convert the ALTREP data to a serializable R object.
- `unserialize(state: SEXP) -> Option<Self>`
  - Reconstruct the ALTREP data from a serialized state.

### `altrep_data::core::InferBase`

Trait for inferring the R base type from a data type's implemented traits.

This is automatically implemented via blanket impls for types that implement
one of the `Alt*Data` traits. It allows the `#[miniextendr]` macro to infer
the base type without requiring an explicit `base = \"...\"` attribute.

**Required methods:**

- `unsafe make_class(class_name: *const i8, pkg_name: *const i8) -> crate::sys::altrep::R_altrep_class_t`
  - Create the ALTREP class handle.
- `unsafe install_methods(cls: crate::sys::altrep::R_altrep_class_t)`
  - Install ALTREP methods on the class.

**Associated items:**

- associated const `BASE: crate::altrep::RBase`

### `altrep_data::traits::AltComplexData`

Trait for types that can back an ALTCOMPLEX vector.

**Required methods:**

- `elt(self: &Self, i: usize) -> Rcomplex`
  - Get the complex element at index `i`.

**Provided methods:**

- `as_slice(self: &Self) -> Option<&[Rcomplex]>`
  - Optional: return a slice if data is contiguous.
- `get_region(self: &Self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize`
  - Optional: bulk read into buffer (clamped to available data).

### `altrep_data::traits::AltIntegerData`

Trait for types that can back an ALTINTEGER vector.

Implement this to create custom integer ALTREP classes.

**Required methods:**

- `elt(self: &Self, i: usize) -> i32`
  - Get the integer element at index `i`.

**Provided methods:**

- `as_slice(self: &Self) -> Option<&[i32]>`
  - Optional: return a pointer to contiguous data if available.
- `get_region(self: &Self, start: usize, len: usize, buf: &mut [i32]) -> usize`
  - Optional: bulk read into buffer. Returns number of elements read.
- `is_sorted(self: &Self) -> Option<Sortedness>`
  - Optional: sortedness hint. Default is unknown.
- `no_na(self: &Self) -> Option<bool>`
  - Optional: does this vector contain any NA values?
- `sum(self: &Self, _na_rm: bool) -> Option<i64>`
  - Optional: optimized sum. Default returns None (use R's default).
- `min(self: &Self, _na_rm: bool) -> Option<i32>`
  - Optional: optimized min. Default returns None (use R's default).
- `max(self: &Self, _na_rm: bool) -> Option<i32>`
  - Optional: optimized max. Default returns None (use R's default).

### `altrep_data::traits::AltListData`

Trait for types that can back an ALTLIST vector.

List elements are arbitrary SEXPs, so this trait works with raw SEXP.

**Required methods:**

- `elt(self: &Self, i: usize) -> SEXP`
  - Get the list element at index `i`.

### `altrep_data::traits::AltLogicalData`

Trait for types that can back an ALTLOGICAL vector.

**Required methods:**

- `elt(self: &Self, i: usize) -> Logical`
  - Get the logical element at index `i`.

**Provided methods:**

- `as_r_slice(self: &Self) -> Option<&[i32]>`
  - Optional: return a slice if data is contiguous i32 (R's internal format).
- `get_region(self: &Self, start: usize, len: usize, buf: &mut [i32]) -> usize`
  - Optional: bulk read into buffer (clamped to available data).
- `is_sorted(self: &Self) -> Option<Sortedness>`
  - Optional: sortedness hint.
- `no_na(self: &Self) -> Option<bool>`
  - Optional: does this vector contain any NA values?
- `sum(self: &Self, _na_rm: bool) -> Option<i64>`
  - Optional: optimized sum (count of TRUE values).

### `altrep_data::traits::AltRawData`

Trait for types that can back an ALTRAW vector.

**Required methods:**

- `elt(self: &Self, i: usize) -> u8`
  - Get the raw byte at index `i`.

**Provided methods:**

- `as_slice(self: &Self) -> Option<&[u8]>`
  - Optional: return a slice if data is contiguous.
- `get_region(self: &Self, start: usize, len: usize, buf: &mut [u8]) -> usize`
  - Optional: bulk read into buffer (clamped to available data).

### `altrep_data::traits::AltRealData`

Trait for types that can back an ALTREAL vector.

**Required methods:**

- `elt(self: &Self, i: usize) -> f64`
  - Get the real element at index `i`.

**Provided methods:**

- `as_slice(self: &Self) -> Option<&[f64]>`
  - Optional: return a pointer to contiguous data if available.
- `get_region(self: &Self, start: usize, len: usize, buf: &mut [f64]) -> usize`
  - Optional: bulk read into buffer (clamped to available data).
- `is_sorted(self: &Self) -> Option<Sortedness>`
  - Optional: sortedness hint.
- `no_na(self: &Self) -> Option<bool>`
  - Optional: does this vector contain any NA values?
- `sum(self: &Self, _na_rm: bool) -> Option<f64>`
  - Optional: optimized sum.
- `min(self: &Self, _na_rm: bool) -> Option<f64>`
  - Optional: optimized min.
- `max(self: &Self, _na_rm: bool) -> Option<f64>`
  - Optional: optimized max.

### `altrep_data::traits::AltStringData`

Trait for types that can back an ALTSTRING vector.

Note: `elt` returns a `&str` which will be converted to CHARSXP.

**Required methods:**

- `elt(self: &Self, i: usize) -> Option<&str>`
  - Get the string element at index `i`.

**Provided methods:**

- `is_sorted(self: &Self) -> Option<Sortedness>`
  - Optional: sortedness hint.
- `no_na(self: &Self) -> Option<bool>`
  - Optional: does this vector contain any NA values?

### `altrep_ext::AltrepSexpExt`

ALTREP-specific extension methods for SEXP.

These methods wrap the free functions in `sys::altrep`, converting
`func(x)` calls to `x.method()` calls. This avoids the
`clippy::not_unsafe_ptr_arg_deref` lint in ALTREP trait method
implementations that receive SEXP as a parameter.

Only data2 (cache) accessors are exposed here; data1 (storage) is
accessed via the `AltrepExtract` trait or the standalone free functions
`altrep_data1_as` / `altrep_data1_as_unchecked` / `altrep_data1_mut` /
`altrep_data1_mut_unchecked`.

**Required methods:**

- `unsafe altrep_data2_raw(self: &Self) -> SEXP`
  - Get the raw SEXP in the ALTREP data2 slot.
- `unsafe altrep_data2_raw_unchecked(self: &Self) -> SEXP`
  - Get the ALTREP data2 slot (unchecked — no thread routing).
- `unsafe set_altrep_data2(self: &Self, data2: SEXP)`
  - Set the ALTREP data2 slot.
- `unsafe set_altrep_data2_unchecked(self: &Self, data2: SEXP)`
  - Set the ALTREP data2 slot (unchecked — no thread routing).

### `altrep_traits::AltComplex`

Complex vector methods.

**Provided methods:**

- `elt(_x: SEXP, _i: R_xlen_t) -> Rcomplex`
  - Get element at index.
- `get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: &mut [Rcomplex]) -> R_xlen_t`
  - Bulk read elements into buffer.

**Associated items:**

- associated const `HAS_ELT: bool`
- associated const `HAS_GET_REGION: bool`

### `altrep_traits::AltInteger`

Integer vector methods.

For ALTINTEGER, you must provide EITHER:
- `HAS_ELT = true` with `elt()` implementation, OR
- `HAS_DATAPTR = true` with `dataptr()` implementation

If neither is provided, R will error at runtime when accessing elements.

**Provided methods:**

- `elt(_x: SEXP, _i: R_xlen_t) -> i32`
  - Get element at index.
- `get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: &mut [i32]) -> R_xlen_t`
  - Bulk read elements into buffer.
- `is_sorted(_x: SEXP) -> i32`
  - Sortedness hint.
- `no_na(_x: SEXP) -> i32`
  - NA-free hint.
- `sum(_x: SEXP, _narm: bool) -> SEXP`
  - Optimized sum.
- `min(_x: SEXP, _narm: bool) -> SEXP`
  - Optimized min.
- `max(_x: SEXP, _narm: bool) -> SEXP`
  - Optimized max.

**Associated items:**

- associated const `HAS_ELT: bool`
- associated const `HAS_GET_REGION: bool`
- associated const `HAS_IS_SORTED: bool`
- associated const `HAS_NO_NA: bool`
- associated const `HAS_SUM: bool`
- associated const `HAS_MIN: bool`
- associated const `HAS_MAX: bool`

### `altrep_traits::AltList`

List vector methods.

**REQUIRED**: `elt` must be implemented (no default).
R will error with "must provide an Elt method" if you don't provide it.

**Required methods:**

- `elt(x: SEXP, i: R_xlen_t) -> SEXP`
  - Get list element at index. Returns any SEXP.

**Provided methods:**

- `set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP)`
  - Set element (for mutable lists).

**Associated items:**

- associated const `HAS_SET_ELT: bool`

### `altrep_traits::AltLogical`

Logical vector methods.

**Provided methods:**

- `elt(_x: SEXP, _i: R_xlen_t) -> i32`
  - Returns i32: 0=FALSE, 1=TRUE, NA_LOGICAL=NA
- `get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: &mut [i32]) -> R_xlen_t`
  - Bulk read elements into buffer.
- `is_sorted(_x: SEXP) -> i32`
  - Sortedness hint.
- `no_na(_x: SEXP) -> i32`
  - NA-free hint.
- `sum(_x: SEXP, _narm: bool) -> SEXP`
  - Sum for logical = count of TRUE values.

**Associated items:**

- associated const `HAS_ELT: bool`
- associated const `HAS_GET_REGION: bool`
- associated const `HAS_IS_SORTED: bool`
- associated const `HAS_NO_NA: bool`
- associated const `HAS_SUM: bool`

### `altrep_traits::AltRaw`

Raw vector methods.

**Provided methods:**

- `elt(_x: SEXP, _i: R_xlen_t) -> u8`
  - Get element at index.
- `get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: &mut [u8]) -> R_xlen_t`
  - Bulk read elements into buffer.

**Associated items:**

- associated const `HAS_ELT: bool`
- associated const `HAS_GET_REGION: bool`

### `altrep_traits::AltReal`

Real vector methods.

**Provided methods:**

- `elt(_x: SEXP, _i: R_xlen_t) -> f64`
  - Get element at index.
- `get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: &mut [f64]) -> R_xlen_t`
  - Bulk read elements into buffer.
- `is_sorted(_x: SEXP) -> i32`
  - Sortedness hint.
- `no_na(_x: SEXP) -> i32`
  - NA-free hint.
- `sum(_x: SEXP, _narm: bool) -> SEXP`
  - Optimized sum.
- `min(_x: SEXP, _narm: bool) -> SEXP`
  - Optimized min.
- `max(_x: SEXP, _narm: bool) -> SEXP`
  - Optimized max.

**Associated items:**

- associated const `HAS_ELT: bool`
- associated const `HAS_GET_REGION: bool`
- associated const `HAS_IS_SORTED: bool`
- associated const `HAS_NO_NA: bool`
- associated const `HAS_SUM: bool`
- associated const `HAS_MIN: bool`
- associated const `HAS_MAX: bool`

### `altrep_traits::AltString`

String vector methods.

**REQUIRED**: `elt` must be implemented (no default).
R will error with "No Elt method found" if you don't provide it.

**Required methods:**

- `elt(x: SEXP, i: R_xlen_t) -> SEXP`
  - Get string element at index. Returns CHARSXP.

**Provided methods:**

- `set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP)`
  - Set element (for mutable strings).
- `is_sorted(_x: SEXP) -> i32`
  - Sortedness hint.
- `no_na(_x: SEXP) -> i32`
  - NA-free hint.

**Associated items:**

- associated const `HAS_SET_ELT: bool`
- associated const `HAS_IS_SORTED: bool`
- associated const `HAS_NO_NA: bool`

### `altrep_traits::AltVec`

Vector-level methods.

All methods are optional with HAS_* gating.

**Provided methods:**

- `dataptr(_x: SEXP, _writable: bool) -> *mut c_void`
  - Get raw data pointer.
- `dataptr_or_null(_x: SEXP) -> *const c_void`
  - Get data pointer without forcing materialization.
- `extract_subset(_x: SEXP, _indx: SEXP, _call: SEXP) -> SEXP`
  - Optimized subsetting.

**Associated items:**

- associated const `HAS_DATAPTR: bool`
- associated const `HAS_DATAPTR_OR_NULL: bool`
- associated const `HAS_EXTRACT_SUBSET: bool`

### `altrep_traits::Altrep`

Base ALTREP methods.

`length` is REQUIRED (no default). All other methods are optional with HAS_* gating.

**Required methods:**

- `length(x: SEXP) -> R_xlen_t`
  - Returns the length of the ALTREP vector.

**Provided methods:**

- `serialized_state(_x: SEXP) -> SEXP`
  - Return serialization state.
- `unserialize(_class: SEXP, _state: SEXP) -> SEXP`
  - Reconstruct ALTREP from serialized state.
- `unserialize_ex(_class: SEXP, _state: SEXP, _attr: SEXP, _objf: i32, _levs: i32) -> SEXP`
  - Extended unserialization with attributes.
- `duplicate(_x: SEXP, _deep: bool) -> SEXP`
  - Duplicate the ALTREP object.
- `duplicate_ex(_x: SEXP, _deep: bool) -> SEXP`
  - Extended duplication.
- `coerce(_x: SEXP, _to_type: SEXPTYPE) -> SEXP`
  - Coerce to another type.
- `inspect(_x: SEXP, _pre: i32, _deep: i32, _pvec: i32, _inspect_subtree: Option<{'function_pointer': {'sig': {'inputs': [['_', {'resolved_path': {'path': 'SEXP', 'id': 236, 'args': None}}], ['_', {'primitive': 'i32'}], ['_', {'primitive': 'i32'}], ['_', {'primitive': 'i32'}]], 'output': None, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>) -> bool`
  - Custom inspection for `.Internal(inspect())`.

**Associated items:**

- associated const `GUARD: AltrepGuard`
- associated const `HAS_SERIALIZED_STATE: bool`
- associated const `HAS_UNSERIALIZE: bool`
- associated const `HAS_UNSERIALIZE_EX: bool`
- associated const `HAS_DUPLICATE: bool`
- associated const `HAS_DUPLICATE_EX: bool`
- associated const `HAS_COERCE: bool`
- associated const `HAS_INSPECT: bool`

### `coerce::Coerce`

Infallible coercion from `Self` to type `R`.

Implement this trait for types that can always be converted to `R`.
Identity and widening conversions should use this trait.

Pair: [`TryCoerce`] for fallible (narrowing) coercions. Strict alternative
on the inbound side: [`crate::from_r::TryFromSexp`].

Works for both scalars and element-wise on slices:
- `i8::coerce() -> i32` (scalar widening)
- `&[i8]::coerce() -> Vec<i32>` (element-wise)

#### Example

```ignore
impl Coerce<i32> for MyType {
    fn coerce(self) -> i32 { ... }
}
```

**Required methods:**

- `coerce(self: Self) -> R`
  - Convert `self` into `R`.

### `coerce::TryCoerce`

Fallible coercion from `Self` to type `R`.

Implement this trait for narrowing conversions that may overflow or lose precision.

Pair: [`Coerce`] for infallible (widening / identity) coercions. Strict
alternative on the inbound side: [`crate::from_r::TryFromSexp`].

**Required methods:**

- `try_coerce(self: Self) -> Result<R, <Self as >::Error>`
  - Attempt to convert `self` into `R`.

**Associated items:**

- associated type `Error`

### `convert::AsDataFrameExt`

Extension trait for wrapping values as [`AsDataFrame`].

Automatically implemented for all `T: IntoDataFrame` (typically `Vec<Row>` where `Row`
derives `DataFrameRow`).

**Provided methods:**

- `wrap_data_frame(self: Self) -> AsDataFrame<Self>`
  - Wrap `self` in [`AsDataFrame`] for R data.frame conversion.

### `convert::AsExternalPtrExt`

Extension trait for wrapping values as [`AsExternalPtr`].

This trait is automatically implemented for all types that implement [`IntoExternalPtr`].

#### Example

```ignore
use miniextendr_api::convert::AsExternalPtrExt;

#[derive(ExternalPtr)]
struct Connection { handle: u64 }

let conn = Connection { handle: 42 };
let wrapped: AsExternalPtr<Connection> = conn.wrap_external_ptr();
```

**Provided methods:**

- `wrap_external_ptr(self: Self) -> AsExternalPtr<Self>`
  - Wrap `self` in [`AsExternalPtr`] for R external pointer conversion.

### `convert::AsListExt`

Extension trait for wrapping values as [`AsList`].

This trait is automatically implemented for all types that implement [`IntoList`].

#### Example

```ignore
use miniextendr_api::convert::AsListExt;

#[derive(IntoList)]
struct Point { x: f64, y: f64 }

let point = Point { x: 1.0, y: 2.0 };
let wrapped: AsList<Point> = point.wrap_list();
```

**Provided methods:**

- `wrap_list(self: Self) -> AsList<Self>`
  - Wrap `self` in [`AsList`] for R list conversion.

### `convert::AsNamedListExt`

Extension trait for wrapping tuple pair collections as [`AsNamedList`].

#### Example

```ignore
let pairs = vec![("x".to_string(), 1i32), ("y".to_string(), 2i32)];
let wrapped = pairs.wrap_named_list();
```

**Provided methods:**

- `wrap_named_list(self: Self) -> AsNamedList<Self>`
  - Wrap `self` in [`AsNamedList`] for named R list conversion.

### `convert::AsNamedVectorExt`

Extension trait for wrapping tuple pair collections as [`AsNamedVector`].

#### Example

```ignore
let pairs = vec![("alice".to_string(), 95.0f64), ("bob".to_string(), 87.5)];
let wrapped = pairs.wrap_named_vector();
```

**Provided methods:**

- `wrap_named_vector(self: Self) -> AsNamedVector<Self>`
  - Wrap `self` in [`AsNamedVector`] for named atomic R vector conversion.

### `convert::AsRNativeExt`

Extension trait for wrapping values as [`AsRNative`].

This trait is automatically implemented for all types that implement [`RNativeType`].

#### Example

```ignore
use miniextendr_api::convert::AsRNativeExt;

let x: f64 = 42.5;
let wrapped: AsRNative<f64> = x.wrap_r_native();
```

**Provided methods:**

- `wrap_r_native(self: Self) -> AsRNative<Self>`
  - Wrap `self` in [`AsRNative`] for native R scalar conversion.

### `convert::AsVctrsExt`

Extension trait for wrapping values as [`AsVctrs`].

Automatically implemented for all `T: IntoVctrs` (typically a `#[derive(Vctrs)]` type).

**Provided methods:**

- `wrap_vctrs(self: Self) -> AsVctrs<Self>`
  - Wrap `self` in [`AsVctrs`] for R vctrs conversion.

### `dataframe::FromDataFrame`

R `data.frame` → Rust data. The data-frame analogue of
[`TryFromSexp`].

Implemented by `#[derive(DataFrameRow)]` for `Vec<Row>` and by the serde row path.

#### Parallel fast path

[`from_dataframe_par`](FromDataFrame::from_dataframe_par) (`feature = "rayon"`) reads the
same rows as [`from_dataframe`](FromDataFrame::from_dataframe), defaulting to the
sequential reader; the derive overrides it with the #765 off-main-thread row assembly.

**Required methods:**

- `from_dataframe(df: &DataFrame) -> Result<Self, DataFrameError>`
  - Read rows back out of a [`DataFrame`].

**Provided methods:**

- `from_dataframe_par(df: &DataFrame) -> Result<Self, DataFrameError>`
  - Parallel row read (`feature = "rayon"`). Same result as `from_dataframe()`.

### `dataframe::IntoDataFrame`

Rust data → R `data.frame`. The data-frame analogue of [`IntoR`].

Implemented by `#[derive(DataFrameRow)]` on a row struct/enum (for `Vec<Row>`), by the
blanket impl for any [`ColumnSource`] (`IntoList`-derived rows), and by the serde column
path. Call it on your data: `rows.into_dataframe()?`.

#### Parallel fast path

[`into_dataframe_par`](IntoDataFrame::into_dataframe_par) (present only with
`feature = "rayon"`) produces the **same** [`DataFrame`] as
[`into_dataframe`](IntoDataFrame::into_dataframe). It defaults to the sequential path, so
every implementor gets a correct `_par` for free; `#[derive(DataFrameRow)]` row types
override it with a genuinely parallel column fill (the #777 flattened `(column,row-range)`
work-list). The verb is stable across feature sets — dropping `_par` degrades cleanly to
the sequential call.

**Required methods:**

- `into_dataframe(self: Self) -> Result<DataFrame, DataFrameError>`
  - Convert this value into a [`DataFrame`].

**Provided methods:**

- `into_dataframe_par(self: Self) -> Result<DataFrame, DataFrameError>`
  - Parallel column fill (`feature = "rayon"`). Same result as `into_dataframe()`.

### `externalptr::IntoExternalPtr`

Marker trait for types that should be converted to R as ExternalPtr.

When a type implements this trait (via `#[derive(ExternalPtr)]`), it gets a
blanket `IntoR` implementation that wraps the value in `ExternalPtr<T>`.

This allows returning the type directly from `#[miniextendr]` functions:

```ignore
#[derive(ExternalPtr)]
struct MyData { value: i32 }

#[miniextendr]
fn create_data(v: i32) -> MyData {
    MyData { value: v }  // Automatically wrapped in ExternalPtr
}
```

### `externalptr::TypedExternal`

Trait for types that can be stored in an ExternalPtr.

This provides the type identification needed for runtime type checking.
Type identification uses R's symbol interning (`Rf_install`) for fast
pointer-based comparison.

#### Type ID vs Type Name

- `TYPE_ID_CSTR`: Namespaced identifier used for type checking (stored in `prot[0]`).
  Format: `"<crate_name>@<crate_version>::<module_path>::<type_name>\0"`

  The crate name and version ensure:
  - Same type from same crate+version → compatible (can share ExternalPtr)
  - Same type name from different crates → incompatible
  - Same type from different crate versions → incompatible

- `TYPE_NAME_CSTR`: Short display name for the R tag (shown when printing).
  Just the type identifier for readability.

**Associated items:**

- associated const `TYPE_NAME: &'static str`
- associated const `TYPE_NAME_CSTR: &'static [u8]`
- associated const `TYPE_ID_CSTR: &'static [u8]`

### `factor::RFactor`

Trait for mapping Rust enums to R factors.

Typically implemented via `#[derive(RFactor)]` for C-style enums.
The derive macro also generates `IntoR` and `TryFromSexp` implementations.

**Required methods:**

- `to_level_index(self: Self) -> i32`
  - Convert variant to 1-based level index.
- `from_level_index(idx: i32) -> Option<Self>`
  - Convert 1-based level index to variant, or `None` if out of range.

### `factor::UnitEnumFactor`

Trait implemented by unit-only enums derived via `#[derive(DataFrameRow)]`.

Provides the level names and 1-based index needed to convert enum values
into R factor SEXPs. Unlike `RFactor`, this trait does **not** require
`Copy` or `MatchArg`, making it usable with `DataFrameRow`-derived types
that only need to participate as factor columns in data frames.

Implemented automatically by `#[derive(DataFrameRow)]` on unit-only enums.
The blanket `impl<T: UnitEnumFactor> IntoR for FactorOptionVec<T>` in
`miniextendr-api` provides the actual SEXP conversion used by the
companion struct's `into_data_frame` method.

#### Safety contract

`to_factor_index` must return a value in `1..=FACTOR_LEVELS.len() as i32`
(or `NA_INTEGER` for missing) to produce a valid R factor SEXP.

**Required methods:**

- `to_factor_index(self: Self) -> i32`
  - Convert `self` to a 1-based R factor level index.
- `from_factor_index(idx: i32) -> Option<Self>`
  - Inverse: 1-based level index → variant, or `None` if out of range.

**Associated items:**

- associated const `FACTOR_LEVELS: &'static [&'static str]`

### `from_r::TryFromSexp`

TryFrom-style trait for converting SEXP to Rust types.

Inbound counterpart of [`crate::into_r::IntoR`]. Strict by construction
(returns `Result`) — for a looser, multi-source coercion path use
[`crate::coerce::Coerce`] / [`crate::coerce::TryCoerce`].

#### Examples

```no_run
use miniextendr_api::SEXP;
use miniextendr_api::from_r::TryFromSexp;

fn example(sexp: SEXP) {
    let value: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
    let text: String = TryFromSexp::try_from_sexp(sexp).unwrap();
}
```

**Required methods:**

- `try_from_sexp(sexp: SEXP) -> Result<Self, <Self as >::Error>`
  - Attempt to convert an R SEXP to this Rust type.

**Provided methods:**

- `unsafe try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, <Self as >::Error>`
  - Convert from SEXP without thread safety checks.

**Associated items:**

- associated type `Error`

### `gc_protect::Protector`

A scope-like GC protection backend.

Functions that allocate multiple intermediate SEXPs can take `&mut impl Protector`
to be generic over the protection mechanism. All protected SEXPs stay protected
until the protector itself is dropped — there is no individual release via this
trait.

For individual release by key, use [`ProtectPool::insert`](crate::protect_pool::ProtectPool::insert)
and [`ProtectPool::release`](crate::protect_pool::ProtectPool::release) directly.

#### Safety

Implementations must ensure that the returned SEXP remains protected from GC
for at least as long as the protector is alive. Callers must not use the
returned SEXP after the protector is dropped.

All methods must be called from the R main thread.

**Required methods:**

- `unsafe protect(self: &mut Self, sexp: SEXP) -> SEXP`
  - Protect a SEXP from garbage collection.

### `into_r::IntoR`

Trait for converting Rust types to R SEXP values.

Outbound counterpart of [`crate::from_r::TryFromSexp`]. This is the **lax**
path for 64-bit integer types — values that overflow `i32` silently land
as `REALSXP`. For the strict alternative, see [`crate::strict`] and the
`#[miniextendr(strict)]` attribute.

#### Required Method

Implementors must provide [`try_into_sexp`](IntoR::try_into_sexp) and
specify [`Error`](IntoR::Error). The other three methods have sensible
defaults.

#### Examples

```no_run
use miniextendr_api::into_r::IntoR;

let sexp = 42i32.into_sexp();
let sexp = "hello".to_string().into_sexp();

// Fallible path:
let result = "hello".try_into_sexp();
assert!(result.is_ok());
```

**Required methods:**

- `try_into_sexp(self: Self) -> Result<crate::SEXP, <Self as >::Error>`
  - Try to convert this value to an R SEXP.

**Provided methods:**

- `unsafe try_into_sexp_unchecked(self: Self) -> Result<crate::SEXP, <Self as >::Error>`
  - Try to convert to SEXP without thread safety checks.
- `into_sexp(self: Self) -> crate::SEXP`
  - Convert this value to an R SEXP, panicking on error.
- `unsafe into_sexp_unchecked(self: Self) -> crate::SEXP`
  - Convert to SEXP without thread safety checks, panicking on error.

**Associated items:**

- associated type `Error`

### `into_r::IntoRAltrep`

Extension trait for ALTREP conversions.

This trait provides ergonomic methods for converting Rust types to R ALTREP
vectors without copying data. The data stays in Rust memory (wrapped in an
ExternalPtr) and R accesses it via ALTREP callbacks.

#### Performance Characteristics

| Operation | Regular (IntoR) | ALTREP (IntoRAltrep) |
|-----------|-----------------|------------------------|
| Creation | O(n) copy | O(1) wrap |
| Memory | Duplicated in R | Single copy in Rust |
| Element access | Direct pointer | Callback (~10ns overhead) |
| DATAPTR ops | O(1) | O(1) if Vec/Box, N/A if lazy |

#### When to Use ALTREP

**Good candidates**:
- ✅ Large vectors (>1000 elements)
- ✅ Lazy/computed data (avoid eager materialization)
- ✅ External data sources (files, databases, APIs)
- ✅ Data that might not be fully accessed by R

**Not recommended**:
- ❌ Small vectors (<100 elements) - copy overhead is negligible
- ❌ Data R will immediately modify (triggers copy anyway)
- ❌ Temporary results (extra indirection not worth it)

#### Example

```rust,ignore
use miniextendr_api::{miniextendr, IntoRAltrep, IntoR, SEXP};

#[miniextendr]
fn large_dataset() -> SEXP {
    let data: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();

    // Zero-copy: wraps pointer instead of copying 1M elements
    data.into_sexp_altrep()
}

#[miniextendr]
fn small_result() -> SEXP {
    let data = vec![1, 2, 3, 4, 5];

    // Regular copy is fine for small data
    data.into_sexp()
}
```

**Required methods:**

- `into_sexp_altrep(self: Self) -> crate::SEXP`
  - Convert to R SEXP using ALTREP zero-copy representation.

**Provided methods:**

- `unsafe into_sexp_altrep_unchecked(self: Self) -> crate::SEXP`
  - Convert to R SEXP using ALTREP, skipping debug thread assertions.
- `into_altrep(self: Self) -> Altrep<Self>`
  - Create an `Altrep<Self>` wrapper.

### `into_r_as::IntoRAs`

Storage-directed conversion to R SEXP.

This trait allows converting Rust values to R with an explicit target storage
type. The conversion is value-based: it succeeds if all values fit the target
type, and fails otherwise.

#### Target Types

- `i32` → R integer (INTSXP)
- `f64` → R numeric (REALSXP)
- `RLogical` → R logical (LGLSXP)
- `u8` → R raw (RAWSXP)
- `String` → R character (STRSXP)

#### Example

```ignore
use miniextendr_api::IntoRAs;

// Convert i64 to R integer (if values fit)
let x: Vec<i64> = vec![1, 2, 3];
let sexp = x.into_r_as::<i32>()?;

// Convert f64 to R integer (if values are integral)
let y: Vec<f64> = vec![1.0, 2.0, 3.0];
let sexp = y.into_r_as::<i32>()?;
```

**Required methods:**

- `into_r_as(self: Self) -> Result<SEXP, StorageCoerceError>`
  - Convert to R SEXP with the specified target storage type.

### `list::IntoList`

Convert things into an R list.

**Required methods:**

- `into_list(self: Self) -> List`
  - Convert `self` into an R list wrapper.

### `list::TryFromList`

Fallible conversion from an R list into a Rust value.

**Required methods:**

- `try_from_list(list: List) -> Result<Self, <Self as >::Error>`
  - Attempt to convert an R list wrapper into `Self`.

**Associated items:**

- associated type `Error`

### `markers::DataFrameRow`

Marker trait for types generated by `#[derive(DataFrameRow)]`.

Automatically implemented by the `DataFrameRow` derive macro. The derive
macro emits a compile-time assertion against this trait for every struct-typed
variant field, giving users a clear error when the inner type is missing the
derive.

This trait has no supertrait — the actual data-frame conversion contract is on the
generated companion `{Name}DataFrame` struct (which implements `ColumnSource`).
The marker is used solely for compile-time assertions via
`_assert_inner_is_dataframe_row::<T>()` generated by the outer derive.

You should not implement this trait manually.

### `markers::WidensToF64`

Marker trait for types that can widen to `f64` without loss.

Manually implemented for specific types to avoid conflicts with identity/
special-case conversions. Used by blanket Coerce implementations.

### `markers::WidensToI32`

Marker trait for types that can widen to `i32` without loss.

Manually implemented for specific types to avoid conflicts with identity/
special-case conversions. Used by blanket Coerce implementations.

### `match_arg::MatchArg`

Trait for enum types that support `match.arg`-style string conversion.

Implementors provide a fixed set of choice strings and bidirectional
conversion between enum variants and their string representations.

Use `#[derive(MatchArg)]` to auto-generate this implementation.

**Required methods:**

- `from_choice(choice: &str) -> Option<Self>`
  - Convert a choice string to the corresponding enum variant.
- `to_choice(self: Self) -> &'static str`
  - Convert the enum variant to its canonical choice string.

**Associated items:**

- associated const `CHOICES: &'static [&'static str]`

### `named_vector::AtomicElement`

Marker trait for types that can be elements of named atomic R vectors.

Each implementation knows how to convert `Vec<Self>` to/from an R atomic
vector (INTSXP, REALSXP, LGLSXP, RAWSXP, or STRSXP).

**Required methods:**

- `vec_to_sexp(values: Vec<Self>) -> SEXP`
  - Convert a Rust vector to an R atomic SEXP.
- `vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError>`
  - Convert an R atomic SEXP to a Rust vector.

### `newtype::FromRNewtype`

Construct a forwarding newtype from its inner value (R → Rust side).

Emitted by `#[derive(TryFromSexp)]`. Powers the `TryFromSexp` container
blankets for `Vec<T>` / `Option<T>` / `Vec<Option<T>>` in this module.

**Required methods:**

- `from_inner(inner: <Self as >::Inner) -> Self`
  - Wrap an inner value into the newtype.

**Associated items:**

- associated type `Inner`

### `newtype::IntoRNewtype`

Unwrap a forwarding newtype into its inner value (Rust → R side).

Emitted by `#[derive(IntoR)]`. Powers the `IntoR` container blankets for
`Option<T>` / `Vec<Option<T>>` in this module.

**Required methods:**

- `into_inner(self: Self) -> <Self as >::Inner`
  - Unwrap the newtype into its inner value.

**Associated items:**

- associated type `Inner`

### `newtype::IntoRVecElement`

How a `Vec<Self>` becomes a single R vector SEXP.

This is the shared element-marker behind the **one** `impl<T: …> IntoR for
Vec<T>` blanket slot. Implemented concretely per type — by `#[derive(IntoR)]`
for newtypes (forwarding to `Vec<Inner>`), and by the `MatchArg` bridge in
`match_arg.rs` for `match.arg` enums (STRSXP by variant name). See the module
docs for why this cannot be two competing blankets.

**Required methods:**

- `elements_into_sexp(values: Vec<Self>) -> SEXP`
  - Convert all elements into one R vector SEXP.

### `optionals::aho_corasick_impl::RAhoCorasickOps`

Adapter trait for exposing `AhoCorasick` operations to R.

Since `AhoCorasick` doesn't implement `Clone` or `Copy`, it's typically
wrapped in an `ExternalPtr` for reuse across R calls.

#### Registration

Registration is automatic when you annotate `impl RAhoCorasickOps for AhoCorasick`
with `#[miniextendr]`.

**Required methods:**

- `patterns_len(self: &Self) -> i32`
  - Number of patterns in the automaton.
- `is_match(self: &Self, haystack: &str) -> bool`
  - Check if haystack matches any pattern.
- `count_matches(self: &Self, haystack: &str) -> i32`
  - Count total matches in haystack.
- `find_all_flat(self: &Self, haystack: &str) -> Vec<i32>`
  - Find all matches, returning flat vec: [pid, start, end, ...]
- `find_first(self: &Self, haystack: &str) -> Vec<i32>`
  - Find first match, returning [pid, start, end] or empty vec.
- `replace_all(self: &Self, haystack: &str, replacement: &str) -> String`
  - Replace all matches with a single replacement.

### `optionals::arrow_impl::RSourced`

Trait for Arrow types that may be backed by R memory.

Types implementing this trait carry optional provenance information:
the original R SEXP whose data buffer backs the Arrow array. When
converting back to R, this enables zero-copy return of the original
vector instead of allocating and copying.

**Required methods:**

- `r_source(self: &Self) -> Option<SEXP>`
  - The original R SEXP if this value is zero-copy from R.
- `nulls_from_sentinels(self: &Self) -> bool`
  - Whether nulls came from R sentinel values (NA_integer_, NA_real_).

### `optionals::borsh_impl::RBorshOps`

Adapter trait for borsh serialization from R.

Provides method-style access to borsh serialization operations.
This trait has a blanket implementation for all types that implement
both `BorshSerialize` and `BorshDeserialize`.

**Required methods:**

- `borsh_serialize(self: &Self) -> Vec<u8>`
  - Serialize to raw bytes.
- `borsh_deserialize(bytes: &[u8]) -> Result<Self, String>`
  - Deserialize from raw bytes.
- `borsh_size(self: &Self) -> usize`
  - Size of serialized form in bytes.

### `optionals::bytes_impl::RBuf`

Adapter trait for exposing byte buffer read operations to R.

This trait wraps the [`bytes::Buf`] trait functionality, providing methods
to read various types from a byte buffer. Implementations must handle
interior mutability since `Buf::get_*` methods consume bytes from the buffer.

#### Example

```ignore
use miniextendr_api::RBuf;
use bytes::Bytes;
use std::cell::RefCell;

struct ReadableBuffer {
    data: RefCell<Bytes>,
}

impl RBuf for ReadableBuffer {
    fn remaining(&self) -> i32 {
        self.data.borrow().remaining() as i32
    }

    fn get_u8(&self) -> Option<i32> {
        let mut buf = self.data.borrow_mut();
        if buf.has_remaining() {
            Some(buf.get_u8() as i32)
        } else {
            None
        }
    }

    fn chunk(&self) -> Vec<u8> {
        self.data.borrow().chunk().to_vec()
    }

    fn copy_to_vec(&self, len: i32) -> Vec<u8> {
        let mut buf = self.data.borrow_mut();
        let len = (len as usize).min(buf.remaining());
        let mut dst = vec![0u8; len];
        buf.copy_to_slice(&mut dst);
        dst
    }
}
```

#### Note

Methods return `Option` or check bounds because R cannot handle Rust panics
gracefully. The `r_get_*` methods return `None` when no bytes remain rather
than panicking.

**Required methods:**

- `remaining(self: &Self) -> i32`
  - Returns the number of bytes remaining in the buffer.
- `get_u8(self: &Self) -> Option<i32>`
  - Gets a single byte from the buffer, advancing the position.
- `chunk(self: &Self) -> Vec<u8>`
  - Returns the current chunk of bytes available for reading without advancing.
- `copy_to_vec(self: &Self, len: i32) -> Vec<u8>`
  - Copies `len` bytes from the buffer into a new Vec, advancing the position.
- `advance(self: &Self, cnt: i32)`
  - Advances the buffer position by `cnt` bytes.

**Provided methods:**

- `has_remaining(self: &Self) -> bool`
  - Returns `true` if there are any bytes remaining.
- `get_i8(self: &Self) -> Option<i32>`
  - Gets a signed byte from the buffer.
- `get_u16(self: &Self) -> Option<i32>`
  - Gets a big-endian u16 from the buffer.
- `get_u16_le(self: &Self) -> Option<i32>`
  - Gets a little-endian u16 from the buffer.
- `get_i16(self: &Self) -> Option<i32>`
  - Gets a big-endian i16 from the buffer.
- `get_i16_le(self: &Self) -> Option<i32>`
  - Gets a little-endian i16 from the buffer.
- `get_u32(self: &Self) -> Option<f64>`
  - Gets a big-endian u32 from the buffer.
- `get_u32_le(self: &Self) -> Option<f64>`
  - Gets a little-endian u32 from the buffer.
- `get_i32(self: &Self) -> Option<i32>`
  - Gets a big-endian i32 from the buffer.
- `get_i32_le(self: &Self) -> Option<i32>`
  - Gets a little-endian i32 from the buffer.
- `get_u64(self: &Self) -> Option<f64>`
  - Gets a big-endian u64 from the buffer.
- `get_u64_le(self: &Self) -> Option<f64>`
  - Gets a little-endian u64 from the buffer.
- `get_i64(self: &Self) -> Option<f64>`
  - Gets a big-endian i64 from the buffer.
- `get_i64_le(self: &Self) -> Option<f64>`
  - Gets a little-endian i64 from the buffer.
- `get_f32(self: &Self) -> Option<f64>`
  - Gets a big-endian f32 from the buffer.
- `get_f32_le(self: &Self) -> Option<f64>`
  - Gets a little-endian f32 from the buffer.
- `get_f64(self: &Self) -> Option<f64>`
  - Gets a big-endian f64 from the buffer.
- `get_f64_le(self: &Self) -> Option<f64>`
  - Gets a little-endian f64 from the buffer.
- `to_vec(self: &Self) -> Vec<u8>`
  - Reads all remaining bytes into a Vec.

### `optionals::bytes_impl::RBufMut`

Adapter trait for exposing byte buffer write operations to R.

This trait wraps the [`bytes::BufMut`] trait functionality, providing methods
to write various types to a byte buffer. Like [`RBuf`], implementations must
handle interior mutability.

#### Example

```ignore
use miniextendr_api::RBufMut;
use bytes::BytesMut;
use std::cell::RefCell;

struct WritableBuffer {
    data: RefCell<BytesMut>,
}

impl RBufMut for WritableBuffer {
    fn remaining_mut(&self) -> i32 {
        i32::try_from(self.data.borrow().remaining_mut()).unwrap_or(i32::MAX)
    }

    fn put_u8(&self, val: i32) {
        self.data.borrow_mut().put_u8(val as u8);
    }

    fn put_slice(&self, src: Vec<u8>) {
        self.data.borrow_mut().put_slice(&src);
    }
}
```

**Required methods:**

- `remaining_mut(self: &Self) -> i32`
  - Returns the number of bytes that can be written without reallocation.
- `put_u8(self: &Self, val: i32)`
  - Writes a single byte to the buffer.
- `put_slice(self: &Self, src: Vec<u8>)`
  - Writes a slice of bytes to the buffer.

**Provided methods:**

- `has_remaining_mut(self: &Self) -> bool`
  - Returns `true` if there is space to write more bytes.
- `put_i8(self: &Self, val: i32)`
  - Writes a signed byte to the buffer.
- `put_u16(self: &Self, _val: i32)`
  - Writes a big-endian u16 to the buffer.
- `put_u16_le(self: &Self, _val: i32)`
  - Writes a little-endian u16 to the buffer.
- `put_i16(self: &Self, _val: i32)`
  - Writes a big-endian i16 to the buffer.
- `put_i16_le(self: &Self, _val: i32)`
  - Writes a little-endian i16 to the buffer.
- `put_u32(self: &Self, _val: f64)`
  - Writes a big-endian u32 to the buffer.
- `put_u32_le(self: &Self, _val: f64)`
  - Writes a little-endian u32 to the buffer.
- `put_i32(self: &Self, _val: i32)`
  - Writes a big-endian i32 to the buffer.
- `put_i32_le(self: &Self, _val: i32)`
  - Writes a little-endian i32 to the buffer.
- `put_u64(self: &Self, _val: f64)`
  - Writes a big-endian u64 to the buffer.
- `put_u64_le(self: &Self, _val: f64)`
  - Writes a little-endian u64 to the buffer.
- `put_i64(self: &Self, _val: f64)`
  - Writes a big-endian i64 to the buffer.
- `put_i64_le(self: &Self, _val: f64)`
  - Writes a little-endian i64 to the buffer.
- `put_f32(self: &Self, _val: f64)`
  - Writes a big-endian f32 to the buffer.
- `put_f32_le(self: &Self, _val: f64)`
  - Writes a little-endian f32 to the buffer.
- `put_f64(self: &Self, _val: f64)`
  - Writes a big-endian f64 to the buffer.
- `put_f64_le(self: &Self, _val: f64)`
  - Writes a little-endian f64 to the buffer.
- `put_bytes(self: &Self, val: i32, n: i32)`
  - Writes `n` copies of byte `val` to the buffer.
- `reserve(self: &Self, _additional: i32)`
  - Reserves capacity for at least `additional` more bytes.
- `len(self: &Self) -> i32`
  - Returns the current length of written data.
- `is_empty(self: &Self) -> bool`
  - Returns `true` if the buffer is empty.
- `clear(self: &Self)`
  - Clears the buffer, removing all written data.

### `optionals::indexmap_impl::RIndexMapOps`

Adapter trait for [`IndexMap`] operations.

Provides ordered dictionary operations from R.
Requires `T: Clone + IntoR` for some methods that return values.

#### Example

```rust,ignore
use indexmap::IndexMap;
use miniextendr_api::indexmap_impl::RIndexMapOps;

#[derive(ExternalPtr)]
struct MyMap(IndexMap<String, i32>);

#[miniextendr]
impl RIndexMapOps for MyMap {}
```

In R:
```r
m <- MyMap$new()
m$insert("foo", 1L)
m$get("foo")        # 1L
m$keys()            # c("foo")
m$len()             # 1L
```

**Required methods:**

- `len(self: &Self) -> i32`
  - Get the number of entries.
- `is_empty(self: &Self) -> bool`
  - Check if the map is empty.
- `keys(self: &Self) -> Vec<String>`
  - Get all keys in order.
- `contains_key(self: &Self, key: &str) -> bool`
  - Check if a key exists.
- `get_index(self: &Self, index: i32) -> Option<(String, T)>`
  - Get the value at an index (0-based).
- `get_key_at(self: &Self, index: i32) -> Option<String>`
  - Get the key at an index (0-based).
- `first(self: &Self) -> Option<(String, T)>`
  - Get the first key-value pair.
- `last(self: &Self) -> Option<(String, T)>`
  - Get the last key-value pair.
- `get_index_of(self: &Self, key: &str) -> i32`
  - Get the index of a key, or -1 if not found.

### `optionals::jiff_impl::RDate`

Adapter trait for [`jiff::civil::Date`].

Provides component accessors, calendar helpers, and formatting for civil dates from R.

**Required methods:**

- `year(self: &Self) -> i32`
- `month(self: &Self) -> i32`
- `day(self: &Self) -> i32`
- `weekday(self: &Self) -> i32`
- `day_of_year(self: &Self) -> i32`
- `first_of_month(self: &Self) -> Date`
- `last_of_month(self: &Self) -> Date`
- `tomorrow(self: &Self) -> Result<Date, String>`
- `yesterday(self: &Self) -> Result<Date, String>`
- `strftime(self: &Self, fmt: &str) -> String`

### `optionals::jiff_impl::RDateTime`

Adapter trait for [`jiff::civil::DateTime`].

Provides component accessors and tz-conversion for civil datetimes from R.

**Required methods:**

- `year(self: &Self) -> i32`
- `month(self: &Self) -> i32`
- `day(self: &Self) -> i32`
- `hour(self: &Self) -> i32`
- `minute(self: &Self) -> i32`
- `second(self: &Self) -> i32`
- `subsec_nanosecond(self: &Self) -> i32`
- `to_date(self: &Self) -> Date`
- `to_time(self: &Self) -> Time`
- `in_tz(self: &Self, iana: &str) -> Result<Zoned, String>`

### `optionals::jiff_impl::RSignedDuration`

Adapter trait for [`jiff::SignedDuration`].

Provides methods to inspect and manipulate signed durations from R.

**Required methods:**

- `as_seconds_f64(self: &Self) -> f64`
  - Get the total duration as floating-point seconds.
- `as_milliseconds(self: &Self) -> i64`
  - Get the total duration in milliseconds (truncated to i64).
- `whole_seconds(self: &Self) -> i64`
  - Get the number of whole seconds.
- `whole_minutes(self: &Self) -> i64`
  - Get the number of whole minutes.
- `whole_hours(self: &Self) -> i64`
  - Get the number of whole hours.
- `whole_days(self: &Self) -> i64`
  - Get the number of whole days.
- `subsec_nanoseconds(self: &Self) -> i32`
  - Get the subsecond nanoseconds component.
- `is_negative(self: &Self) -> bool`
  - Check if the duration is negative.
- `is_zero(self: &Self) -> bool`
  - Check if the duration is zero.
- `abs(self: &Self) -> SignedDuration`
  - Get the absolute value of this duration.

### `optionals::jiff_impl::RSpan`

Adapter trait for [`jiff::Span`].

Provides component accessors and arithmetic helpers for `Span` values from R.

**Required methods:**

- `get_years(self: &Self) -> i64`
- `get_months(self: &Self) -> i64`
- `get_weeks(self: &Self) -> i64`
- `get_days(self: &Self) -> i64`
- `get_hours(self: &Self) -> i64`
- `get_minutes(self: &Self) -> i64`
- `get_seconds(self: &Self) -> i64`
- `get_milliseconds(self: &Self) -> i64`
- `get_microseconds(self: &Self) -> i64`
- `get_nanoseconds(self: &Self) -> i64`
- `is_zero(self: &Self) -> bool`
- `is_negative(self: &Self) -> bool`
- `negate(self: &Self) -> Span`
- `abs(self: &Self) -> Span`

### `optionals::jiff_impl::RTime`

Adapter trait for [`jiff::civil::Time`].

Provides component accessors and date-combination for civil times from R.

**Required methods:**

- `hour(self: &Self) -> i32`
- `minute(self: &Self) -> i32`
- `second(self: &Self) -> i32`
- `subsec_nanosecond(self: &Self) -> i32`
- `on(self: &Self, year: i16, month: i8, day: i8) -> DateTime`

### `optionals::jiff_impl::RTimestamp`

Adapter trait for [`jiff::Timestamp`].

Provides component accessors and tz conversion for timestamps from R.

**Required methods:**

- `as_second(self: &Self) -> i64`
- `as_millisecond(self: &Self) -> i64`
- `subsec_nanosecond(self: &Self) -> i32`
- `to_zoned_in(self: &Self, iana: &str) -> Result<Zoned, String>`
- `strftime(self: &Self, fmt: &str) -> String`

### `optionals::jiff_impl::RZoned`

Adapter trait for [`jiff::Zoned`].

Provides component accessors, tz conversion, and formatting for zoned datetimes from R.

**Required methods:**

- `iana_name(self: &Self) -> Option<String>`
- `year(self: &Self) -> i32`
- `month(self: &Self) -> i32`
- `day(self: &Self) -> i32`
- `hour(self: &Self) -> i32`
- `minute(self: &Self) -> i32`
- `second(self: &Self) -> i32`
- `in_tz(self: &Self, iana: &str) -> Result<Zoned, String>`
- `start_of_day(self: &Self) -> Result<Zoned, String>`
- `strftime(self: &Self, fmt: &str) -> String`

### `optionals::nalgebra_impl::RMatrixOps`

Adapter trait for [`nalgebra::DMatrix`] operations.

Provides common matrix operations accessible from R.
Automatically implemented for `DMatrix<f64>`.

#### Example

```rust,ignore
use nalgebra::DMatrix;
use miniextendr_api::nalgebra_impl::RMatrixOps;

#[derive(ExternalPtr)]
struct MyMatrix(DMatrix<f64>);

#[miniextendr]
impl RMatrixOps for MyMatrix {}
```

In R:
```r
m <- MyMatrix$new(matrix(1:4, 2, 2))
m$nrows()        # 2L
m$ncols()        # 2L
m$determinant()  # -2.0
m$transpose()    # Returns transposed matrix
```

**Required methods:**

- `nrows(self: &Self) -> i32`
  - Get the number of rows.
- `ncols(self: &Self) -> i32`
  - Get the number of columns.
- `shape(self: &Self) -> (i32, i32)`
  - Get the shape as (rows, cols).
- `is_square(self: &Self) -> bool`
  - Check if the matrix is square.
- `is_empty(self: &Self) -> bool`
  - Check if the matrix is empty.
- `transpose(self: &Self) -> DMatrix<f64>`
  - Compute the transpose.
- `determinant(self: &Self) -> f64`
  - Compute the determinant (for square matrices).
- `trace(self: &Self) -> f64`
  - Compute the trace (sum of diagonal elements).
- `diagonal(self: &Self) -> DVector<f64>`
  - Get the diagonal as a vector.
- `norm(self: &Self) -> f64`
  - Compute the Frobenius norm.
- `try_inverse(self: &Self) -> Option<DMatrix<f64>>`
  - Compute the matrix inverse (returns None if singular).
- `sum(self: &Self) -> f64`
  - Compute the sum of all elements.
- `mean(self: &Self) -> f64`
  - Compute the mean of all elements.
- `min(self: &Self) -> f64`
  - Get the minimum element.
- `max(self: &Self) -> f64`
  - Get the maximum element.
- `scale(self: &Self, s: f64) -> DMatrix<f64>`
  - Scale the matrix by a scalar.
- `add(self: &Self, other: &Self) -> DMatrix<f64>`
  - Add another matrix.
- `sub(self: &Self, other: &Self) -> DMatrix<f64>`
  - Subtract another matrix.
- `mul(self: &Self, other: &Self) -> DMatrix<f64>`
  - Matrix multiplication.
- `component_mul(self: &Self, other: &Self) -> DMatrix<f64>`
  - Element-wise multiplication.
- `row_sum(self: &Self) -> DVector<f64>`
  - Sum of each row, returned as a column vector.
- `column_sum(self: &Self) -> DVector<f64>`
  - Sum of each column, returned as a row vector.
- `row_mean(self: &Self) -> DVector<f64>`
  - Mean of each row.
- `column_mean(self: &Self) -> DVector<f64>`
  - Mean of each column.

### `optionals::nalgebra_impl::RVectorOps`

Adapter trait for [`nalgebra::DVector`] operations.

Provides common vector operations accessible from R.
Automatically implemented for `DVector<f64>`.

#### Example

```rust,ignore
use nalgebra::DVector;
use miniextendr_api::nalgebra_impl::RVectorOps;

#[derive(ExternalPtr)]
struct MyVector(DVector<f64>);

#[miniextendr]
impl RVectorOps for MyVector {}
```

In R:
```r
v <- MyVector$new(c(3, 4))
v$norm()      # 5.0 (Euclidean norm)
v$len()       # 2L
```

**Required methods:**

- `len(self: &Self) -> i32`
  - Get the number of elements in the vector.
- `is_empty(self: &Self) -> bool`
  - Check if the vector is empty.
- `norm(self: &Self) -> f64`
  - Compute the Euclidean (L2) norm.
- `norm_squared(self: &Self) -> f64`
  - Compute the squared Euclidean norm (avoids sqrt).
- `norm_l1(self: &Self) -> f64`
  - Compute the L1 norm (sum of absolute values).
- `norm_linf(self: &Self) -> f64`
  - Compute the L-infinity norm (maximum absolute value).
- `sum(self: &Self) -> f64`
  - Compute the sum of all elements.
- `mean(self: &Self) -> f64`
  - Compute the mean of all elements.
- `min(self: &Self) -> f64`
  - Get the minimum element.
- `max(self: &Self) -> f64`
  - Get the maximum element.
- `argmin(self: &Self) -> i32`
  - Get the index of the minimum element (0-based).
- `argmax(self: &Self) -> i32`
  - Get the index of the maximum element (0-based).
- `dot(self: &Self, other: &Self) -> f64`
  - Compute the dot product with another vector.
- `normalize(self: &Self) -> DVector<f64>`
  - Return a normalized (unit) vector.
- `scale(self: &Self, s: f64) -> DVector<f64>`
  - Scale the vector by a scalar.
- `add(self: &Self, other: &Self) -> DVector<f64>`
  - Add another vector.
- `sub(self: &Self, other: &Self) -> DVector<f64>`
  - Subtract another vector.
- `component_mul(self: &Self, other: &Self) -> DVector<f64>`
  - Element-wise multiplication.

### `optionals::ndarray_impl::RNdArrayOps`

Adapter trait for ndarray array operations.

Provides common array operations accessible from R.
Implemented for `Array1<f64>`, `Array2<f64>`, and `ArrayD<f64>`.

#### Example

```rust,ignore
use ndarray::Array1;
use miniextendr_api::ndarray_impl::RNdArrayOps;

#[derive(ExternalPtr)]
struct MyArray(Array1<f64>);

#[miniextendr]
impl RNdArrayOps for MyArray {}
```

In R:
```r
arr <- MyArray$new(c(1, 2, 3, 4, 5))
arr$len()          # 5L
arr$sum()          # 15.0
arr$mean()         # 3.0
arr$shape()        # c(5L)
```

**Required methods:**

- `len(self: &Self) -> i32`
  - Get the total number of elements.
- `is_empty(self: &Self) -> bool`
  - Check if the array is empty.
- `ndim(self: &Self) -> i32`
  - Get the number of dimensions (ndim).
- `shape(self: &Self) -> Vec<i32>`
  - Get the shape as a vector of dimensions.
- `sum(self: &Self) -> f64`
  - Compute the sum of all elements.
- `mean(self: &Self) -> f64`
  - Compute the mean of all elements.
- `min(self: &Self) -> f64`
  - Get the minimum element.
- `max(self: &Self) -> f64`
  - Get the maximum element.
- `product(self: &Self) -> f64`
  - Compute the product of all elements.
- `var(self: &Self) -> f64`
  - Compute the variance (population variance, ddof=0).
- `std(self: &Self) -> f64`
  - Compute the standard deviation (population, ddof=0).

### `optionals::ndarray_impl::RNdIndex`

Adapter trait for n-dimensional array indexing and slicing.

Provides R-style array subsetting for `ArrayD` (dynamic dimension arrays).
Unlike `RNdSlice` (1D) and `RNdSlice2D` (2D), this trait handles arrays
of arbitrary dimension.

#### Example

```rust,ignore
use ndarray::ArrayD;
use miniextendr_api::ndarray_impl::RNdIndex;

#[derive(ExternalPtr)]
struct MyNdArray(ArrayD<f64>);

#[miniextendr]
impl RNdIndex for MyNdArray {}
```

In R:
```r
# Create a 3D array (2x3x4)
arr <- MyNdArray$new(array(1:24, dim = c(2, 3, 4)))
arr$get_nd(c(0L, 1L, 2L))     # Element at [0,1,2]
arr$slice_nd(c(0L, 0L, 0L), c(2L, 2L, 2L))  # Subarray [0:2, 0:2, 0:2]
arr$flatten()                 # Flatten to 1D vector
```

**Required methods:**

- `get_nd(self: &Self, indices: Vec<i32>) -> Option<<Self as >::Elem>`
  - Get the element at the given n-dimensional index (0-indexed).
- `slice_nd(self: &Self, start: Vec<i32>, end: Vec<i32>) -> Option<Vec<<Self as >::Elem>>`
  - Extract a subarray from start (inclusive) to end (exclusive).
- `shape_nd(self: &Self) -> Vec<i32>`
  - Get the shape of the array.
- `ndim(self: &Self) -> i32`
  - Get the number of dimensions.
- `len_nd(self: &Self) -> i32`
  - Get the total number of elements.
- `flatten(self: &Self) -> Vec<<Self as >::Elem>`
  - Flatten the array to a 1D vector in Fortran (column-major) order.
- `flatten_c(self: &Self) -> Vec<<Self as >::Elem>`
  - Flatten the array to a 1D vector in C (row-major) order.
- `axis_slice(self: &Self, axis: i32, index: i32) -> Vec<<Self as >::Elem>`
  - Get elements along a specific axis at the given index.
- `reshape(self: &Self, new_shape: Vec<i32>) -> Option<Vec<<Self as >::Elem>>`
  - Reshape the array to new dimensions (data must fit exactly).

**Provided methods:**

- `is_valid_nd(self: &Self, indices: Vec<i32>) -> bool`
  - Check if the given index is valid.

**Associated items:**

- associated type `Elem`

### `optionals::ndarray_impl::RNdSlice`

Adapter trait for ndarray slicing and indexing operations.

Provides element access and subarray extraction accessible from R.
Unlike `RNdArrayOps` which provides aggregate operations, `RNdSlice`
focuses on accessing individual elements and extracting subarrays.

#### Example

```rust,ignore
use ndarray::Array1;
use miniextendr_api::ndarray_impl::RNdSlice;

#[derive(ExternalPtr)]
struct MyArray(Array1<f64>);

#[miniextendr]
impl RNdSlice for MyArray {}
```

In R:
```r
arr <- MyArray$new(c(1, 2, 3, 4, 5))
arr$get(2L)              # Get element at index 2 (0-indexed): 3.0
arr$slice_1d(1L, 4L)     # Slice [1:4): c(2, 3, 4)
arr$first()              # First element: 1.0
arr$last()               # Last element: 5.0
```

**Required methods:**

- `get(self: &Self, index: i32) -> Option<<Self as >::Elem>`
  - Get the element at the given flat index (0-indexed).
- `first(self: &Self) -> Option<<Self as >::Elem>`
  - Get the first element, or None if empty.
- `last(self: &Self) -> Option<<Self as >::Elem>`
  - Get the last element, or None if empty.
- `slice_1d(self: &Self, start: i32, end: i32) -> Vec<<Self as >::Elem>`
  - Extract a 1D slice as a new Vec (0-indexed, exclusive end).

**Provided methods:**

- `get_many(self: &Self, indices: Vec<i32>) -> Vec<Option<<Self as >::Elem>>`
  - Get elements at the given indices.
- `is_valid_index(self: &Self, index: i32) -> bool`
  - Check if the given index is valid.

**Associated items:**

- associated type `Elem`

### `optionals::ndarray_impl::RNdSlice2D`

Adapter trait for 2D array row/column access.

Provides row and column extraction for matrices.

#### Example

```rust,ignore
use ndarray::Array2;
use miniextendr_api::ndarray_impl::RNdSlice2D;

#[derive(ExternalPtr)]
struct MyMatrix(Array2<f64>);

#[miniextendr]
impl RNdSlice2D for MyMatrix {}
```

In R:
```r
mat <- MyMatrix$new(matrix(1:6, nrow=2, ncol=3))
mat$row(0L)     # First row: c(1, 3, 5)
mat$col(1L)     # Second column: c(3, 4)
mat$get_2d(0L, 1L)  # Element at [0,1]: 3
```

**Required methods:**

- `get_2d(self: &Self, row: i32, col: i32) -> Option<<Self as >::Elem>`
  - Get the element at [row, col] (0-indexed).
- `row(self: &Self, row: i32) -> Vec<<Self as >::Elem>`
  - Get a row as a vector.
- `col(self: &Self, col: i32) -> Vec<<Self as >::Elem>`
  - Get a column as a vector.
- `diag(self: &Self) -> Vec<<Self as >::Elem>`
  - Get the diagonal elements.
- `nrows(self: &Self) -> i32`
  - Get the number of rows.
- `ncols(self: &Self) -> i32`
  - Get the number of columns.

**Associated items:**

- associated type `Elem`

### `optionals::num_bigint_impl::RBigIntBitOps`

Adapter trait for [`BigInt`] bitwise operations.

Provides bitwise operations on arbitrary-precision integers from R.
Note: Bitwise operations on negative BigInt use two's complement representation.

#### Example

```rust,ignore
use num_bigint::BigInt;
use miniextendr_api::num_bigint_impl::RBigIntBitOps;

#[derive(ExternalPtr)]
struct MyBigInt(BigInt);

#[miniextendr]
impl RBigIntBitOps for MyBigInt {}
```

In R:
```r
x <- MyBigInt$from_str("255")
x$bit_and_str("15")$as_string()  # "15"
x$shl(4)$as_string()             # "4080"
x$count_ones()                   # 8
```

**Required methods:**

- `bit_and_str(self: &Self, other: &str) -> Result<BigInt, String>`
  - Bitwise AND with another BigInt (passed as string).
- `bit_or_str(self: &Self, other: &str) -> Result<BigInt, String>`
  - Bitwise OR with another BigInt (passed as string).
- `bit_xor_str(self: &Self, other: &str) -> Result<BigInt, String>`
  - Bitwise XOR with another BigInt (passed as string).
- `bit_not(self: &Self) -> BigInt`
  - Bitwise NOT (two's complement).
- `shl(self: &Self, n: u32) -> BigInt`
  - Left shift by n bits.
- `shr(self: &Self, n: u32) -> BigInt`
  - Right shift by n bits (arithmetic shift for signed).
- `count_ones(self: &Self) -> i64`
  - Count the number of set bits (ones) in the absolute value.
- `trailing_zeros(self: &Self) -> Option<i64>`
  - Count trailing zeros in the absolute value.
- `bit(self: &Self, n: u64) -> bool`
  - Get bit at position n (0-indexed from LSB).
- `set_bit(self: &Self, n: u64) -> BigInt`
  - Set bit at position n (0-indexed from LSB).
- `clear_bit(self: &Self, n: u64) -> BigInt`
  - Clear bit at position n (0-indexed from LSB).

### `optionals::num_bigint_impl::RBigIntOps`

Adapter trait for [`BigInt`] operations.

Provides arbitrary-precision integer arithmetic from R.
Automatically implemented for `num_bigint::BigInt`.

#### Example

```rust,ignore
use num_bigint::BigInt;
use miniextendr_api::num_bigint_impl::RBigIntOps;

#[derive(ExternalPtr)]
struct MyBigInt(BigInt);

#[miniextendr]
impl RBigIntOps for MyBigInt {}
```

In R:
```r
x <- MyBigInt$from_str("12345678901234567890")
y <- MyBigInt$from_str("98765432109876543210")
x$add(y)$as_string()  # "111111111011111111100"
x$is_positive()       # TRUE
x$bit_length()        # 64
```

**Required methods:**

- `as_string(self: &Self) -> String`
  - Convert to string representation.
- `is_zero(self: &Self) -> bool`
  - Check if zero.
- `is_positive(self: &Self) -> bool`
  - Check if positive (> 0).
- `is_negative(self: &Self) -> bool`
  - Check if negative (< 0).
- `sign(self: &Self) -> i32`
  - Get the sign as an integer: -1, 0, or 1.
- `bit_length(self: &Self) -> i64`
  - Get the number of bits needed to represent this number.
- `abs(self: &Self) -> BigInt`
  - Get the absolute value.
- `neg(self: &Self) -> BigInt`
  - Negate the value.
- `add_str(self: &Self, other: &str) -> Result<BigInt, String>`
  - Add another BigInt (passed as string).
- `sub_str(self: &Self, other: &str) -> Result<BigInt, String>`
  - Subtract another BigInt (passed as string).
- `mul_str(self: &Self, other: &str) -> Result<BigInt, String>`
  - Multiply by another BigInt (passed as string).
- `div_str(self: &Self, other: &str) -> Result<BigInt, String>`
  - Divide by another BigInt (passed as string).
- `rem_str(self: &Self, other: &str) -> Result<BigInt, String>`
  - Remainder after division (passed as string).
- `pow(self: &Self, exp: u32) -> BigInt`
  - Raise to a power (u32 exponent).
- `gcd_str(self: &Self, other: &str) -> Result<BigInt, String>`
  - Get the greatest common divisor with another BigInt.
- `to_bytes_be(self: &Self) -> Vec<u8>`
  - Convert to bytes (big-endian).
- `to_bytes_le(self: &Self) -> Vec<u8>`
  - Convert to bytes (little-endian).

### `optionals::num_bigint_impl::RBigUintBitOps`

Adapter trait for [`BigUint`] bitwise operations.

Provides bitwise operations on arbitrary-precision unsigned integers from R.

**Required methods:**

- `bit_and_str(self: &Self, other: &str) -> Result<BigUint, String>`
  - Bitwise AND with another BigUint (passed as string).
- `bit_or_str(self: &Self, other: &str) -> Result<BigUint, String>`
  - Bitwise OR with another BigUint (passed as string).
- `bit_xor_str(self: &Self, other: &str) -> Result<BigUint, String>`
  - Bitwise XOR with another BigUint (passed as string).
- `shl(self: &Self, n: u32) -> BigUint`
  - Left shift by n bits.
- `shr(self: &Self, n: u32) -> BigUint`
  - Right shift by n bits.
- `count_ones(self: &Self) -> i64`
  - Count the number of set bits (ones).
- `trailing_zeros(self: &Self) -> Option<i64>`
  - Count trailing zeros.
- `bit(self: &Self, n: u64) -> bool`
  - Get bit at position n (0-indexed from LSB).
- `set_bit(self: &Self, n: u64) -> BigUint`
  - Set bit at position n (0-indexed from LSB).
- `clear_bit(self: &Self, n: u64) -> BigUint`
  - Clear bit at position n (0-indexed from LSB).

### `optionals::num_bigint_impl::RBigUintOps`

Adapter trait for [`BigUint`] operations.

Provides arbitrary-precision unsigned integer arithmetic from R.
Automatically implemented for `num_bigint::BigUint`.

**Required methods:**

- `as_string(self: &Self) -> String`
  - Convert to string representation.
- `is_zero(self: &Self) -> bool`
  - Check if zero.
- `is_one(self: &Self) -> bool`
  - Check if this is one.
- `bit_length(self: &Self) -> i64`
  - Get the number of bits needed to represent this number.
- `add_str(self: &Self, other: &str) -> Result<BigUint, String>`
  - Add another BigUint (passed as string).
- `sub_str(self: &Self, other: &str) -> Result<BigUint, String>`
  - Subtract another BigUint (passed as string). Returns error if result would be negative.
- `mul_str(self: &Self, other: &str) -> Result<BigUint, String>`
  - Multiply by another BigUint (passed as string).
- `div_str(self: &Self, other: &str) -> Result<BigUint, String>`
  - Divide by another BigUint (passed as string).
- `rem_str(self: &Self, other: &str) -> Result<BigUint, String>`
  - Remainder after division (passed as string).
- `pow(self: &Self, exp: u32) -> BigUint`
  - Raise to a power (u32 exponent).
- `gcd_str(self: &Self, other: &str) -> Result<BigUint, String>`
  - Get the greatest common divisor with another BigUint.
- `to_bytes_be(self: &Self) -> Vec<u8>`
  - Convert to bytes (big-endian).
- `to_bytes_le(self: &Self) -> Vec<u8>`
  - Convert to bytes (little-endian).

### `optionals::num_complex_impl::RComplexOps`

Adapter trait for [`Complex<f64>`] operations.

Provides complex number inspection methods from R.
Automatically implemented for `Complex<f64>`.

#### Example

```rust,ignore
use num_complex::Complex;
use miniextendr_api::num_complex_impl::RComplexOps;

#[derive(ExternalPtr)]
struct MyComplex(Complex<f64>);

#[miniextendr]
impl RComplexOps for MyComplex {}
```

In R:
```r
z <- MyComplex$new(3, 4)
z$re()         # 3
z$im()         # 4
z$norm()       # 5 (magnitude)
z$arg()        # 0.927... (phase in radians)
z$conj()       # MyComplex(3, -4)
```

**Required methods:**

- `re(self: &Self) -> f64`
  - Get the real part.
- `im(self: &Self) -> f64`
  - Get the imaginary part.
- `norm(self: &Self) -> f64`
  - Get the magnitude (absolute value): sqrt(re² + im²).
- `norm_sqr(self: &Self) -> f64`
  - Get the squared magnitude: re² + im².
- `arg(self: &Self) -> f64`
  - Get the phase angle in radians.
- `is_finite(self: &Self) -> bool`
  - Check if this is a finite complex number.
- `is_infinite(self: &Self) -> bool`
  - Check if this is an infinite complex number.
- `is_nan(self: &Self) -> bool`
  - Check if this is NaN (either part).
- `is_normal(self: &Self) -> bool`
  - Check if this is normal (not zero, infinite, NaN, or subnormal).
- `conj(self: &Self) -> Complex<f64>`
  - Get the complex conjugate.
- `inv(self: &Self) -> Complex<f64>`
  - Get the reciprocal (1/z).

### `optionals::num_traits_impl::RFloat`

Adapter trait for [`num_traits::Float`].

Provides floating-point operations for any type implementing `Float`.
Automatically implemented via blanket impl.

#### Methods

##### Classification
- `r_is_nan()` - Check if NaN
- `r_is_infinite()` - Check if infinite
- `r_is_finite()` - Check if finite (not NaN or infinite)
- `r_is_normal()` - Check if normal (not zero, subnormal, infinite, or NaN)
- `r_is_sign_positive()` - Check if sign bit is positive
- `r_is_sign_negative()` - Check if sign bit is negative

##### Rounding
- `r_floor()` - Round towards negative infinity
- `r_ceil()` - Round towards positive infinity
- `r_round()` - Round to nearest integer
- `r_trunc()` - Round towards zero
- `r_fract()` - Fractional part

##### Mathematical
- `r_abs()` - Absolute value
- `r_sqrt()` - Square root
- `r_cbrt()` - Cube root
- `r_exp()` - Exponential (e^x)
- `r_exp2()` - 2^x
- `r_ln()` - Natural logarithm
- `r_log2()` - Base-2 logarithm
- `r_log10()` - Base-10 logarithm
- `r_sin()`, `r_cos()`, `r_tan()` - Trigonometric functions
- `r_asin()`, `r_acos()`, `r_atan()` - Inverse trigonometric
- `r_sinh()`, `r_cosh()`, `r_tanh()` - Hyperbolic functions

##### Special values
- `r_infinity()` - Positive infinity
- `r_neg_infinity()` - Negative infinity
- `r_nan()` - Not a Number
- `r_min_value()` - Smallest finite value
- `r_max_value()` - Largest finite value
- `r_epsilon()` - Machine epsilon

#### Example

```ignore
#[derive(ExternalPtr)]
struct MyFloat(f64);

#[miniextendr]
impl RFloat for MyFloat {}
```

In R:
```r
x <- MyFloat$new(3.7)
x$r_floor()        # 3.0
x$r_ceil()         # 4.0
x$r_is_finite()    # TRUE
x$r_sqrt()         # 1.923538
```

**Required methods:**

- `is_nan(self: &Self) -> bool`
  - Check if the value is NaN.
- `is_infinite(self: &Self) -> bool`
  - Check if the value is infinite.
- `is_finite(self: &Self) -> bool`
  - Check if the value is finite (not NaN or infinite).
- `is_normal(self: &Self) -> bool`
  - Check if the value is normal (not zero, subnormal, infinite, or NaN).
- `is_sign_positive(self: &Self) -> bool`
  - Check if the sign bit is positive.
- `is_sign_negative(self: &Self) -> bool`
  - Check if the sign bit is negative.
- `floor(self: &Self) -> Self`
  - Round towards negative infinity.
- `ceil(self: &Self) -> Self`
  - Round towards positive infinity.
- `round(self: &Self) -> Self`
  - Round to nearest integer.
- `trunc(self: &Self) -> Self`
  - Round towards zero (truncate).
- `fract(self: &Self) -> Self`
  - Get the fractional part.
- `abs(self: &Self) -> Self`
  - Get the absolute value.
- `signum(self: &Self) -> Self`
  - Get the sign of the number (1.0, -1.0, or NaN).
- `sqrt(self: &Self) -> Self`
  - Compute the square root.
- `cbrt(self: &Self) -> Self`
  - Compute the cube root.
- `exp(self: &Self) -> Self`
  - Compute e^x.
- `exp2(self: &Self) -> Self`
  - Compute 2^x.
- `ln(self: &Self) -> Self`
  - Compute the natural logarithm.
- `log2(self: &Self) -> Self`
  - Compute the base-2 logarithm.
- `log10(self: &Self) -> Self`
  - Compute the base-10 logarithm.
- `sin(self: &Self) -> Self`
  - Compute sine.
- `cos(self: &Self) -> Self`
  - Compute cosine.
- `tan(self: &Self) -> Self`
  - Compute tangent.
- `asin(self: &Self) -> Self`
  - Compute arcsine.
- `acos(self: &Self) -> Self`
  - Compute arccosine.
- `atan(self: &Self) -> Self`
  - Compute arctangent.
- `sinh(self: &Self) -> Self`
  - Compute hyperbolic sine.
- `cosh(self: &Self) -> Self`
  - Compute hyperbolic cosine.
- `tanh(self: &Self) -> Self`
  - Compute hyperbolic tangent.
- `asinh(self: &Self) -> Self`
  - Compute inverse hyperbolic sine.
- `acosh(self: &Self) -> Self`
  - Compute inverse hyperbolic cosine.
- `atanh(self: &Self) -> Self`
  - Compute inverse hyperbolic tangent.
- `infinity() -> Self`
  - Get positive infinity.
- `neg_infinity() -> Self`
  - Get negative infinity.
- `nan() -> Self`
  - Get NaN.
- `min_value() -> Self`
  - Get the smallest finite value.
- `max_value() -> Self`
  - Get the largest finite value.
- `epsilon() -> Self`
  - Get the machine epsilon.
- `powi(self: &Self, n: i32) -> Self`
  - Compute x^n for integer n.
- `powf(self: &Self, y: &Self) -> Self`
  - Compute x^y for float y.
- `recip(self: &Self) -> Self`
  - Compute the reciprocal (1/x).

### `optionals::num_traits_impl::RNum`

Adapter trait for [`num_traits::Num`].

Provides basic numeric operations for any type implementing `Num`.
Automatically implemented via blanket impl.

#### Methods

- `r_zero()` - Returns the additive identity (zero)
- `r_one()` - Returns the multiplicative identity (one)
- `r_is_zero()` - Check if value equals zero
- `r_is_one()` - Check if value equals one

#### Example

```ignore
use num_traits::Num;

#[derive(ExternalPtr)]
struct BigNum(i128);

// Blanket impl provides all methods automatically
#[miniextendr]
impl RNum for BigNum {}
```

**Required methods:**

- `zero() -> Self`
  - Get the additive identity (zero).
- `one() -> Self`
  - Get the multiplicative identity (one).
- `is_zero(self: &Self) -> bool`
  - Check if this value is zero.
- `is_one(self: &Self) -> bool`
  - Check if this value equals one.

### `optionals::num_traits_impl::RSigned`

Adapter trait for [`num_traits::Signed`].

Provides signed number operations for any type implementing `Signed`.
Automatically implemented via blanket impl.

#### Methods

- `r_abs()` - Absolute value
- `r_signum()` - Sign of the number (-1, 0, or 1)
- `r_is_positive()` - Check if positive
- `r_is_negative()` - Check if negative

#### Example

```ignore
#[derive(ExternalPtr)]
struct SignedInt(i64);

#[miniextendr]
impl RSigned for SignedInt {}
```

**Required methods:**

- `abs(self: &Self) -> Self`
  - Get the absolute value.
- `signum(self: &Self) -> Self`
  - Get the sign of the number.
- `is_positive(self: &Self) -> bool`
  - Check if the value is positive.
- `is_negative(self: &Self) -> bool`
  - Check if the value is negative.

### `optionals::ordered_float_impl::ROrderedFloatOps`

Adapter trait for [`OrderedFloat`] operations.

Provides NaN-safe numeric operations from R.
Automatically implemented for `OrderedFloat<T>` where T: FloatCore.

#### Why OrderedFloat?

Standard floats in Rust don't implement `Ord` because NaN breaks ordering.
`OrderedFloat` wraps floats to provide total ordering where NaN < all values.
This is useful for sorting, using floats as map keys, etc.

#### Example

```rust,ignore
use ordered_float::OrderedFloat;
use miniextendr_api::ordered_float_impl::ROrderedFloatOps;

#[derive(ExternalPtr)]
struct MyFloat(OrderedFloat<f64>);

#[miniextendr]
impl ROrderedFloatOps for MyFloat {}
```

In R:
```r
x <- MyFloat$new(3.14)
x$is_nan()       # FALSE
x$is_infinite()  # FALSE
x$is_finite()    # TRUE
x$floor()        # 3.0
x$ceil()         # 4.0
```

**Required methods:**

- `inner(self: &Self) -> f64`
  - Get the inner float value.
- `is_nan(self: &Self) -> bool`
  - Check if the value is NaN.
- `is_infinite(self: &Self) -> bool`
  - Check if the value is infinite (positive or negative).
- `is_finite(self: &Self) -> bool`
  - Check if the value is finite (not NaN or infinite).
- `is_positive(self: &Self) -> bool`
  - Check if the value is positive.
- `is_negative(self: &Self) -> bool`
  - Check if the value is negative.
- `floor(self: &Self) -> f64`
  - Get the floor (largest integer <= self).
- `ceil(self: &Self) -> f64`
  - Get the ceiling (smallest integer >= self).
- `round(self: &Self) -> f64`
  - Round to nearest integer.
- `trunc(self: &Self) -> f64`
  - Truncate toward zero.
- `fract(self: &Self) -> f64`
  - Get the fractional part.
- `abs(self: &Self) -> f64`
  - Get the absolute value.
- `signum(self: &Self) -> f64`
  - Get the sign: -1.0, 0.0, or 1.0.
- `min_with(self: &Self, other: f64) -> f64`
  - Return the minimum of self and other (NaN-safe).
- `max_with(self: &Self, other: f64) -> f64`
  - Return the maximum of self and other (NaN-safe).
- `clamp_to(self: &Self, min: f64, max: f64) -> f64`
  - Clamp the value to a range.

### `optionals::rand_impl::RDistributionOps`

Adapter trait for exposing probability distributions to R.

This trait provides methods for sampling from any probability distribution.
Implementations typically wrap both a distribution and an RNG together,
using interior mutability for the RNG state.

#### Methods

- `r_sample()` - Draw a single sample from the distribution
- `r_sample_n(n)` - Draw n samples from the distribution
- `r_sample_vec(n)` - Alias for sample_n

#### Example

```ignore
use std::cell::RefCell;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Normal, Distribution};

#[derive(ExternalPtr)]
struct NormalDist {
    dist: Normal<f64>,
    rng: RefCell<StdRng>,
}

impl NormalDist {
    fn new(mean: f64, std_dev: f64, seed: u64) -> Self {
        Self {
            dist: Normal::new(mean, std_dev).unwrap(),
            rng: RefCell::new(StdRng::seed_from_u64(seed)),
        }
    }
}

impl RDistributionOps<f64> for NormalDist {
    fn sample(&self) -> f64 {
        self.dist.sample(&mut *self.rng.borrow_mut())
    }
}

#[miniextendr]
impl RDistributionOps<f64> for NormalDist {}
```

In R:
```r
dist <- NormalDist$new(mean = 0, std_dev = 1, seed = 42L)
dist$r_sample()        # Single sample
dist$r_sample_n(100L)  # 100 samples
```

#### Design Note

Like `RIterator` and `RRngOps`, this trait does NOT have a blanket impl
because sampling requires mutable RNG state, but R's ExternalPtr pattern
provides `&self`. Users must use interior mutability (RefCell, Mutex, etc.).

**Required methods:**

- `sample(self: &Self) -> T`
  - Draw a single sample from the distribution.

**Provided methods:**

- `sample_n(self: &Self, n: i32) -> Vec<T>`
  - Draw n samples from the distribution.
- `sample_vec(self: &Self, n: i32) -> Vec<T>`
  - Draw n samples from the distribution (alias for sample_n).
- `mean(self: &Self) -> Option<f64>`
  - Get the mean/expected value of the distribution, if known.
- `variance(self: &Self) -> Option<f64>`
  - Get the variance of the distribution, if known.
- `std_dev(self: &Self) -> Option<f64>`
  - Get the standard deviation of the distribution, if known.

### `optionals::rand_impl::RDistributions`

Direct access to R's native distribution functions.

These methods bypass `rand`'s generic distribution machinery and call
R's optimized C implementations directly. Use these when you need:

- **Standard normal**: `standard_normal()` uses `norm_rand()`
- **Exponential(1)**: `standard_exp()` uses `exp_rand()`
- **Uniform integer**: `uniform_index(n)` uses `R_unif_index()`

#### Example

```ignore
use miniextendr_api::rand_impl::{RRng, RDistributions};

#[miniextendr(rng)]
fn sample_distributions() -> Vec<f64> {
    let mut rng = RRng::new();
    vec![
        rng.standard_normal(),  // N(0, 1)
        rng.standard_exp(),     // Exp(1)
        rng.uniform_index(100) as f64,  // Uniform integer in [0, 100)
    ]
}
```

**Required methods:**

- `standard_normal(self: &mut Self) -> f64`
  - Generate a standard normal random value (mean 0, sd 1).
- `standard_exp(self: &mut Self) -> f64`
  - Generate a standard exponential random value (rate 1).
- `uniform_index(self: &mut Self, n: usize) -> usize`
  - Generate a uniform random integer in [0, n).
- `uniform_f64(self: &mut Self) -> f64`
  - Generate a uniform random f64 in [0, 1).

### `optionals::rand_impl::RRngOps`

Adapter trait for exposing any [`rand::RngExt`] to R.

This trait provides R-friendly methods for random number generation.
It has a blanket implementation for all types implementing `Rng`,
so any Rust RNG can be exposed to R with no additional code.

#### Methods

- `r_random_f64()` - Random float in [0, 1)
- `r_random_i32()` - Random i32 (full range)
- `r_random_bool()` - Random boolean (50/50)
- `r_gen_range_f64(low, high)` - Random float in [low, high)
- `r_gen_range_i32(low, high)` - Random integer in [low, high)
- `r_gen_bool(p)` - Bernoulli trial with probability p
- `r_shuffle(items)` - Shuffle a vector in place
- `r_sample(items, n)` - Sample n items without replacement

#### Example

```ignore
use std::cell::RefCell;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(ExternalPtr)]
struct MyRng(RefCell<StdRng>);

impl MyRng {
    fn new(seed: u64) -> Self {
        Self(RefCell::new(StdRng::seed_from_u64(seed)))
    }
}

impl RRngOps for MyRng {
    fn random_f64(&self) -> f64 {
        use rand::RngExt;
        self.0.borrow_mut().random()
    }
    // ... implement other methods using self.0.borrow_mut()
}

#[miniextendr]
impl RRngOps for MyRng {}
```

In R:
```r
rng <- MyRng$new(42L)
rng$r_random_f64()           # Random float in [0, 1)
rng$r_gen_range_f64(0, 10)   # Random float in [0, 10)
rng$r_gen_bool(0.3)          # TRUE with 30% probability
```

#### Design Note

Like `RIterator`, this trait does NOT have a blanket impl because
`rand::RngExt` methods require `&mut self`, but R's ExternalPtr pattern
provides `&self`. Users must implement manually using interior mutability.

**Required methods:**

- `random_f64(self: &Self) -> f64`
  - Generate a random f64 in [0, 1).
- `random_i32(self: &Self) -> i32`
  - Generate a random i32 covering the full i32 range.
- `random_bool(self: &Self) -> bool`
  - Generate a random boolean (50% chance each).
- `gen_range_f64(self: &Self, low: f64, high: f64) -> f64`
  - Generate a random f64 in [low, high).
- `gen_range_i32(self: &Self, low: i32, high: i32) -> i32`
  - Generate a random i32 in [low, high).
- `gen_bool(self: &Self, p: f64) -> bool`
  - Generate a boolean with probability `p` of being true.

**Provided methods:**

- `random_f64_vec(self: &Self, n: i32) -> Vec<f64>`
  - Fill a vector with random f64 values in [0, 1).
- `gen_range_f64_vec(self: &Self, n: i32, low: f64, high: f64) -> Vec<f64>`
  - Fill a vector with random f64 values in [low, high).
- `gen_range_i32_vec(self: &Self, n: i32, low: i32, high: i32) -> Vec<i32>`
  - Fill a vector with random i32 values in [low, high).
- `gen_bool_vec(self: &Self, n: i32, p: f64) -> Vec<bool>`
  - Fill a vector with random booleans with probability `p` of true.

### `optionals::rayon_bridge::ParCollectR`

Extension trait for collecting indexed parallel iterators directly into R memory.

This is the primary bridge between Rayon's parallel computation and R's data
structures. It extends every `IndexedParallelIterator` with a `.collect_r()`
method that writes directly into pre-allocated R memory — zero intermediate
allocation.

Most parallel iterator chains are indexed (known length):
- `slice.par_iter().map(...)` — indexed
- `(0..n).into_par_iter().map(...)` — indexed
- `vec.into_par_iter().map(...)` — indexed
- `.enumerate()`, `.zip()`, `.take()`, `.skip()` — indexed

For non-indexed iterators (`.filter()`, `.flat_map()`), use
[`par_collect_sexp()`] which collects via an intermediate `Vec<T>`.

#### Example

```ignore
use miniextendr_api::rayon_bridge::{rayon::prelude::*, ParCollectR};

#[miniextendr]
fn parallel_sqrt(x: &[f64]) -> SEXP {
    // Zero-copy: writes sqrt values directly into R memory
    x.par_iter().map(|&v| v.sqrt()).collect_r()
}
```

#### Thread Safety

`.collect_r()` must be called from the worker thread or main thread, NOT
from inside a Rayon parallel context. The R vector allocation uses
`with_r_thread()`, which only works from those threads.

**Provided methods:**

- `collect_r(self: Self) -> SEXP`
  - Collect this indexed parallel iterator directly into an R SEXP vector.

### `optionals::rayon_bridge::RParallelExtend`

Adapter trait for parallel collection extension.

This trait provides a way to extend collections in parallel, useful for
building up large collections from multiple sources efficiently.

#### Interior Mutability

Since `ExternalPtr` only provides `&self`, implementations must use interior
mutability (e.g., `RefCell`, `Mutex`) to allow modification.

#### Example

```ignore
use miniextendr_api::rayon_bridge::RParallelExtend;
use std::sync::Mutex;

#[derive(ExternalPtr)]
struct ParallelBuffer {
    data: Mutex<Vec<f64>>,
}

impl RParallelExtend<f64> for ParallelBuffer {
    fn par_extend(&self, items: Vec<f64>) {
        // Use par_extend from rayon
        let mut guard = self.data.lock().unwrap();
        guard.par_extend(items);
    }
}

#[miniextendr]
impl RParallelExtend<f64> for ParallelBuffer {}
```

**Required methods:**

- `par_extend(self: &Self, items: Vec<T>)`
  - Extends the collection with items from a Vec in parallel.

**Provided methods:**

- `par_extend_from_slice(self: &Self, items: &[T])`
  - Extends the collection with items from a slice (clones items).
- `par_len(self: &Self) -> i32`
  - Returns the current length of the collection.
- `par_is_empty(self: &Self) -> bool`
  - Returns true if the collection is empty.
- `par_clear(self: &Self)`
  - Clears the collection.
- `par_reserve(self: &Self, _additional: i32)`
  - Reserves capacity for at least `additional` more elements.

### `optionals::rayon_bridge::RParallelIterator`

Adapter trait for exposing parallel iteration operations to R.

This trait provides a way to expose Rayon's parallel iteration capabilities
to R via `#[miniextendr]`. Unlike the standard `ParallelIterator`
trait, this adapter is designed to work with `ExternalPtr<T>` which only provides
`&self` access.

#### Design

The trait is designed around non-consuming parallel operations:
- Aggregations (sum, min, max, mean, count)
- Predicates (any, all, find)
- Transformations that return new collections (map, filter)

#### Interior Mutability

Since `ExternalPtr` provides `&self` and parallel iteration typically works
with owned data or `&[T]` slices, implementations should either:
1. Store data in a way that allows parallel access (e.g., `Vec<T>`, `[T]`)
2. Use interior mutability if the iteration consumes cached state

#### Example

```ignore
use miniextendr_api::rayon_bridge::RParallelIterator;
use miniextendr_api::ExternalPtr;

#[derive(ExternalPtr)]
struct ParallelData {
    values: Vec<f64>,
}

impl RParallelIterator for ParallelData {
    type Item = f64;

    fn par_iter(&self) -> impl rayon::iter::ParallelIterator<Item = Self::Item> + '_ {
        self.values.par_iter().copied()
    }
}

#[miniextendr]
impl RParallelIterator for ParallelData {}
```

In R:
```r
data <- ParallelData$new(as.numeric(1:1000000))
data$r_par_sum()      # Fast parallel sum
data$r_par_mean()     # Parallel mean
data$r_par_min()      # Parallel minimum
data$r_par_count()    # Count elements
```

**Required methods:**

- `par_iter(self: &Self) -> impl rayon::iter::ParallelIterator`
  - Returns a parallel iterator over the elements.

**Provided methods:**

- `par_len(self: &Self) -> i32`
  - Returns the number of elements, if known.
- `par_sum(self: &Self) -> f64`
  - Computes the parallel sum of f64 elements.
- `par_sum_int(self: &Self) -> i32`
  - Computes the parallel sum of i32 elements.
- `par_sum_i64(self: &Self) -> f64`
  - Computes the parallel sum of i64 elements (returned as f64 for R).
- `par_mean(self: &Self) -> f64`
  - Computes the parallel mean of f64 elements.
- `par_min(self: &Self) -> Option<<Self as >::Item>`
  - Finds the parallel minimum.
- `par_max(self: &Self) -> Option<<Self as >::Item>`
  - Finds the parallel maximum.
- `par_min_f64(self: &Self) -> f64`
  - Finds the parallel minimum f64 (handles NaN).
- `par_max_f64(self: &Self) -> f64`
  - Finds the parallel maximum f64 (handles NaN).
- `par_count(self: &Self) -> i32`
  - Counts the number of elements in parallel.
- `par_product(self: &Self) -> f64`
  - Computes the parallel product of f64 elements.
- `par_any_gt(self: &Self, threshold: f64) -> bool`
  - Returns true if any element satisfies the predicate (greater than threshold).
- `par_all_gt(self: &Self, threshold: f64) -> bool`
  - Returns true if all elements satisfy the predicate (greater than threshold).
- `par_any_lt(self: &Self, threshold: f64) -> bool`
  - Returns true if any element satisfies the predicate (less than threshold).
- `par_all_lt(self: &Self, threshold: f64) -> bool`
  - Returns true if all elements satisfy the predicate (less than threshold).
- `par_count_gt(self: &Self, threshold: f64) -> i32`
  - Counts elements greater than threshold.
- `par_count_lt(self: &Self, threshold: f64) -> i32`
  - Counts elements less than threshold.
- `par_count_eq(self: &Self, value: f64, epsilon: f64) -> i32`
  - Counts elements equal to value (within epsilon for floats).
- `par_variance(self: &Self) -> f64`
  - Computes variance in parallel.
- `par_std_dev(self: &Self) -> f64`
  - Computes standard deviation in parallel.
- `par_filter_gt(self: &Self, threshold: f64) -> Vec<f64>`
  - Collects elements greater than threshold into a `Vec<f64>`.
- `par_filter_lt(self: &Self, threshold: f64) -> Vec<f64>`
  - Collects elements less than threshold into a `Vec<f64>`.
- `par_scale(self: &Self, factor: f64) -> Vec<f64>`
  - Applies a scalar operation and collects results (multiply by factor).
- `par_offset(self: &Self, offset: f64) -> Vec<f64>`
  - Applies offset and collects results (add offset).
- `par_clamp(self: &Self, min: f64, max: f64) -> Vec<f64>`
  - Clamps values to range and collects results.
- `par_abs(self: &Self) -> Vec<f64>`
  - Applies absolute value and collects results.
- `par_sqrt(self: &Self) -> Vec<f64>`
  - Applies square root and collects results.
- `par_pow(self: &Self, exp: f64) -> Vec<f64>`
  - Applies power and collects results.
- `par_ln(self: &Self) -> Vec<f64>`
  - Applies natural log and collects results.
- `par_exp(self: &Self) -> Vec<f64>`
  - Applies exp and collects results.

**Associated items:**

- associated type `Item`

### `optionals::regex_impl::RCaptureGroups`

Adapter trait for capture group access.

Provides methods to access regex capture groups from R.

#### Example

```rust,ignore
use regex::Regex;
use miniextendr_api::regex_impl::{CaptureGroups, RCaptureGroups};

let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
let caps = CaptureGroups::capture(&re, "Date: 2024-01-15").unwrap();

assert_eq!(caps.get(0), Some("2024-01-15".to_string()));  // whole match
assert_eq!(caps.get(1), Some("2024".to_string()));        // year
assert_eq!(caps.get(2), Some("01".to_string()));          // month
assert_eq!(caps.get(3), Some("15".to_string()));          // day
```

**Required methods:**

- `get(self: &Self, i: i32) -> Option<String>`
  - Get a capture group by index (0 = whole match).
- `get_named(self: &Self, name: &str) -> Option<String>`
  - Get a capture group by name.
- `len(self: &Self) -> i32`
  - Get the number of capture groups.
- `is_empty(self: &Self) -> bool`
  - Check if there are no capture groups.
- `all_groups(self: &Self) -> Vec<Option<String>>`
  - Get all capture groups as a vector (None for non-matching groups).

### `optionals::regex_impl::RRegexOps`

Adapter trait for [`Regex`] operations.

Provides string replacement and matching operations from R.
Automatically implemented for `regex::Regex`.

#### Example

```rust,ignore
use regex::Regex;
use miniextendr_api::regex_impl::RRegexOps;

#[derive(ExternalPtr)]
struct MyPattern(Regex);

#[miniextendr]
impl RRegexOps for MyPattern {}
```

In R:
```r
pat <- compile_regex("\\d+")
pat$replace_first("abc123def456", "X")  # "abcXdef456"
pat$replace_all("abc123def456", "X")    # "abcXdefX"
pat$is_match("test123")                 # TRUE
pat$find("test123")                     # "123"
```

**Required methods:**

- `replace_first(self: &Self, text: &str, replacement: &str) -> String`
  - Replace the first match in the text.
- `replace_all(self: &Self, text: &str, replacement: &str) -> String`
  - Replace all matches in the text.
- `is_match(self: &Self, text: &str) -> bool`
  - Check if the pattern matches anywhere in the text.
- `find(self: &Self, text: &str) -> Option<String>`
  - Find the first match and return it, or None if no match.
- `find_all(self: &Self, text: &str) -> Vec<String>`
  - Find all non-overlapping matches.
- `split(self: &Self, text: &str) -> Vec<String>`
  - Split the text by the pattern.
- `captures_len(self: &Self) -> i32`
  - Get the number of capture groups (including the whole match).

### `optionals::rust_decimal_impl::RDecimalOps`

Adapter trait for [`Decimal`] operations.

Provides fixed-precision decimal arithmetic from R.
Automatically implemented for `rust_decimal::Decimal`.

#### Example

```rust,ignore
use rust_decimal::Decimal;
use miniextendr_api::rust_decimal_impl::RDecimalOps;

#[derive(ExternalPtr)]
struct MyDecimal(Decimal);

#[miniextendr]
impl RDecimalOps for MyDecimal {}
```

In R:
```r
x <- MyDecimal$from_str("123.456")
x$add_str("0.001")$as_string()  # "123.457"
x$round(2)$as_string()          # "123.46"
x$scale()                       # 3
```

**Required methods:**

- `as_string(self: &Self) -> String`
  - Convert to string representation.
- `is_zero(self: &Self) -> bool`
  - Check if zero.
- `is_positive(self: &Self) -> bool`
  - Check if positive (> 0).
- `is_negative(self: &Self) -> bool`
  - Check if negative (< 0).
- `sign(self: &Self) -> i32`
  - Get the sign as an integer: -1, 0, or 1.
- `scale(self: &Self) -> i32`
  - Get the number of decimal places.
- `abs(self: &Self) -> Decimal`
  - Get the absolute value.
- `neg(self: &Self) -> Decimal`
  - Negate the value.
- `add_str(self: &Self, other: &str) -> Result<Decimal, String>`
  - Add another Decimal (passed as string).
- `sub_str(self: &Self, other: &str) -> Result<Decimal, String>`
  - Subtract another Decimal (passed as string).
- `mul_str(self: &Self, other: &str) -> Result<Decimal, String>`
  - Multiply by another Decimal (passed as string).
- `div_str(self: &Self, other: &str) -> Result<Decimal, String>`
  - Divide by another Decimal (passed as string).
- `rem_str(self: &Self, other: &str) -> Result<Decimal, String>`
  - Remainder after division (passed as string).
- `round(self: &Self, dp: i32) -> Decimal`
  - Round to the specified number of decimal places.
- `floor(self: &Self) -> Decimal`
  - Round toward negative infinity.
- `ceil(self: &Self) -> Decimal`
  - Round toward positive infinity.
- `trunc(self: &Self) -> Decimal`
  - Truncate toward zero (remove decimal places).
- `fract(self: &Self) -> Decimal`
  - Get the fractional part (value - trunc).
- `as_f64(self: &Self) -> f64`
  - Convert to f64 (may lose precision).
- `as_i64(self: &Self) -> Option<i64>`
  - Try to convert to i64 (returns None if out of range or has decimals).
- `normalize(self: &Self) -> Decimal`
  - Normalize - remove trailing zeros.
- `is_integer(self: &Self) -> bool`
  - Check if the value is an integer (no fractional part).

### `optionals::serde_impl::RDeserialize`

Adapter trait for [`serde::Deserialize`].

Provides JSON deserialization for R, allowing JSON strings to be parsed
into Rust types.

#### Methods

- `from_json(s)` - Parse JSON string, returning None on failure
- `from_json_result(s)` - Parse JSON string with detailed error

#### Example

```rust,ignore
#[derive(Deserialize, ExternalPtr)]
struct Config { name: String, value: i32 }

#[miniextendr]
impl RDeserialize for Config {}
```

In R:
```r
cfg <- Config$from_json('{"name":"test","value":42}')
```

**Required methods:**

- `from_json(s: &str) -> Option<Self>`
  - Parse a JSON string into this type.
- `from_json_result(s: &str) -> Result<Self, String>`
  - Parse a JSON string with detailed error information.

### `optionals::serde_impl::RJsonBridge`

Bridge trait for direct Rust struct to R list conversion via `serde_json::Value`.

This converts Rust types to/from native R lists without going through
a JSON string intermediate. The path is: `Rust struct -> serde_json::Value -> R SEXP`
(and vice versa), which avoids the overhead of serializing to and parsing
from a JSON string.

#### Example

```rust,ignore
use serde::{Serialize, Deserialize};
use miniextendr_api::serde_impl::RJsonBridge;

#[derive(Serialize, Deserialize)]
struct Config {
    name: String,
    value: i32,
    tags: Vec<String>,
}

// The blanket impl provides to_r_list() and from_r_list() automatically.
fn export_config(cfg: &Config) -> SEXP {
    cfg.to_r_list()
}

fn import_config(sexp: SEXP) -> Result<Config, String> {
    Config::from_r_list(sexp)
}
```

In R, the result is a native named list:
```r
cfg <- export_config(cfg_ptr)
cfg$name   # "my-app"
cfg$value  # 42L
cfg$tags   # c("alpha", "beta")
```

**Required methods:**

- `to_r_list(self: &Self) -> SEXP`
  - Convert this value to a native R list/vector via `serde_json::Value`.
- `from_r_list(sexp: SEXP) -> Result<Self, String>`
  - Create a value of this type from an R object via `serde_json::Value`.

### `optionals::serde_impl::RJsonValueOps`

Adapter trait for JSON value inspection.

#### Registration

Registration is automatic when you annotate `impl RJsonValueOps for JsonValue`
with `#[miniextendr]`.

**Required methods:**

- `is_null(self: &Self) -> bool`
  - Check if this is a null value.
- `is_boolean(self: &Self) -> bool`
  - Check if this is a boolean.
- `is_number(self: &Self) -> bool`
  - Check if this is a number.
- `is_string(self: &Self) -> bool`
  - Check if this is a string.
- `is_array(self: &Self) -> bool`
  - Check if this is an array.
- `is_object(self: &Self) -> bool`
  - Check if this is an object.
- `type_name(self: &Self) -> String`
  - Get the type name.
- `to_json_string(self: &Self) -> String`
  - Serialize to compact JSON string.
- `to_json_string_pretty(self: &Self) -> String`
  - Serialize to pretty JSON string.
- `as_bool(self: &Self) -> Option<bool>`
  - Get as boolean if this is a boolean.
- `as_i64(self: &Self) -> Option<i64>`
  - Get as integer if this is an integer.
- `as_f64(self: &Self) -> Option<f64>`
  - Get as float if this is a number.
- `as_str(self: &Self) -> Option<String>`
  - Get as string if this is a string.
- `array_len(self: &Self) -> Option<i32>`
  - Get array length if this is an array.
- `object_keys(self: &Self) -> Vec<String>`
  - Get object keys if this is an object.

### `optionals::serde_impl::RSerialize`

Adapter trait for [`serde::Serialize`].

Provides JSON serialization for R, allowing Rust types to be converted
to JSON strings for storage, transmission, or interop with other tools.

#### Methods

- `r_to_json()` - Serialize to compact JSON string
- `r_to_json_pretty()` - Serialize to pretty-printed JSON
- `r_to_json_value()` - Serialize to a JSON value (for inspection)

#### Example

```rust,ignore
#[derive(Serialize, ExternalPtr)]
struct Point { x: f64, y: f64 }

#[miniextendr]
impl RSerialize for Point {}
```

**Required methods:**

- `to_json(self: &Self) -> Result<String, String>`
  - Serialize to a compact JSON string.
- `to_json_pretty(self: &Self) -> Result<String, String>`
  - Serialize to a pretty-printed JSON string with indentation.

### `optionals::time_impl::RDateTimeFormat`

Adapter trait for formatting and parsing datetime types.

Provides format/parse operations for `time::OffsetDateTime` and `time::Date`.
Uses `time` crate's format description syntax.

#### Format Syntax

The format string uses bracketed component specifiers:
- `[year]` - 4-digit year
- `[month]` - Month (01-12)
- `[day]` - Day of month (01-31)
- `[hour]` - Hour (00-23)
- `[minute]` - Minute (00-59)
- `[second]` - Second (00-59)
- `[subsecond]` - Fractional seconds
- `[offset_hour]`, `[offset_minute]` - Timezone offset

See `time` crate documentation for full format specification.

#### Example

```rust,ignore
use time::OffsetDateTime;
use miniextendr_api::time_impl::RDateTimeFormat;

let now = OffsetDateTime::now_utc();
let formatted = now.format("[year]-[month]-[day] [hour]:[minute]:[second]");
// e.g., "2024-01-15 14:30:00"

let parsed = OffsetDateTime::r_parse(
    "2024-01-15 14:30:00 +00:00:00",
    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
);
```

**Required methods:**

- `format(self: &Self, fmt: &str) -> Result<String, String>`
  - Format using a format description string.
- `parse(s: &str, fmt: &str) -> Result<Self, String>`
  - Parse from a string using a format description.

### `optionals::time_impl::RDuration`

Adapter trait for [`time::Duration`].

Provides methods to inspect and manipulate durations from R.
Automatically implemented for `time::Duration`.

#### Methods

- `as_seconds_f64()` - Total duration as floating-point seconds
- `as_milliseconds()` - Total duration in milliseconds (i64)
- `whole_days()` - Number of whole days
- `whole_hours()` - Number of whole hours
- `whole_minutes()` - Number of whole minutes
- `whole_seconds()` - Number of whole seconds
- `is_negative()` - Check if duration is negative
- `is_zero()` - Check if duration is zero
- `abs()` - Absolute value of duration

#### Example

```rust,ignore
use time::Duration;
use miniextendr_api::time_impl::RDuration;

#[derive(ExternalPtr)]
struct MyDuration(Duration);

#[miniextendr]
impl RDuration for MyDuration {}
```

In R:
```r
d <- MyDuration$new(...)
d$as_seconds_f64()  # e.g., 3661.5
d$whole_hours()     # e.g., 1
d$is_negative()     # FALSE
```

**Required methods:**

- `as_seconds_f64(self: &Self) -> f64`
  - Get the total duration as floating-point seconds.
- `as_milliseconds(self: &Self) -> i64`
  - Get the total duration in milliseconds.
- `whole_days(self: &Self) -> i64`
  - Get the number of whole days in the duration.
- `whole_hours(self: &Self) -> i64`
  - Get the number of whole hours in the duration.
- `whole_minutes(self: &Self) -> i64`
  - Get the number of whole minutes in the duration.
- `whole_seconds(self: &Self) -> i64`
  - Get the number of whole seconds in the duration.
- `subsec_nanoseconds(self: &Self) -> i32`
  - Get the subsecond nanoseconds component.
- `is_negative(self: &Self) -> bool`
  - Check if the duration is negative.
- `is_zero(self: &Self) -> bool`
  - Check if the duration is zero.
- `abs(self: &Self) -> Duration`
  - Get the absolute value of this duration.

### `optionals::toml_impl::RTomlOps`

Adapter trait for TOML value inspection.

#### Registration

Registration is automatic when you annotate `impl RTomlOps for TomlValue`
with `#[miniextendr]`.

**Required methods:**

- `is_string(self: &Self) -> bool`
  - Check if this is a string value.
- `is_integer(self: &Self) -> bool`
  - Check if this is an integer value.
- `is_float(self: &Self) -> bool`
  - Check if this is a float value.
- `is_boolean(self: &Self) -> bool`
  - Check if this is a boolean value.
- `is_datetime(self: &Self) -> bool`
  - Check if this is a datetime value.
- `is_array(self: &Self) -> bool`
  - Check if this is an array.
- `is_table(self: &Self) -> bool`
  - Check if this is a table.
- `type_name(self: &Self) -> String`
  - Get the type name as a string.
- `to_toml_string(self: &Self) -> String`
  - Serialize to TOML string.
- `to_toml_string_pretty(self: &Self) -> String`
  - Serialize to pretty TOML string.
- `as_str(self: &Self) -> Option<String>`
  - Get as string if this is a string value.
- `as_integer(self: &Self) -> Option<i64>`
  - Get as integer if this is an integer value.
- `as_float(self: &Self) -> Option<f64>`
  - Get as float if this is a float value.
- `as_bool(self: &Self) -> Option<bool>`
  - Get as boolean if this is a boolean value.
- `array_len(self: &Self) -> Option<i32>`
  - Get array length if this is an array.
- `table_keys(self: &Self) -> Vec<String>`
  - Get table keys if this is a table.

### `optionals::url_impl::RUrlOps`

Adapter trait for [`Url`] operations.

Provides URL inspection and manipulation methods for R.
Automatically implemented for `Url`.

#### Example

```rust,ignore
use url::Url;
use miniextendr_api::url_impl::RUrlOps;

#[derive(ExternalPtr)]
struct MyUrl(Url);

#[miniextendr]
impl RUrlOps for MyUrl {}
```

In R:
```r
u <- MyUrl$new("https://example.com:8080/path?query=1#frag")
u$scheme()     # "https"
u$host()       # "example.com"
u$port()       # 8080 (NA if not specified)
u$path()       # "/path"
u$query()      # "query=1" (NA if none)
u$fragment()   # "frag" (NA if none)
u$as_str()     # full URL string
```

**Required methods:**

- `scheme(self: &Self) -> String`
  - Get the URL scheme (e.g., "https", "http", "ftp").
- `host(self: &Self) -> Option<String>`
  - Get the host string, if present.
- `port(self: &Self) -> Option<u16>`
  - Get the port number, if explicitly specified.
- `port_or_known_default(self: &Self) -> Option<u16>`
  - Get the port or default port for the scheme.
- `path(self: &Self) -> String`
  - Get the path component.
- `query(self: &Self) -> Option<String>`
  - Get the query string, if present.
- `fragment(self: &Self) -> Option<String>`
  - Get the fragment identifier, if present.
- `as_str(self: &Self) -> String`
  - Get the full URL as a string.
- `username(self: &Self) -> String`
  - Get the username, if present.
- `password(self: &Self) -> Option<String>`
  - Get the password, if present.
- `cannot_be_a_base(self: &Self) -> bool`
  - Check if this URL cannot be a base (has opaque path).
- `origin(self: &Self) -> String`
  - Get the origin as a string.

### `optionals::uuid_impl::RUuidOps`

Adapter trait for [`Uuid`] operations.

Provides UUID inspection and generation methods from R.
Automatically implemented for `uuid::Uuid`.

#### Example

```rust,ignore
use uuid::Uuid;
use miniextendr_api::uuid_impl::RUuidOps;

#[derive(ExternalPtr)]
struct MyUuid(Uuid);

#[miniextendr]
impl RUuidOps for MyUuid {}
```

In R:
```r
id <- MyUuid$new_v4()
id$version()       # 4
id$is_nil()        # FALSE
id$as_bytes()      # raw(16)
id$to_hyphenated() # "550e8400-e29b-41d4-a716-446655440000"
```

**Required methods:**

- `version(self: &Self) -> i32`
  - Get the UUID version number (1-8, or 0 for nil).
- `variant(self: &Self) -> String`
  - Get the UUID variant.
- `is_nil(self: &Self) -> bool`
  - Check if this is a nil (all zeros) UUID.
- `is_max(self: &Self) -> bool`
  - Check if this is the max (all ones) UUID.
- `as_bytes(self: &Self) -> Vec<u8>`
  - Get the UUID as a 16-byte raw vector.
- `to_hyphenated(self: &Self) -> String`
  - Get the UUID as hyphenated string (standard format).
- `to_simple(self: &Self) -> String`
  - Get the UUID as simple string (no hyphens).
- `to_urn(self: &Self) -> String`
  - Get the UUID as URN format (urn:uuid:...).

### `r_coerce::RCoerceCharacter`

Trait for types that can be coerced to `character` via `as.character()`.

This typically produces a string representation of the object.
For single values, return a single-element vector; for collections,
return a vector with one element per item.

**Required methods:**

- `as_character(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R character vector.

### `r_coerce::RCoerceComplex`

Trait for types that can be coerced to `complex` via `as.complex()`.

The result should be an R complex vector (CPLXSXP).

**Required methods:**

- `as_complex(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R complex vector.

### `r_coerce::RCoerceDataFrame`

Trait for types that can be coerced to `data.frame` via `as.data.frame()`.

#### Example

```ignore
use miniextendr_api::r_coerce::{RCoerceDataFrame, RCoerceError};
use miniextendr_api::List;

impl RCoerceDataFrame for MyStruct {
    fn as_data_frame(&self) -> Result<List, RCoerceError> {
        Ok(List::from_pairs(vec![
            ("col1", self.field1.clone()),
            ("col2", self.field2.clone()),
        ])
        .set_class_str(&["data.frame"])
        .set_row_names_int(self.field1.len()))
    }
}
```

**Required methods:**

- `as_data_frame(self: &Self) -> Result<crate::List, RCoerceError>`
  - Convert to an R data.frame.

### `r_coerce::RCoerceDate`

Trait for types that can be coerced to `Date` via `as.Date()`.

The result should be an R Date object (numeric with "Date" class).

**Required methods:**

- `as_date(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R Date.

### `r_coerce::RCoerceEnvironment`

Trait for types that can be coerced to `environment` via `as.environment()`.

The result should be an R environment.

**Required methods:**

- `as_environment(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R environment.

### `r_coerce::RCoerceFactor`

Trait for types that can be coerced to `factor` via `as.factor()`.

The result should be an R factor (integer vector with levels attribute).

**Required methods:**

- `as_factor(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R factor.

### `r_coerce::RCoerceFunction`

Trait for types that can be coerced to `function` via `as.function()`.

The result should be an R function (closure).

**Required methods:**

- `as_function(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R function.

### `r_coerce::RCoerceInteger`

Trait for types that can be coerced to `integer` via `as.integer()`.

The result should be an R integer vector (INTSXP).

**Required methods:**

- `as_integer(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R integer vector.

### `r_coerce::RCoerceList`

Trait for types that can be coerced to `list` via `as.list()`.

#### Example

```ignore
impl RCoerceList for MyStruct {
    fn as_list(&self) -> Result<List, RCoerceError> {
        Ok(List::from_pairs(vec![
            ("field1", self.field1.clone()),
            ("field2", self.field2.clone()),
        ]))
    }
}
```

**Required methods:**

- `as_list(self: &Self) -> Result<crate::List, RCoerceError>`
  - Convert to an R list.

### `r_coerce::RCoerceLogical`

Trait for types that can be coerced to `logical` via `as.logical()`.

The result should be an R logical vector (LGLSXP).

**Required methods:**

- `as_logical(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R logical vector.

### `r_coerce::RCoerceMatrix`

Trait for types that can be coerced to `matrix` via `as.matrix()`.

The result should be an R matrix with appropriate dimensions.

**Required methods:**

- `as_matrix(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R matrix.

### `r_coerce::RCoerceNumeric`

Trait for types that can be coerced to `numeric`/`double` via `as.numeric()`.

The result should be an R numeric vector (REALSXP).

**Required methods:**

- `as_numeric(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R numeric vector.

### `r_coerce::RCoercePOSIXct`

Trait for types that can be coerced to `POSIXct` via `as.POSIXct()`.

The result should be an R POSIXct object (numeric with "POSIXct", "POSIXt" class).

**Required methods:**

- `as_posixct(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R POSIXct.

### `r_coerce::RCoerceRaw`

Trait for types that can be coerced to `raw` via `as.raw()`.

The result should be an R raw vector (RAWSXP).

**Required methods:**

- `as_raw(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R raw vector.

### `r_coerce::RCoerceVector`

Trait for types that can be coerced to a generic `vector` via `as.vector()`.

This is the most general vector coercion, typically stripping attributes.

**Required methods:**

- `as_vector(self: &Self) -> Result<crate::SEXP, RCoerceError>`
  - Convert to an R vector.

### `refcount_protect::MapStorage`

Trait abstracting over map implementations for arena storage.

This allows [`Arena`] to be generic over the underlying map type,
supporting both `BTreeMap` and `HashMap`.

**Required methods:**

- `get(self: &Self, key: &usize) -> Option<&Entry>`
  - Get an entry by key.
- `get_mut(self: &mut Self, key: &usize) -> Option<&mut Entry>`
  - Get a mutable entry by key.
- `insert(self: &mut Self, key: usize, entry: Entry) -> Option<Entry>`
  - Insert an entry, returning the old value if present.
- `remove(self: &mut Self, key: &usize) -> Option<Entry>`
  - Remove an entry by key.
- `contains_key(self: &Self, key: &usize) -> bool`
  - Check if a key exists.
- `for_each_entry<F>(self: &Self, f: F)`
  - Iterate over all entries.
- `clear(self: &mut Self)`
  - Clear all entries.

**Provided methods:**

- `reserve(self: &mut Self, _additional: usize)`
  - Reserve capacity for additional entries.
- `decrement_and_maybe_remove(self: &mut Self, key: &usize) -> Option<(bool, usize)>`
  - Decrement the count for a key and remove if zero.

### `refcount_protect::ThreadLocalArenaOps`

Trait providing default implementations for all thread-local arena methods.

Implementors only need to provide [`with_state`](Self::with_state) to access
the thread-local state; all 14 arena methods are provided as defaults.

The `define_thread_local_arena!` macro generates both the struct and the
`ThreadLocalArenaOps` impl, so this trait is an implementation detail.
Import it when calling methods on thread-local arena types:

```ignore
use miniextendr_api::refcount_protect::{ThreadLocalArena, ThreadLocalArenaOps};
unsafe { ThreadLocalArena::protect(x) };
```

**Required methods:**

- `with_state<R, F>(f: F) -> R`
  - Access the thread-local state.

**Provided methods:**

- `unsafe init()`
  - Initialize the arena with default capacity (called automatically on first use).
- `unsafe init_with_capacity(capacity: usize)`
  - Initialize the arena with specific capacity.
- `unsafe protect(x: SEXP) -> SEXP`
  - Protect a SEXP, incrementing its reference count.
- `unsafe unprotect(x: SEXP)`
  - Unprotect a SEXP.
- `unsafe try_unprotect(x: SEXP) -> bool`
  - Try to unprotect a SEXP.
- `unsafe protect_fast(x: SEXP) -> SEXP`
  - Protect without checking initialization.
- `unsafe unprotect_fast(x: SEXP)`
  - Unprotect without checking initialization.
- `unsafe try_unprotect_fast(x: SEXP) -> bool`
  - Try to unprotect without checking initialization.
- `is_protected(x: SEXP) -> bool`
  - Check if a SEXP is protected.
- `ref_count(x: SEXP) -> usize`
  - Get reference count.
- `len() -> usize`
  - Number of protected SEXPs.
- `is_empty() -> bool`
  - Check if empty.
- `capacity() -> usize`
  - Get capacity.
- `unsafe clear()`
  - Clear all protections.

**Associated items:**

- associated type `Map`

### `serde::traits::RDeserializeNative`

Adapter trait for direct R deserialization (R SEXP -> Rust).

This trait provides methods to deserialize R objects directly to Rust types
without going through an intermediate format like JSON.

#### Type Mappings

| R Type | Rust Type |
|--------|-----------|
| `logical(1)` | `bool` |
| `integer(1)` | `i32` |
| `numeric(1)` | `f64` |
| `character(1)` | `String` |
| NA values | `Option<T>::None` |
| atomic vectors | `Vec<primitive>` or `Box<[primitive]>` |
| lists | `Vec<T>` |
| named lists | struct or `HashMap<String, T>` |
| NULL | `()` or `Option<T>::None` |

#### Registration

```rust,ignore
use miniextendr_api::serde_r::RDeserializeNative;
use serde::Deserialize;

#[derive(Deserialize, ExternalPtr)]
struct Config { name: String, value: i32 }

#[miniextendr]
impl RDeserializeNative for Config {}
```

#### R Usage

```r
# Create from R list
data <- list(name = "test", value = 42L)
cfg <- Config$from_r(data)

# Now cfg is a Config external pointer
cfg$name   # "test"
cfg$value  # 42L
```

**Required methods:**

- `from_r(sexp: SEXP) -> Result<Self, String>`
  - Deserialize from an R object.

### `serde::traits::RSerializeNative`

Adapter trait for direct R serialization (Rust -> R SEXP).

This trait provides methods to serialize Rust types directly to R objects
without going through an intermediate format like JSON.

#### Type Mappings

| Rust Type | R Type |
|-----------|--------|
| `bool` | `logical(1)` |
| `i32` | `integer(1)` |
| `f64` | `numeric(1)` |
| `String` | `character(1)` |
| `Option<T>::None` | NA or NULL |
| `Vec<primitive>` | atomic vector |
| `Vec<struct>` | list of lists |
| `HashMap<String, T>` | named list |
| `struct` | named list |

#### Registration

```rust,ignore
use miniextendr_api::serde_r::RSerializeNative;
use serde::Serialize;

#[derive(Serialize, ExternalPtr)]
struct Config { name: String, value: i32 }

#[miniextendr]
impl RSerializeNative for Config {}
```

#### R Usage

```r
cfg <- Config$new("test", 42L)
data <- cfg$to_r()
# Returns: list(name = "test", value = 42L)

# Access fields directly
data$name   # "test"
data$value  # 42L
```

**Required methods:**

- `to_r(self: &Self) -> Result<SEXP, String>`
  - Serialize this value to a native R object.

### `sexp_ext::SexpExt`

Extension trait for SEXP providing safe(r) accessors and type checking.

This trait provides idiomatic Rust methods for working with SEXPs,
equivalent to R's inline macros and type checking functions.

**Required methods:**

- `type_of(self: &Self) -> SEXPTYPE`
  - Get the type of this SEXP.
- `is_null_or_nil(self: &Self) -> bool`
  - Check if this SEXP is null or R_NilValue.
- `len(self: &Self) -> usize`
  - Get the length of this SEXP as `usize`.
- `xlength(self: &Self) -> R_xlen_t`
  - Get the length as `R_xlen_t`.
- `unsafe xlength_unchecked(self: &Self) -> R_xlen_t`
  - Get the length as `R_xlen_t` without thread checks.
- `unsafe len_unchecked(self: &Self) -> usize`
  - Get the length without thread checks.
- `unsafe as_slice<T>(self: &Self) -> &'static [T]`
  - Get a slice view of this SEXP's data.
- `unsafe as_slice_unchecked<T>(self: &Self) -> &'static [T]`
  - Get a slice view without thread checks.
- `unsafe as_mut_slice<T>(self: &Self) -> &'static mut [T]`
  - Get a mutable slice view of this SEXP's data.
- `is_integer(self: &Self) -> bool`
  - Check if this SEXP is an integer vector (INTSXP).
- `is_real(self: &Self) -> bool`
  - Check if this SEXP is a real/numeric vector (REALSXP).
- `is_logical(self: &Self) -> bool`
  - Check if this SEXP is a logical vector (LGLSXP).
- `is_character(self: &Self) -> bool`
  - Check if this SEXP is a character/string vector (STRSXP).
- `is_raw(self: &Self) -> bool`
  - Check if this SEXP is a raw vector (RAWSXP).
- `is_complex(self: &Self) -> bool`
  - Check if this SEXP is a complex vector (CPLXSXP).
- `is_list(self: &Self) -> bool`
  - Check if this SEXP is a list/generic vector (VECSXP).
- `is_external_ptr(self: &Self) -> bool`
  - Check if this SEXP is an external pointer (EXTPTRSXP).
- `is_environment(self: &Self) -> bool`
  - Check if this SEXP is an environment (ENVSXP).
- `is_symbol(self: &Self) -> bool`
  - Check if this SEXP is a symbol (SYMSXP).
- `is_language(self: &Self) -> bool`
  - Check if this SEXP is a language object (LANGSXP).
- `is_altrep(self: &Self) -> bool`
  - Check if this SEXP is an ALTREP object.
- `is_empty(self: &Self) -> bool`
  - Check if this `SEXP` contains any elements.
- `is_nil(self: &Self) -> bool`
  - Check if this SEXP is R's `NULL` (NILSXP).
- `is_factor(self: &Self) -> bool`
  - Check if this SEXP is a factor.
- `is_pair_list(self: &Self) -> bool`
  - Check if this SEXP is a pairlist (LISTSXP or NILSXP).
- `is_matrix(self: &Self) -> bool`
  - Check if this SEXP is a matrix.
- `is_array(self: &Self) -> bool`
  - Check if this SEXP is an array.
- `is_function(self: &Self) -> bool`
  - Check if this SEXP is a function (closure, builtin, or special).
- `is_s4(self: &Self) -> bool`
  - Check if this SEXP is an S4 object.
- `is_data_frame(self: &Self) -> bool`
  - Check if this SEXP is a data.frame.
- `is_numeric(self: &Self) -> bool`
  - Check if this SEXP is a numeric type (integer, logical, or real, excluding factors).
- `is_number(self: &Self) -> bool`
  - Check if this SEXP is a number type (numeric or complex).
- `is_vector_atomic(self: &Self) -> bool`
  - Check if this SEXP is an atomic vector.
- `is_vector_list(self: &Self) -> bool`
  - Check if this SEXP is a vector list (VECSXP or EXPRSXP).
- `is_vector(self: &Self) -> bool`
  - Check if this SEXP is a vector (atomic vector or list).
- `is_object(self: &Self) -> bool`
  - Check if this SEXP is an R "object" (has a class attribute).
- `coerce(self: &Self, target: SEXPTYPE) -> SEXP`
  - Coerce this SEXP to the given type, returning a new SEXP.
- `as_logical(self: &Self) -> Option<bool>`
  - Extract a scalar logical value.
- `as_integer(self: &Self) -> Option<i32>`
  - Extract a scalar integer value.
- `as_real(self: &Self) -> Option<f64>`
  - Extract a scalar real value.
- `as_char(self: &Self) -> SEXP`
  - Extract a scalar CHARSXP from this SEXP.
- `get_attr(self: &Self, name: SEXP) -> SEXP`
  - Get an attribute by symbol.
- `set_attr(self: &Self, name: SEXP, val: SEXP)`
  - Set an attribute by symbol.
- `get_names(self: &Self) -> SEXP`
  - Get the `names` attribute.
- `set_names(self: &Self, names: SEXP)`
  - Set the `names` attribute.
- `get_class(self: &Self) -> SEXP`
  - Get the `class` attribute.
- `set_class(self: &Self, class: SEXP)`
  - Set the `class` attribute.
- `get_dim(self: &Self) -> SEXP`
  - Get the `dim` attribute.
- `set_dim(self: &Self, dim: SEXP)`
  - Set the `dim` attribute.
- `get_dimnames(self: &Self) -> SEXP`
  - Get the `dimnames` attribute.
- `set_dimnames(self: &Self, dimnames: SEXP)`
  - Set the `dimnames` attribute.
- `get_levels(self: &Self) -> SEXP`
  - Get the `levels` attribute (factors).
- `set_levels(self: &Self, levels: SEXP)`
  - Set the `levels` attribute (factors).
- `get_row_names(self: &Self) -> SEXP`
  - Get the `row.names` attribute.
- `set_row_names(self: &Self, row_names: SEXP)`
  - Set the `row.names` attribute.
- `inherits_class(self: &Self, class: &std::ffi::CStr) -> bool`
  - Check if this SEXP inherits from a class.
- `string_elt(self: &Self, i: isize) -> SEXP`
  - Get the i-th CHARSXP element from a STRSXP.
- `string_elt_str(self: &Self, i: isize) -> Option<&str>`
  - Get the i-th string element as `Option<&str>`.
- `set_string_elt(self: &Self, i: isize, charsxp: SEXP)`
  - Set the i-th CHARSXP element of a STRSXP.
- `is_na_string(self: &Self) -> bool`
  - Check if this CHARSXP is `NA_character_`.
- `vector_elt(self: &Self, i: isize) -> SEXP`
  - Get the i-th element of a VECSXP (generic vector / list).
- `set_vector_elt(self: &Self, i: isize, val: SEXP)`
  - Set the i-th element of a VECSXP.
- `integer_elt(self: &Self, i: isize) -> i32`
  - Get the i-th integer element.
- `real_elt(self: &Self, i: isize) -> f64`
  - Get the i-th real element.
- `logical_elt(self: &Self, i: isize) -> i32`
  - Get the i-th logical element (raw i32: 0/1/NA_LOGICAL).
- `complex_elt(self: &Self, i: isize) -> Rcomplex`
  - Get the i-th complex element.
- `raw_elt(self: &Self, i: isize) -> u8`
  - Get the i-th raw element.
- `set_integer_elt(self: &Self, i: isize, v: i32)`
  - Set the i-th integer element.
- `set_real_elt(self: &Self, i: isize, v: f64)`
  - Set the i-th real element.
- `set_logical_elt(self: &Self, i: isize, v: i32)`
  - Set the i-th logical element (raw i32: 0/1/NA_LOGICAL).
- `set_complex_elt(self: &Self, i: isize, v: Rcomplex)`
  - Set the i-th complex element.
- `set_raw_elt(self: &Self, i: isize, v: u8)`
  - Set the i-th raw element.
- `printname(self: &Self) -> SEXP`
  - Get the print name (CHARSXP) of a symbol (SYMSXP).
- `r_char(self: &Self) -> *const ::std::os::raw::c_char`
  - Get the C string pointer from a CHARSXP.
- `r_char_str(self: &Self) -> Option<&str>`
  - Get a `&str` from a CHARSXP. Returns `None` for `NA_character_`.
- `resize(self: &Self, newlen: R_xlen_t) -> SEXP`
  - Resize a vector to a new length, returning a (possibly new) SEXP.
- `duplicate(self: &Self) -> SEXP`
  - Deep-copy this SEXP. Equivalent to R's `Rf_duplicate(x)`.
- `shallow_duplicate(self: &Self) -> SEXP`
  - Shallow-copy this SEXP. Equivalent to R's `Rf_shallow_duplicate(x)`.
- `unsafe string_elt_unchecked(self: &Self, i: isize) -> SEXP`
  - Get the i-th CHARSXP from a STRSXP. No thread check.
- `unsafe set_string_elt_unchecked(self: &Self, i: isize, charsxp: SEXP)`
  - Set the i-th CHARSXP of a STRSXP. No thread check.
- `unsafe vector_elt_unchecked(self: &Self, i: isize) -> SEXP`
  - Get the i-th element of a VECSXP. No thread check.
- `unsafe set_vector_elt_unchecked(self: &Self, i: isize, val: SEXP)`
  - Set the i-th element of a VECSXP. No thread check.
- `unsafe get_attr_unchecked(self: &Self, name: SEXP) -> SEXP`
  - Get an attribute by symbol. No thread check.
- `unsafe set_attr_unchecked(self: &Self, name: SEXP, val: SEXP)`
  - Set an attribute by symbol. No thread check.
- `unsafe r_char_unchecked(self: &Self) -> *const ::std::os::raw::c_char`
  - Get C string pointer from a CHARSXP. No thread check.

**Provided methods:**

- `get_attr_opt(self: &Self, name: SEXP) -> Option<SEXP>`
  - Get an attribute by symbol, returning `None` for `R_NilValue`.

### `sexp_types::RNativeType`

Marker trait for types that correspond to R's native vector element types.

This enables blanket implementations for `TryFromSexp` and safe conversions.

**Required methods:**

- `unsafe dataptr_mut(sexp: SEXP) -> *mut Self`
  - Get mutable pointer to vector data.
- `elt(sexp: SEXP, i: isize) -> Self`
  - Read the i-th element via the appropriate `*_ELT` accessor.

**Associated items:**

- associated const `SEXP_TYPE: SEXPTYPE`
- associated const `R_NA: Self`

### `trait_abi::TraitView`

Trait for view types that can be created from SEXP via trait dispatch.

This trait is implemented by the macro-generated `<Trait>View` structs.
It provides a common interface for:
- Querying whether an object implements a trait
- Creating a view from an SEXP

#### Generated by `#[miniextendr]` on traits

When you write:
```ignore
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
}
```

The macro generates `CounterView` that implements `TraitView`:
```ignore
impl TraitView for CounterView {
    const TAG: mx_tag = TAG_COUNTER;

    unsafe fn from_raw_parts(data: *mut c_void, vtable: *const c_void) -> Self {
        Self {
            data,
            vtable: vtable.cast::<CounterVTable>(),
        }
    }
}
```

#### Usage

```ignore
// Try to get a Counter view from an R object
let view = CounterView::try_from_sexp(obj)?;

// Call methods through the view
view.increment();
let val = view.value();
```

#### Safety

The `from_raw_parts` method is unsafe because:
- `data` must be a valid pointer to the concrete object
- `vtable` must be a valid pointer to the trait's vtable
- The pointers must remain valid for the lifetime of the view

**Required methods:**

- `unsafe from_raw_parts(data: *mut c_void, vtable: *const c_void) -> Self`
  - Create a view from raw data and vtable pointers.

**Provided methods:**

- `unsafe try_from_sexp(sexp: SEXP) -> Option<Self>`
  - Try to create a view from an R SEXP.
- `unsafe try_from_sexp_or_error(sexp: SEXP, trait_name: &str) -> Result<Self, String>`
  - Try to create a view, returning an error message on failure.

**Associated items:**

- associated const `TAG: mx_tag`

### `vctrs::IntoVctrs`

Trait for converting Rust types into vctrs-compatible R objects.

This trait provides the `into_vctrs()` method which converts a Rust
value into an R SEXP with proper vctrs class structure.

#### Implementation

Types implementing this trait should:
1. Convert their data to the appropriate R SEXP type
2. Apply the vctrs class structure using [`new_vctr`], [`new_rcrd`], or [`new_list_of`]

#### Example

```ignore
struct Percent(Vec<f64>);

impl VctrsClass for Percent {
    const CLASS_NAME: &'static str = "vctrs_percent";
    const KIND: VctrsKind = VctrsKind::Vctr;
    const BASE_TYPE: Option<SEXPTYPE> = Some(SEXPTYPE::REALSXP);
}

impl IntoVctrs for Percent {
    fn into_vctrs(self) -> Result<SEXP, VctrsBuildError> {
        use miniextendr_api::IntoR;
        let data = self.0.into_r();
        new_vctr(
            data,
            &[Self::CLASS_NAME],
            &self.attrs(),
            Some(Self::INHERIT_BASE_TYPE),
        )
    }
}
```

**Required methods:**

- `into_vctrs(self: Self) -> Result<SEXP, VctrsBuildError>`
  - Convert this value into a vctrs-compatible R object.

### `vctrs::VctrsClass`

Trait for types that can describe their vctrs class metadata.

Implement this trait to define how a Rust type should be represented
as a vctrs-compatible R object.

#### Example

```ignore
struct Percent(Vec<f64>);

impl VctrsClass for Percent {
    const CLASS_NAME: &'static str = "vctrs_percent";
    const KIND: VctrsKind = VctrsKind::Vctr;
    const BASE_TYPE: Option<SEXPTYPE> = Some(SEXPTYPE::REALSXP);
    const INHERIT_BASE_TYPE: bool = false;
}
```

**Provided methods:**

- `additional_classes() -> &'static [&'static str]`
  - Additional class names to include (after the primary class).
- `attrs(self: &Self) -> Vec<(&'static str, SEXP)>`
  - Additional attributes to set on the object.

**Associated items:**

- associated const `CLASS_NAME: &'static str`
- associated const `KIND: VctrsKind`
- associated const `BASE_TYPE: Option<SEXPTYPE>`
- associated const `INHERIT_BASE_TYPE: bool`
- associated const `ABBR: Option<&'static str>`

### `vctrs::VctrsListOf`

Marker trait for vctrs list_of types.

list_of types are lists where all elements are expected to share
a common prototype (element type).

#### Example

```ignore
/// A list of integer vectors
struct IntVecList(Vec<Vec<i32>>);

impl VctrsClass for IntVecList {
    const CLASS_NAME: &'static str = "vctrs_int_list";
    const KIND: VctrsKind = VctrsKind::ListOf;
}

impl VctrsListOf for IntVecList {
    fn ptype_expr() -> &'static str {
        "integer()"
    }
}
```

**Required methods:**

- `ptype_expr() -> &'static str`
  - An R expression that evaluates to the prototype.

**Provided methods:**

- `fixed_size() -> Option<i32>`
  - Optional fixed size for list elements.

### `vctrs::VctrsRecord`

Marker trait for vctrs record types.

Record types are vctrs classes backed by named lists where all fields
have equal length. Each "element" of the record is a row across all fields.

#### Example

```ignore
/// A rational number represented as numerator/denominator
struct Rational {
    n: Vec<i32>,  // numerators
    d: Vec<i32>,  // denominators
}

impl VctrsClass for Rational {
    const CLASS_NAME: &'static str = "vctrs_rational";
    const KIND: VctrsKind = VctrsKind::Rcrd;
}

impl VctrsRecord for Rational {
    fn field_names() -> &'static [&'static str] {
        &["n", "d"]
    }
}
```

**Required methods:**

- `field_names() -> &'static [&'static str]`
  - The names of the record fields.

---

## Functions

### `abi::mx_tag_from_path`

```rust
const mx_tag_from_path(path: &str) -> mx_tag
```

Create a new type tag from a string path.

This is a helper for generating deterministic tags from type/trait paths.
Uses FNV-1a hash to produce a 128-bit tag (two independent 64-bit hashes).

#### Arguments

* `path` - Fully-qualified path like `"mypackage::MyType"` or `"mypackage::Foo"`

#### Returns

A deterministic [`mx_tag`] for the given path.

#### Example

```
use miniextendr_api::abi::mx_tag_from_path;

const TAG_FOO: miniextendr_api::abi::mx_tag = mx_tag_from_path("mypackage::Foo");
const TAG_BAR: miniextendr_api::abi::mx_tag = mx_tag_from_path("mypackage::Bar");

// Same path produces same tag
assert_eq!(TAG_FOO, mx_tag_from_path("mypackage::Foo"));

// Different paths produce different tags
assert_ne!(TAG_FOO, TAG_BAR);
```

#### Note

This function is `const` to enable compile-time tag generation.
The hash is deterministic across compilations.

### `adapter_traits::__rclone_build_vtable`

```rust
const __rclone_build_vtable<__ImplT>() -> RCloneVTable
```

Build a vtable for a concrete type implementing `RClone`.
Generated from source location line 368, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__rcopy_build_vtable`

```rust
const __rcopy_build_vtable<__ImplT>() -> RCopyVTable
```

Build a vtable for a concrete type implementing `RCopy`.
Generated from source location line 454, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__rdebug_build_vtable`

```rust
const __rdebug_build_vtable<__ImplT>() -> RDebugVTable
```

Build a vtable for a concrete type implementing `RDebug`.
Generated from source location line 55, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__rdefault_build_vtable`

```rust
const __rdefault_build_vtable<__ImplT>() -> RDefaultVTable
```

Build a vtable for a concrete type implementing `RDefault`.
Generated from source location line 407, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__rdisplay_build_vtable`

```rust
const __rdisplay_build_vtable<__ImplT>() -> RDisplayVTable
```

Build a vtable for a concrete type implementing `RDisplay`.
Generated from source location line 97, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__rerror_build_vtable`

```rust
const __rerror_build_vtable<__ImplT>() -> RErrorVTable
```

Build a vtable for a concrete type implementing `RError`.
Generated from source location line 264, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__rfromstr_build_vtable`

```rust
const __rfromstr_build_vtable<__ImplT>() -> RFromStrVTable
```

Build a vtable for a concrete type implementing `RFromStr`.
Generated from source location line 328, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__rhash_build_vtable`

```rust
const __rhash_build_vtable<__ImplT>() -> RHashVTable
```

Build a vtable for a concrete type implementing `RHash`.
Generated from source location line 132, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__riterator_build_vtable`

```rust
const __riterator_build_vtable<__ImplT>() -> RIteratorVTable
```

Build a vtable for a concrete type implementing `RIterator`.
Generated from source location line 540, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__rord_build_vtable`

```rust
const __rord_build_vtable<__ImplT>() -> ROrdVTable
```

Build a vtable for a concrete type implementing `ROrd`.
Generated from source location line 164, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::__rpartialord_build_vtable`

```rust
const __rpartialord_build_vtable<__ImplT>() -> RPartialOrdVTable
```

Build a vtable for a concrete type implementing `RPartialOrd`.
Generated from source location line 204, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `altrep::assert_altrep_class_uniqueness`

```rust
assert_altrep_class_uniqueness()
```

Assert that all registered ALTREP class names are unique.

Must be called after all ALTREP registrations (builtin, arrow, user-defined)
have completed in `package_init()`. Panics with a clear message if any
duplicate class name is found.

### `altrep::make_class_by_base`

```rust
unsafe make_class_by_base(class_name: *const i8, pkg_name: *const i8, base: RBase) -> crate::sys::altrep::R_altrep_class_t
```

Create an ALTREP class handle based on the runtime base type.

Validates the returned handle and panics if registration fails.

#### Safety
Must be called during R initialization (after `set_altrep_dll_info`).

### `altrep::validate_altrep_class`

```rust
validate_altrep_class(cls: crate::sys::altrep::R_altrep_class_t, class_name: &std::ffi::CStr, base: RBase) -> crate::sys::altrep::R_altrep_class_t
```

Validate that an ALTREP class handle was successfully created.

Panics with a descriptive message if the class handle is null, indicating
that `R_make_alt*_class()` failed during registration.

Also records the class name for later duplicate detection via
[`assert_altrep_class_uniqueness`].

#### Arguments
* `cls` - The class handle returned by `R_make_alt*_class()`
* `class_name` - The name of the ALTREP class (for diagnostics)
* `base` - The base R type (for diagnostics)

### `altrep_bridge::install_base`

```rust
unsafe install_base<T>(cls: crate::sys::altrep::R_altrep_class_t)
```

Install base ALTREP methods (always installs length, conditionally installs optional).
#### Safety
Must be called during R initialization with a valid ALTREP class handle.

### `altrep_bridge::install_cplx`

```rust
unsafe install_cplx<T>(cls: R_altrep_class_t)
```

Install complex-specific methods.
#### Safety
Must be called during R initialization with a valid ALTREP class handle.

### `altrep_bridge::install_int`

```rust
unsafe install_int<T>(cls: R_altrep_class_t)
```

Install integer-specific methods.
#### Safety
Must be called during R initialization with a valid ALTREP class handle.

### `altrep_bridge::install_lgl`

```rust
unsafe install_lgl<T>(cls: R_altrep_class_t)
```

Install logical-specific methods.
#### Safety
Must be called during R initialization with a valid ALTREP class handle.

### `altrep_bridge::install_list`

```rust
unsafe install_list<T>(cls: R_altrep_class_t)
```

Install list-specific methods.
#### Safety
Must be called during R initialization with a valid ALTREP class handle.
Note: Elt is always installed for ALTLIST (required).

### `altrep_bridge::install_raw`

```rust
unsafe install_raw<T>(cls: R_altrep_class_t)
```

Install raw-specific methods.
#### Safety
Must be called during R initialization with a valid ALTREP class handle.

### `altrep_bridge::install_real`

```rust
unsafe install_real<T>(cls: R_altrep_class_t)
```

Install real-specific methods.
#### Safety
Must be called during R initialization with a valid ALTREP class handle.

### `altrep_bridge::install_str`

```rust
unsafe install_str<T>(cls: R_altrep_class_t)
```

Install string-specific methods.
#### Safety
Must be called during R initialization with a valid ALTREP class handle.
Note: Elt is always installed for ALTSTRING (required).

### `altrep_bridge::install_vec`

```rust
unsafe install_vec<T>(cls: crate::sys::altrep::R_altrep_class_t)
```

Install vector-level methods.
#### Safety
Must be called during R initialization with a valid ALTREP class handle.

### `altrep_bridge::t_coerce`

```rust
unsafe t_coerce<T>(x: crate::SEXP, to_type: crate::SEXPTYPE) -> crate::SEXP
```

Trampoline for Coerce method.
#### Safety
`x` must be a valid SEXP for the ALTREP class backed by `T`.

### `altrep_bridge::t_cplx_elt`

```rust
unsafe t_cplx_elt<T>(x: crate::SEXP, i: crate::R_xlen_t) -> crate::Rcomplex
```

Trampoline for complex Elt method.
#### Safety
`x` must be a valid ALTREP CPLXSXP and `i` within bounds.

### `altrep_bridge::t_cplx_get_region`

```rust
unsafe t_cplx_get_region<T>(x: crate::SEXP, i: crate::R_xlen_t, n: crate::R_xlen_t, out: *mut crate::Rcomplex) -> crate::R_xlen_t
```

Trampoline for complex Get_region method.
#### Safety
`x` must be a valid ALTREP CPLXSXP and `out` a valid buffer of at least `n` elements.

### `altrep_bridge::t_dataptr`

```rust
unsafe t_dataptr<T>(x: crate::SEXP, w: crate::Rboolean) -> *mut core::ffi::c_void
```

Trampoline for Dataptr method.
#### Safety
`x` must be a valid SEXP for the ALTREP class backed by `T`.

### `altrep_bridge::t_dataptr_or_null`

```rust
unsafe t_dataptr_or_null<T>(x: crate::SEXP) -> *const core::ffi::c_void
```

Trampoline for Dataptr_or_null method.
#### Safety
`x` must be a valid SEXP for the ALTREP class backed by `T`.

### `altrep_bridge::t_duplicate`

```rust
unsafe t_duplicate<T>(x: crate::SEXP, deep: crate::Rboolean) -> crate::SEXP
```

Trampoline for Duplicate method.
#### Safety
`x` must be a valid SEXP for the ALTREP class backed by `T`.

### `altrep_bridge::t_duplicate_ex`

```rust
unsafe t_duplicate_ex<T>(x: crate::SEXP, deep: crate::Rboolean) -> crate::SEXP
```

Trampoline for DuplicateEX method (extended duplication).
#### Safety
`x` must be a valid SEXP for the ALTREP class backed by `T`.

### `altrep_bridge::t_extract_subset`

```rust
unsafe t_extract_subset<T>(x: crate::SEXP, indx: crate::SEXP, call: crate::SEXP) -> crate::SEXP
```

Trampoline for Extract_subset method.
#### Safety
`x`, `indx`, and `call` must be valid SEXPs.

### `altrep_bridge::t_inspect`

```rust
unsafe t_inspect<T>(x: crate::SEXP, pre: i32, deep: i32, pvec: i32, inspect_subtree: Option<{'function_pointer': {'sig': {'inputs': [['_', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['_', {'primitive': 'i32'}], ['_', {'primitive': 'i32'}], ['_', {'primitive': 'i32'}]], 'output': None, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>) -> crate::Rboolean
```

Trampoline for Inspect method.
#### Safety
`x` must be a valid SEXP for the ALTREP class backed by `T`.

### `altrep_bridge::t_int_elt`

```rust
unsafe t_int_elt<T>(x: crate::SEXP, i: crate::R_xlen_t) -> i32
```

Trampoline for integer Elt method.
#### Safety
`x` must be a valid ALTREP INTSXP and `i` within bounds.

### `altrep_bridge::t_int_get_region`

```rust
unsafe t_int_get_region<T>(x: crate::SEXP, i: crate::R_xlen_t, n: crate::R_xlen_t, out: *mut i32) -> crate::R_xlen_t
```

Trampoline for integer Get_region method.
#### Safety
`x` must be a valid ALTREP INTSXP and `out` a valid buffer of at least `n` elements.

### `altrep_bridge::t_int_is_sorted`

```rust
unsafe t_int_is_sorted<T>(x: crate::SEXP) -> i32
```

Trampoline for integer Is_sorted method.
#### Safety
`x` must be a valid ALTREP INTSXP.

### `altrep_bridge::t_int_max`

```rust
unsafe t_int_max<T>(x: crate::SEXP, narm: crate::Rboolean) -> crate::SEXP
```

Trampoline for integer Max method.
#### Safety
`x` must be a valid ALTREP INTSXP.

### `altrep_bridge::t_int_min`

```rust
unsafe t_int_min<T>(x: crate::SEXP, narm: crate::Rboolean) -> crate::SEXP
```

Trampoline for integer Min method.
#### Safety
`x` must be a valid ALTREP INTSXP.

### `altrep_bridge::t_int_no_na`

```rust
unsafe t_int_no_na<T>(x: crate::SEXP) -> i32
```

Trampoline for integer No_NA method.
#### Safety
`x` must be a valid ALTREP INTSXP.

### `altrep_bridge::t_int_sum`

```rust
unsafe t_int_sum<T>(x: crate::SEXP, narm: crate::Rboolean) -> crate::SEXP
```

Trampoline for integer Sum method.
#### Safety
`x` must be a valid ALTREP INTSXP.

### `altrep_bridge::t_length`

```rust
unsafe t_length<T>(x: crate::SEXP) -> crate::R_xlen_t
```

Trampoline for Length method.
#### Safety
`x` must be a valid SEXP for the ALTREP class backed by `T`.

### `altrep_bridge::t_lgl_elt`

```rust
unsafe t_lgl_elt<T>(x: crate::SEXP, i: crate::R_xlen_t) -> i32
```

Trampoline for logical Elt method.
#### Safety
`x` must be a valid ALTREP LGLSXP and `i` within bounds.

### `altrep_bridge::t_lgl_get_region`

```rust
unsafe t_lgl_get_region<T>(x: crate::SEXP, i: crate::R_xlen_t, n: crate::R_xlen_t, out: *mut i32) -> crate::R_xlen_t
```

Trampoline for logical Get_region method.
#### Safety
`x` must be a valid ALTREP LGLSXP and `out` a valid buffer of at least `n` elements.

### `altrep_bridge::t_lgl_is_sorted`

```rust
unsafe t_lgl_is_sorted<T>(x: crate::SEXP) -> i32
```

Trampoline for logical Is_sorted method.
#### Safety
`x` must be a valid ALTREP LGLSXP.

### `altrep_bridge::t_lgl_no_na`

```rust
unsafe t_lgl_no_na<T>(x: crate::SEXP) -> i32
```

Trampoline for logical No_NA method.
#### Safety
`x` must be a valid ALTREP LGLSXP.

### `altrep_bridge::t_lgl_sum`

```rust
unsafe t_lgl_sum<T>(x: crate::SEXP, narm: crate::Rboolean) -> crate::SEXP
```

Trampoline for logical Sum method.
#### Safety
`x` must be a valid ALTREP LGLSXP.

### `altrep_bridge::t_list_elt`

```rust
unsafe t_list_elt<T>(x: crate::SEXP, i: crate::R_xlen_t) -> crate::SEXP
```

Trampoline for list Elt method (REQUIRED for ALTLIST).
#### Safety
`x` must be a valid ALTREP VECSXP and `i` within bounds.

### `altrep_bridge::t_list_set_elt`

```rust
unsafe t_list_set_elt<T>(x: crate::SEXP, i: crate::R_xlen_t, v: crate::SEXP)
```

Trampoline for list Set_elt method.
#### Safety
`x` must be a valid ALTREP VECSXP and `v` a valid SEXP.

### `altrep_bridge::t_raw_elt`

```rust
unsafe t_raw_elt<T>(x: crate::SEXP, i: crate::R_xlen_t) -> crate::Rbyte
```

Trampoline for raw Elt method.
#### Safety
`x` must be a valid ALTREP RAWSXP and `i` within bounds.

### `altrep_bridge::t_raw_get_region`

```rust
unsafe t_raw_get_region<T>(x: crate::SEXP, i: crate::R_xlen_t, n: crate::R_xlen_t, out: *mut crate::Rbyte) -> crate::R_xlen_t
```

Trampoline for raw Get_region method.
#### Safety
`x` must be a valid ALTREP RAWSXP and `out` a valid buffer of at least `n` elements.

### `altrep_bridge::t_real_elt`

```rust
unsafe t_real_elt<T>(x: crate::SEXP, i: crate::R_xlen_t) -> f64
```

Trampoline for real Elt method.
#### Safety
`x` must be a valid ALTREP REALSXP and `i` within bounds.

### `altrep_bridge::t_real_get_region`

```rust
unsafe t_real_get_region<T>(x: crate::SEXP, i: crate::R_xlen_t, n: crate::R_xlen_t, out: *mut f64) -> crate::R_xlen_t
```

Trampoline for real Get_region method.
#### Safety
`x` must be a valid ALTREP REALSXP and `out` a valid buffer of at least `n` elements.

### `altrep_bridge::t_real_is_sorted`

```rust
unsafe t_real_is_sorted<T>(x: crate::SEXP) -> i32
```

Trampoline for real Is_sorted method.
#### Safety
`x` must be a valid ALTREP REALSXP.

### `altrep_bridge::t_real_max`

```rust
unsafe t_real_max<T>(x: crate::SEXP, narm: crate::Rboolean) -> crate::SEXP
```

Trampoline for real Max method.
#### Safety
`x` must be a valid ALTREP REALSXP.

### `altrep_bridge::t_real_min`

```rust
unsafe t_real_min<T>(x: crate::SEXP, narm: crate::Rboolean) -> crate::SEXP
```

Trampoline for real Min method.
#### Safety
`x` must be a valid ALTREP REALSXP.

### `altrep_bridge::t_real_no_na`

```rust
unsafe t_real_no_na<T>(x: crate::SEXP) -> i32
```

Trampoline for real No_NA method.
#### Safety
`x` must be a valid ALTREP REALSXP.

### `altrep_bridge::t_real_sum`

```rust
unsafe t_real_sum<T>(x: crate::SEXP, narm: crate::Rboolean) -> crate::SEXP
```

Trampoline for real Sum method.
#### Safety
`x` must be a valid ALTREP REALSXP.

### `altrep_bridge::t_serialized_state`

```rust
unsafe t_serialized_state<T>(x: crate::SEXP) -> crate::SEXP
```

Trampoline for Serialized_state method.
#### Safety
`x` must be a valid SEXP for the ALTREP class backed by `T`.

### `altrep_bridge::t_str_elt`

```rust
unsafe t_str_elt<T>(x: crate::SEXP, i: crate::R_xlen_t) -> crate::SEXP
```

Trampoline for string Elt method (REQUIRED for ALTSTRING).
#### Safety
`x` must be a valid ALTREP STRSXP and `i` within bounds.

### `altrep_bridge::t_str_is_sorted`

```rust
unsafe t_str_is_sorted<T>(x: crate::SEXP) -> i32
```

Trampoline for string Is_sorted method.
#### Safety
`x` must be a valid ALTREP STRSXP.

### `altrep_bridge::t_str_no_na`

```rust
unsafe t_str_no_na<T>(x: crate::SEXP) -> i32
```

Trampoline for string No_NA method.
#### Safety
`x` must be a valid ALTREP STRSXP.

### `altrep_bridge::t_str_set_elt`

```rust
unsafe t_str_set_elt<T>(x: crate::SEXP, i: crate::R_xlen_t, v: crate::SEXP)
```

Trampoline for string Set_elt method.
#### Safety
`x` must be a valid ALTREP STRSXP and `v` a valid CHARSXP.

### `altrep_bridge::t_unserialize`

```rust
unsafe t_unserialize<T>(class: crate::SEXP, state: crate::SEXP) -> crate::SEXP
```

Trampoline for Unserialize method.
#### Safety
`class` and `state` must be valid SEXPs from R.

### `altrep_bridge::t_unserialize_ex`

```rust
unsafe t_unserialize_ex<T>(class: crate::SEXP, state: crate::SEXP, attr: crate::SEXP, objf: ::std::os::raw::c_int, levs: ::std::os::raw::c_int) -> crate::SEXP
```

Trampoline for UnserializeEX method (extended unserialization with attributes).
#### Safety
`class`, `state`, and `attr` must be valid SEXPs from R.

### `altrep_data::core::materialize_altrep_data2`

```rust
unsafe materialize_altrep_data2<T>(x: crate::SEXP) -> *mut core::ffi::c_void
```

Materialize an ALTREP SEXP into a plain R vector in data2.

Called by `__impl_altvec_dataptr` when the custom `dataptr()` returns `None`.
Allocates a destination vector via `alloc_r_vector_unchecked`, fills it from
`T::elt()` (which goes through R's ALTREP Elt dispatch), stores in data2,
and returns DATAPTR of data2.

#### Safety
- `x` must be a valid ALTREP SEXP of element type `T`
- Must be called on R's main thread

### `altrep_impl::altrep_region_buf`

```rust
unsafe altrep_region_buf<T>(buf: *mut T, len: usize) -> &'static mut [T]
```

Create a mutable slice from an ALTREP `get_region` output buffer pointer.

Called by the bridge trampolines (`altrep_bridge.rs`) to convert the raw
`*mut T` buffer from R's ALTREP dispatch into a `&mut [T]` before passing
it to the trait methods.

#### Safety

- `buf` must be a valid, aligned, writable pointer to at least `len` elements of `T`.
- The caller must ensure no aliasing references to the same memory exist.
- This is guaranteed when called from R's ALTREP `Get_region` dispatch, which
  provides a freshly allocated buffer.

### `altrep_impl::checked_mkchar`

```rust
unsafe checked_mkchar(s: &str) -> crate::SEXP
```

Create a CHARSXP from a Rust string, with checked length conversion.

#### Safety

Must be called from R's main thread.

#### Panics

Panics if `s.len() > i32::MAX`.

### `altrep_sexp::ensure_materialized`

```rust
unsafe ensure_materialized(sexp: crate::SEXP) -> crate::SEXP
```

If `sexp` is ALTREP, force materialization and return the SEXP.
If not ALTREP, return as-is (no-op).

This is the main entry point for ensuring a SEXP is safe to access
from non-R threads. After materialization, the data pointer is stable
and the SEXP can be freely sent across threads.

Called automatically by `TryFromSexp for SEXP` — you only need to call
this directly in `extern "C-unwind"` functions that receive raw SEXPs.

For contiguous types (INTSXP, REALSXP, LGLSXP, RAWSXP, CPLXSXP),
calls `DATAPTR_RO` to trigger materialization. For STRSXP, iterates
`STRING_ELT` to force each element to materialize.

#### Safety

Must be called on the R main thread (materialization invokes R internals).

### `backtrace::miniextendr_panic_hook`

```rust
miniextendr_panic_hook()
```

Register the miniextendr panic hook.

If `MINIEXTENDR_BACKTRACE` is set to `true` or `1`, the default Rust
panic hook runs (full traceback printed to stderr); otherwise the hook
swallows the panic output silently so the R error (emitted by
`panic_message_to_r_error`) is what users see.

Idempotent within a DLL instance: the first call installs, subsequent
calls are no-ops. If the DLL is unloaded and loaded again, the new
instance has its own `INSTALLED` flag and installs afresh.

### `cached_class::set_posixct_tz`

```rust
set_posixct_tz(sexp: crate::SEXP, iana: &str)
```

Set class = `c("POSIXct", "POSIXt")` and tzone = `iana` on an SEXP.

Used by the jiff integration to round-trip `Zoned` timezone identity.
Falls back to `"UTC"` for zones without an IANA name (e.g., fixed-offset zones).

#### Safety

`sexp` must be a valid REALSXP. Must be called on R's main thread.

### `cached_class::set_posixct_utc`

```rust
set_posixct_utc(sexp: crate::SEXP)
```

Set class = `c("POSIXct", "POSIXt")` and tzone = `"UTC"` on an SEXP.

Uses cached class vector + tzone string — zero allocations after first call.

#### Safety

`sexp` must be a valid REALSXP. Must be called on R's main thread.

### `condition::repanic_if_rust_error`

```rust
unsafe repanic_if_rust_error(sexp: crate::SEXP)
```

Inspect a SEXP returned by a trait-ABI vtable shim and, if it is a tagged
error value, re-panic with the reconstructed [`RCondition`].

This is the "re-panic at the View boundary" step of Approach 1 from the
issue-345 plan. The caller (a generated View method wrapper) does:

```ignore
let result = { vtable_call };
::miniextendr_api::trait_abi::repanic_if_rust_error(result);
// ... convert result normally if we reach here
```

When `sexp` is a tagged error value:
- `RCondition::Error` / `RCondition::Warning` / etc. → `panic_any!(cond)`.
  The outer `with_r_unwind_protect` in the consumer's C entry point will
  catch this and produce a tagged SEXP for the consumer's R wrapper.

When `sexp` is a normal value: this is a no-op.

#### Safety

Must be called from R's main thread. `sexp` must be a valid (possibly
tagged) SEXP.

### `encoding::encoding_info`

```rust
encoding_info() -> Option<&'static REncodingInfo>
```

Return the cached encoding info (if `miniextendr_encoding_init()` has run).

### `encoding::miniextendr_assert_utf8_locale`

```rust
miniextendr_assert_utf8_locale()
```

Assert that R's locale is UTF-8.

Called once from `R_init_*` (package init). Errors if the R session
does not use UTF-8, since `charsxp_to_str` assumes all CHARSXP bytes
are valid UTF-8.

Uses `l10n_info()[["UTF-8"]]` which is public R API.

### `encoding::miniextendr_encoding_init`

```rust
miniextendr_encoding_init()
```

Initialize / snapshot R's encoding state.

Intended to be called once from `R_init_*` (package init).

### `error::r_warning`

```rust
r_warning(msg: &str)
```

Raise an R warning with the given message.

Unlike `r_stop`, this returns normally after issuing the warning.
Automatically routes to R's main thread if called from a worker thread.

### `error_value::make_rust_condition_value`

```rust
make_rust_condition_value(message: &str, kind: &str, class: Option<&str>, call: Option<crate::SEXP>) -> crate::SEXP
```

Build a tagged condition-value SEXP for transport across the Rust→R boundary.

Used for all Rust-origin failures and user-facing conditions. The R-side
switch in `condition_check_lines` reads `.val$kind` to select the condition
type and `.val$class` to prepend optional user classes before the standard
`rust_*` layering.

#### Safety

Must be called from R's main thread (standard R API constraint).
The returned SEXP is unprotected — caller must protect if needed.

#### PROTECT discipline

Every fresh allocation (msg, kind, optional class, true-marker) is protected
before the next allocation that might trigger a GC barrier. The `prot` counter
is incremented on each `Rf_protect` and balanced by `Rf_unprotect(prot)` at
exit on all branches. This pattern was established by PR #344 commit `af6b4875`
to fix a `recursive gc invocation` segfault on R-devel.

#### Arguments

* `message` - Human-readable condition message
* `kind` - Condition kind — one of the constants in [`kind`].
* `class` - Optional user-supplied class name to prepend to the layered vector
* `call` - Optional R call SEXP for error context. When `None`, uses `R_NilValue`.

### `expression::dollar_extract`

```rust
unsafe dollar_extract(target: crate::SEXP, name: &str) -> Result<crate::SEXP, String>
```

Build and evaluate `target$name` — the R `$` extraction operator.

This is a convenience wrapper that avoids hand-rolling
`Rf_install("$") + Rf_lang3(...) + R_tryEvalSilent(...)` ladders.
Equivalent to:

```ignore
RCall::new("$")
    .arg(target)
    .arg(SEXP::scalar_string_from_str(name))
    .eval_base()
```

but uses the more direct LANGSXP form internally and protects all
intermediate allocations via RAII.

#### Safety

- Must be called from the R main thread.
- `target` must be a valid SEXP (typically a list, environment, or S4
  object that supports `$` extraction).

#### Returns

- `Ok(SEXP)` with the extracted value (unprotected — caller should protect if needed).
- `Err(String)` with the R error message if `$` extraction fails or the
  evaluation errors.

### `expression::r_eval_str`

```rust
unsafe r_eval_str(code: &str, env: crate::SEXP) -> Result<crate::SEXP, String>
```

Parse a string of R source and evaluate it in `env`.

This is the runtime workhorse behind the [`r_str!`](crate::r_str) and
[`r!`](crate::r) macros. It performs the full
`R_ParseVector` → check status → `Rf_eval` ladder with correct GC
protection on every intermediate SEXP, so callers never have to hand-roll
`OwnedProtect` around the parse tree.

Only the **last** top-level expression's value is returned (matching R's
`eval(parse(text = ...))` semantics): each parsed expression is evaluated in
order so that side effects (assignments, `library()`, …) take effect, and
the value of the final one is returned. An empty / whitespace-only string
yields `R_NilValue`.

#### Safety

- Must be called from (or routed to) the R main thread. The parse and eval
  FFI calls go through the checked `#[r_ffi_checked]` variants, which
  serialize onto the R thread via `with_r_thread`, so calling from a
  worker thread is sound — but the returned SEXP must not outlive the R
  session.
- `env` must be a valid ENVSXP.

#### Returns

- `Ok(SEXP)` with the value of the last expression (**unprotected** — the
  caller should protect it if further allocations will occur before use).
- `Err(String)` if parsing fails (syntax error / incomplete input) or if
  evaluation raises an R error. The error is captured via
  `R_tryEvalSilent` + `geterrmessage()`, so it never longjmps through Rust
  frames.

#### Example

```ignore
use miniextendr_api::expression::r_eval_str;
use miniextendr_api::sys::R_GlobalEnv;

unsafe {
    let three = r_eval_str("1L + 2L", R_GlobalEnv)?;
    // three is an INTSXP holding 3
}
```

### `expression::r_eval_str_global`

```rust
unsafe r_eval_str_global(code: &str) -> Result<crate::SEXP, String>
```

Parse and evaluate a string of R source in `R_GlobalEnv`.

Convenience wrapper over [`r_eval_str`] for the common case. See that
function for safety and return semantics.

#### Safety

Same as [`r_eval_str`].

### `externalptr::altrep_helpers::altrep_data1_as`

```rust
unsafe altrep_data1_as<T>(x: crate::SEXP) -> Option<super::ExternalPtr<T>>
```

Extract the ALTREP data1 slot as a typed `ExternalPtr<T>`.

This is a convenience function for ALTREP implementations that store
their data in an `ExternalPtr` in the data1 slot.

#### Safety

- `x` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### Example

```ignore
impl Altrep for MyAltrepClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        match unsafe { altrep_data1_as::<MyData>(x) } {
            Some(ext) => ext.data.len() as R_xlen_t,
            None => 0,
        }
    }
}
```

### `externalptr::altrep_helpers::altrep_data1_as_unchecked`

```rust
unsafe altrep_data1_as_unchecked<T>(x: crate::SEXP) -> Option<super::ExternalPtr<T>>
```

Extract the ALTREP data1 slot (unchecked version).

Skips thread safety checks for performance-critical ALTREP callbacks.

#### Safety

- `x` must be a valid ALTREP SEXP
- Must be called from the R main thread (guaranteed in ALTREP callbacks)

### `externalptr::altrep_helpers::altrep_data1_mut`

```rust
unsafe altrep_data1_mut<T>(x: crate::SEXP) -> Option<&'static mut T>
```

Get a mutable reference to data in ALTREP data1 slot via `ErasedExternalPtr`.

This is useful for ALTREP methods that need to mutate the underlying data.

#### Safety

- `x` must be a valid ALTREP SEXP
- Must be called from the R main thread
- The caller must ensure no other references to the data exist

#### Example

```ignore
fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
    match unsafe { altrep_data1_mut::<MyData>(x) } {
        Some(data) => data.buffer.as_mut_ptr().cast(),
        None => core::ptr::null_mut(),
    }
}
```

### `externalptr::altrep_helpers::altrep_data1_mut_unchecked`

```rust
unsafe altrep_data1_mut_unchecked<T>(x: crate::SEXP) -> Option<&'static mut T>
```

Get a mutable reference to data in ALTREP data1 slot (unchecked version).

Skips thread safety checks for performance-critical ALTREP callbacks.

#### Safety

- `x` must be a valid ALTREP SEXP
- Must be called from the R main thread (guaranteed in ALTREP callbacks)
- The caller must ensure no other references to the data exist

### `externalptr::altrep_helpers::altrep_data2_as`

```rust
unsafe altrep_data2_as<T>(x: crate::SEXP) -> Option<super::ExternalPtr<T>>
```

Extract the ALTREP data2 slot as a typed `ExternalPtr<T>`.

Similar to `altrep_data1_as`, but for the data2 slot.

#### Safety

- `x` must be a valid ALTREP SEXP
- Must be called from the R main thread

### `externalptr::altrep_helpers::altrep_data2_as_unchecked`

```rust
unsafe altrep_data2_as_unchecked<T>(x: crate::SEXP) -> Option<super::ExternalPtr<T>>
```

Extract the ALTREP data2 slot (unchecked version).

Skips thread safety checks for performance-critical ALTREP callbacks.

#### Safety

- `x` must be a valid ALTREP SEXP
- Must be called from the R main thread (guaranteed in ALTREP callbacks)

### `factor::build_factor`

```rust
build_factor(indices: &[i32], levels: crate::SEXP) -> crate::SEXP
```

Build a factor SEXP from indices and a levels STRSXP.

### `factor::build_factor_with_levels`

```rust
build_factor_with_levels(indices: &[i32], level_names: &[&str]) -> crate::SEXP
```

Build a factor SEXP from indices and level names in a single call.

Builds the levels STRSXP via [`build_levels_sexp`] and protects it
across the [`build_factor`] allocation, so callers don't need to manage
the levels protection themselves. The returned factor SEXP is **not**
protected — caller must protect or return it.

This is the recommended path for one-shot factor construction; for
repeated calls with the same levels prefer caching via
[`build_levels_sexp_cached`] (no protection needed because the cached
SEXP is on R's precious list).

See CLAUDE.md "PROTECT discipline against R-devel GC" for why this
matters even though `build_levels_sexp` uses symbol PRINTNAMEs for the
per-element CHARSXPs — the container STRSXP itself is freshly allocated
and unprotected.

### `factor::build_levels_sexp`

```rust
build_levels_sexp(levels: &[&str]) -> crate::SEXP
```

Build a levels STRSXP using symbol PRINTNAMEs for permanent CHARSXP protection.

The returned STRSXP is NOT protected - caller must protect or preserve it.

### `factor::build_levels_sexp_cached`

```rust
build_levels_sexp_cached(levels: &[&str]) -> crate::SEXP
```

Build a levels STRSXP and preserve it permanently (for caching).

### `factor::factor_from_sexp`

```rust
factor_from_sexp<T>(sexp: crate::SEXP) -> Result<T, crate::from_r::SexpError>
```

Convert an R factor SEXP to a single enum value.

### `factor::unit_factor_option_vec_from_sexp`

```rust
unit_factor_option_vec_from_sexp<T>(sexp: crate::SEXP) -> Result<Vec<Option<T>>, crate::from_r::SexpError>
```

Convert an R factor SEXP to a `Vec<Option<T>>` using [`UnitEnumFactor`] (NA → `None`).

Used by the enum DataFrame reader to reconstruct `as_factor` columns.
Unlike `factor_option_vec_from_sexp` (which requires `RFactor + MatchArg`),
this accepts any `UnitEnumFactor` — including `#[derive(DataFrameRow)]`
unit-only enums that do not implement `RFactor`.

### `ffi_guard::guarded_ffi_call`

```rust
guarded_ffi_call<F, R>(f: F, mode: GuardMode, source: crate::panic_telemetry::PanicSource) -> R
```

Execute `f` inside an FFI guard selected by `mode`.

On panic:
- Extracts the panic message from the payload.
- Fires [`crate::panic_telemetry`] with `source`.
- For [`GuardMode::CatchUnwind`]: raises R error via `Rf_error` (diverges — never returns).
- For [`GuardMode::RUnwind`]: delegates to `with_r_unwind_protect_sourced`.

#### Parameters

- `f`: The closure to execute.
- `mode`: Which guard strategy to use.
- `source`: Attribution for telemetry if a panic occurs.

#### Note on `fallback`

`GuardMode::CatchUnwind` diverges on panic (`Rf_error` never returns), so no
fallback value is needed. If you need a fallback (e.g. connection trampolines
that must return a value on panic without calling R), use
[`guarded_ffi_call_with_fallback`] instead.

### `ffi_guard::guarded_ffi_call_with_fallback`

```rust
guarded_ffi_call_with_fallback<F, R>(f: F, fallback: R, source: crate::panic_telemetry::PanicSource) -> R
```

Execute `f` inside a `CatchUnwind` guard, returning `fallback` on panic.

Unlike [`guarded_ffi_call`] with `CatchUnwind` (which diverges via `Rf_error`),
this variant returns the `fallback` value instead of raising an R error.
This is needed for connection trampolines where panicking through R/C frames
is UB but raising an R error is also undesirable (the caller expects a return
value indicating failure).

Telemetry is fired before returning the fallback.

### `gc_protect::tls::current_count`

```rust
current_count() -> Option<i32>
```

Get the current scope's protection count.

Returns `None` if no scope is active.

### `gc_protect::tls::has_active_scope`

```rust
has_active_scope() -> bool
```

Check if there is an active TLS scope.

### `gc_protect::tls::protect`

```rust
unsafe protect(x: crate::SEXP) -> TlsRoot
```

Protect a value using the current TLS scope.

#### Panics

Panics if called outside of a [`with_protect_scope`] block.

#### Safety

- Must be called from the R main thread
- `x` must be a valid SEXP
- Must be called within a [`with_protect_scope`] block

#### Example

```ignore
tls::with_protect_scope(|| {
    let x = tls::protect(some_sexp);
    // use x...
})
```

### `gc_protect::tls::protect_raw`

```rust
unsafe protect_raw(x: crate::SEXP) -> crate::SEXP
```

Protect a value, returning the raw SEXP.

#### Panics

Panics if called outside of a [`with_protect_scope`] block.

#### Safety

Same as [`protect`].

### `gc_protect::tls::scope_depth`

```rust
scope_depth() -> usize
```

Get the nesting depth of TLS scopes.

### `gc_protect::tls::with_protect_scope`

```rust
unsafe with_protect_scope<F, R>(f: F) -> R
```

Execute a closure with a protect scope that is accessible via TLS.

#### Safety

Must be called from the R main thread.

### `growth_debug::get_count`

```rust
get_count(name: &'static str) -> u64
```

Get the current growth count for a named collection (for testing).

### `growth_debug::record_growth`

```rust
record_growth(name: &'static str)
```

Increment the growth counter for the named collection.

### `growth_debug::report_and_reset`

```rust
report_and_reset()
```

Print all growth counters to stderr and reset them.

### `growth_debug::reset`

```rust
reset()
```

Reset all growth counters.

### `init::package_init`

```rust
unsafe package_init(dll: *mut crate::sys::DllInfo, pkg_name: &std::ffi::CStr)
```

Initialize a miniextendr R package.

This performs all initialization steps in the correct order:

1. Install panic hook for better error messages
2. Record main thread ID (and optionally spawn worker thread)
3. Assert UTF-8 locale
4. Set ALTREP package name
5. Register mx_abi C-callables for cross-package trait dispatch
6. Register all `#[miniextendr]` routines and ALTREP classes
7. Lock down dynamic symbols

#### Safety

Must be called from R's main thread during `R_init_*`.
`dll` must be a valid pointer provided by R.
`pkg_name` must be a valid null-terminated C string that lives for the
duration of the R session (typically a string literal).

### `list::accumulator::collect_list`

```rust
unsafe collect_list<'a, I, T>(scope: &'a crate::gc_protect::ProtectScope, iter: I) -> crate::gc_protect::Root<'a>
```

Collect an iterator into an R list with bounded protect stack usage.

This is a convenience wrapper around [`ListAccumulator`] for iterator-based
collection. Each element is converted via [`IntoR`].

#### Safety

Must be called from the R main thread.

#### Example

```ignore
unsafe fn squares(n: usize) -> SEXP {
    let scope = ProtectScope::new();
    collect_list(&scope, (0..n).map(|i| (i * i) as i32)).get()
}
```

### `match_arg::choices_sexp`

```rust
choices_sexp<T>() -> crate::SEXP
```

Build an R character vector (STRSXP) from the choices of a `MatchArg` type.

This is called by generated choices-helper C wrappers to provide the
choice list to `base::match.arg()` in the R wrapper.

### `match_arg::escape_r_string`

```rust
escape_r_string(s: &str) -> String
```

Escape a Rust `&str` for embedding inside an R double-quoted string literal.

Handles `\`, `"`, newline, carriage return, and tab — the characters R
recognises as escape sequences inside `"..."`. Used when formatting
`MatchArg::CHOICES` into the default of a generated R wrapper formal, so
that a choice like `say "hi"` or `c:\path` cannot produce syntactically
invalid R code.

### `match_arg::match_arg_from_sexp`

```rust
match_arg_from_sexp<T>(sexp: crate::SEXP) -> Result<T, MatchArgError>
```

Extract a single string from an R SEXP and match it against a `MatchArg` type.

Used by the generated `TryFromSexp for T` implementation (single-value `match.arg`).

### `match_arg::match_arg_vec_from_sexp`

```rust
match_arg_vec_from_sexp<T>(sexp: crate::SEXP) -> Result<Vec<T>, MatchArgError>
```

Extract multiple strings from an R SEXP (STRSXP) and match each against
the choices of a `MatchArg` type.

Used by the generated C wrapper for `match_arg + several_ok` parameters
(`match.arg` with `several.ok = TRUE`).

NULL input returns all variants (matching R's `match.arg` default with `several.ok = TRUE`).

Note: factors (INTSXP) are not handled here — the R wrapper coerces factors
to character before the `.Call()` boundary.

### `match_arg::match_arg_vec_into_sexp`

```rust
match_arg_vec_into_sexp<T>(values: Vec<T>) -> crate::SEXP
```

Convert a `Vec<T: MatchArg>` to an R character vector (STRSXP).

Each element is written as its canonical choice string via [`MatchArg::to_choice`].
Empty choice strings are stored as `R_BlankString` (parity with [`choices_sexp`]).

Called by the [`MatchArg`]→[`IntoRVecElement`](crate::newtype::IntoRVecElement)
bridge below, which backs `IntoR for Vec<MyEnum>`.

### `missing::is_missing_arg`

```rust
is_missing_arg(sexp: crate::SEXP) -> bool
```

Check if a SEXP is the missing argument sentinel.

### `mx_abi::mx_abi_register`

```rust
unsafe mx_abi_register(pkg_name: &std::ffi::CStr)
```

Register the mx_* C-callables with R.

Called during package init (`R_init_*`) to make `mx_wrap`, `mx_get`,
and `mx_query` available to consumer packages via `R_GetCCallable`.

#### Safety

Must be called from R's main thread during package initialization.
`pkg_name` must be a valid null-terminated C string.

### `mx_abi::mx_get`

```rust
unsafe mx_get(sexp: crate::SEXP) -> *mut crate::abi::mx_erased
```

Extract an erased object pointer from an R external pointer.

Returns null if the SEXP is not an external pointer or doesn't carry
the miniextendr tag.

Registered as `"mx_get"` via `R_RegisterCCallable`.

#### Safety

`sexp` must be a valid SEXP. Must be called on R's main thread.

### `mx_abi::mx_query`

```rust
unsafe mx_query(sexp: crate::SEXP, tag: crate::abi::mx_tag) -> *const std::ffi::c_void
```

Query an object for an interface vtable by tag.

Returns the vtable pointer, or null if the type does not implement
the requested trait.

Registered as `"mx_query"` via `R_RegisterCCallable`.

#### Safety

`sexp` must be a valid SEXP. Must be called on R's main thread.

### `mx_abi::mx_wrap`

```rust
unsafe mx_wrap(ptr: *mut crate::abi::mx_erased) -> crate::SEXP
```

Wrap an erased object pointer in an R external pointer.

Registered as `"mx_wrap"` via `R_RegisterCCallable`.

#### Safety

`ptr` must point to a valid `mx_erased` allocated by a miniextendr constructor.
Must be called on R's main thread.

### `optionals::aho_corasick_impl::aho_builder`

```rust
aho_builder(patterns: &[String], ascii_case_insensitive: bool, match_kind: &str) -> Result<AhoCorasick, crate::from_r::SexpError>
```

Compile an Aho-Corasick automaton with custom options.

#### Arguments

* `patterns` - Patterns to search for
* `ascii_case_insensitive` - Enable case-insensitive matching (ASCII only)
* `match_kind` - Match semantics: "standard", "leftmost-first", or "leftmost-longest"

#### Errors

Returns an error if:
- Pattern vector is empty
- Unknown match_kind value
- Building the automaton fails

#### Example

```ignore
let ac = aho_builder(&["Foo".to_string()], true, "leftmost-longest")?;
```

### `optionals::aho_corasick_impl::aho_compile`

```rust
aho_compile(patterns: &[String]) -> Result<AhoCorasick, crate::from_r::SexpError>
```

Compile an Aho-Corasick automaton from patterns.

#### Errors

Returns an error if building the automaton fails.

#### Example

```ignore
let ac = aho_compile(&["foo".to_string(), "bar".to_string()])?;
```

### `optionals::aho_corasick_impl::aho_count_matches`

```rust
aho_count_matches(ac: &AhoCorasick, haystack: &str) -> usize
```

Count total number of matches.

### `optionals::aho_corasick_impl::aho_find_all`

```rust
aho_find_all(ac: &AhoCorasick, haystack: &str) -> Vec<(usize, usize, usize)>
```

Find all matches in a haystack.

Returns a vector of `(pattern_id, start, end)` tuples where:
- `pattern_id` is 0-based (convert to 1-based for R)
- `start` and `end` are byte offsets into the haystack

#### Example

```ignore
let ac = aho_compile(&["foo".to_string(), "bar".to_string()])?;
let matches = aho_find_all(&ac, "foo and bar");
// matches: [(0, 0, 3), (1, 8, 11)]
```

### `optionals::aho_corasick_impl::aho_find_all_flat`

```rust
aho_find_all_flat(ac: &AhoCorasick, haystack: &str) -> Vec<i32>
```

Find all matches and return as a flattened vector suitable for R.

Returns a flat vector: `[pid1, start1, end1, pid2, start2, end2, ...]`
where pattern IDs are **1-based** for R compatibility.

This format can be easily reshaped to a matrix in R:
```r
result <- matrix(matches, ncol = 3, byrow = TRUE)
colnames(result) <- c("pattern_id", "start", "end")
```

### `optionals::aho_corasick_impl::aho_find_first`

```rust
aho_find_first(ac: &AhoCorasick, haystack: &str) -> Option<(usize, usize, usize)>
```

Find the first (leftmost) match.

Returns `Some((pattern_id, start, end))` or `None` if no match.
Pattern ID is 0-based.

### `optionals::aho_corasick_impl::aho_is_match`

```rust
aho_is_match(ac: &AhoCorasick, haystack: &str) -> bool
```

Check if any pattern matches in the haystack.

### `optionals::aho_corasick_impl::aho_replace_all`

```rust
aho_replace_all(ac: &AhoCorasick, haystack: &str, replacement: &str) -> String
```

Replace all matches with a single replacement string.

### `optionals::aho_corasick_impl::aho_replace_all_with`

```rust
aho_replace_all_with(ac: &AhoCorasick, haystack: &str, replacements: &[String]) -> Result<String, crate::from_r::SexpError>
```

Replace matches with pattern-specific replacements.

`replacements` must have the same length as the number of patterns.

### `optionals::arrow_impl::alloc_r_backed_buffer`

```rust
unsafe alloc_r_backed_buffer<T>(len: usize) -> (arrow_buffer::Buffer, crate::SEXP)
```

Allocate an Arrow Buffer backed by a new R vector.

The returned buffer points into a freshly allocated R vector (REALSXP,
INTSXP, or RAWSXP depending on `T`). When this buffer is later used in
an Arrow array and that array is converted back to R via `IntoR`, the
SEXP pointer recovery will find the original R vector — zero-copy
round-trip for the Rust→Arrow→R direction.

Returns `(buffer, sexp)` so callers can also work with the SEXP directly.

#### Safety

Must be called on R's main thread.

#### Allocation failure

If `Rf_allocVector` cannot satisfy the request, R longjmps from inside
the allocation. `R_UnwindProtect` catches the longjmp, the framework
recognises the R-origin path via `RErrorMarker`, and `R_ContinueUnwind`
re-raises R's original `Error: vector memory exhausted (limit reached?)`
verbatim. The error message is R's, not Rust's — no tagged-SEXP synthesis
happens on this path (cf. `with_r_unwind_protect`). The two
`.expect()` calls in the body are therefore unreachable in production —
they guard against logic errors (wrong `len` encoding or unexpected null
pointer), not against OOM.

#### Example

```ignore
let (buffer, _sexp) = unsafe { alloc_r_backed_buffer::<f64>(1000) };
let values = arrow_buffer::ScalarBuffer::<f64>::from(buffer);
// Fill values via unsafe mutable access, then:
let array = Float64Array::new(values, None);
// array.into_sexp() → returns the original REALSXP (zero-copy)
```

### `optionals::arrow_impl::posixct_to_timestamp`

```rust
posixct_to_timestamp(sexp: crate::SEXP) -> Result<TimestampSecondArray, crate::from_r::SexpError>
```

Convert R POSIXct to Arrow TimestampSecondArray.

R POSIXct values are doubles (seconds since Unix epoch, possibly fractional).
Arrow TimestampSecondArray uses i64 seconds. Fractional seconds are truncated.
Timezone from R's "tzone" attribute is preserved if present.

### `optionals::bitflags_impl::flags_from_i32_strict`

```rust
flags_from_i32_strict<T>(value: i32) -> Option<T>
```

Convert an `i32` to a bitflags type (strict - unknown bits cause error).

Returns `None` if the value contains bits not defined in the flags type.

#### Example

```ignore
let flags: Option<Permissions> = flags_from_i32_strict(0b011);
```

### `optionals::bitflags_impl::flags_from_i32_truncate`

```rust
flags_from_i32_truncate<T>(value: i32) -> T
```

Convert an `i32` to a bitflags type (truncating - unknown bits are ignored).

Returns empty flags if the value cannot be converted to the Bits type.

#### Example

```ignore
let flags: Permissions = flags_from_i32_truncate(0b111111);
// Only defined bits are kept
```

### `optionals::bitflags_impl::flags_to_i32`

```rust
flags_to_i32<T>(flags: T) -> Option<i32>
```

Convert a bitflags value to `i32`.

Returns `None` if the bits value doesn't fit in `i32`.

### `optionals::bitvec_impl::bitvec_count_ones`

```rust
bitvec_count_ones(bits: &RBitVec) -> usize
```

Count the number of set bits (ones).

### `optionals::bitvec_impl::bitvec_count_zeros`

```rust
bitvec_count_zeros(bits: &RBitVec) -> usize
```

Count the number of unset bits (zeros).

### `optionals::bitvec_impl::bitvec_from_bools`

```rust
bitvec_from_bools(bools: &[bool]) -> RBitVec
```

Create a bit vector from a slice of booleans.

### `optionals::bitvec_impl::bitvec_to_bools`

```rust
bitvec_to_bools(bits: &RBitVec) -> Vec<bool>
```

Convert a bit vector to a Vec of booleans.

### `optionals::borsh_impl::borsh_from_raw`

```rust
borsh_from_raw<T>(sexp: crate::SEXP) -> Result<T, crate::from_r::SexpError>
```

Deserialize from R raw vector.

### `optionals::borsh_impl::borsh_to_raw`

```rust
borsh_to_raw<T>(value: &T) -> crate::SEXP
```

Serialize a borsh value to R raw vector.

### `optionals::jiff_impl::datetime_vec_to_rcrd`

```rust
datetime_vec_to_rcrd(dts: &[DateTime]) -> crate::SEXP
```

Convert a slice of `DateTime`s into a vctrs `jiff_datetime` rcrd SEXP.

### `optionals::jiff_impl::span_vec_to_rcrd`

```rust
span_vec_to_rcrd(spans: &[Span]) -> crate::SEXP
```

Convert a slice of `Span`s into a vctrs `jiff_span` rcrd SEXP.

### `optionals::jiff_impl::time_vec_to_rcrd`

```rust
time_vec_to_rcrd(times: &[Time]) -> crate::SEXP
```

Convert a slice of `Time`s into a vctrs `jiff_time` rcrd SEXP.

### `optionals::jiff_impl::vctrs_support::datetime_vec_to_rcrd`

```rust
datetime_vec_to_rcrd(dts: &[DateTime]) -> crate::SEXP
```

Convert a slice of `DateTime`s into a vctrs `jiff_datetime` rcrd SEXP.

Fields: `year`, `month`, `day`, `hour`, `minute`, `second`, `nanosecond`.

### `optionals::jiff_impl::vctrs_support::span_vec_to_rcrd`

```rust
span_vec_to_rcrd(spans: &[Span]) -> crate::SEXP
```

Convert a slice of `Span`s into a vctrs `jiff_span` rcrd SEXP.

Fields: `years`, `months`, `weeks`, `days`, `hours`, `minutes`, `seconds`,
`milliseconds`, `microseconds`, `nanoseconds` — all as integer (`INTSXP`).

### `optionals::jiff_impl::vctrs_support::time_vec_to_rcrd`

```rust
time_vec_to_rcrd(times: &[Time]) -> crate::SEXP
```

Convert a slice of `Time`s into a vctrs `jiff_time` rcrd SEXP.

Fields: `hour`, `minute`, `second`, `nanosecond`.

### `optionals::jiff_impl::vctrs_support::zoned_vec_to_rcrd`

```rust
zoned_vec_to_rcrd(zones: &[Zoned]) -> crate::SEXP
```

Convert a slice of `Zoned`s into a vctrs `jiff_zoned` rcrd SEXP.

Fields: `timestamp` (REALSXP, seconds since epoch), `tz` (STRSXP, IANA name).

### `optionals::jiff_impl::zoned_vec_to_rcrd`

```rust
zoned_vec_to_rcrd(zones: &[Zoned]) -> crate::SEXP
```

Convert a slice of `Zoned`s into a vctrs `jiff_zoned` rcrd SEXP.

### `optionals::log_impl::drain_log_queue`

```rust
drain_log_queue()
```

Drain all buffered cross-thread log records to R's console.

Must be called from R's main thread — silently returns otherwise.
Idempotent: if the queue is empty, this is a no-op.

If records were dropped due to queue overflow, a single `WARN`-level
message ("N log records dropped due to queue overflow") is emitted first,
and the overflow counter is reset to zero.

The FFI trampoline in `unwind_protect.rs` calls this automatically on every
FFI exit path (normal return, Rust panic, and caught R longjmp). Direct
callers are rare but supported (e.g. end-of-batch flush points).

### `optionals::log_impl::install_r_logger`

```rust
install_r_logger()
```

Install the R console logger.

Call this once during package initialization (from `package_init()`).
If a logger is already installed (by another package or the user),
this is a no-op.

Default level: `Off` (all output suppressed until the downstream package
calls `set_log_level("info")` or similar).

### `optionals::log_impl::set_log_level`

```rust
set_log_level(level: &str)
```

Set the log level filter from a string.

Valid levels: "error", "warn", "info", "debug", "trace", "off"
(case-insensitive). Invalid strings default to `"info"`.

### `optionals::ndarray_impl::from_r_array`

```rust
unsafe from_r_array<T>(sexp: crate::SEXP) -> Result<ArrayViewD<'static, T>, crate::from_r::SexpError>
```

Create an `ArrayViewD` from an R N-dimensional array without copying.

Returns a Fortran-order (column-major) view that directly references R's
array storage. This is true zero-copy access for arrays of any dimension.

Plain vectors (no dim attribute) are treated as 1D arrays.

#### Safety

- The returned view is only valid as long as the R object is protected.
- The SEXP must be of the correct type for `T`.

#### Example

```ignore
use miniextendr_api::ndarray_impl::from_r_array;

#[miniextendr]
fn nd_sum(x: SEXP) -> f64 {
    let view = unsafe { from_r_array::<f64>(x).unwrap() };
    view.sum()
}
```

### `optionals::ndarray_impl::from_r_array3`

```rust
unsafe from_r_array3<T>(sexp: crate::SEXP) -> Result<ArrayView3<'static, T>, crate::from_r::SexpError>
```

Create an `ArrayView3` from an R 3D array without copying.

Returns a Fortran-order (column-major) view that directly references R's
array storage. This is true zero-copy access.

#### Safety

- The returned view is only valid as long as the R object is protected.
- The SEXP must be of the correct type for `T`.
- The SEXP must be a 3D array (have a `dim` attribute of length 3).

#### Example

```ignore
use miniextendr_api::ndarray_impl::from_r_array3;

#[miniextendr]
fn cube_sum(x: SEXP) -> f64 {
    let view = unsafe { from_r_array3::<f64>(x).unwrap() };
    view.sum()
}
```

### `optionals::ndarray_impl::from_r_matrix`

```rust
unsafe from_r_matrix<T>(sexp: crate::SEXP) -> Result<ArrayView2<'static, T>, crate::from_r::SexpError>
```

Create an `ArrayView2` from an R matrix without copying.

Returns a Fortran-order (column-major) view that directly references R's
matrix storage. This is true zero-copy access.

#### Safety

- The returned view is only valid as long as the R object is protected.
- The SEXP must be of the correct type for `T`.
- The SEXP must be a matrix (dim attribute of length 2) or a plain vector
  (which is treated as an n×1 column matrix).

#### Example

```ignore
use miniextendr_api::ndarray_impl::from_r_matrix;

#[miniextendr]
fn matrix_sum(x: SEXP) -> f64 {
    let view = unsafe { from_r_matrix::<f64>(x).unwrap() };
    view.sum()
}
```

### `optionals::ndarray_impl::from_r_slice`

```rust
unsafe from_r_slice<T>(sexp: crate::SEXP) -> Result<ArrayView1<'static, T>, crate::from_r::SexpError>
```

Create an `ArrayView1` from an R vector without copying.

#### Safety

The returned view is only valid as long as the R object is protected.
The SEXP must be of the correct type for `T`.

#### Example

```ignore
use miniextendr_api::ndarray_impl::from_r_slice;

#[miniextendr]
fn sum_view(x: SEXP) -> f64 {
    let view = unsafe { from_r_slice::<f64>(x).unwrap() };
    view.sum()
}
```

### `optionals::num_complex_impl::from_rcomplex`

```rust
from_rcomplex(r: crate::Rcomplex) -> Complex<f64>
```

Convert R's `Rcomplex` to `Complex<f64>`.

### `optionals::num_complex_impl::is_na_complex`

```rust
is_na_complex(c: &Complex<f64>) -> bool
```

Check if a `Complex<f64>` value is NA (either part is `NA_REAL`).

### `optionals::num_complex_impl::is_na_rcomplex`

```rust
is_na_rcomplex(r: &crate::Rcomplex) -> bool
```

Check if an `Rcomplex` value is NA.

A complex is NA if either the real or imaginary part is `NA_REAL`.
We use bit comparison for reliable detection since `NA_REAL` is a specific NaN payload.

### `optionals::num_complex_impl::na_rcomplex`

```rust
na_rcomplex() -> crate::Rcomplex
```

Create an NA complex value (both parts are `NA_REAL`).

### `optionals::num_complex_impl::to_rcomplex`

```rust
to_rcomplex(c: Complex<f64>) -> crate::Rcomplex
```

Convert `Complex<f64>` to R's `Rcomplex`.

### `optionals::rayon_bridge::new_r_array`

```rust
new_r_array<T, NDIM, F>(dims: [usize; NDIM], f: F) -> crate::rarray::RArray<T, NDIM>
```

Pre-allocate an N-dimensional R array and return it as [`RArray<T, NDIM>`][crate::rarray::RArray].

This is like [`with_r_array`] but returns a typed wrapper instead of raw SEXP.

#### Example

```ignore
let array: RArray<f64, 3> = new_r_array([2, 3, 4], |slab, slab_idx| {
    for (i, slot) in slab.iter_mut().enumerate() {
        *slot = (slab_idx * 100 + i) as f64;
    }
});
```

### `optionals::rayon_bridge::new_r_matrix`

```rust
new_r_matrix<T, F>(nrow: usize, ncol: usize, f: F) -> crate::rarray::RMatrix<T>
```

Pre-allocate an R matrix and return it as [`RMatrix<T>`][crate::rarray::RMatrix].

This is like [`with_r_matrix`] but returns a typed wrapper instead of raw SEXP.

#### Example

```ignore
let matrix: RMatrix<f64> = new_r_matrix(3, 4, |col, col_idx| {
    for (row, slot) in col.iter_mut().enumerate() {
        *slot = (row + col_idx * 10) as f64;
    }
});
```

### `optionals::rayon_bridge::par_collect_sexp`

```rust
par_collect_sexp<T, I>(iter: I) -> crate::SEXP
```

Collect a non-indexed parallel iterator into an R vector (SEXP).

For iterators that lose their index (`.filter()`, `.flat_map()`, `.par_bridge()`),
this function collects to an intermediate `Vec<T>` then converts to R.

For indexed iterators (the common case), prefer [`ParCollectR::collect_r()`]
which writes directly into R memory with zero intermediate allocation.

#### Example

```ignore
use miniextendr_api::rayon_bridge;

// filter() loses index — must use par_collect_sexp
let sexp = rayon_bridge::par_collect_sexp(
    data.par_iter().filter(|&&x| x > 0.0).copied()
);
```

### `optionals::rayon_bridge::par_map`

```rust
par_map<T, U, F>(input: &[T], f: F) -> crate::SEXP
```

Transform an input slice into a new R vector, element-wise in parallel.

Allocates an output R vector of the same length as `input`, then fills it
in parallel using chunk-based dispatch. Each element `output[i] = f(&input[i])`.

This is the parallel equivalent of `input.iter().map(f).collect::<Vec<U>>()`,
but writes directly into R memory (zero intermediate allocation).

#### Example

```ignore
// Parallel sqrt of an R numeric vector
fn parallel_sqrt(x: &[f64]) -> SEXP {
    par_map(x, |&v| v.sqrt())
}

// Type conversion: i32 → f64
fn int_to_double(x: &[i32]) -> SEXP {
    par_map(x, |&v| v as f64)
}
```

### `optionals::rayon_bridge::par_map2`

```rust
par_map2<T, U, V, F>(a: &[T], b: &[U], f: F) -> crate::SEXP
```

Two-input element-wise parallel map into a new R vector.

Like [`par_map`] but zips two input slices: `output[i] = f(&a[i], &b[i])`.

#### Panics

Panics if `a.len() != b.len()`.

#### Example

```ignore
// Parallel element-wise addition
fn vec_add(a: &[f64], b: &[f64]) -> SEXP {
    par_map2(a, b, |&x, &y| x + y)
}

// Weighted transform
fn weighted_sqrt(values: &[f64], weights: &[f64]) -> SEXP {
    par_map2(values, weights, |&v, &w| v.sqrt() * w)
}
```

### `optionals::rayon_bridge::par_map3`

```rust
par_map3<A, B, C, V, F>(a: &[A], b: &[B], c: &[C], f: F) -> crate::SEXP
```

Three-input element-wise parallel map into a new R vector.

Like [`par_map2`] but zips three input slices: `output[i] = f(&a[i], &b[i], &c[i])`.

#### Panics

Panics if the three slices have different lengths.

#### Example

```ignore
// Fused multiply-add: a * b + c
fn fma(a: &[f64], b: &[f64], c: &[f64]) -> SEXP {
    par_map3(a, b, c, |&x, &y, &z| x * y + z)
}
```

### `optionals::rayon_bridge::perf::in_rayon_thread`

```rust
in_rayon_thread() -> bool
```

Check if in a Rayon thread.

### `optionals::rayon_bridge::perf::num_threads`

```rust
num_threads() -> usize
```

Get number of threads in Rayon pool.

### `optionals::rayon_bridge::perf::thread_index`

```rust
thread_index() -> Option<usize>
```

Get thread index.

### `optionals::rayon_bridge::reduce::max`

```rust
max(slice: &[f64]) -> crate::SEXP
```

Parallel maximum.

### `optionals::rayon_bridge::reduce::mean`

```rust
mean(slice: &[f64]) -> crate::SEXP
```

Parallel mean.

### `optionals::rayon_bridge::reduce::min`

```rust
min(slice: &[f64]) -> crate::SEXP
```

Parallel minimum.

### `optionals::rayon_bridge::reduce::sum`

```rust
sum(slice: &[f64]) -> crate::SEXP
```

Parallel sum → R scalar (f64).

### `optionals::rayon_bridge::reduce::sum_int`

```rust
sum_int(slice: &[i32]) -> crate::SEXP
```

Parallel sum → R scalar (i32).

### `optionals::rayon_bridge::with_r_array`

```rust
with_r_array<T, NDIM, F>(dims: [usize; NDIM], f: F) -> crate::SEXP
```

Pre-allocate an N-dimensional R array, fill slabs in parallel, return the SEXP.

The closure `f(slab, slab_idx)` is called in parallel for each slab:
- `slab`: mutable slice of one slab (product of all dims except the last)
- `slab_idx`: slab index along the last dimension (0-based)

For dims `[d0, d1, ..., dN]`, each slab has `d0 * d1 * ... * d(N-1)` elements
and there are `dN` slabs total.

#### Example

```ignore
// Create a 2x3x4 array, 4 slabs of 6 elements each
with_r_array([2, 3, 4], |slab: &mut [f64], slab_idx: usize| {
    for (i, val) in slab.iter_mut().enumerate() {
        *val = (slab_idx * 100 + i) as f64;
    }
});
```

#### Protection

The array is PROTECTED during parallel writes. After return, UNPROTECT
is called and the SEXP becomes the caller's responsibility.

### `optionals::rayon_bridge::with_r_matrix`

```rust
with_r_matrix<T, F>(nrow: usize, ncol: usize, f: F) -> crate::SEXP
```

Pre-allocate an R matrix, fill columns in parallel, return the SEXP.

The closure `f(column, col_idx)` is called in parallel for each column:
- `column`: mutable slice of length `nrow` (one column)
- `col_idx`: column index (0-based)

R matrices are column-major, so each column is a contiguous slice.

#### Example

```ignore
// Create a 100x50 matrix, fill by column
with_r_matrix(100, 50, |col: &mut [f64], col_idx: usize| {
    for (row, slot) in col.iter_mut().enumerate() {
        *slot = (row + col_idx * 1000) as f64;
    }
});
```

#### Protection

The matrix is PROTECTED during parallel writes. After return, UNPROTECT
is called and the SEXP becomes the caller's responsibility.

### `optionals::rayon_bridge::with_r_vec`

```rust
with_r_vec<T, F>(len: usize, f: F) -> crate::SEXP
```

Pre-allocate an R vector, fill chunks in parallel, return the SEXP.

The closure `f(chunk, offset)` is called in parallel for each chunk:
- `chunk`: mutable slice to fill
- `offset`: starting index of this chunk in the overall vector

The framework handles splitting and parallel dispatch via Rayon's
`par_chunks_mut`. **Chunk boundaries are deterministic** for a given
vector length and thread count, making parallel RNG reproducible.

#### Type Mapping

`T` must implement [`RNativeType`]:
- `f64` → `REALSXP`
- `i32` → `INTSXP`
- `RLogical` → `LGLSXP`
- `u8` → `RAWSXP`
- `Rcomplex` → `CPLXSXP`

#### Example

```ignore
// Fill with sqrt(index) — framework handles parallelism
with_r_vec(1000, |chunk: &mut [f64], offset: usize| {
    for (i, slot) in chunk.iter_mut().enumerate() {
        *slot = ((offset + i) as f64).sqrt();
    }
});

// Reproducible parallel RNG (seed from offset)
with_r_vec(1000, |chunk: &mut [f64], offset| {
    let mut rng = ChaChaRng::seed_from_u64(42 + offset as u64);
    for slot in chunk { *slot = rng.gen(); }
});
```

#### Protection

The vector is PROTECTED during parallel writes. After return, UNPROTECT
is called and the SEXP becomes the caller's responsibility.

### `optionals::rayon_bridge::with_r_vec_map`

```rust
with_r_vec_map<T, F>(len: usize, f: F) -> crate::SEXP
```

Pre-allocate an R vector, fill element-wise in parallel, return the SEXP.

This is syntactic sugar over [`with_r_vec`] for the common case where each
element depends only on its index.

#### Example

```ignore
// Fill with sqrt(index)
with_r_vec_map(1000, |i: usize| (i as f64).sqrt());
```

### `optionals::regex_impl::try_compile`

```rust
try_compile(pattern: &str) -> Result<Regex, regex::Error>
```

Compile a regex pattern and return an error-typed Result.

This is a convenience function for use in `#[miniextendr]` functions
that want to handle regex compilation errors gracefully.

#### Example

```ignore
use miniextendr_api::regex_impl::try_compile;

#[miniextendr]
fn safe_compile(pattern: String) -> Result<ExternalPtr<Regex>, String> {
    try_compile(&pattern)
        .map(ExternalPtr::new)
        .map_err(|e| e.to_string())
}
```

### `optionals::serde_impl::json_from_sexp`

```rust
json_from_sexp(sexp: crate::SEXP) -> Result<JsonValue, crate::from_r::SexpError>
```

Convert an R object to a JSON value with default options.

Mapping rules (R -> JSON):
- `NULL` -> `Null`
- Scalar `LGLSXP` -> `Bool` (NA -> `Null`)
- Scalar `INTSXP` -> `Number` (NA -> `Null`)
- Scalar `REALSXP` -> `Number` (NA/NaN/Inf -> error)
- Scalar `STRSXP` -> `String` (NA -> `Null`)
- Vector of length > 1 -> `Array`
- Named `VECSXP` -> `Object`
- Unnamed `VECSXP` -> `Array`
- Factor -> `String` (via levels)

#### Errors

Returns an error for:
- `NaN` or `Inf` values (JSON has no representation)
- Unsupported R types (e.g., CLOSXP, ENVSXP)

### `optionals::serde_impl::json_from_sexp_permissive`

```rust
json_from_sexp_permissive(sexp: crate::SEXP) -> Result<JsonValue, crate::from_r::SexpError>
```

Convert R to JSON with permissive handling (NA/NaN/Inf -> Null).

### `optionals::serde_impl::json_from_sexp_strict`

```rust
json_from_sexp_strict(sexp: crate::SEXP) -> Result<JsonValue, crate::from_r::SexpError>
```

Convert R to JSON with strict NA handling (errors on NA).

### `optionals::serde_impl::json_from_sexp_with`

```rust
json_from_sexp_with(sexp: crate::SEXP, opts: &JsonOptions) -> Result<JsonValue, crate::from_r::SexpError>
```

Convert an R object to a JSON value with custom options.

#### Example

```rust,ignore
let opts = JsonOptions::default()
    .na(NaHandling::String("NA".into()))
    .nan(SpecialFloatHandling::Null);
let json = json_from_sexp_with(sexp, &opts)?;
```

### `optionals::serde_impl::json_into_sexp`

```rust
json_into_sexp(value: &JsonValue) -> crate::SEXP
```

Convert a JSON value to an R object.

### `optionals::sha2_impl::sha256_bytes`

```rust
sha256_bytes(data: &[u8]) -> String
```

Compute SHA-256 hash of raw bytes.

Returns a 64-character lowercase hex string.

#### Example

```ignore
let hash = sha256_bytes(b"hello world");
assert_eq!(hash.len(), 64);
```

### `optionals::sha2_impl::sha256_bytes_vec`

```rust
sha256_bytes_vec(data: &[&[u8]]) -> Vec<String>
```

Compute SHA-256 hashes for a vector of byte slices.

Returns a vector of 64-character lowercase hex strings.

### `optionals::sha2_impl::sha256_str`

```rust
sha256_str(s: &str) -> String
```

Compute SHA-256 hash of a UTF-8 string.

Returns a 64-character lowercase hex string.

#### Example

```ignore
let hash = sha256_str("hello world");
assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
```

### `optionals::sha2_impl::sha256_str_vec`

```rust
sha256_str_vec(strings: &[&str]) -> Vec<String>
```

Compute SHA-256 hashes for a vector of strings.

Returns a vector of 64-character lowercase hex strings.

### `optionals::sha2_impl::sha512_bytes`

```rust
sha512_bytes(data: &[u8]) -> String
```

Compute SHA-512 hash of raw bytes.

Returns a 128-character lowercase hex string.

#### Example

```ignore
let hash = sha512_bytes(b"hello world");
assert_eq!(hash.len(), 128);
```

### `optionals::sha2_impl::sha512_bytes_vec`

```rust
sha512_bytes_vec(data: &[&[u8]]) -> Vec<String>
```

Compute SHA-512 hashes for a vector of byte slices.

Returns a vector of 128-character lowercase hex strings.

### `optionals::sha2_impl::sha512_str`

```rust
sha512_str(s: &str) -> String
```

Compute SHA-512 hash of a UTF-8 string.

Returns a 128-character lowercase hex string.

#### Example

```ignore
let hash = sha512_str("hello world");
assert_eq!(hash.len(), 128);
```

### `optionals::sha2_impl::sha512_str_vec`

```rust
sha512_str_vec(strings: &[&str]) -> Vec<String>
```

Compute SHA-512 hashes for a vector of strings.

Returns a vector of 128-character lowercase hex strings.

### `optionals::tabled_impl::builder_to_string`

```rust
builder_to_string(builder: Builder) -> String
```

Build a table from a builder for dynamic schemas.

#### Example

```ignore
use tabled::builder::Builder;

let mut builder = Builder::new();
builder.push_record(["Name", "Value"]);
builder.push_record(["foo", "42"]);
builder.push_record(["bar", "99"]);

let table = builder_to_string(builder);
```

### `optionals::tabled_impl::table_from_vecs`

```rust
table_from_vecs<S>(headers: &[S], rows: &[Vec<S>]) -> String
```

Build a table from column headers and rows.

#### Example

```ignore
let headers = vec!["Name", "Age"];
let rows = vec![
    vec!["Alice", "30"],
    vec!["Bob", "25"],
];
let table = table_from_vecs(&headers, &rows);
```

### `optionals::tabled_impl::table_to_string`

```rust
table_to_string<T>(rows: &[T]) -> String
```

Format rows as a table string.

Uses default tabled formatting (ASCII box drawing).

#### Example

```ignore
use tabled::Tabled;

#[derive(Tabled)]
struct Item { name: String, count: i32 }

let items = vec![
    Item { name: "apple".into(), count: 5 },
    Item { name: "banana".into(), count: 3 },
];
let table = table_to_string(&items);
```

### `optionals::tabled_impl::table_to_string_opts`

```rust
table_to_string_opts<T>(rows: &[T], max_width: Option<usize>, align: &str, _trim: bool) -> String
```

Format rows with custom options.

#### Arguments

* `rows` - Data to format
* `max_width` - Optional maximum column width (truncates)
* `align` - Alignment: "left", "right", or "center"
* `trim` - Whether to trim whitespace

#### Example

```ignore
let table = table_to_string_opts(&items, Some(20), "center", true);
```

### `optionals::tabled_impl::table_to_string_styled`

```rust
table_to_string_styled<T>(rows: &[T], style: &str) -> String
```

Format a table with a specific style.

Available styles: "ascii", "modern", "markdown", "rounded", "blank"

### `optionals::toml_impl::toml_from_str`

```rust
toml_from_str(s: &str) -> Result<TomlValue, crate::from_r::SexpError>
```

Parse a TOML document string into a `TomlValue`.

#### Errors

Returns an error if the string is not valid TOML.

#### Example

```ignore
let value = toml_from_str("[package]\nname = \"my-pkg\"")?;
```

### `optionals::toml_impl::toml_to_string`

```rust
toml_to_string(v: &TomlValue) -> String
```

Serialize a `TomlValue` to a TOML string.

#### Example

```ignore
let text = toml_to_string(&value);
```

### `optionals::toml_impl::toml_to_string_pretty`

```rust
toml_to_string_pretty(v: &TomlValue) -> String
```

Serialize a `TomlValue` to a pretty-printed TOML string.

### `optionals::url_impl::url_helpers::is_valid`

```rust
is_valid(s: &str) -> bool
```

Check if a string is a valid URL.

### `optionals::url_impl::url_helpers::join`

```rust
join(base: &super::Url, path: &str) -> Result<super::Url, String>
```

Join a base URL with a relative path.

### `optionals::url_impl::url_helpers::parse`

```rust
parse(s: &str) -> Result<super::Url, String>
```

Parse a URL string, returning an error message on failure.

### `optionals::uuid_impl::uuid_helpers::from_bytes`

```rust
from_bytes(bytes: Vec<u8>) -> Result<super::Uuid, String>
```

Parse a UUID from bytes.

### `optionals::uuid_impl::uuid_helpers::max`

```rust
max() -> super::Uuid
```

Get the max UUID (all ones).

### `optionals::uuid_impl::uuid_helpers::new_v4`

```rust
new_v4() -> super::Uuid
```

Generate a new random (v4) UUID.

### `optionals::uuid_impl::uuid_helpers::nil`

```rust
nil() -> super::Uuid
```

Get the nil UUID (all zeros).

### `optionals::uuid_impl::uuid_helpers::parse_str`

```rust
parse_str(s: &str) -> Result<super::Uuid, String>
```

Parse a UUID from a string (any format).

### `panic_telemetry::clear_panic_telemetry_hook`

```rust
clear_panic_telemetry_hook()
```

Remove the current panic telemetry hook, if any.

### `panic_telemetry::set_panic_telemetry_hook`

```rust
set_panic_telemetry_hook(f: impl Fn + Send + Sync)
```

Register a panic telemetry hook.

The hook is called with a [`PanicReport`] each time a Rust panic is about
to be converted into an R error. Only one hook can be active at a time;
calling this again replaces (and drops) the previous hook.

#### Thread Safety

The hook may be called from any thread (worker thread, main R thread, etc.).
Ensure your closure is safe to call concurrently.

It is safe to call `set_panic_telemetry_hook` or `clear_panic_telemetry_hook`
from within a hook — the lock is released before the hook is invoked.

### `r_coerce::is_supported_as_generic`

```rust
is_supported_as_generic(generic: &str) -> bool
```

Check if a generic name is a supported `as.<class>()` generic.

### `r_coerce::r_generic_to_method`

```rust
const r_generic_to_method(generic: &str) -> Option<&'static str>
```

Maps an R generic name to the corresponding trait method name.

This is used by the proc-macro to validate `#[miniextendr(as = "...")]` attributes.

#### Returns

The Rust method name that corresponds to the R generic, or `None` if the
generic is not supported.

### `r_memory::init_sexprec_data_offset`

```rust
unsafe init_sexprec_data_offset()
```

Compute and store the SEXPREC data offset by measuring a real R vector.

Must be called from R's main thread during package init.

#### Safety

Must be called on R's main thread with R initialized.

### `r_memory::sexprec_data_offset`

```rust
sexprec_data_offset() -> usize
```

Get the computed SEXPREC data offset.

Returns 0 if not yet initialized.

### `r_memory::try_recover_r_sexp`

```rust
unsafe try_recover_r_sexp(data_ptr: *const u8, expected_type: crate::SEXPTYPE, expected_len: usize) -> Option<crate::SEXP>
```

Try to recover the source R SEXP from a data pointer.

Given a pointer that may point into an R vector's data area, this
subtracts the known SEXPREC header size to get a candidate SEXP, then
verifies it:
1. The SEXP type tag (bits 0-4 of sxpinfo) matches `expected_type`
2. `ALTREP(candidate)` is false (only non-ALTREP vectors have fixed-offset data)
3. `XLENGTH(candidate)` matches `expected_len` (safe for non-ALTREP)

Returns `None` if:
- The offset hasn't been initialized yet
- The pointer doesn't come from an R vector
- The candidate SEXP has the wrong type or length
- The candidate is an ALTREP vector (data not at fixed offset from SEXP)

#### Why this is outside Rust's memory model (see #63)

This is a conservative-GC-style probe, analogous to Boehm GC scanning
the heap without allocation provenance. We compute a speculative pointer
via `wrapping_byte_sub` (well-defined pointer arithmetic) and read the
first 4 bytes (sxpinfo bits) to check whether the address looks like the
start of a SEXPREC. For pointers that did not come from an R SEXP, that
read has no valid allocation provenance under Rust's Stacked / Tree
Borrows model — it's defined behavior at the hardware level (the heap
is contiguous mapped memory), but Miri correctly flags it as UB.

We guard the read with a 4096-byte address floor (below which the
candidate would cross into unmapped memory), the ALTREP bit check
(prevents calling dispatch fns on garbage), and the length check
(filters random garbage with high probability). Callers that cannot
tolerate a false positive must not rely on this path alone.

To keep Miri green, the whole recovery is a no-op under `#[cfg(miri)]`:
we always return `None`, and callers fall back to the copy path. This
is not a correctness change — the copy path is always a valid alternative.

#### Safety

Must be called on R's main thread. The data pointer must be valid
(i.e., it must point to readable memory for at least `expected_len`
elements, which is guaranteed if it came from an Arrow buffer).

### `raw_conversions::raw_from_bytes`

```rust
raw_from_bytes<T>(bytes: &[u8]) -> Result<T, RawError>
```

Decode a POD value from raw bytes.

### `raw_conversions::raw_slice_from_bytes`

```rust
raw_slice_from_bytes<T>(bytes: &[u8]) -> Result<Vec<T>, RawError>
```

Decode a slice of POD values from raw bytes.

### `raw_conversions::raw_slice_to_bytes`

```rust
raw_slice_to_bytes<T>(values: &[T]) -> Vec<u8>
```

Encode a slice of POD values to raw bytes.

### `raw_conversions::raw_to_bytes`

```rust
raw_to_bytes<T>(value: &T) -> Vec<u8>
```

Encode a POD value to raw bytes.

### `registry::collect_r_wrappers`

```rust
collect_r_wrappers() -> Vec<std::borrow::Cow<'static, str>>
```

Collect all R wrapper entries, sorted by priority and deduplicated.

Within each priority group, S7 class definitions are topologically sorted
so parents are defined before children (S7 `parent = X` requires X to exist).

Host-only — wasm32 doesn't run wrapper-gen.

### `registry::miniextendr_register_routines`

```rust
unsafe miniextendr_register_routines(dll: *mut crate::sys::DllInfo)
```

Register all `#[miniextendr]` routines and ALTREP classes with R.

Called from `package_init()` during `R_init_*` (via `miniextendr_init!`).
Everything else is automatic.

#### Safety

Must be called from R's main thread during `R_init_*`.
`dll` must be a valid pointer provided by R.

### `registry::miniextendr_write_wasm_registry`

```rust
unsafe miniextendr_write_wasm_registry(path_sexp: crate::SEXP) -> crate::SEXP
```

C-callable entry point for `wasm_registry.rs` generation via cdylib.

Pairs with [`miniextendr_write_wrappers`]: same cdylib, separate `.Call`,
independent output path. Host-only; the generated file itself is then
consumed at compile time by the user crate's wasm32 build via
`install_wasm_runtime_slices`.

#### Safety

`path_sexp` must be a valid STRSXP of length >= 1.

### `registry::miniextendr_write_wrappers`

```rust
unsafe miniextendr_write_wrappers(path_sexp: crate::SEXP) -> crate::SEXP
```

C-callable entry point for R wrapper generation via cdylib.

Called from Makevars via Rscript: loads the cdylib with `dyn.load()`,
then `.Call("miniextendr_write_wrappers", path)` to write
`R/miniextendr-wrappers.R`. NAMESPACE generation is left to roxygen2
(`devtools::document()`).

#### Safety

`path_sexp` must be a valid STRSXP of length >= 1.

### `registry::universal_query`

```rust
unsafe universal_query(ptr: *mut crate::abi::mx_erased, trait_tag: crate::abi::mx_tag) -> *const std::os::raw::c_void
```

Universal query function for trait dispatch.

Scans [`MX_TRAIT_DISPATCH`] for a matching `(concrete_tag, trait_tag)` pair.
Returns the vtable pointer, or null if the trait is not implemented.

This replaces per-type query functions — a single function handles all types
by reading from the global dispatch table.

#### Safety

- `ptr` must point to a valid `mx_erased` with a valid base vtable.
- Must be called on R's main thread.

### `registry::write_r_wrappers_to_file`

```rust
write_r_wrappers_to_file(path: &str)
```

Write all R wrapper entries to a file.

Called from [`miniextendr_write_wrappers`] (via cdylib `dyn.load`/`.Call`).
All distributed_slice entries from `#[miniextendr]` items are available
because the cdylib includes all symbols by design.

Host-only — wasm32 doesn't run wrapper-gen.

### `rng::with_rng`

```rust
with_rng<F, R>(f: F) -> R
```

Scope guard for RNG operations.

Executes a closure with RNG state properly managed.
This is a convenience wrapper around [`RngGuard`].

#### Example

```ignore
use miniextendr_api::rng::with_rng;
use miniextendr_api::sys::unif_rand;

let values = with_rng(|| {
    (0..10).map(|_| unsafe { unif_rand() }).collect::<Vec<_>>()
});
```

#### Warning

Like [`RngGuard`], this relies on Rust drop semantics and won't
properly clean up if R longjumps. For R-exposed functions, use
`#[miniextendr(rng)]` instead.

### `s4_helpers::s4_class_name`

```rust
unsafe s4_class_name(obj: crate::SEXP) -> Option<String>
```

Extract the S4 class name from an object.

Reads the `class` attribute and returns the first element as a `String`.
Returns `None` if the object has no class attribute or the attribute is empty.

#### Safety

- `obj` must be a valid SEXP.
- Must be called from the R main thread.

### `s4_helpers::s4_get_slot`

```rust
unsafe s4_get_slot(obj: crate::SEXP, slot_name: &str) -> Result<crate::SEXP, String>
```

Get the value of a named slot from an S4 object.

Uses R's `slot(obj, name)` to access the slot value.

#### Safety

- `obj` must be a valid S4 SEXP with the named slot.
- Must be called from the R main thread.

#### Returns

- `Ok(SEXP)` with the slot value (unprotected).
- `Err(String)` if the slot doesn't exist or another R error occurs.

### `s4_helpers::s4_has_slot`

```rust
unsafe s4_has_slot(obj: crate::SEXP, slot_name: &str) -> bool
```

Check if an S4 object has a named slot.

Attempts to access the slot via [`s4_get_slot`]. Returns `true` if the
slot exists and is accessible, `false` if accessing it errors (i.e.,
the slot does not exist).

#### Safety

- `obj` must be a valid SEXP (typically an S4 object).
- Must be called from the R main thread.

### `s4_helpers::s4_is`

```rust
unsafe s4_is(obj: crate::SEXP) -> bool
```

Check if a SEXP is an S4 object.

#### Safety

- `obj` must be a valid SEXP.
- Must be called from the R main thread.

### `s4_helpers::s4_set_slot`

```rust
unsafe s4_set_slot(obj: crate::SEXP, slot_name: &str, value: crate::SEXP) -> Result<(), String>
```

Set the value of a named slot on an S4 object.

Uses R's `slot(obj, name) <- value` to assign the slot value.

#### Safety

- `obj` must be a valid S4 SEXP with the named slot.
- `value` must be a valid SEXP of the appropriate type for the slot.
- Must be called from the R main thread.

#### Returns

- `Ok(())` on success.
- `Err(String)` if the slot doesn't exist or the value type is incompatible.

### `serde::columnar::dispatch_to_dataframes`

```rust
dispatch_to_dataframes<O, E, I>(iter: I, nrow_hint: Option<usize>, names: DispatchNames) -> Result<crate::list::List, super::error::RSerdeError>
```

Stream a `Result`-yielding iterator into a named `list(ok = df, err = df)`.

Maintains two [`SerdeRowBuilder`]s and dispatches each row to the
appropriate one based on its `Result` variant. The output names default to
`"ok"` / `"err"`; pass [`DispatchNames`] to override.

Unlike a manual `iter.partition(Result::is_ok)` + two `iter_to_dataframe`
calls, this preserves the streaming property — rows are dispatched as they
arrive, no double-materialisation.

#### Empty sides

A side that received zero rows produces a 0-row, 0-column data.frame in
the returned named list. The list always has both slots so downstream R
code can rely on a stable shape (`res$ok` / `res$err`).

#### When to use this vs [`result_to_dataframe`]

Use [`super::result_to_dataframe`] when you already have a `&[Result<T, E>]`
in memory; it offers richer shape control (`Auto` / `Collated` / `Split`
with custom sentinel). Use `dispatch_to_dataframes` when the rows arrive
incrementally and you specifically want the streaming two-builder shape.

#### Errors

- A row fails to serialize (propagates from either builder).
- Schema mismatch on later rows of the same variant.

#### Example

```ignore
use miniextendr_api::serde::dispatch_to_dataframes;

#[derive(Serialize)] struct Ok_ { id: i32, val: f64 }
#[derive(Serialize)] struct Err_ { id: i32, reason: String }

let rows = (0..10).map(|i| if i % 3 == 0 {
    Err(Err_ { id: i, reason: "skip".into() })
} else {
    Ok(Ok_ { id: i, val: i as f64 * 0.5 })
});

let named = dispatch_to_dataframes(rows, None, DispatchNames::default())?;
// named$ok  -> id, val
// named$err -> id, reason
```

### `serde::columnar::hashmap_to_dataframe`

```rust
hashmap_to_dataframe<K, V>(map: &std::collections::HashMap<K, V>, key_column: &str) -> Result<crate::dataframe::DataFrame, super::error::RSerdeError>
```

Serialize a [`HashMap`] to an R data.frame.

Keys are sorted by their `Ord` impl to produce a deterministic row order.
For maps with non-`Ord` keys (or callers happy with insertion-order
non-determinism), wrap into a `BTreeMap` first or convert manually.

Output column order matches [`map_to_dataframe`]: `<key_column>` first,
then `V`'s flattened serde fields in declaration order.

#### Errors

Same as [`map_to_dataframe`].

### `serde::columnar::iter_to_dataframe`

```rust
iter_to_dataframe<T, I>(iter: I, nrow_hint: Option<usize>) -> Result<crate::dataframe::DataFrame, super::error::RSerdeError>
```

Stream rows from an iterator into a columnar data.frame.

Schema is taken from the **first row**; subsequent rows must match that
schema. If a later row introduces a field not seen in the first row,
returns [`RSerdeError::Message`]. Fields present in the first row but
missing from a later row are NA-padded.

`nrow_hint` lets callers pre-size column buffers; `None` is fine — buffers
grow exponentially via `Vec::push`.

#### When to use this vs [`vec_to_dataframe`]

Use [`vec_to_dataframe`] when you already have a `&[T]` in memory. Use
`iter_to_dataframe` when the rows arrive incrementally (a file iterator,
a DB cursor, a generator) — materialising into a `Vec` first would defeat
the purpose.

#### Errors

- A row fails to serialize.
- A row introduces a field not present in the first row's schema.
- Column assembly fails.

#### Example

```rust,ignore
#[derive(serde::Serialize)]
struct Row { id: i32, name: String }

let rows = (0..10).map(|i| Row { id: i, name: format!("item_{i}") });
let df = iter_to_dataframe(rows, Some(10))?;
```

### `serde::columnar::map_to_dataframe`

```rust
map_to_dataframe<K, V>(map: &std::collections::BTreeMap<K, V>, key_column: &str) -> Result<crate::dataframe::DataFrame, super::error::RSerdeError>
```

Serialize a [`BTreeMap`](std::collections::BTreeMap) to an R data.frame
with the keys as one column and the value struct's fields as the rest.

Output column order: `<key_column>` first, then `V`'s flattened serde
fields in declaration order. Nested struct flattening, `#[serde(flatten)]`,
and `#[serde(skip_serializing_if)]` all work the same way as in
[`vec_to_dataframe`].

`BTreeMap`'s ordered iteration gives a deterministic row order. For
[`HashMap`], see [`hashmap_to_dataframe`].

#### Errors

- `V` does not serialize as a struct or map.
- Underlying column-buffer assembly fails.

#### Example

```ignore
use std::collections::BTreeMap;
use serde::Serialize;

#[derive(Serialize)]
struct Summary {
    cmax: f64,
    tmax: f64,
}

let summary: BTreeMap<i32, Summary> = /* … */;
let df = map_to_dataframe(&summary, "subject")?;
// Columns: subject, cmax, tmax
```

### `serde::columnar::par_iter_to_dataframe`

```rust
par_iter_to_dataframe<T, I>(iter: I, nrow_hint: Option<usize>) -> Result<crate::dataframe::DataFrame, super::error::RSerdeError>
```

Parallel counterpart to [`iter_to_dataframe`]: fan row→column serialisation
out across rayon for CPU-bound row work.

#### Strategy: per-thread scratch + merge

1. **Materialise + discover** (main thread). The iterator is collected into
   a `Vec<T>` and the schema is discovered from the **first row** — the same
   homogeneous-schema contract as [`iter_to_dataframe`]. The collection is
   necessary: rayon needs an indexed source to split into ordered chunks,
   and the schema must be shared by every worker.
2. **Fan out** (worker threads, zero R API). Rows are split into contiguous
   chunks; each worker fills a *local* `Vec<ColumnBuffer>` against the shared
   schema using pure-Rust serde extraction — no SEXP allocation, no
   `ProtectScope`, no R main-thread contact. This mirrors the invariant of
   the row-oriented serde paths: the parallel region touches only Rust data.
3. **Merge in row order** (main thread). Chunk results come back ordered
   (rayon's [`collect`](rayon::iter::ParallelIterator::collect) on an `IndexedParallelIterator` preserves index order), so
   concatenating each chunk's column buffers reproduces the original row
   order exactly. The merged buffers are assembled into a [`DataFrame`].

#### Schema scope (homogeneous only)

Like [`iter_to_dataframe`], the schema is fixed from the first row. Two
shapes are **rejected** here because they need the R main thread or a
reconciliation step that defeats the per-thread-merge model:

- **Generic (list) columns** — a column whose values serialise to arbitrary
  SEXPs (the `Generic` fallback) must allocate on the R main thread. Such a
  schema returns an error pointing back to [`iter_to_dataframe`].
- **Growing / heterogeneous schema** — rows that introduce new fields under
  parallelism would produce divergent per-thread schemas needing a union
  merge. Out of scope for this homogeneous variant; use
  [`par_iter_to_dataframe_growing`] (union-schema, still parallel) or
  [`SerdeRowBuilder::grow_schema`] sequentially.

#### Equivalence

For any input whose schema is fully atomic (no `Generic` column), the result
is identical column-for-column and row-for-row to
`iter_to_dataframe(rows, nrow_hint)`.

#### Errors

- A row fails to serialize.
- A row introduces a field not present in the first row's schema.
- The discovered schema contains a `Generic` (list) column.
- Column assembly fails.

#### Example

```rust,ignore
use rayon::prelude::*;

#[derive(serde::Serialize)]
struct Row { id: i32, name: String }

let rows: Vec<Row> = (0..10_000)
    .map(|i| Row { id: i, name: format!("item_{i}") })
    .collect();
let df = par_iter_to_dataframe(rows, Some(10_000))?;
```

### `serde::columnar::par_iter_to_dataframe_growing`

```rust
par_iter_to_dataframe_growing<T, I>(iter: I, nrow_hint: Option<usize>) -> Result<crate::dataframe::DataFrame, super::error::RSerdeError>
```

Parallel, growing-schema counterpart to [`par_iter_to_dataframe`]: the
rayon-backed analogue of [`vec_to_dataframe`]'s union-schema path (#936).

Where [`par_iter_to_dataframe`] fixes the schema from the first row and
rejects rows that introduce new fields, this variant computes the **union**
of fields across *all* rows — rows may freely introduce fields the others
lack (heterogeneous structs via untagged enums, maps with divergent keys).
Fields a row doesn't carry are NA-padded, exactly like
[`vec_to_dataframe`].

#### How it stays parallel

Divergent per-thread schemas are the hard part of a growing schema under
fan-out: one worker may discover field `x` while another discovers `y`,
and neither sees the other's columns. Instead of reconciling per-thread
column sets after the fact (re-indexing + NA back-fill per chunk), the
build separates discovery from fill:

1. **Parallel union discovery.** Each worker probes its chunk's rows into a
   local `SchemaAccumulator` (pure Rust, no R contact).
2. **Global resolution** (main thread, cheap). The per-chunk accumulators
   are merged in chunk (= row) order and resolved through the same
   candidate lattice as [`vec_to_dataframe`] — so cross-chunk type clashes
   behave identically to the sequential union path: first-seen wins,
   a typed probe beats a None-only (`Generic`) probe regardless of which
   chunk saw it.
3. **Parallel fill against the shared union schema.** Identical to the
   homogeneous fan-out: every worker fills a local column set with one slot
   per union field, NA-padding fields its rows lack. The ordered merge then
   needs no reconciliation at all.

#### Schema scope

**Generic (list) columns are rejected**, same as [`par_iter_to_dataframe`]:
per-cell SEXP allocation needs the R main thread. This includes the
all-`None` case — a field that is `None` in *every* row resolves to
`Generic` (nothing typed it) and is rejected here, where
[`vec_to_dataframe`] would downgrade it to a logical-NA column at assembly.
Fall back to [`vec_to_dataframe`] for such schemas.

#### Equivalence

For any input whose union schema is fully atomic (no `Generic` column), the
result is identical column-for-column and row-for-row to
`vec_to_dataframe(&rows)`. Note this is the *union* semantics — it differs
from sequential [`SerdeRowBuilder::grow_schema`], which types each column
from its first-seen probe only (an early all-`None` window can lock a
column to `Generic` there; the union path resolves it from any later row).

#### Errors

- A row fails to serialize during the fill pass.
- No row yields any fields (e.g. every row is a top-level `None`).
- The union schema contains a `Generic` (list) column.
- Column assembly fails.

#### Example

```rust,ignore
#[derive(serde::Serialize)]
#[serde(untagged)]
enum Row {
    Old { id: i32 },
    New { id: i32, score: f64 },
}

let rows: Vec<Row> = load_mixed_rows();
// Columns: id, score — `score` is NA on every `Old` row.
let df = par_iter_to_dataframe_growing(rows, None)?;
```

### `serde::columnar::result_to_dataframe`

```rust
result_to_dataframe<T, E, S>(rows: &[Result<T, E>], shape: ResultShape<S>) -> Result<DataFrameShape, super::error::RSerdeError>
```

Partition a slice of `Result<T, E>` into Ok and Err data.frames.

The shape of the return is controlled by [`ResultShape`]. The output is
always a [`DataFrameShape`]; convert via [`crate::IntoR`] to get the
equivalent R-side SEXP at the `#[miniextendr]` function boundary.

#### Sentinel

For [`Auto`](ResultShape::Auto) and [`Split`](ResultShape::Split), the
`empty_ok_sentinel` field is the value placed in the `results` slot
when every row is `Err`. Any [`crate::IntoR`] type works — `NULL`,
`FALSE`, `NA`, an empty zero-row data.frame, etc. The sentinel is only
allocated when needed.

#### GC discipline

All intermediate data.frames are protected via
[`NamedDataFrameListBuilder`]'s `ProtectScope` while the helper is on
the stack; the returned [`DataFrameShape`] owns the inner SEXPs until
the caller consumes it.

#### Errors

- `T` or `E` does not serialize as a struct or map.
- Underlying column-buffer assembly fails.

#### Example

```ignore
# use miniextendr_api::serde::{result_to_dataframe, ResultShape};
# use serde::Serialize;
# #[derive(Serialize)] struct Obs { id: i32, value: f64 }
# #[derive(Serialize)] struct Err { id: i32, reason: String }
let rows: Vec<Result<Obs, Err>> = /* … */ vec![];
// Default dispatch: bare on all-Ok, list otherwise.
let shape = result_to_dataframe(&rows, ResultShape::Auto { empty_ok_sentinel: () })?;
```

### `serde::columnar::vec_to_dataframe`

```rust
vec_to_dataframe<T>(rows: &[T]) -> Result<crate::dataframe::DataFrame, super::error::RSerdeError>
```

Convert a slice of serializable structs to an R
[`DataFrame`] in columnar layout.

Each field of `T` becomes a column (R atomic vector). Nested structs are
recursively flattened into prefixed columns (`parent_child` naming).

This is the serde column path's `Rust → R` entry point. It produces the same
[`DataFrame`] the rest of the unified interface uses
(the same type returned by [`IntoDataFrame`](crate::dataframe::IntoDataFrame)), so the
result supports post-assembly editing through [`DataFrame`]'s own methods:

```ignore
vec_to_dataframe(&rows)?
    .rename("hashes_blake3", "hash")
    .with_column("status", status_sexp)
    .drop("internal_id")
```

#### Supported Field Types

| Rust Type | R Column Type |
|-----------|---------------|
| `bool` | `logical` |
| `i8/i16/i32` | `integer` |
| `i64/u64/f32/f64` | `numeric` |
| `String/&str` | `character` |
| `Option<T>` | Same type with NA for `None` |
| `Option<T>` (every row `None`) | `logical` NA column — R coerces to the surrounding type on first use (`c(NA, 1L)` → integer, `c(NA, "x")` → character) |
| Nested struct | Recursively flattened with `parent_child` naming |
| Other | Falls back to per-element list column |

### `serde::columnar::vec_to_dataframe_split`

```rust
vec_to_dataframe_split<T>(rows: &[T], shape: SplitShape) -> Result<DataFrameShape, super::error::RSerdeError>
```

Partition a slice of serializable enum rows into per-variant data.frames.

The output shape is selected via [`SplitShape`]:

- [`PerVariantList`](SplitShape::PerVariantList) returns the historical
  `list(VariantA = df, …)` shape (single-variant short-circuit to a
  bare data.frame).
- [`PerVariantListWithTag`](SplitShape::PerVariantListWithTag) is the
  same shape but each per-variant data.frame carries a leading
  variant-tag column. Use when downstream `rbind`/`bind_rows` needs
  the tag to survive.
- [`Collated`](SplitShape::Collated) returns one data.frame with all
  variants stacked, plus a leading variant-tag column. Other-variant
  fields are NA-filled per row.

Each variant's per-variant data.frame contains only that variant's
fields. For internally-tagged enums (`#[serde(tag = "...")]`), the
implicit tag column is dropped from each partition before any explicit
tag column is added back.

Variant-name casing: whatever serde emits. PascalCase by default;
override with `#[serde(rename_all = "snake_case")]` on the enum.

#### Errors

- Any row serializes without a variant name (i.e. it's not an enum) —
  use [`vec_to_dataframe`] for plain structs instead.
- [`Collated`](SplitShape::Collated) on empty input — the variant set
  is unknowable.
- Underlying column-buffer assembly fails.

### `serde::dataframe_de::dataframe_to_vec`

```rust
dataframe_to_vec<T>(sexp: crate::SEXP) -> Result<Vec<T>, super::error::RSerdeError>
```

Convert an R `data.frame` SEXP into a `Vec<T>`.

Each row of the data.frame is deserialized as one `T`. `T` must be a flat
struct (no nested `#[derive(Deserialize)]` structs as fields — see
"Limitations" below).

#### Type mapping

| R column type | Rust field type |
|---|---|
| `logical` | `bool` / `Option<bool>` |
| `integer` | `i32` / `Option<i32>` (also `i8`, `i16`, `i64`, `u*` via overflow check) |
| `numeric` | `f64` / `Option<f64>` |
| `character` | `String` / `Option<String>` |
| `factor` | `String` / `Option<String>` (label) or `i32` / `Option<i32>` (1-based code) |
| NA cell | `Option<T>` → `None`; non-optional → error |

#### Errors

- `sexp` does not inherit from `"data.frame"`.
- A column required by `T`'s schema is missing.
- A cell's R type does not match the Rust field type.
- A non-`Option<…>` field encounters an NA cell.

#### Limitations

1. **Nested struct un-flattening uses single-underscore prefix matching**
   ([#688](https://github.com/A2-ai/miniextendr/issues/688)). Flat fields
   whose name contains `_` are interpreted as nested-struct paths; rename
   the R column if you need a flat string-typed field with an underscore.
   `#[serde(flatten)]` is not supported.

### `serde::dataframe_de::dataframe_to_vec_borrowed`

```rust
dataframe_to_vec_borrowed<'a, T>(sexp: crate::SEXP) -> Result<BorrowedRows<'a, T>, super::error::RSerdeError>
```

Deserialise a data.frame into a `BorrowedRows<'a, T>` that holds the
source SEXP rooted for `'a`.

Sister function to [`dataframe_to_vec`]: same per-row deserialisation,
same flat-struct limitations ([#688]), but the returned handle
keeps the input SEXP protected on the R protect stack while the caller
holds it. Use this when [`with_dataframe_rows`]'s closure shape is too
restrictive — e.g., a parser that threads rows through multiple helper
functions before producing a summary.

#### Cost vs alternatives

One extra `Rf_protect` entry vs. [`with_dataframe_rows`], plus the
[`Protected`](crate::Protected) wrapper itself. Prefer the closure form
when it fits.

#### Note on the lifetime parameter

`T: for<'b> serde::Deserialize<'b>` is equivalent to `DeserializeOwned`
today, so character fields materialise as `String` (no zero-copy yet).
The `'a` lifetime ties the protect entry to the returned handle; future
work threading `'a` through [`DataFrame`]
would enable true zero-copy `T = Borrowed<'a> { name: &'a str }` — see
the issue thread on #671 for the design discussion.

[#688]: https://github.com/A2-ai/miniextendr/issues/688

### `serde::dataframe_de::with_dataframe_rows`

```rust
with_dataframe_rows<T, F, R>(sexp: crate::SEXP, f: F) -> Result<R, super::error::RSerdeError>
```

Pass a slice of deserialized rows to a scoped callback.

The `T: for<'a> Deserialize<'a>` bound is equivalent to `DeserializeOwned`,
so character fields materialise as `String` (same as [`dataframe_to_vec`]).
The advantage of this surface over `dataframe_to_vec` is ergonomic: the
callback receives `&[T]`, the borrow is scoped to `f`, and no intermediate
row-list SEXPs are allocated on the R heap.

For a zero-copy variant where `T` can hold `&'a str` borrowing from the SEXP's
CHARSXP cache, use the `BorrowedRows<'a, T>` RAII type from #671b (ships
separately on top of [`Protected`](crate::Protected)).

The input SEXP is rooted with an `OwnedProtect` for the duration of the call,
so a Rust caller passing a freshly-built data.frame is safe under GC (a
`.Call` caller's argument frame would also protect it).

#### Errors

Same as [`dataframe_to_vec`].

#### Limitations

Same as [`dataframe_to_vec`] — single-underscore nested-struct path
matching.

### `serde::traits::from_r`

```rust
from_r<T>(sexp: crate::SEXP) -> Result<T, super::error::RSerdeError>
```

Convenience function to deserialize from R.

#### Example

```rust,ignore
use miniextendr_api::serde_r::from_r;

let point: Point = from_r(sexp)?;
```

### `serde::traits::to_r`

```rust
to_r<T>(value: &T) -> Result<crate::SEXP, super::error::RSerdeError>
```

Convenience function to serialize a value to R.

#### Example

```rust,ignore
use miniextendr_api::serde_r::to_r;

let point = Point { x: 1.0, y: 2.0 };
let sexp = to_r(&point)?;
```

### `strict::checked_into_sexp_i64`

```rust
checked_into_sexp_i64(val: i64) -> crate::SEXP
```

Convert `i64` to R integer, panicking if outside i32 range.

The valid range is `(i32::MIN, i32::MAX]` — `i32::MIN` is excluded because
it is `NA_integer_` in R.

### `strict::checked_into_sexp_isize`

```rust
checked_into_sexp_isize(val: isize) -> crate::SEXP
```

Convert `isize` to R integer, panicking if outside i32 range.

### `strict::checked_into_sexp_u64`

```rust
checked_into_sexp_u64(val: u64) -> crate::SEXP
```

Convert `u64` to R integer, panicking if > i32::MAX.

### `strict::checked_into_sexp_usize`

```rust
checked_into_sexp_usize(val: usize) -> crate::SEXP
```

Convert `usize` to R integer, panicking if > i32::MAX.

### `strict::checked_option_i64_into_sexp`

```rust
checked_option_i64_into_sexp(val: Option<i64>) -> crate::SEXP
```

Convert `Option<i64>` to R integer in strict mode.
Panics if `Some(x)` is outside i32 range. `None` becomes `NA_integer_`.

### `strict::checked_option_isize_into_sexp`

```rust
checked_option_isize_into_sexp(val: Option<isize>) -> crate::SEXP
```

Convert `Option<isize>` to R integer in strict mode.

### `strict::checked_option_u64_into_sexp`

```rust
checked_option_u64_into_sexp(val: Option<u64>) -> crate::SEXP
```

Convert `Option<u64>` to R integer in strict mode.
Panics if `Some(x)` exceeds i32::MAX. `None` becomes `NA_integer_`.

### `strict::checked_option_usize_into_sexp`

```rust
checked_option_usize_into_sexp(val: Option<usize>) -> crate::SEXP
```

Convert `Option<usize>` to R integer in strict mode.

### `strict::checked_try_from_sexp_i64`

```rust
checked_try_from_sexp_i64(sexp: crate::SEXP, param: &str) -> i64
```

Convert R SEXP to `i64` in strict mode.

Only INTSXP and REALSXP are accepted. RAWSXP and LGLSXP are rejected.
For REALSXP, uses `TryCoerce` to reject fractional, NaN, and out-of-range values.

### `strict::checked_try_from_sexp_isize`

```rust
checked_try_from_sexp_isize(sexp: crate::SEXP, param: &str) -> isize
```

Convert R SEXP to `isize` in strict mode.

### `strict::checked_try_from_sexp_u64`

```rust
checked_try_from_sexp_u64(sexp: crate::SEXP, param: &str) -> u64
```

Convert R SEXP to `u64` in strict mode.

### `strict::checked_try_from_sexp_usize`

```rust
checked_try_from_sexp_usize(sexp: crate::SEXP, param: &str) -> usize
```

Convert R SEXP to `usize` in strict mode.

### `strict::checked_vec_i64_into_sexp`

```rust
checked_vec_i64_into_sexp(val: Vec<i64>) -> crate::SEXP
```

Convert `Vec<i64>` to R integer vector, panicking if any element is outside i32 range.

### `strict::checked_vec_isize_into_sexp`

```rust
checked_vec_isize_into_sexp(val: Vec<isize>) -> crate::SEXP
```

Convert `Vec<isize>` to R integer vector, panicking if any element is outside i32 range.

### `strict::checked_vec_option_i64_into_sexp`

```rust
checked_vec_option_i64_into_sexp(val: Vec<Option<i64>>) -> crate::SEXP
```

Convert `Vec<Option<i64>>` to R integer vector in strict mode.
Panics if any `Some(x)` value is outside i32 range. `None` becomes `NA_INTEGER`.

### `strict::checked_vec_option_isize_into_sexp`

```rust
checked_vec_option_isize_into_sexp(val: Vec<Option<isize>>) -> crate::SEXP
```

Convert `Vec<Option<isize>>` to R integer vector in strict mode.

### `strict::checked_vec_option_u64_into_sexp`

```rust
checked_vec_option_u64_into_sexp(val: Vec<Option<u64>>) -> crate::SEXP
```

Convert `Vec<Option<u64>>` to R integer vector in strict mode.

### `strict::checked_vec_option_usize_into_sexp`

```rust
checked_vec_option_usize_into_sexp(val: Vec<Option<usize>>) -> crate::SEXP
```

Convert `Vec<Option<usize>>` to R integer vector in strict mode.

### `strict::checked_vec_try_from_sexp_i64`

```rust
checked_vec_try_from_sexp_i64(sexp: crate::SEXP, param: &str) -> Vec<i64>
```

Convert R SEXP to `Vec<i64>` in strict mode.

### `strict::checked_vec_try_from_sexp_isize`

```rust
checked_vec_try_from_sexp_isize(sexp: crate::SEXP, param: &str) -> Vec<isize>
```

Convert R SEXP to `Vec<isize>` in strict mode.

### `strict::checked_vec_try_from_sexp_u64`

```rust
checked_vec_try_from_sexp_u64(sexp: crate::SEXP, param: &str) -> Vec<u64>
```

Convert R SEXP to `Vec<u64>` in strict mode.

### `strict::checked_vec_try_from_sexp_usize`

```rust
checked_vec_try_from_sexp_usize(sexp: crate::SEXP, param: &str) -> Vec<usize>
```

Convert R SEXP to `Vec<usize>` in strict mode.

### `strict::checked_vec_u64_into_sexp`

```rust
checked_vec_u64_into_sexp(val: Vec<u64>) -> crate::SEXP
```

Convert `Vec<u64>` to R integer vector, panicking if any element > i32::MAX.

### `strict::checked_vec_usize_into_sexp`

```rust
checked_vec_usize_into_sexp(val: Vec<usize>) -> crate::SEXP
```

Convert `Vec<usize>` to R integer vector, panicking if any element > i32::MAX.

### `sys::ALTREP`

```rust
unsafe ALTREP(x: crate::sexp::SEXP) -> ::std::os::raw::c_int
```

 Check if a SEXP is an ALTREP object (returns non-zero if true).

 Use `SexpExt::is_altrep()` instead of calling this directly.
Checked wrapper for `ALTREP`. Calls `ALTREP_unchecked` and routes through `with_r_thread`.
Generated from source location line 664, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::ALTREP_unchecked`

```rust
unsafe ALTREP_unchecked(x: SEXP) -> ::std::os::raw::c_int
```

 Check if a SEXP is an ALTREP object (returns non-zero if true).

 Use `SexpExt::is_altrep()` instead of calling this directly.
Unchecked FFI binding for `ALTREP`.
Generated from source location line 664, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::ATTRIB`

```rust
unsafe ATTRIB(x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Get the attributes pairlist of a SEXP.

 Returns R_NilValue if no attributes.
Checked wrapper for `ATTRIB`. Calls `ATTRIB_unchecked` and routes through `with_r_thread`.
Generated from source location line 625, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::ATTRIB_unchecked`

```rust
unsafe ATTRIB_unchecked(x: SEXP) -> SEXP
```

 Get the attributes pairlist of a SEXP.

 Returns R_NilValue if no attributes.
Unchecked FFI binding for `ATTRIB`.
Generated from source location line 625, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CAAR`

```rust
unsafe CAAR(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `CAAR`. Calls `CAAR_unchecked` and routes through `with_r_thread`.
Generated from source location line 564, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CAAR_unchecked`

```rust
unsafe CAAR_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `CAAR`.
Generated from source location line 564, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CAD4R`

```rust
unsafe CAD4R(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `CAD4R`. Calls `CAD4R_unchecked` and routes through `with_r_thread`.
Generated from source location line 570, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CAD4R_unchecked`

```rust
unsafe CAD4R_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `CAD4R`.
Generated from source location line 570, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CADDDR`

```rust
unsafe CADDDR(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `CADDDR`. Calls `CADDDR_unchecked` and routes through `with_r_thread`.
Generated from source location line 569, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CADDDR_unchecked`

```rust
unsafe CADDDR_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `CADDDR`.
Generated from source location line 569, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CADDR`

```rust
unsafe CADDR(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `CADDR`. Calls `CADDR_unchecked` and routes through `with_r_thread`.
Generated from source location line 568, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CADDR_unchecked`

```rust
unsafe CADDR_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `CADDR`.
Generated from source location line 568, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CADR`

```rust
unsafe CADR(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `CADR`. Calls `CADR_unchecked` and routes through `with_r_thread`.
Generated from source location line 566, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CADR_unchecked`

```rust
unsafe CADR_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `CADR`.
Generated from source location line 566, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CAR`

```rust
unsafe CAR(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `CAR`. Calls `CAR_unchecked` and routes through `with_r_thread`.
Generated from source location line 562, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CAR_unchecked`

```rust
unsafe CAR_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `CAR`.
Generated from source location line 562, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CDAR`

```rust
unsafe CDAR(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `CDAR`. Calls `CDAR_unchecked` and routes through `with_r_thread`.
Generated from source location line 565, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CDAR_unchecked`

```rust
unsafe CDAR_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `CDAR`.
Generated from source location line 565, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CDDR`

```rust
unsafe CDDR(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `CDDR`. Calls `CDDR_unchecked` and routes through `with_r_thread`.
Generated from source location line 567, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CDDR_unchecked`

```rust
unsafe CDDR_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `CDDR`.
Generated from source location line 567, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CDR`

```rust
unsafe CDR(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `CDR`. Calls `CDR_unchecked` and routes through `with_r_thread`.
Generated from source location line 563, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::CDR_unchecked`

```rust
unsafe CDR_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `CDR`.
Generated from source location line 563, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::COMPLEX_ELT`

```rust
unsafe COMPLEX_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t) -> crate::sexp_types::Rcomplex
```

Checked wrapper for `COMPLEX_ELT`. Calls `COMPLEX_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 589, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::COMPLEX_ELT_unchecked`

```rust
unsafe COMPLEX_ELT_unchecked(x: SEXP, i: R_xlen_t) -> Rcomplex
```

Unchecked FFI binding for `COMPLEX_ELT`.
Generated from source location line 589, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::COMPLEX_OR_NULL`

```rust
unsafe COMPLEX_OR_NULL(x: crate::sexp::SEXP) -> *const crate::sexp_types::Rcomplex
```

Checked wrapper for `COMPLEX_OR_NULL`. Calls `COMPLEX_OR_NULL_unchecked` and routes through `with_r_thread`.
Generated from source location line 582, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::COMPLEX_OR_NULL_unchecked`

```rust
unsafe COMPLEX_OR_NULL_unchecked(x: SEXP) -> *const Rcomplex
```

Unchecked FFI binding for `COMPLEX_OR_NULL`.
Generated from source location line 582, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::DATAPTR_OR_NULL`

```rust
unsafe DATAPTR_OR_NULL(x: crate::sexp::SEXP) -> *const ::std::os::raw::c_void
```

Checked wrapper for `DATAPTR_OR_NULL`. Calls `DATAPTR_OR_NULL_unchecked` and routes through `with_r_thread`.
Generated from source location line 534, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::DATAPTR_OR_NULL_unchecked`

```rust
unsafe DATAPTR_OR_NULL_unchecked(x: SEXP) -> *const ::std::os::raw::c_void
```

Unchecked FFI binding for `DATAPTR_OR_NULL`.
Generated from source location line 534, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::DATAPTR_RO`

```rust
unsafe DATAPTR_RO(x: crate::sexp::SEXP) -> *const ::std::os::raw::c_void
```

Checked wrapper for `DATAPTR_RO`. Calls `DATAPTR_RO_unchecked` and routes through `with_r_thread`.
Generated from source location line 533, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::DATAPTR_RO_unchecked`

```rust
unsafe DATAPTR_RO_unchecked(x: SEXP) -> *const ::std::os::raw::c_void
```

Unchecked FFI binding for `DATAPTR_RO`.
Generated from source location line 533, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::GetRNGstate`

```rust
unsafe GetRNGstate()
```

 Save the current RNG state from R's global state.

 Must be called before using `unif_rand()`, `norm_rand()`, etc.
 The state is restored with `PutRNGstate()`.

 # Example

 ```ignore
 unsafe {
     GetRNGstate();
     let x = unif_rand();
     let y = norm_rand();
     PutRNGstate();
 }
 ```
Checked wrapper for `GetRNGstate`. Calls `GetRNGstate_unchecked` and routes through `with_r_thread`.
Generated from source location line 1564, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::GetRNGstate_unchecked`

```rust
unsafe GetRNGstate_unchecked()
```

 Save the current RNG state from R's global state.

 Must be called before using `unif_rand()`, `norm_rand()`, etc.
 The state is restored with `PutRNGstate()`.

 # Example

 ```ignore
 unsafe {
     GetRNGstate();
     let x = unif_rand();
     let y = norm_rand();
     PutRNGstate();
 }
 ```
Unchecked FFI binding for `GetRNGstate`.
Generated from source location line 1564, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::INTEGER_ELT`

```rust
unsafe INTEGER_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t) -> ::std::os::raw::c_int
```

Checked wrapper for `INTEGER_ELT`. Calls `INTEGER_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 586, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::INTEGER_ELT_unchecked`

```rust
unsafe INTEGER_ELT_unchecked(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int
```

Unchecked FFI binding for `INTEGER_ELT`.
Generated from source location line 586, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::INTEGER_OR_NULL`

```rust
unsafe INTEGER_OR_NULL(x: crate::sexp::SEXP) -> *const ::std::os::raw::c_int
```

Checked wrapper for `INTEGER_OR_NULL`. Calls `INTEGER_OR_NULL_unchecked` and routes through `with_r_thread`.
Generated from source location line 580, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::INTEGER_OR_NULL_unchecked`

```rust
unsafe INTEGER_OR_NULL_unchecked(x: SEXP) -> *const ::std::os::raw::c_int
```

Unchecked FFI binding for `INTEGER_OR_NULL`.
Generated from source location line 580, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::LENGTH`

```rust
unsafe LENGTH(x: crate::sexp::SEXP) -> ::std::os::raw::c_int
```

 Get the length of a SEXP as `int` (for short vectors < 2^31).

 For long vectors, use `Rf_xlength()` instead.
 Returns 0 for R_NilValue.
Checked wrapper for `LENGTH`. Calls `LENGTH_unchecked` and routes through `with_r_thread`.
Generated from source location line 609, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::LENGTH_unchecked`

```rust
unsafe LENGTH_unchecked(x: SEXP) -> ::std::os::raw::c_int
```

 Get the length of a SEXP as `int` (for short vectors < 2^31).

 For long vectors, use `Rf_xlength()` instead.
 Returns 0 for R_NilValue.
Unchecked FFI binding for `LENGTH`.
Generated from source location line 609, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::LEVELS`

```rust
unsafe LEVELS(x: crate::sexp::SEXP) -> ::std::os::raw::c_int
```

 Get the LEVELS field (for factors).
Checked wrapper for `LEVELS`. Calls `LEVELS_unchecked` and routes through `with_r_thread`.
Generated from source location line 643, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::LEVELS_unchecked`

```rust
unsafe LEVELS_unchecked(x: SEXP) -> ::std::os::raw::c_int
```

 Get the LEVELS field (for factors).
Unchecked FFI binding for `LEVELS`.
Generated from source location line 643, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::LOGICAL_ELT`

```rust
unsafe LOGICAL_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t) -> ::std::os::raw::c_int
```

Checked wrapper for `LOGICAL_ELT`. Calls `LOGICAL_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 588, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::LOGICAL_ELT_unchecked`

```rust
unsafe LOGICAL_ELT_unchecked(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int
```

Unchecked FFI binding for `LOGICAL_ELT`.
Generated from source location line 588, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::LOGICAL_OR_NULL`

```rust
unsafe LOGICAL_OR_NULL(x: crate::sexp::SEXP) -> *const ::std::os::raw::c_int
```

Checked wrapper for `LOGICAL_OR_NULL`. Calls `LOGICAL_OR_NULL_unchecked` and routes through `with_r_thread`.
Generated from source location line 579, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::LOGICAL_OR_NULL_unchecked`

```rust
unsafe LOGICAL_OR_NULL_unchecked(x: SEXP) -> *const ::std::os::raw::c_int
```

Unchecked FFI binding for `LOGICAL_OR_NULL`.
Generated from source location line 579, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::OBJECT`

```rust
unsafe OBJECT(x: crate::sexp::SEXP) -> ::std::os::raw::c_int
```

 Check if SEXP has the "object" bit set (has a class).

 Returns non-zero if object has a class attribute.
Checked wrapper for `OBJECT`. Calls `OBJECT_unchecked` and routes through `with_r_thread`.
Generated from source location line 637, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::OBJECT_unchecked`

```rust
unsafe OBJECT_unchecked(x: SEXP) -> ::std::os::raw::c_int
```

 Check if SEXP has the "object" bit set (has a class).

 Returns non-zero if object has a class attribute.
Unchecked FFI binding for `OBJECT`.
Generated from source location line 637, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::PRINTNAME`

```rust
unsafe PRINTNAME(x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Get the print name (CHARSXP) of a symbol (SYMSXP)
Checked wrapper for `PRINTNAME`. Calls `PRINTNAME_unchecked` and routes through `with_r_thread`.
Generated from source location line 724, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::PRINTNAME_unchecked`

```rust
unsafe PRINTNAME_unchecked(x: SEXP) -> SEXP
```

 Get the print name (CHARSXP) of a symbol (SYMSXP)
Unchecked FFI binding for `PRINTNAME`.
Generated from source location line 724, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::PutRNGstate`

```rust
unsafe PutRNGstate()
```

 Restore the RNG state to R's global state.

 Must be called after using `unif_rand()`, `norm_rand()`, etc.
 to ensure R's `.Random.seed` is updated.
Checked wrapper for `PutRNGstate`. Calls `PutRNGstate_unchecked` and routes through `with_r_thread`.
Generated from source location line 1570, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::PutRNGstate_unchecked`

```rust
unsafe PutRNGstate_unchecked()
```

 Restore the RNG state to R's global state.

 Must be called after using `unif_rand()`, `norm_rand()`, etc.
 to ensure R's `.Random.seed` is updated.
Unchecked FFI binding for `PutRNGstate`.
Generated from source location line 1570, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::RAW_ELT`

```rust
unsafe RAW_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t) -> crate::sexp_types::Rbyte
```

Checked wrapper for `RAW_ELT`. Calls `RAW_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 590, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::RAW_ELT_unchecked`

```rust
unsafe RAW_ELT_unchecked(x: SEXP, i: R_xlen_t) -> Rbyte
```

Unchecked FFI binding for `RAW_ELT`.
Generated from source location line 590, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::RAW_OR_NULL`

```rust
unsafe RAW_OR_NULL(x: crate::sexp::SEXP) -> *const crate::sexp_types::Rbyte
```

Checked wrapper for `RAW_OR_NULL`. Calls `RAW_OR_NULL_unchecked` and routes through `with_r_thread`.
Generated from source location line 583, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::RAW_OR_NULL_unchecked`

```rust
unsafe RAW_OR_NULL_unchecked(x: SEXP) -> *const Rbyte
```

Unchecked FFI binding for `RAW_OR_NULL`.
Generated from source location line 583, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::REAL_ELT`

```rust
unsafe REAL_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t) -> f64
```

Checked wrapper for `REAL_ELT`. Calls `REAL_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 587, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::REAL_ELT_unchecked`

```rust
unsafe REAL_ELT_unchecked(x: SEXP, i: R_xlen_t) -> f64
```

Unchecked FFI binding for `REAL_ELT`.
Generated from source location line 587, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::REAL_OR_NULL`

```rust
unsafe REAL_OR_NULL(x: crate::sexp::SEXP) -> *const f64
```

Checked wrapper for `REAL_OR_NULL`. Calls `REAL_OR_NULL_unchecked` and routes through `with_r_thread`.
Generated from source location line 581, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::REAL_OR_NULL_unchecked`

```rust
unsafe REAL_OR_NULL_unchecked(x: SEXP) -> *const f64
```

Unchecked FFI binding for `REAL_OR_NULL`.
Generated from source location line 581, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::REprintf`

```rust
unsafe REprintf(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char)
```

Print to R's stderr (via R_ShowMessage or error console).

#### Safety

- Must be called from the R main thread
- `fmt` and `arg1` must be valid null-terminated C strings

### `sys::REprintf_unchecked`

```rust
unsafe REprintf_unchecked(arg1: *const ::std::os::raw::c_char)
```

Unchecked variadic `REprintf`; call checked wrapper when possible.

### `sys::R_CHAR`

```rust
unsafe R_CHAR(x: crate::sexp::SEXP) -> *const ::std::os::raw::c_char
```

 Get the C string pointer from a CHARSXP — encapsulated by SexpExt::r_char()
Checked wrapper for `R_CHAR`. Calls `R_CHAR_unchecked` and routes through `with_r_thread`.
Generated from source location line 727, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_CHAR_unchecked`

```rust
unsafe R_CHAR_unchecked(x: SEXP) -> *const ::std::os::raw::c_char
```

 Get the C string pointer from a CHARSXP — encapsulated by SexpExt::r_char()
Unchecked FFI binding for `R_CHAR`.
Generated from source location line 727, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_CheckStack`

```rust
unsafe R_CheckStack()
```

 Check for R stack overflow.

 Throws an R error if stack is nearly exhausted.
Checked wrapper for `R_CheckStack`. Calls `R_CheckStack_unchecked` and routes through `with_r_thread`.
Generated from source location line 1906, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_CheckStack2`

```rust
unsafe R_CheckStack2(extra: usize)
```

 Check for R stack overflow with extra space requirement.

 # Parameters

 - `extra`: Additional bytes needed
Checked wrapper for `R_CheckStack2`. Calls `R_CheckStack2_unchecked` and routes through `with_r_thread`.
Generated from source location line 1913, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_CheckStack2_unchecked`

```rust
unsafe R_CheckStack2_unchecked(extra: usize)
```

 Check for R stack overflow with extra space requirement.

 # Parameters

 - `extra`: Additional bytes needed
Unchecked FFI binding for `R_CheckStack2`.
Generated from source location line 1913, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_CheckStack_unchecked`

```rust
unsafe R_CheckStack_unchecked()
```

 Check for R stack overflow.

 Throws an R error if stack is nearly exhausted.
Unchecked FFI binding for `R_CheckStack`.
Generated from source location line 1906, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_CheckUserInterrupt`

```rust
unsafe R_CheckUserInterrupt()
```

Checked wrapper for `R_CheckUserInterrupt`. Calls `R_CheckUserInterrupt_unchecked` and routes through `with_r_thread`.
Generated from source location line 710, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_CheckUserInterrupt_unchecked`

```rust
unsafe R_CheckUserInterrupt_unchecked()
```

Unchecked FFI binding for `R_CheckUserInterrupt`.
Generated from source location line 710, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ExpandFileName`

```rust
unsafe R_ExpandFileName(s: *const ::std::os::raw::c_char) -> *const ::std::os::raw::c_char
```

 Expand a filename, resolving `~` and environment variables.

 # Returns

 Pointer to expanded path (in R's internal buffer, do not free).
Checked wrapper for `R_ExpandFileName`. Calls `R_ExpandFileName_unchecked` and routes through `with_r_thread`.
Generated from source location line 1855, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ExpandFileName_unchecked`

```rust
unsafe R_ExpandFileName_unchecked(s: *const ::std::os::raw::c_char) -> *const ::std::os::raw::c_char
```

 Expand a filename, resolving `~` and environment variables.

 # Returns

 Pointer to expanded path (in R's internal buffer, do not free).
Unchecked FFI binding for `R_ExpandFileName`.
Generated from source location line 1855, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ExternalPtrAddr`

```rust
unsafe R_ExternalPtrAddr(s: crate::sexp::SEXP) -> *mut ::std::os::raw::c_void
```

Checked wrapper for `R_ExternalPtrAddr`. Calls `R_ExternalPtrAddr_unchecked` and routes through `with_r_thread`.
Generated from source location line 363, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ExternalPtrAddrFn`

```rust
unsafe R_ExternalPtrAddrFn(s: crate::sexp::SEXP) -> DL_FUNC
```

Checked wrapper for `R_ExternalPtrAddrFn`. Calls `R_ExternalPtrAddrFn_unchecked` and routes through `with_r_thread`.
Generated from source location line 372, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ExternalPtrAddrFn_unchecked`

```rust
unsafe R_ExternalPtrAddrFn_unchecked(s: SEXP) -> DL_FUNC
```

Unchecked FFI binding for `R_ExternalPtrAddrFn`.
Generated from source location line 372, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ExternalPtrAddr_unchecked`

```rust
unsafe R_ExternalPtrAddr_unchecked(s: SEXP) -> *mut ::std::os::raw::c_void
```

Unchecked FFI binding for `R_ExternalPtrAddr`.
Generated from source location line 363, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_FindNamespace`

```rust
unsafe R_FindNamespace(info: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Find a registered namespace by name. **Longjmps on error** — prefer
 `REnv::package_namespace()` which wraps this safely.
Checked wrapper for `R_FindNamespace`. Calls `R_FindNamespace_unchecked` and routes through `with_r_thread`.
Generated from source location line 914, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_FindNamespace_unchecked`

```rust
unsafe R_FindNamespace_unchecked(info: SEXP) -> SEXP
```

 Find a registered namespace by name. **Longjmps on error** — prefer
 `REnv::package_namespace()` which wraps this safely.
Unchecked FFI binding for `R_FindNamespace`.
Generated from source location line 914, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_MakeExternalPtrFn`

```rust
unsafe R_MakeExternalPtrFn(p: DL_FUNC, tag: crate::sexp::SEXP, prot: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Added in R 3.4.0
Checked wrapper for `R_MakeExternalPtrFn`. Calls `R_MakeExternalPtrFn_unchecked` and routes through `with_r_thread`.
Generated from source location line 371, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_MakeExternalPtrFn_unchecked`

```rust
unsafe R_MakeExternalPtrFn_unchecked(p: DL_FUNC, tag: SEXP, prot: SEXP) -> SEXP
```

 Added in R 3.4.0
Unchecked FFI binding for `R_MakeExternalPtrFn`.
Generated from source location line 371, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_MakeWeakRef`

```rust
unsafe R_MakeWeakRef(key: crate::sexp::SEXP, val: crate::sexp::SEXP, fin: crate::sexp::SEXP, onexit: crate::sexp_types::Rboolean) -> crate::sexp::SEXP
```

 Create a weak reference.

 # Parameters

 - `key`: The key object (weak reference target)
 - `val`: The value to associate
 - `fin`: Finalizer function (or R_NilValue)
 - `onexit`: Whether to run finalizer on R exit
Checked wrapper for `R_MakeWeakRef`. Calls `R_MakeWeakRef_unchecked` and routes through `with_r_thread`.
Generated from source location line 2217, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_MakeWeakRefC`

```rust
unsafe R_MakeWeakRefC(key: crate::sexp::SEXP, val: crate::sexp::SEXP, fin: crate::sexp_types::R_CFinalizer_t, onexit: crate::sexp_types::Rboolean) -> crate::sexp::SEXP
```

 Create a weak reference with C finalizer.
Checked wrapper for `R_MakeWeakRefC`. Calls `R_MakeWeakRefC_unchecked` and routes through `with_r_thread`.
Generated from source location line 2220, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_MakeWeakRefC_unchecked`

```rust
unsafe R_MakeWeakRefC_unchecked(key: SEXP, val: SEXP, fin: R_CFinalizer_t, onexit: Rboolean) -> SEXP
```

 Create a weak reference with C finalizer.
Unchecked FFI binding for `R_MakeWeakRefC`.
Generated from source location line 2220, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_MakeWeakRef_unchecked`

```rust
unsafe R_MakeWeakRef_unchecked(key: SEXP, val: SEXP, fin: SEXP, onexit: Rboolean) -> SEXP
```

 Create a weak reference.

 # Parameters

 - `key`: The key object (weak reference target)
 - `val`: The value to associate
 - `fin`: Finalizer function (or R_NilValue)
 - `onexit`: Whether to run finalizer on R exit
Unchecked FFI binding for `R_MakeWeakRef`.
Generated from source location line 2217, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ParseVector`

```rust
unsafe R_ParseVector(text: crate::sexp::SEXP, n: ::std::os::raw::c_int, status: *mut ParseStatus, srcfile: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Parse R source text into an EXPRSXP (a list of parsed expressions).

 `text` is a STRSXP holding the source, `n` is the number of expressions
 to parse (`-1` for all), `status` receives the [`ParseStatus`] outcome,
 and `srcfile` is a srcref/`R_NilValue`. Allocates; protect the result.

 Prefer the safe [`crate::expression::r_eval_str`] wrapper, which does the
 STRSXP construction, status check, and protection bookkeeping for you.
Checked wrapper for `R_ParseVector`. Calls `R_ParseVector_unchecked` and routes through `with_r_thread`.
Generated from source location line 951, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ParseVector_unchecked`

```rust
unsafe R_ParseVector_unchecked(text: SEXP, n: ::std::os::raw::c_int, status: *mut ParseStatus, srcfile: SEXP) -> SEXP
```

 Parse R source text into an EXPRSXP (a list of parsed expressions).

 `text` is a STRSXP holding the source, `n` is the number of expressions
 to parse (`-1` for all), `status` receives the [`ParseStatus`] outcome,
 and `srcfile` is a srcref/`R_NilValue`. Allocates; protect the result.

 Prefer the safe [`crate::expression::r_eval_str`] wrapper, which does the
 STRSXP construction, status check, and protection bookkeeping for you.
Unchecked FFI binding for `R_ParseVector`.
Generated from source location line 951, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_PreserveObject`

```rust
unsafe R_PreserveObject(object: crate::sexp::SEXP)
```

 Add a SEXP to the global precious list, preventing GC indefinitely.

 **Cost: O(1) but allocates a CONSXP cell** — creates GC pressure on every
 call. The precious list is a global linked list (`R_PreciousList`).

 Use only for long-lived objects (e.g., ExternalPtr stored across R calls).
 For temporary protection within a function, prefer `Rf_protect`.
Checked wrapper for `R_PreserveObject`. Calls `R_PreserveObject_unchecked` and routes through `with_r_thread`.
Generated from source location line 466, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_PreserveObject_unchecked`

```rust
unsafe R_PreserveObject_unchecked(object: SEXP)
```

 Add a SEXP to the global precious list, preventing GC indefinitely.

 **Cost: O(1) but allocates a CONSXP cell** — creates GC pressure on every
 call. The precious list is a global linked list (`R_PreciousList`).

 Use only for long-lived objects (e.g., ExternalPtr stored across R calls).
 For temporary protection within a function, prefer `Rf_protect`.
Unchecked FFI binding for `R_PreserveObject`.
Generated from source location line 466, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ProtectWithIndex`

```rust
unsafe R_ProtectWithIndex(s: crate::sexp::SEXP, index: *mut ::std::os::raw::c_int)
```

 Protect a SEXP and record its stack index for later `R_Reprotect`.

 **Cost: O(1)** — same array write as `Rf_protect`, plus stores the index.
 No allocation. Use when you need to replace a protected value in-place
 (e.g., inside a loop that allocates) without unprotect/re-protect churn.
Checked wrapper for `R_ProtectWithIndex`. Calls `R_ProtectWithIndex_unchecked` and routes through `with_r_thread`.
Generated from source location line 2194, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ProtectWithIndex_unchecked`

```rust
unsafe R_ProtectWithIndex_unchecked(s: SEXP, index: *mut ::std::os::raw::c_int)
```

 Protect a SEXP and record its stack index for later `R_Reprotect`.

 **Cost: O(1)** — same array write as `Rf_protect`, plus stores the index.
 No allocation. Use when you need to replace a protected value in-place
 (e.g., inside a loop that allocates) without unprotect/re-protect churn.
Unchecked FFI binding for `R_ProtectWithIndex`.
Generated from source location line 2194, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_RegisterFinalizer`

```rust
unsafe R_RegisterFinalizer(s: crate::sexp::SEXP, fun: crate::sexp::SEXP)
```

Checked wrapper for `R_RegisterFinalizer`. Calls `R_RegisterFinalizer_unchecked` and routes through `with_r_thread`.
Generated from source location line 373, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_RegisterFinalizerEx`

```rust
unsafe R_RegisterFinalizerEx(s: crate::sexp::SEXP, fun: crate::sexp::SEXP, onexit: crate::sexp_types::Rboolean)
```

Checked wrapper for `R_RegisterFinalizerEx`. Calls `R_RegisterFinalizerEx_unchecked` and routes through `with_r_thread`.
Generated from source location line 375, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_RegisterFinalizerEx_unchecked`

```rust
unsafe R_RegisterFinalizerEx_unchecked(s: SEXP, fun: SEXP, onexit: Rboolean)
```

Unchecked FFI binding for `R_RegisterFinalizerEx`.
Generated from source location line 375, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_RegisterFinalizer_unchecked`

```rust
unsafe R_RegisterFinalizer_unchecked(s: SEXP, fun: SEXP)
```

Unchecked FFI binding for `R_RegisterFinalizer`.
Generated from source location line 373, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ReleaseObject`

```rust
unsafe R_ReleaseObject(object: crate::sexp::SEXP)
```

 Remove a SEXP from the global precious list, allowing GC.

 **Cost: O(n)** — linear scan of the entire precious list to find and unlink
 the cons cell. With `R_HASH_PRECIOUS` env var, O(bucket_size) average
 via a 1069-bucket hash table, but this is off by default.
Checked wrapper for `R_ReleaseObject`. Calls `R_ReleaseObject_unchecked` and routes through `with_r_thread`.
Generated from source location line 473, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_ReleaseObject_unchecked`

```rust
unsafe R_ReleaseObject_unchecked(object: SEXP)
```

 Remove a SEXP from the global precious list, allowing GC.

 **Cost: O(n)** — linear scan of the entire precious list to find and unlink
 the cons cell. With `R_HASH_PRECIOUS` env var, O(bucket_size) average
 via a 1069-bucket hash table, but this is off by default.
Unchecked FFI binding for `R_ReleaseObject`.
Generated from source location line 473, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_Reprotect`

```rust
unsafe R_Reprotect(s: crate::sexp::SEXP, index: ::std::os::raw::c_int)
```

 Replace the SEXP at a previously recorded protect stack index.

 **Cost: O(1)** — direct array write (`R_PPStack[index] = s`). No allocation.

 # Safety

 `index` must be from a previous `R_ProtectWithIndex` call and the
 stack must not have been unprotected past that index.
Checked wrapper for `R_Reprotect`. Calls `R_Reprotect_unchecked` and routes through `with_r_thread`.
Generated from source location line 2205, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_Reprotect_unchecked`

```rust
unsafe R_Reprotect_unchecked(s: SEXP, index: ::std::os::raw::c_int)
```

 Replace the SEXP at a previously recorded protect stack index.

 **Cost: O(1)** — direct array write (`R_PPStack[index] = s`). No allocation.

 # Safety

 `index` must be from a previous `R_ProtectWithIndex` call and the
 stack must not have been unprotected past that index.
Unchecked FFI binding for `R_Reprotect`.
Generated from source location line 2205, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_RunPendingFinalizers`

```rust
unsafe R_RunPendingFinalizers()
```

 Run pending finalizers.
Checked wrapper for `R_RunPendingFinalizers`. Calls `R_RunPendingFinalizers_unchecked` and routes through `with_r_thread`.
Generated from source location line 2229, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_RunPendingFinalizers_unchecked`

```rust
unsafe R_RunPendingFinalizers_unchecked()
```

 Run pending finalizers.
Unchecked FFI binding for `R_RunPendingFinalizers`.
Generated from source location line 2229, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_WeakRefKey`

```rust
unsafe R_WeakRefKey(w: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Get the key from a weak reference.
Checked wrapper for `R_WeakRefKey`. Calls `R_WeakRefKey_unchecked` and routes through `with_r_thread`.
Generated from source location line 2223, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_WeakRefKey_unchecked`

```rust
unsafe R_WeakRefKey_unchecked(w: SEXP) -> SEXP
```

 Get the key from a weak reference.
Unchecked FFI binding for `R_WeakRefKey`.
Generated from source location line 2223, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_WeakRefValue`

```rust
unsafe R_WeakRefValue(w: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Get the value from a weak reference.
Checked wrapper for `R_WeakRefValue`. Calls `R_WeakRefValue_unchecked` and routes through `with_r_thread`.
Generated from source location line 2226, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_WeakRefValue_unchecked`

```rust
unsafe R_WeakRefValue_unchecked(w: SEXP) -> SEXP
```

 Get the value from a weak reference.
Unchecked FFI binding for `R_WeakRefValue`.
Generated from source location line 2226, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_alloc`

```rust
unsafe R_alloc(nelem: usize, eltsize: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char
```

 Allocate memory on R's memory stack.

 This memory is automatically freed when the calling R function returns,
 or can be freed earlier with `vmaxset()`.

 # Parameters

 - `nelem`: Number of elements to allocate
 - `eltsize`: Size of each element in bytes

 # Returns

 Pointer to allocated memory (as `char*` for compatibility with S).
Checked wrapper for `R_alloc`. Calls `R_alloc_unchecked` and routes through `with_r_thread`.
Generated from source location line 1661, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_allocLD`

```rust
unsafe R_allocLD(nelem: usize) -> *mut f64
```

 Allocate an array of long doubles on R's memory stack.

 # Parameters

 - `nelem`: Number of long double elements to allocate
Checked wrapper for `R_allocLD`. Calls `R_allocLD_unchecked` and routes through `with_r_thread`.
Generated from source location line 1668, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_allocLD_unchecked`

```rust
unsafe R_allocLD_unchecked(nelem: usize) -> *mut f64
```

 Allocate an array of long doubles on R's memory stack.

 # Parameters

 - `nelem`: Number of long double elements to allocate
Unchecked FFI binding for `R_allocLD`.
Generated from source location line 1668, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_alloc_unchecked`

```rust
unsafe R_alloc_unchecked(nelem: usize, eltsize: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char
```

 Allocate memory on R's memory stack.

 This memory is automatically freed when the calling R function returns,
 or can be freed earlier with `vmaxset()`.

 # Parameters

 - `nelem`: Number of elements to allocate
 - `eltsize`: Size of each element in bytes

 # Returns

 Pointer to allocated memory (as `char*` for compatibility with S).
Unchecked FFI binding for `R_alloc`.
Generated from source location line 1661, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_altrep_data1`

```rust
unsafe R_altrep_data1(x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `R_altrep_data1`. Calls `R_altrep_data1_unchecked` and routes through `with_r_thread`.
Generated from source location line 656, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_altrep_data1_unchecked`

```rust
unsafe R_altrep_data1_unchecked(x: SEXP) -> SEXP
```

Unchecked FFI binding for `R_altrep_data1`.
Generated from source location line 656, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_altrep_data2`

```rust
unsafe R_altrep_data2(x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `R_altrep_data2`. Calls `R_altrep_data2_unchecked` and routes through `with_r_thread`.
Generated from source location line 657, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_altrep_data2_unchecked`

```rust
unsafe R_altrep_data2_unchecked(x: SEXP) -> SEXP
```

Unchecked FFI binding for `R_altrep_data2`.
Generated from source location line 657, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_atof`

```rust
unsafe R_atof(str: *const ::std::os::raw::c_char) -> f64
```

 Convert string to double, always using '.' as decimal point.

 Also accepts "NA" as input, returning NA_REAL.
Checked wrapper for `R_atof`. Calls `R_atof_unchecked` and routes through `with_r_thread`.
Generated from source location line 1860, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_atof_unchecked`

```rust
unsafe R_atof_unchecked(str: *const ::std::os::raw::c_char) -> f64
```

 Convert string to double, always using '.' as decimal point.

 Also accepts "NA" as input, returning NA_REAL.
Unchecked FFI binding for `R_atof`.
Generated from source location line 1860, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_calloc_gc`

```rust
unsafe R_calloc_gc(nelem: usize, eltsize: usize) -> *mut ::std::os::raw::c_void
```

 GC-aware calloc.

 Triggers GC if allocation fails, then retries.
 Memory must be freed with `free()`.
Checked wrapper for `R_calloc_gc`. Calls `R_calloc_gc_unchecked` and routes through `with_r_thread`.
Generated from source location line 1703, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_calloc_gc_unchecked`

```rust
unsafe R_calloc_gc_unchecked(nelem: usize, eltsize: usize) -> *mut ::std::os::raw::c_void
```

 GC-aware calloc.

 Triggers GC if allocation fails, then retries.
 Memory must be freed with `free()`.
Unchecked FFI binding for `R_calloc_gc`.
Generated from source location line 1703, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_compute_identical`

```rust
unsafe R_compute_identical(x: crate::sexp::SEXP, y: crate::sexp::SEXP, flags: ::std::os::raw::c_int) -> crate::sexp_types::Rboolean
```

 Check if two R objects are identical (deep semantic equality).

 This is the C implementation of R's `identical()` function.

 # Flags

 Use the `IDENT_*` constants below. Flags are inverted: set bit = disable that check.

 **Default from R**: `IDENT_USE_CLOENV` (16) - ignore closure environments

 # Returns

 `TRUE` if identical, `FALSE` otherwise.

 # Performance

 Fast-path: Returns `TRUE` immediately if pointers are equal.
Checked wrapper for `R_compute_identical`. Calls `R_compute_identical_unchecked` and routes through `with_r_thread`.
Generated from source location line 767, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_compute_identical_unchecked`

```rust
unsafe R_compute_identical_unchecked(x: SEXP, y: SEXP, flags: ::std::os::raw::c_int) -> Rboolean
```

 Check if two R objects are identical (deep semantic equality).

 This is the C implementation of R's `identical()` function.

 # Flags

 Use the `IDENT_*` constants below. Flags are inverted: set bit = disable that check.

 **Default from R**: `IDENT_USE_CLOENV` (16) - ignore closure environments

 # Returns

 `TRUE` if identical, `FALSE` otherwise.

 # Performance

 Fast-path: Returns `TRUE` immediately if pointers are equal.
Unchecked FFI binding for `R_compute_identical`.
Generated from source location line 767, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_csort`

```rust
unsafe R_csort(x: *mut crate::sexp_types::Rcomplex, n: ::std::os::raw::c_int)
```

 Sort a complex vector in place.

 # Parameters

 - `x`: Pointer to Rcomplex array
 - `n`: Number of elements
Checked wrapper for `R_csort`. Calls `R_csort_unchecked` and routes through `with_r_thread`.
Generated from source location line 1743, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_csort_unchecked`

```rust
unsafe R_csort_unchecked(x: *mut Rcomplex, n: ::std::os::raw::c_int)
```

 Sort a complex vector in place.

 # Parameters

 - `x`: Pointer to Rcomplex array
 - `n`: Number of elements
Unchecked FFI binding for `R_csort`.
Generated from source location line 1743, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_existsVarInFrame`

```rust
unsafe R_existsVarInFrame(rho: crate::sexp::SEXP, symbol: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

 Check if a variable exists in an environment frame.

 Does not search enclosing environments.
Checked wrapper for `R_existsVarInFrame`. Calls `R_existsVarInFrame_unchecked` and routes through `with_r_thread`.
Generated from source location line 2055, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_existsVarInFrame_unchecked`

```rust
unsafe R_existsVarInFrame_unchecked(rho: SEXP, symbol: SEXP) -> Rboolean
```

 Check if a variable exists in an environment frame.

 Does not search enclosing environments.
Unchecked FFI binding for `R_existsVarInFrame`.
Generated from source location line 2055, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_forceAndCall`

```rust
unsafe R_forceAndCall(e: crate::sexp::SEXP, n: ::std::os::raw::c_int, rho: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `R_forceAndCall`. Calls `R_forceAndCall_unchecked` and routes through `with_r_thread`.
Generated from source location line 940, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_forceAndCall_unchecked`

```rust
unsafe R_forceAndCall_unchecked(e: SEXP, n: ::std::os::raw::c_int, rho: SEXP) -> SEXP
```

Unchecked FFI binding for `R_forceAndCall`.
Generated from source location line 940, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_free_tmpnam`

```rust
unsafe R_free_tmpnam(name: *mut ::std::os::raw::c_char)
```

 Free a temporary filename allocated by `R_tmpnam` or `R_tmpnam2`.
Checked wrapper for `R_free_tmpnam`. Calls `R_free_tmpnam_unchecked` and routes through `with_r_thread`.
Generated from source location line 1901, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_free_tmpnam_unchecked`

```rust
unsafe R_free_tmpnam_unchecked(name: *mut ::std::os::raw::c_char)
```

 Free a temporary filename allocated by `R_tmpnam` or `R_tmpnam2`.
Unchecked FFI binding for `R_free_tmpnam`.
Generated from source location line 1901, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_gc`

```rust
unsafe R_gc()
```

 Run the R garbage collector.

 Forces a full garbage collection cycle.
Checked wrapper for `R_gc`. Calls `R_gc_unchecked` and routes through `with_r_thread`.
Generated from source location line 1641, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_gc_running`

```rust
unsafe R_gc_running() -> ::std::os::raw::c_int
```

 Check if the garbage collector is currently running.

 Returns non-zero if GC is in progress.
Checked wrapper for `R_gc_running`. Calls `R_gc_running_unchecked` and routes through `with_r_thread`.
Generated from source location line 1646, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_gc_running_unchecked`

```rust
unsafe R_gc_running_unchecked() -> ::std::os::raw::c_int
```

 Check if the garbage collector is currently running.

 Returns non-zero if GC is in progress.
Unchecked FFI binding for `R_gc_running`.
Generated from source location line 1646, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_gc_unchecked`

```rust
unsafe R_gc_unchecked()
```

 Run the R garbage collector.

 Forces a full garbage collection cycle.
Unchecked FFI binding for `R_gc`.
Generated from source location line 1641, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_isort`

```rust
unsafe R_isort(x: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int)
```

 Sort an integer vector in place (ascending order).

 # Parameters

 - `x`: Pointer to integer array
 - `n`: Number of elements
Checked wrapper for `R_isort`. Calls `R_isort_unchecked` and routes through `with_r_thread`.
Generated from source location line 1727, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_isort_unchecked`

```rust
unsafe R_isort_unchecked(x: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int)
```

 Sort an integer vector in place (ascending order).

 # Parameters

 - `x`: Pointer to integer array
 - `n`: Number of elements
Unchecked FFI binding for `R_isort`.
Generated from source location line 1727, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_malloc_gc`

```rust
unsafe R_malloc_gc(size: usize) -> *mut ::std::os::raw::c_void
```

 GC-aware malloc.

 Triggers GC if allocation fails, then retries.
 Memory must be freed with `free()`.
Checked wrapper for `R_malloc_gc`. Calls `R_malloc_gc_unchecked` and routes through `with_r_thread`.
Generated from source location line 1697, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_malloc_gc_unchecked`

```rust
unsafe R_malloc_gc_unchecked(size: usize) -> *mut ::std::os::raw::c_void
```

 GC-aware malloc.

 Triggers GC if allocation fails, then retries.
 Memory must be freed with `free()`.
Unchecked FFI binding for `R_malloc_gc`.
Generated from source location line 1697, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_max_col`

```rust
unsafe R_max_col(matrix: *const f64, nr: *const ::std::os::raw::c_int, nc: *const ::std::os::raw::c_int, maxes: *mut ::std::os::raw::c_int, ties_meth: *const ::std::os::raw::c_int)
```

 Find column maxima in a matrix.

 # Parameters

 - `matrix`: Column-major matrix data
 - `nr`: Number of rows
 - `nc`: Number of columns
 - `maxes`: Output array for column maxima indices (1-indexed)
 - `ties_meth`: How to handle ties (1=first, 2=random, 3=last)
Checked wrapper for `R_max_col`. Calls `R_max_col_unchecked` and routes through `with_r_thread`.
Generated from source location line 1964, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_max_col_unchecked`

```rust
unsafe R_max_col_unchecked(matrix: *const f64, nr: *const ::std::os::raw::c_int, nc: *const ::std::os::raw::c_int, maxes: *mut ::std::os::raw::c_int, ties_meth: *const ::std::os::raw::c_int)
```

 Find column maxima in a matrix.

 # Parameters

 - `matrix`: Column-major matrix data
 - `nr`: Number of rows
 - `nc`: Number of columns
 - `maxes`: Output array for column maxima indices (1-indexed)
 - `ties_meth`: How to handle ties (1=first, 2=random, 3=last)
Unchecked FFI binding for `R_max_col`.
Generated from source location line 1964, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_nchar`

```rust
unsafe R_nchar(x: crate::sexp::SEXP, ntype: ::std::os::raw::c_int, allowNA: crate::sexp_types::Rboolean, keepNA: crate::sexp_types::Rboolean, msg_name: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int
```

 Get the number of characters in a string/character.

 # Parameters

 - `x`: A string SEXP
 - `ntype`: Type of count (0=bytes, 1=chars, 2=width)
 - `allowNA`: Whether to allow NA values
 - `keepNA`: Whether to keep NA in result
 - `msg_name`: Name for error messages

 # Returns

 Character count or -1 on error.
Checked wrapper for `R_nchar`. Calls `R_nchar_unchecked` and routes through `with_r_thread`.
Generated from source location line 2020, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_nchar_unchecked`

```rust
unsafe R_nchar_unchecked(x: SEXP, ntype: ::std::os::raw::c_int, allowNA: Rboolean, keepNA: Rboolean, msg_name: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int
```

 Get the number of characters in a string/character.

 # Parameters

 - `x`: A string SEXP
 - `ntype`: Type of count (0=bytes, 1=chars, 2=width)
 - `allowNA`: Whether to allow NA values
 - `keepNA`: Whether to keep NA in result
 - `msg_name`: Name for error messages

 # Returns

 Character count or -1 on error.
Unchecked FFI binding for `R_nchar`.
Generated from source location line 2020, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_qsort`

```rust
unsafe R_qsort(v: *mut f64, i: usize, j: usize)
```

 Quicksort doubles in place.

 # Parameters

 - `v`: Pointer to double array
 - `i`: Start index (1-indexed for R compatibility)
 - `j`: End index (1-indexed)
Checked wrapper for `R_qsort`. Calls `R_qsort_unchecked` and routes through `with_r_thread`.
Generated from source location line 1809, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_qsort_I`

```rust
unsafe R_qsort_I(v: *mut f64, indx: *mut ::std::os::raw::c_int, i: ::std::os::raw::c_int, j: ::std::os::raw::c_int)
```

 Quicksort doubles with index array.

 # Parameters

 - `v`: Pointer to double array
 - `indx`: Pointer to index array (permuted alongside v)
 - `i`: Start index (1-indexed)
 - `j`: End index (1-indexed)
Checked wrapper for `R_qsort_I`. Calls `R_qsort_I_unchecked` and routes through `with_r_thread`.
Generated from source location line 1819, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_qsort_I_unchecked`

```rust
unsafe R_qsort_I_unchecked(v: *mut f64, indx: *mut ::std::os::raw::c_int, i: ::std::os::raw::c_int, j: ::std::os::raw::c_int)
```

 Quicksort doubles with index array.

 # Parameters

 - `v`: Pointer to double array
 - `indx`: Pointer to index array (permuted alongside v)
 - `i`: Start index (1-indexed)
 - `j`: End index (1-indexed)
Unchecked FFI binding for `R_qsort_I`.
Generated from source location line 1819, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_qsort_int`

```rust
unsafe R_qsort_int(iv: *mut ::std::os::raw::c_int, i: usize, j: usize)
```

 Quicksort integers in place.

 # Parameters

 - `iv`: Pointer to integer array
 - `i`: Start index (1-indexed)
 - `j`: End index (1-indexed)
Checked wrapper for `R_qsort_int`. Calls `R_qsort_int_unchecked` and routes through `with_r_thread`.
Generated from source location line 1833, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_qsort_int_I`

```rust
unsafe R_qsort_int_I(iv: *mut ::std::os::raw::c_int, indx: *mut ::std::os::raw::c_int, i: ::std::os::raw::c_int, j: ::std::os::raw::c_int)
```

 Quicksort integers with index array.

 # Parameters

 - `iv`: Pointer to integer array
 - `indx`: Pointer to index array
 - `i`: Start index (1-indexed)
 - `j`: End index (1-indexed)
Checked wrapper for `R_qsort_int_I`. Calls `R_qsort_int_I_unchecked` and routes through `with_r_thread`.
Generated from source location line 1843, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_qsort_int_I_unchecked`

```rust
unsafe R_qsort_int_I_unchecked(iv: *mut ::std::os::raw::c_int, indx: *mut ::std::os::raw::c_int, i: ::std::os::raw::c_int, j: ::std::os::raw::c_int)
```

 Quicksort integers with index array.

 # Parameters

 - `iv`: Pointer to integer array
 - `indx`: Pointer to index array
 - `i`: Start index (1-indexed)
 - `j`: End index (1-indexed)
Unchecked FFI binding for `R_qsort_int_I`.
Generated from source location line 1843, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_qsort_int_unchecked`

```rust
unsafe R_qsort_int_unchecked(iv: *mut ::std::os::raw::c_int, i: usize, j: usize)
```

 Quicksort integers in place.

 # Parameters

 - `iv`: Pointer to integer array
 - `i`: Start index (1-indexed)
 - `j`: End index (1-indexed)
Unchecked FFI binding for `R_qsort_int`.
Generated from source location line 1833, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_qsort_unchecked`

```rust
unsafe R_qsort_unchecked(v: *mut f64, i: usize, j: usize)
```

 Quicksort doubles in place.

 # Parameters

 - `v`: Pointer to double array
 - `i`: Start index (1-indexed for R compatibility)
 - `j`: End index (1-indexed)
Unchecked FFI binding for `R_qsort`.
Generated from source location line 1809, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_realloc_gc`

```rust
unsafe R_realloc_gc(ptr: *mut ::std::os::raw::c_void, size: usize) -> *mut ::std::os::raw::c_void
```

 GC-aware realloc.

 Triggers GC if allocation fails, then retries.
 Memory must be freed with `free()`.
Checked wrapper for `R_realloc_gc`. Calls `R_realloc_gc_unchecked` and routes through `with_r_thread`.
Generated from source location line 1709, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_realloc_gc_unchecked`

```rust
unsafe R_realloc_gc_unchecked(ptr: *mut ::std::os::raw::c_void, size: usize) -> *mut ::std::os::raw::c_void
```

 GC-aware realloc.

 Triggers GC if allocation fails, then retries.
 Memory must be freed with `free()`.
Unchecked FFI binding for `R_realloc_gc`.
Generated from source location line 1709, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_removeVarFromFrame`

```rust
unsafe R_removeVarFromFrame(symbol: crate::sexp::SEXP, env: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Remove a variable from an environment frame.

 # Returns

 The removed value, or R_NilValue if not found.
Checked wrapper for `R_removeVarFromFrame`. Calls `R_removeVarFromFrame_unchecked` and routes through `with_r_thread`.
Generated from source location line 2062, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_removeVarFromFrame_unchecked`

```rust
unsafe R_removeVarFromFrame_unchecked(symbol: SEXP, env: SEXP) -> SEXP
```

 Remove a variable from an environment frame.

 # Returns

 The removed value, or R_NilValue if not found.
Unchecked FFI binding for `R_removeVarFromFrame`.
Generated from source location line 2062, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_rsort`

```rust
unsafe R_rsort(x: *mut f64, n: ::std::os::raw::c_int)
```

 Sort a double vector in place (ascending order).

 # Parameters

 - `x`: Pointer to double array
 - `n`: Number of elements
Checked wrapper for `R_rsort`. Calls `R_rsort_unchecked` and routes through `with_r_thread`.
Generated from source location line 1735, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_rsort_unchecked`

```rust
unsafe R_rsort_unchecked(x: *mut f64, n: ::std::os::raw::c_int)
```

 Sort a double vector in place (ascending order).

 # Parameters

 - `x`: Pointer to double array
 - `n`: Number of elements
Unchecked FFI binding for `R_rsort`.
Generated from source location line 1735, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_sample_kind`

```rust
unsafe R_sample_kind() -> Sampletype
```

 Get the current discrete uniform sample method.
Checked wrapper for `R_sample_kind`. Calls `R_sample_kind_unchecked` and routes through `with_r_thread`.
Generated from source location line 1603, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_sample_kind_unchecked`

```rust
unsafe R_sample_kind_unchecked() -> Sampletype
```

 Get the current discrete uniform sample method.
Unchecked FFI binding for `R_sample_kind`.
Generated from source location line 1603, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_set_altrep_data1`

```rust
unsafe R_set_altrep_data1(x: crate::sexp::SEXP, v: crate::sexp::SEXP)
```

Checked wrapper for `R_set_altrep_data1`. Calls `R_set_altrep_data1_unchecked` and routes through `with_r_thread`.
Generated from source location line 658, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_set_altrep_data1_unchecked`

```rust
unsafe R_set_altrep_data1_unchecked(x: SEXP, v: SEXP)
```

Unchecked FFI binding for `R_set_altrep_data1`.
Generated from source location line 658, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_set_altrep_data2`

```rust
unsafe R_set_altrep_data2(x: crate::sexp::SEXP, v: crate::sexp::SEXP)
```

Checked wrapper for `R_set_altrep_data2`. Calls `R_set_altrep_data2_unchecked` and routes through `with_r_thread`.
Generated from source location line 659, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_set_altrep_data2_unchecked`

```rust
unsafe R_set_altrep_data2_unchecked(x: SEXP, v: SEXP)
```

Unchecked FFI binding for `R_set_altrep_data2`.
Generated from source location line 659, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_strtod`

```rust
unsafe R_strtod(c: *const ::std::os::raw::c_char, end: *mut *mut ::std::os::raw::c_char) -> f64
```

 Convert string to double with end pointer, using '.' as decimal point.

 Like `strtod()` but locale-independent.
Checked wrapper for `R_strtod`. Calls `R_strtod_unchecked` and routes through `with_r_thread`.
Generated from source location line 1865, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_strtod_unchecked`

```rust
unsafe R_strtod_unchecked(c: *const ::std::os::raw::c_char, end: *mut *mut ::std::os::raw::c_char) -> f64
```

 Convert string to double with end pointer, using '.' as decimal point.

 Like `strtod()` but locale-independent.
Unchecked FFI binding for `R_strtod`.
Generated from source location line 1865, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_tmpnam`

```rust
unsafe R_tmpnam(prefix: *const ::std::os::raw::c_char, tempdir: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char
```

 Generate a temporary filename.

 # Parameters

 - `prefix`: Filename prefix
 - `tempdir`: Directory for temp file

 # Returns

 Newly allocated string (must be freed with `R_free_tmpnam`).
Checked wrapper for `R_tmpnam`. Calls `R_tmpnam_unchecked` and routes through `with_r_thread`.
Generated from source location line 1878, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_tmpnam2`

```rust
unsafe R_tmpnam2(prefix: *const ::std::os::raw::c_char, tempdir: *const ::std::os::raw::c_char, fileext: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char
```

 Generate a temporary filename with extension.

 # Parameters

 - `prefix`: Filename prefix
 - `tempdir`: Directory for temp file
 - `fileext`: File extension (e.g., ".txt")

 # Returns

 Newly allocated string (must be freed with `R_free_tmpnam`).
Checked wrapper for `R_tmpnam2`. Calls `R_tmpnam2_unchecked` and routes through `with_r_thread`.
Generated from source location line 1894, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_tmpnam2_unchecked`

```rust
unsafe R_tmpnam2_unchecked(prefix: *const ::std::os::raw::c_char, tempdir: *const ::std::os::raw::c_char, fileext: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char
```

 Generate a temporary filename with extension.

 # Parameters

 - `prefix`: Filename prefix
 - `tempdir`: Directory for temp file
 - `fileext`: File extension (e.g., ".txt")

 # Returns

 Newly allocated string (must be freed with `R_free_tmpnam`).
Unchecked FFI binding for `R_tmpnam2`.
Generated from source location line 1894, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_tmpnam_unchecked`

```rust
unsafe R_tmpnam_unchecked(prefix: *const ::std::os::raw::c_char, tempdir: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char
```

 Generate a temporary filename.

 # Parameters

 - `prefix`: Filename prefix
 - `tempdir`: Directory for temp file

 # Returns

 Newly allocated string (must be freed with `R_free_tmpnam`).
Unchecked FFI binding for `R_tmpnam`.
Generated from source location line 1878, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_tryEval`

```rust
unsafe R_tryEval(expr: crate::sexp::SEXP, env: crate::sexp::SEXP, error_occurred: *mut ::std::os::raw::c_int) -> crate::sexp::SEXP
```

Checked wrapper for `R_tryEval`. Calls `R_tryEval_unchecked` and routes through `with_r_thread`.
Generated from source location line 934, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_tryEval_unchecked`

```rust
unsafe R_tryEval_unchecked(expr: SEXP, env: SEXP, error_occurred: *mut ::std::os::raw::c_int) -> SEXP
```

Unchecked FFI binding for `R_tryEval`.
Generated from source location line 934, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_unif_index`

```rust
unsafe R_unif_index(dn: f64) -> f64
```

 Generate a uniform random index in [0, dn).

 Used for sampling without bias for large n.

 # Important

 Must call `GetRNGstate()` before and `PutRNGstate()` after.
Checked wrapper for `R_unif_index`. Calls `R_unif_index_unchecked` and routes through `with_r_thread`.
Generated from source location line 1600, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::R_unif_index_unchecked`

```rust
unsafe R_unif_index_unchecked(dn: f64) -> f64
```

 Generate a uniform random index in [0, dn).

 Used for sampling without bias for large n.

 # Important

 Must call `GetRNGstate()` before and `PutRNGstate()` after.
Unchecked FFI binding for `R_unif_index`.
Generated from source location line 1600, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_GetOption1`

```rust
unsafe Rf_GetOption1(tag: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Get an R option value.

 Equivalent to `getOption("name")` in R.

 # Parameters

 - `tag`: Symbol for option name
Checked wrapper for `Rf_GetOption1`. Calls `Rf_GetOption1_unchecked` and routes through `with_r_thread`.
Generated from source location line 2135, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_GetOption1_unchecked`

```rust
unsafe Rf_GetOption1_unchecked(tag: SEXP) -> SEXP
```

 Get an R option value.

 Equivalent to `getOption("name")` in R.

 # Parameters

 - `tag`: Symbol for option name
Unchecked FFI binding for `Rf_GetOption1`.
Generated from source location line 2135, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_GetOptionDigits`

```rust
unsafe Rf_GetOptionDigits() -> ::std::os::raw::c_int
```

 Get the `digits` option.

 Returns the value of `getOption("digits")`.
Checked wrapper for `Rf_GetOptionDigits`. Calls `Rf_GetOptionDigits_unchecked` and routes through `with_r_thread`.
Generated from source location line 2141, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_GetOptionDigits_unchecked`

```rust
unsafe Rf_GetOptionDigits_unchecked() -> ::std::os::raw::c_int
```

 Get the `digits` option.

 Returns the value of `getOption("digits")`.
Unchecked FFI binding for `Rf_GetOptionDigits`.
Generated from source location line 2141, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_PairToVectorList`

```rust
unsafe Rf_PairToVectorList(x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Convert a pairlist to a generic vector (list).
Checked wrapper for `Rf_PairToVectorList`. Calls `Rf_PairToVectorList_unchecked` and routes through `with_r_thread`.
Generated from source location line 2235, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_PairToVectorList_unchecked`

```rust
unsafe Rf_PairToVectorList_unchecked(x: SEXP) -> SEXP
```

 Convert a pairlist to a generic vector (list).
Unchecked FFI binding for `Rf_PairToVectorList`.
Generated from source location line 2235, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_PrintValue`

```rust
unsafe Rf_PrintValue(x: crate::sexp::SEXP)
```

 Print an R value to the console.

 Uses R's standard print method for the object.
Checked wrapper for `Rf_PrintValue`. Calls `Rf_PrintValue_unchecked` and routes through `with_r_thread`.
Generated from source location line 2038, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_PrintValue_unchecked`

```rust
unsafe Rf_PrintValue_unchecked(x: SEXP)
```

 Print an R value to the console.

 Uses R's standard print method for the object.
Unchecked FFI binding for `Rf_PrintValue`.
Generated from source location line 2038, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_S3Class`

```rust
unsafe Rf_S3Class(object: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Get the S3 class of an S4 object.
Checked wrapper for `Rf_S3Class`. Calls `Rf_S3Class_unchecked` and routes through `with_r_thread`.
Generated from source location line 2123, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_S3Class_unchecked`

```rust
unsafe Rf_S3Class_unchecked(object: SEXP) -> SEXP
```

 Get the S3 class of an S4 object.
Unchecked FFI binding for `Rf_S3Class`.
Generated from source location line 2123, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarComplex`

```rust
unsafe Rf_ScalarComplex(x: crate::sexp_types::Rcomplex) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_ScalarComplex`. Calls `Rf_ScalarComplex_unchecked` and routes through `with_r_thread`.
Generated from source location line 516, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarComplex_unchecked`

```rust
unsafe Rf_ScalarComplex_unchecked(x: Rcomplex) -> SEXP
```

Unchecked FFI binding for `Rf_ScalarComplex`.
Generated from source location line 516, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarInteger`

```rust
unsafe Rf_ScalarInteger(x: ::std::os::raw::c_int) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_ScalarInteger`. Calls `Rf_ScalarInteger_unchecked` and routes through `with_r_thread`.
Generated from source location line 518, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarInteger_unchecked`

```rust
unsafe Rf_ScalarInteger_unchecked(x: ::std::os::raw::c_int) -> SEXP
```

Unchecked FFI binding for `Rf_ScalarInteger`.
Generated from source location line 518, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarLogical`

```rust
unsafe Rf_ScalarLogical(x: ::std::os::raw::c_int) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_ScalarLogical`. Calls `Rf_ScalarLogical_unchecked` and routes through `with_r_thread`.
Generated from source location line 520, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarLogical_unchecked`

```rust
unsafe Rf_ScalarLogical_unchecked(x: ::std::os::raw::c_int) -> SEXP
```

Unchecked FFI binding for `Rf_ScalarLogical`.
Generated from source location line 520, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarRaw`

```rust
unsafe Rf_ScalarRaw(x: crate::sexp_types::Rbyte) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_ScalarRaw`. Calls `Rf_ScalarRaw_unchecked` and routes through `with_r_thread`.
Generated from source location line 522, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarRaw_unchecked`

```rust
unsafe Rf_ScalarRaw_unchecked(x: Rbyte) -> SEXP
```

Unchecked FFI binding for `Rf_ScalarRaw`.
Generated from source location line 522, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarReal`

```rust
unsafe Rf_ScalarReal(x: f64) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_ScalarReal`. Calls `Rf_ScalarReal_unchecked` and routes through `with_r_thread`.
Generated from source location line 524, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarReal_unchecked`

```rust
unsafe Rf_ScalarReal_unchecked(x: f64) -> SEXP
```

Unchecked FFI binding for `Rf_ScalarReal`.
Generated from source location line 524, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarString`

```rust
unsafe Rf_ScalarString(x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_ScalarString`. Calls `Rf_ScalarString_unchecked` and routes through `with_r_thread`.
Generated from source location line 526, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ScalarString_unchecked`

```rust
unsafe Rf_ScalarString_unchecked(x: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_ScalarString`.
Generated from source location line 526, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_VectorToPairList`

```rust
unsafe Rf_VectorToPairList(x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Convert a generic vector (list) to a pairlist.
Checked wrapper for `Rf_VectorToPairList`. Calls `Rf_VectorToPairList_unchecked` and routes through `with_r_thread`.
Generated from source location line 2239, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_VectorToPairList_unchecked`

```rust
unsafe Rf_VectorToPairList_unchecked(x: SEXP) -> SEXP
```

 Convert a generic vector (list) to a pairlist.
Unchecked FFI binding for `Rf_VectorToPairList`.
Generated from source location line 2239, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_alloc3DArray`

```rust
unsafe Rf_alloc3DArray(sexptype: crate::sexp_types::SEXPTYPE, nrow: ::std::os::raw::c_int, ncol: ::std::os::raw::c_int, nface: ::std::os::raw::c_int) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_alloc3DArray`. Calls `Rf_alloc3DArray_unchecked` and routes through `with_r_thread`.
Generated from source location line 488, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_alloc3DArray_unchecked`

```rust
unsafe Rf_alloc3DArray_unchecked(sexptype: SEXPTYPE, nrow: ::std::os::raw::c_int, ncol: ::std::os::raw::c_int, nface: ::std::os::raw::c_int) -> SEXP
```

Unchecked FFI binding for `Rf_alloc3DArray`.
Generated from source location line 488, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocArray`

```rust
unsafe Rf_allocArray(sexptype: crate::sexp_types::SEXPTYPE, dims: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_allocArray`. Calls `Rf_allocArray_unchecked` and routes through `with_r_thread`.
Generated from source location line 486, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocArray_unchecked`

```rust
unsafe Rf_allocArray_unchecked(sexptype: SEXPTYPE, dims: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_allocArray`.
Generated from source location line 486, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocLang`

```rust
unsafe Rf_allocLang(n: ::std::os::raw::c_int) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_allocLang`. Calls `Rf_allocLang_unchecked` and routes through `with_r_thread`.
Generated from source location line 500, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocLang_unchecked`

```rust
unsafe Rf_allocLang_unchecked(n: ::std::os::raw::c_int) -> SEXP
```

Unchecked FFI binding for `Rf_allocLang`.
Generated from source location line 500, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocMatrix`

```rust
unsafe Rf_allocMatrix(sexptype: crate::sexp_types::SEXPTYPE, nrow: ::std::os::raw::c_int, ncol: ::std::os::raw::c_int) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_allocMatrix`. Calls `Rf_allocMatrix_unchecked` and routes through `with_r_thread`.
Generated from source location line 480, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocMatrix_unchecked`

```rust
unsafe Rf_allocMatrix_unchecked(sexptype: SEXPTYPE, nrow: ::std::os::raw::c_int, ncol: ::std::os::raw::c_int) -> SEXP
```

Unchecked FFI binding for `Rf_allocMatrix`.
Generated from source location line 480, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocS4Object`

```rust
unsafe Rf_allocS4Object() -> crate::sexp::SEXP
```

Checked wrapper for `Rf_allocS4Object`. Calls `Rf_allocS4Object_unchecked` and routes through `with_r_thread`.
Generated from source location line 502, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocS4Object_unchecked`

```rust
unsafe Rf_allocS4Object_unchecked() -> SEXP
```

Unchecked FFI binding for `Rf_allocS4Object`.
Generated from source location line 502, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocSExp`

```rust
unsafe Rf_allocSExp(sexptype: crate::sexp_types::SEXPTYPE) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_allocSExp`. Calls `Rf_allocSExp_unchecked` and routes through `with_r_thread`.
Generated from source location line 504, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocSExp_unchecked`

```rust
unsafe Rf_allocSExp_unchecked(sexptype: SEXPTYPE) -> SEXP
```

Unchecked FFI binding for `Rf_allocSExp`.
Generated from source location line 504, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocVector`

```rust
unsafe Rf_allocVector(sexptype: crate::sexp_types::SEXPTYPE, length: crate::sexp_types::R_xlen_t) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_allocVector`. Calls `Rf_allocVector_unchecked` and routes through `with_r_thread`.
Generated from source location line 478, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_allocVector_unchecked`

```rust
unsafe Rf_allocVector_unchecked(sexptype: SEXPTYPE, length: R_xlen_t) -> SEXP
```

Unchecked FFI binding for `Rf_allocVector`.
Generated from source location line 478, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_any_duplicated`

```rust
unsafe Rf_any_duplicated(x: crate::sexp::SEXP, fromLast: crate::sexp_types::Rboolean) -> crate::sexp_types::R_xlen_t
```

 Find first duplicated element.

 # Parameters

 - `x`: Vector to search
 - `fromLast`: If TRUE, search from end

 # Returns

 0 if no duplicates, otherwise 1-indexed position of first duplicate.
Checked wrapper for `Rf_any_duplicated`. Calls `Rf_any_duplicated_unchecked` and routes through `with_r_thread`.
Generated from source location line 2108, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_any_duplicated_unchecked`

```rust
unsafe Rf_any_duplicated_unchecked(x: SEXP, fromLast: Rboolean) -> R_xlen_t
```

 Find first duplicated element.

 # Parameters

 - `x`: Vector to search
 - `fromLast`: If TRUE, search from end

 # Returns

 0 if no duplicates, otherwise 1-indexed position of first duplicate.
Unchecked FFI binding for `Rf_any_duplicated`.
Generated from source location line 2108, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_applyClosure`

```rust
unsafe Rf_applyClosure(call: crate::sexp::SEXP, op: crate::sexp::SEXP, args: crate::sexp::SEXP, rho: crate::sexp::SEXP, suppliedvars: crate::sexp::SEXP, check: crate::sexp_types::Rboolean) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_applyClosure`. Calls `Rf_applyClosure_unchecked` and routes through `with_r_thread`.
Generated from source location line 926, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_applyClosure_unchecked`

```rust
unsafe Rf_applyClosure_unchecked(call: SEXP, op: SEXP, args: SEXP, rho: SEXP, suppliedvars: SEXP, check: Rboolean) -> SEXP
```

Unchecked FFI binding for `Rf_applyClosure`.
Generated from source location line 926, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asChar`

```rust
unsafe Rf_asChar(x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_asChar`. Calls `Rf_asChar_unchecked` and routes through `with_r_thread`.
Generated from source location line 797, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asChar_unchecked`

```rust
unsafe Rf_asChar_unchecked(x: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_asChar`.
Generated from source location line 797, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asInteger`

```rust
unsafe Rf_asInteger(x: crate::sexp::SEXP) -> ::std::os::raw::c_int
```

Checked wrapper for `Rf_asInteger`. Calls `Rf_asInteger_unchecked` and routes through `with_r_thread`.
Generated from source location line 793, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asInteger_unchecked`

```rust
unsafe Rf_asInteger_unchecked(x: SEXP) -> ::std::os::raw::c_int
```

Unchecked FFI binding for `Rf_asInteger`.
Generated from source location line 793, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asLogical`

```rust
unsafe Rf_asLogical(x: crate::sexp::SEXP) -> ::std::os::raw::c_int
```

Checked wrapper for `Rf_asLogical`. Calls `Rf_asLogical_unchecked` and routes through `with_r_thread`.
Generated from source location line 791, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asLogical_unchecked`

```rust
unsafe Rf_asLogical_unchecked(x: SEXP) -> ::std::os::raw::c_int
```

Unchecked FFI binding for `Rf_asLogical`.
Generated from source location line 791, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asReal`

```rust
unsafe Rf_asReal(x: crate::sexp::SEXP) -> f64
```

Checked wrapper for `Rf_asReal`. Calls `Rf_asReal_unchecked` and routes through `with_r_thread`.
Generated from source location line 795, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asReal_unchecked`

```rust
unsafe Rf_asReal_unchecked(x: SEXP) -> f64
```

Unchecked FFI binding for `Rf_asReal`.
Generated from source location line 795, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asS4`

```rust
unsafe Rf_asS4(object: crate::sexp::SEXP, flag: crate::sexp_types::Rboolean, complete: ::std::os::raw::c_int) -> crate::sexp::SEXP
```

 Convert to an S4 object.

 # Parameters

 - `object`: Object to convert
 - `flag`: Conversion flag
Checked wrapper for `Rf_asS4`. Calls `Rf_asS4_unchecked` and routes through `with_r_thread`.
Generated from source location line 2119, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_asS4_unchecked`

```rust
unsafe Rf_asS4_unchecked(object: SEXP, flag: Rboolean, complete: ::std::os::raw::c_int) -> SEXP
```

 Convert to an S4 object.

 # Parameters

 - `object`: Object to convert
 - `flag`: Conversion flag
Unchecked FFI binding for `Rf_asS4`.
Generated from source location line 2119, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_charIsASCII`

```rust
unsafe Rf_charIsASCII(x: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_charIsASCII`. Calls `Rf_charIsASCII_unchecked` and routes through `with_r_thread`.
Generated from source location line 324, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_charIsASCII_unchecked`

```rust
unsafe Rf_charIsASCII_unchecked(x: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_charIsASCII`.
Generated from source location line 324, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_charIsLatin1`

```rust
unsafe Rf_charIsLatin1(x: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_charIsLatin1`. Calls `Rf_charIsLatin1_unchecked` and routes through `with_r_thread`.
Generated from source location line 328, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_charIsLatin1_unchecked`

```rust
unsafe Rf_charIsLatin1_unchecked(x: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_charIsLatin1`.
Generated from source location line 328, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_charIsUTF8`

```rust
unsafe Rf_charIsUTF8(x: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_charIsUTF8`. Calls `Rf_charIsUTF8_unchecked` and routes through `with_r_thread`.
Generated from source location line 326, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_charIsUTF8_unchecked`

```rust
unsafe Rf_charIsUTF8_unchecked(x: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_charIsUTF8`.
Generated from source location line 326, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_classgets`

```rust
unsafe Rf_classgets(vec: crate::sexp::SEXP, klass: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Set the class attribute of a vector.

 Equivalent to R's `class(vec) <- klass` syntax.
 The "gets" suffix indicates this is a setter function.

 # Returns

 Returns the modified vector (like all "*gets" functions).
Checked wrapper for `Rf_classgets`. Calls `Rf_classgets_unchecked` and routes through `with_r_thread`.
Generated from source location line 879, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_classgets_unchecked`

```rust
unsafe Rf_classgets_unchecked(vec: SEXP, klass: SEXP) -> SEXP
```

 Set the class attribute of a vector.

 Equivalent to R's `class(vec) <- klass` syntax.
 The "gets" suffix indicates this is a setter function.

 # Returns

 Returns the modified vector (like all "*gets" functions).
Unchecked FFI binding for `Rf_classgets`.
Generated from source location line 879, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_coerceVector`

```rust
unsafe Rf_coerceVector(v: crate::sexp::SEXP, sexptype: crate::sexp_types::SEXPTYPE) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_coerceVector`. Calls `Rf_coerceVector_unchecked` and routes through `with_r_thread`.
Generated from source location line 799, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_coerceVector_unchecked`

```rust
unsafe Rf_coerceVector_unchecked(v: SEXP, sexptype: SEXPTYPE) -> SEXP
```

Unchecked FFI binding for `Rf_coerceVector`.
Generated from source location line 799, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_cons`

```rust
unsafe Rf_cons(car: crate::sexp::SEXP, cdr: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_cons`. Calls `Rf_cons_unchecked` and routes through `with_r_thread`.
Generated from source location line 507, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_cons_unchecked`

```rust
unsafe Rf_cons_unchecked(car: SEXP, cdr: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_cons`.
Generated from source location line 507, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_copyMostAttrib`

```rust
unsafe Rf_copyMostAttrib(source: crate::sexp::SEXP, target: crate::sexp::SEXP)
```

 Copy most attributes from source to target.

 Copies all attributes except names, dim, and dimnames.
Checked wrapper for `Rf_copyMostAttrib`. Calls `Rf_copyMostAttrib_unchecked` and routes through `with_r_thread`.
Generated from source location line 2095, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_copyMostAttrib_unchecked`

```rust
unsafe Rf_copyMostAttrib_unchecked(source: SEXP, target: SEXP)
```

 Copy most attributes from source to target.

 Copies all attributes except names, dim, and dimnames.
Unchecked FFI binding for `Rf_copyMostAttrib`.
Generated from source location line 2095, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_defineVar`

```rust
unsafe Rf_defineVar(symbol: crate::sexp::SEXP, value: crate::sexp::SEXP, rho: crate::sexp::SEXP)
```

Checked wrapper for `Rf_defineVar`. Calls `Rf_defineVar_unchecked` and routes through `with_r_thread`.
Generated from source location line 905, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_defineVar_unchecked`

```rust
unsafe Rf_defineVar_unchecked(symbol: SEXP, value: SEXP, rho: SEXP)
```

Unchecked FFI binding for `Rf_defineVar`.
Generated from source location line 905, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_dimgets`

```rust
unsafe Rf_dimgets(vec: crate::sexp::SEXP, val: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Set the `dim` attribute; returns the updated object.
Checked wrapper for `Rf_dimgets`. Calls `Rf_dimgets_unchecked` and routes through `with_r_thread`.
Generated from source location line 741, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_dimgets_unchecked`

```rust
unsafe Rf_dimgets_unchecked(vec: SEXP, val: SEXP) -> SEXP
```

 Set the `dim` attribute; returns the updated object.
Unchecked FFI binding for `Rf_dimgets`.
Generated from source location line 741, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_dimnamesgets`

```rust
unsafe Rf_dimnamesgets(vec: crate::sexp::SEXP, val: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Set the dimnames attribute of an array/matrix.

 Equivalent to R's `dimnames(vec) <- val` syntax.
 The "gets" suffix indicates this is a setter function.

 # Returns

 Returns the modified vector.
Checked wrapper for `Rf_dimnamesgets`. Calls `Rf_dimnamesgets_unchecked` and routes through `with_r_thread`.
Generated from source location line 890, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_dimnamesgets_unchecked`

```rust
unsafe Rf_dimnamesgets_unchecked(vec: SEXP, val: SEXP) -> SEXP
```

 Set the dimnames attribute of an array/matrix.

 Equivalent to R's `dimnames(vec) <- val` syntax.
 The "gets" suffix indicates this is a setter function.

 # Returns

 Returns the modified vector.
Unchecked FFI binding for `Rf_dimnamesgets`.
Generated from source location line 890, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_duplicate`

```rust
unsafe Rf_duplicate(s: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_duplicate`. Calls `Rf_duplicate_unchecked` and routes through `with_r_thread`.
Generated from source location line 745, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_duplicate_unchecked`

```rust
unsafe Rf_duplicate_unchecked(s: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_duplicate`.
Generated from source location line 745, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_elt`

```rust
unsafe Rf_elt(list: crate::sexp::SEXP, i: ::std::os::raw::c_int) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_elt`. Calls `Rf_elt_unchecked` and routes through `with_r_thread`.
Generated from source location line 857, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_elt_unchecked`

```rust
unsafe Rf_elt_unchecked(list: SEXP, i: ::std::os::raw::c_int) -> SEXP
```

Unchecked FFI binding for `Rf_elt`.
Generated from source location line 857, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_error`

```rust
unsafe Rf_error(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) -> never
```

Checked wrapper for `Rf_error` - panics if called from non-main thread.
Common usage: `Rf_error(c"%s".as_ptr(), message.as_ptr())`

#### Safety

- Must be called from the R main thread
- `fmt` and `arg1` must be valid null-terminated C strings

### `sys::Rf_error_unchecked`

```rust
unsafe Rf_error_unchecked(arg1: *const ::std::os::raw::c_char) -> never
```

Unchecked variadic `Rf_error`; call checked wrapper when possible.

### `sys::Rf_errorcall`

```rust
unsafe Rf_errorcall(call: crate::sexp::SEXP, fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) -> never
```

Checked wrapper for `Rf_errorcall` - panics if called from non-main thread.

#### Safety

- Must be called from the R main thread
- `call` must be a valid SEXP or R_NilValue
- `fmt` and `arg1` must be valid null-terminated C strings

### `sys::Rf_errorcall_unchecked`

```rust
unsafe Rf_errorcall_unchecked(arg1: SEXP, arg2: *const ::std::os::raw::c_char) -> never
```

Unchecked variadic `Rf_errorcall`; call checked wrapper when possible.

### `sys::Rf_eval`

```rust
unsafe Rf_eval(expr: crate::sexp::SEXP, rho: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_eval`. Calls `Rf_eval_unchecked` and routes through `with_r_thread`.
Generated from source location line 924, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_eval_unchecked`

```rust
unsafe Rf_eval_unchecked(expr: SEXP, rho: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_eval`.
Generated from source location line 924, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_findFun`

```rust
unsafe Rf_findFun(symbol: crate::sexp::SEXP, rho: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_findFun`. Calls `Rf_findFun_unchecked` and routes through `with_r_thread`.
Generated from source location line 909, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_findFun_unchecked`

```rust
unsafe Rf_findFun_unchecked(symbol: SEXP, rho: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_findFun`.
Generated from source location line 909, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_findVar`

```rust
unsafe Rf_findVar(symbol: crate::sexp::SEXP, rho: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_findVar`. Calls `Rf_findVar_unchecked` and routes through `with_r_thread`.
Generated from source location line 899, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_findVarInFrame`

```rust
unsafe Rf_findVarInFrame(rho: crate::sexp::SEXP, symbol: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_findVarInFrame`. Calls `Rf_findVarInFrame_unchecked` and routes through `with_r_thread`.
Generated from source location line 901, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_findVarInFrame3`

```rust
unsafe Rf_findVarInFrame3(rho: crate::sexp::SEXP, symbol: crate::sexp::SEXP, doget: crate::sexp_types::Rboolean) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_findVarInFrame3`. Calls `Rf_findVarInFrame3_unchecked` and routes through `with_r_thread`.
Generated from source location line 903, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_findVarInFrame3_unchecked`

```rust
unsafe Rf_findVarInFrame3_unchecked(rho: SEXP, symbol: SEXP, doget: Rboolean) -> SEXP
```

Unchecked FFI binding for `Rf_findVarInFrame3`.
Generated from source location line 903, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_findVarInFrame_unchecked`

```rust
unsafe Rf_findVarInFrame_unchecked(rho: SEXP, symbol: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_findVarInFrame`.
Generated from source location line 901, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_findVar_unchecked`

```rust
unsafe Rf_findVar_unchecked(symbol: SEXP, rho: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_findVar`.
Generated from source location line 899, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_getAttrib`

```rust
unsafe Rf_getAttrib(vec: crate::sexp::SEXP, name: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Read an attribute from an object by symbol (e.g. `R_NamesSymbol`).

 Returns `R_NilValue` if the attribute is not set.
Checked wrapper for `Rf_getAttrib`. Calls `Rf_getAttrib_unchecked` and routes through `with_r_thread`.
Generated from source location line 735, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_getAttrib_unchecked`

```rust
unsafe Rf_getAttrib_unchecked(vec: SEXP, name: SEXP) -> SEXP
```

 Read an attribute from an object by symbol (e.g. `R_NamesSymbol`).

 Returns `R_NilValue` if the attribute is not set.
Unchecked FFI binding for `Rf_getAttrib`.
Generated from source location line 735, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_getCharCE`

```rust
unsafe Rf_getCharCE(x: crate::sexp::SEXP) -> crate::sexp_types::cetype_t
```

Checked wrapper for `Rf_getCharCE`. Calls `Rf_getCharCE_unchecked` and routes through `with_r_thread`.
Generated from source location line 322, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_getCharCE_unchecked`

```rust
unsafe Rf_getCharCE_unchecked(x: SEXP) -> cetype_t
```

Unchecked FFI binding for `Rf_getCharCE`.
Generated from source location line 322, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_inherits`

```rust
unsafe Rf_inherits(x: crate::sexp::SEXP, klass: *const ::std::os::raw::c_char) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_inherits`. Calls `Rf_inherits_unchecked` and routes through `with_r_thread`.
Generated from source location line 809, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_inherits_unchecked`

```rust
unsafe Rf_inherits_unchecked(x: SEXP, klass: *const ::std::os::raw::c_char) -> Rboolean
```

Unchecked FFI binding for `Rf_inherits`.
Generated from source location line 809, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_install`

```rust
unsafe Rf_install(name: *const ::std::os::raw::c_char) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_install`. Calls `Rf_install_unchecked` and routes through `with_r_thread`.
Generated from source location line 722, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_installChar`

```rust
unsafe Rf_installChar(x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Install a symbol from a CHARSXP.

 Like `Rf_install()` but takes a CHARSXP instead of C string.
Checked wrapper for `Rf_installChar`. Calls `Rf_installChar_unchecked` and routes through `with_r_thread`.
Generated from source location line 2247, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_installChar_unchecked`

```rust
unsafe Rf_installChar_unchecked(x: SEXP) -> SEXP
```

 Install a symbol from a CHARSXP.

 Like `Rf_install()` but takes a CHARSXP instead of C string.
Unchecked FFI binding for `Rf_installChar`.
Generated from source location line 2247, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_install_unchecked`

```rust
unsafe Rf_install_unchecked(name: *const ::std::os::raw::c_char) -> SEXP
```

Unchecked FFI binding for `Rf_install`.
Generated from source location line 722, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isArray`

```rust
unsafe Rf_isArray(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isArray`. Calls `Rf_isArray_unchecked` and routes through `with_r_thread`.
Generated from source location line 831, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isArray_unchecked`

```rust
unsafe Rf_isArray_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isArray`.
Generated from source location line 831, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isComplex`

```rust
unsafe Rf_isComplex(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isComplex`. Calls `Rf_isComplex_unchecked` and routes through `with_r_thread`.
Generated from source location line 821, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isComplex_unchecked`

```rust
unsafe Rf_isComplex_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isComplex`.
Generated from source location line 821, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isDataFrame`

```rust
unsafe Rf_isDataFrame(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isDataFrame`. Calls `Rf_isDataFrame_unchecked` and routes through `with_r_thread`.
Generated from source location line 847, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isDataFrame_unchecked`

```rust
unsafe Rf_isDataFrame_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isDataFrame`.
Generated from source location line 847, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isEnvironment`

```rust
unsafe Rf_isEnvironment(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isEnvironment`. Calls `Rf_isEnvironment_unchecked` and routes through `with_r_thread`.
Generated from source location line 825, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isEnvironment_unchecked`

```rust
unsafe Rf_isEnvironment_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isEnvironment`.
Generated from source location line 825, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isExpression`

```rust
unsafe Rf_isExpression(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isExpression`. Calls `Rf_isExpression_unchecked` and routes through `with_r_thread`.
Generated from source location line 823, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isExpression_unchecked`

```rust
unsafe Rf_isExpression_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isExpression`.
Generated from source location line 823, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isFactor`

```rust
unsafe Rf_isFactor(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isFactor`. Calls `Rf_isFactor_unchecked` and routes through `with_r_thread`.
Generated from source location line 849, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isFactor_unchecked`

```rust
unsafe Rf_isFactor_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isFactor`.
Generated from source location line 849, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isFunction`

```rust
unsafe Rf_isFunction(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isFunction`. Calls `Rf_isFunction_unchecked` and routes through `with_r_thread`.
Generated from source location line 841, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isFunction_unchecked`

```rust
unsafe Rf_isFunction_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isFunction`.
Generated from source location line 841, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isInteger`

```rust
unsafe Rf_isInteger(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isInteger`. Calls `Rf_isInteger_unchecked` and routes through `with_r_thread`.
Generated from source location line 851, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isInteger_unchecked`

```rust
unsafe Rf_isInteger_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isInteger`.
Generated from source location line 851, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isLanguage`

```rust
unsafe Rf_isLanguage(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isLanguage`. Calls `Rf_isLanguage_unchecked` and routes through `with_r_thread`.
Generated from source location line 845, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isLanguage_unchecked`

```rust
unsafe Rf_isLanguage_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isLanguage`.
Generated from source location line 845, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isList`

```rust
unsafe Rf_isList(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isList`. Calls `Rf_isList_unchecked` and routes through `with_r_thread`.
Generated from source location line 835, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isList_unchecked`

```rust
unsafe Rf_isList_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isList`.
Generated from source location line 835, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isLogical`

```rust
unsafe Rf_isLogical(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isLogical`. Calls `Rf_isLogical_unchecked` and routes through `with_r_thread`.
Generated from source location line 817, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isLogical_unchecked`

```rust
unsafe Rf_isLogical_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isLogical`.
Generated from source location line 817, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isMatrix`

```rust
unsafe Rf_isMatrix(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isMatrix`. Calls `Rf_isMatrix_unchecked` and routes through `with_r_thread`.
Generated from source location line 833, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isMatrix_unchecked`

```rust
unsafe Rf_isMatrix_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isMatrix`.
Generated from source location line 833, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isNewList`

```rust
unsafe Rf_isNewList(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isNewList`. Calls `Rf_isNewList_unchecked` and routes through `with_r_thread`.
Generated from source location line 837, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isNewList_unchecked`

```rust
unsafe Rf_isNewList_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isNewList`.
Generated from source location line 837, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isNull`

```rust
unsafe Rf_isNull(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isNull`. Calls `Rf_isNull_unchecked` and routes through `with_r_thread`.
Generated from source location line 813, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isNull_unchecked`

```rust
unsafe Rf_isNull_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isNull`.
Generated from source location line 813, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isObject`

```rust
unsafe Rf_isObject(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isObject`. Calls `Rf_isObject_unchecked` and routes through `with_r_thread`.
Generated from source location line 853, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isObject_unchecked`

```rust
unsafe Rf_isObject_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isObject`.
Generated from source location line 853, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isOrdered`

```rust
unsafe Rf_isOrdered(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

 Check if a factor is ordered.
Checked wrapper for `Rf_isOrdered`. Calls `Rf_isOrdered_unchecked` and routes through `with_r_thread`.
Generated from source location line 2153, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isOrdered_unchecked`

```rust
unsafe Rf_isOrdered_unchecked(s: SEXP) -> Rboolean
```

 Check if a factor is ordered.
Unchecked FFI binding for `Rf_isOrdered`.
Generated from source location line 2153, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isPairList`

```rust
unsafe Rf_isPairList(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isPairList`. Calls `Rf_isPairList_unchecked` and routes through `with_r_thread`.
Generated from source location line 839, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isPairList_unchecked`

```rust
unsafe Rf_isPairList_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isPairList`.
Generated from source location line 839, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isPrimitive`

```rust
unsafe Rf_isPrimitive(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isPrimitive`. Calls `Rf_isPrimitive_unchecked` and routes through `with_r_thread`.
Generated from source location line 843, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isPrimitive_unchecked`

```rust
unsafe Rf_isPrimitive_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isPrimitive`.
Generated from source location line 843, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isReal`

```rust
unsafe Rf_isReal(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isReal`. Calls `Rf_isReal_unchecked` and routes through `with_r_thread`.
Generated from source location line 819, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isReal_unchecked`

```rust
unsafe Rf_isReal_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isReal`.
Generated from source location line 819, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isS4`

```rust
unsafe Rf_isS4(arg1: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Check if a SEXP is an S4 object.

#### Safety

- `arg1` must be a valid SEXP

### `sys::Rf_isString`

```rust
unsafe Rf_isString(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isString`. Calls `Rf_isString_unchecked` and routes through `with_r_thread`.
Generated from source location line 827, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isString_unchecked`

```rust
unsafe Rf_isString_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isString`.
Generated from source location line 827, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isSymbol`

```rust
unsafe Rf_isSymbol(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

Checked wrapper for `Rf_isSymbol`. Calls `Rf_isSymbol_unchecked` and routes through `with_r_thread`.
Generated from source location line 815, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isSymbol_unchecked`

```rust
unsafe Rf_isSymbol_unchecked(s: SEXP) -> Rboolean
```

Unchecked FFI binding for `Rf_isSymbol`.
Generated from source location line 815, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isUnordered`

```rust
unsafe Rf_isUnordered(s: crate::sexp::SEXP) -> crate::sexp_types::Rboolean
```

 Check if a factor is unordered.
Checked wrapper for `Rf_isUnordered`. Calls `Rf_isUnordered_unchecked` and routes through `with_r_thread`.
Generated from source location line 2157, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isUnordered_unchecked`

```rust
unsafe Rf_isUnordered_unchecked(s: SEXP) -> Rboolean
```

 Check if a factor is unordered.
Unchecked FFI binding for `Rf_isUnordered`.
Generated from source location line 2157, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isUnsorted`

```rust
unsafe Rf_isUnsorted(x: crate::sexp::SEXP, strictly: crate::sexp_types::Rboolean) -> ::std::os::raw::c_int
```

 Check if a vector is unsorted.

 # Parameters

 - `x`: Vector to check
 - `strictly`: If TRUE, check for strictly increasing
Checked wrapper for `Rf_isUnsorted`. Calls `Rf_isUnsorted_unchecked` and routes through `with_r_thread`.
Generated from source location line 2166, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_isUnsorted_unchecked`

```rust
unsafe Rf_isUnsorted_unchecked(x: SEXP, strictly: Rboolean) -> ::std::os::raw::c_int
```

 Check if a vector is unsorted.

 # Parameters

 - `x`: Vector to check
 - `strictly`: If TRUE, check for strictly increasing
Unchecked FFI binding for `Rf_isUnsorted`.
Generated from source location line 2166, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_lang1`

```rust
unsafe Rf_lang1(s: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a language object (call) with 1 element (the function).

Rust equivalent of R's inline `Rf_lang1(s)`.
Creates a call like `f()` where `s` is the function.

#### Safety

- `s` must be a valid SEXP (typically a symbol or closure)
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_lang2`

```rust
unsafe Rf_lang2(s: crate::sexp::SEXP, t: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a language object (call) with function and 1 argument.

Rust equivalent of R's inline `Rf_lang2(s, t)`.
Creates a call like `f(arg)` where `s` is the function and `t` is the argument.

#### Safety

- Both SEXPs must be valid
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_lang3`

```rust
unsafe Rf_lang3(s: crate::sexp::SEXP, t: crate::sexp::SEXP, u: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a language object (call) with function and 2 arguments.

Rust equivalent of R's inline `Rf_lang3(s, t, u)`.
Creates a call like `f(arg1, arg2)`.

#### Safety

- All SEXPs must be valid
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_lang4`

```rust
unsafe Rf_lang4(s: crate::sexp::SEXP, t: crate::sexp::SEXP, u: crate::sexp::SEXP, v: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a language object (call) with function and 3 arguments.

Rust equivalent of R's inline `Rf_lang4(s, t, u, v)`.
Creates a call like `f(arg1, arg2, arg3)`.

#### Safety

- All SEXPs must be valid
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_lang5`

```rust
unsafe Rf_lang5(s: crate::sexp::SEXP, t: crate::sexp::SEXP, u: crate::sexp::SEXP, v: crate::sexp::SEXP, w: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a language object (call) with function and 4 arguments.

Rust equivalent of R's inline `Rf_lang5(s, t, u, v, w)`.
Creates a call like `f(arg1, arg2, arg3, arg4)`.

#### Safety

- All SEXPs must be valid
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_lang6`

```rust
unsafe Rf_lang6(s: crate::sexp::SEXP, t: crate::sexp::SEXP, u: crate::sexp::SEXP, v: crate::sexp::SEXP, w: crate::sexp::SEXP, x: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a language object (call) with function and 5 arguments.

Rust equivalent of R's inline `Rf_lang6(s, t, u, v, w, x)`.
Creates a call like `f(arg1, arg2, arg3, arg4, arg5)`.

#### Safety

- All SEXPs must be valid
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_lastElt`

```rust
unsafe Rf_lastElt(list: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_lastElt`. Calls `Rf_lastElt_unchecked` and routes through `with_r_thread`.
Generated from source location line 859, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_lastElt_unchecked`

```rust
unsafe Rf_lastElt_unchecked(list: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_lastElt`.
Generated from source location line 859, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_lcons`

```rust
unsafe Rf_lcons(car: crate::sexp::SEXP, cdr: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_lcons`. Calls `Rf_lcons_unchecked` and routes through `with_r_thread`.
Generated from source location line 508, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_lcons_unchecked`

```rust
unsafe Rf_lcons_unchecked(car: SEXP, cdr: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_lcons`.
Generated from source location line 508, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_lengthgets`

```rust
unsafe Rf_lengthgets(x: crate::sexp::SEXP, newlen: crate::sexp_types::R_xlen_t) -> crate::sexp::SEXP
```

 Set vector length.

 For short vectors (length < 2^31).
Checked wrapper for `Rf_lengthgets`. Calls `Rf_lengthgets_unchecked` and routes through `with_r_thread`.
Generated from source location line 2180, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_lengthgets_unchecked`

```rust
unsafe Rf_lengthgets_unchecked(x: SEXP, newlen: R_xlen_t) -> SEXP
```

 Set vector length.

 For short vectors (length < 2^31).
Unchecked FFI binding for `Rf_lengthgets`.
Generated from source location line 2180, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_list1`

```rust
unsafe Rf_list1(s: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a pairlist with 1 element.

Rust equivalent of R's inline `Rf_list1(s)`.

#### Safety

- `s` must be a valid SEXP
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_list2`

```rust
unsafe Rf_list2(s: crate::sexp::SEXP, t: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a pairlist with 2 elements.

Rust equivalent of R's inline `Rf_list2(s, t)`.

#### Safety

- Both SEXPs must be valid
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_list3`

```rust
unsafe Rf_list3(s: crate::sexp::SEXP, t: crate::sexp::SEXP, u: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a pairlist with 3 elements.

Rust equivalent of R's inline `Rf_list3(s, t, u)`.

#### Safety

- All SEXPs must be valid
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_list4`

```rust
unsafe Rf_list4(s: crate::sexp::SEXP, t: crate::sexp::SEXP, u: crate::sexp::SEXP, v: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Build a pairlist with 4 elements.

Rust equivalent of R's inline `Rf_list4(s, t, u, v)`.

#### Safety

- All SEXPs must be valid
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_listAppend`

```rust
unsafe Rf_listAppend(s: crate::sexp::SEXP, t: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_listAppend`. Calls `Rf_listAppend_unchecked` and routes through `with_r_thread`.
Generated from source location line 863, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_listAppend_unchecked`

```rust
unsafe Rf_listAppend_unchecked(s: SEXP, t: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_listAppend`.
Generated from source location line 863, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_match`

```rust
unsafe Rf_match(x: crate::sexp::SEXP, table: crate::sexp::SEXP, nomatch: ::std::os::raw::c_int) -> crate::sexp::SEXP
```

 Match elements of first vector in second vector.

 Like R's `match()` function.

 # Parameters

 - `x`: Vector of values to match
 - `table`: Vector to match against
 - `nomatch`: Value to return for non-matches

 # Returns

 Integer vector of match positions (1-indexed, nomatch for non-matches).
Checked wrapper for `Rf_match`. Calls `Rf_match_unchecked` and routes through `with_r_thread`.
Generated from source location line 2087, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_match_unchecked`

```rust
unsafe Rf_match_unchecked(x: SEXP, table: SEXP, nomatch: ::std::os::raw::c_int) -> SEXP
```

 Match elements of first vector in second vector.

 Like R's `match()` function.

 # Parameters

 - `x`: Vector of values to match
 - `table`: Vector to match against
 - `nomatch`: Value to return for non-matches

 # Returns

 Integer vector of match positions (1-indexed, nomatch for non-matches).
Unchecked FFI binding for `Rf_match`.
Generated from source location line 2087, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_mkChar`

```rust
unsafe Rf_mkChar(s: *const ::std::os::raw::c_char) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_mkChar`. Calls `Rf_mkChar_unchecked` and routes through `with_r_thread`.
Generated from source location line 307, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_mkCharLen`

```rust
unsafe Rf_mkCharLen(s: *const ::std::os::raw::c_char, len: i32) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_mkCharLen`. Calls `Rf_mkCharLen_unchecked` and routes through `with_r_thread`.
Generated from source location line 309, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_mkCharLenCE`

```rust
unsafe Rf_mkCharLenCE(x: *const ::std::os::raw::c_char, len: ::std::os::raw::c_int, ce: crate::sexp_types::cetype_t) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_mkCharLenCE`. Calls `Rf_mkCharLenCE_unchecked` and routes through `with_r_thread`.
Generated from source location line 311, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_mkCharLenCE_unchecked`

```rust
unsafe Rf_mkCharLenCE_unchecked(x: *const ::std::os::raw::c_char, len: ::std::os::raw::c_int, ce: cetype_t) -> SEXP
```

Unchecked FFI binding for `Rf_mkCharLenCE`.
Generated from source location line 311, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_mkCharLen_unchecked`

```rust
unsafe Rf_mkCharLen_unchecked(s: *const ::std::os::raw::c_char, len: i32) -> SEXP
```

Unchecked FFI binding for `Rf_mkCharLen`.
Generated from source location line 309, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_mkChar_unchecked`

```rust
unsafe Rf_mkChar_unchecked(s: *const ::std::os::raw::c_char) -> SEXP
```

Unchecked FFI binding for `Rf_mkChar`.
Generated from source location line 307, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_mkString`

```rust
unsafe Rf_mkString(s: *const ::std::os::raw::c_char) -> crate::sexp::SEXP
```

Create a length-1 string vector from a C string.

Rust equivalent of R's inline `Rf_mkString(s)`, which is
shorthand for `ScalarString(mkChar(s))`.

#### Safety

- `s` must be a valid null-terminated C string
- Must be called from R's main thread
- Result must be protected from GC

### `sys::Rf_namesgets`

```rust
unsafe Rf_namesgets(vec: crate::sexp::SEXP, val: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Set the `names` attribute; returns the updated object.
Checked wrapper for `Rf_namesgets`. Calls `Rf_namesgets_unchecked` and routes through `with_r_thread`.
Generated from source location line 738, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_namesgets_unchecked`

```rust
unsafe Rf_namesgets_unchecked(vec: SEXP, val: SEXP) -> SEXP
```

 Set the `names` attribute; returns the updated object.
Unchecked FFI binding for `Rf_namesgets`.
Generated from source location line 738, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ncols`

```rust
unsafe Rf_ncols(x: crate::sexp::SEXP) -> ::std::os::raw::c_int
```

Checked wrapper for `Rf_ncols`. Calls `Rf_ncols_unchecked` and routes through `with_r_thread`.
Generated from source location line 805, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_ncols_unchecked`

```rust
unsafe Rf_ncols_unchecked(x: SEXP) -> ::std::os::raw::c_int
```

Unchecked FFI binding for `Rf_ncols`.
Generated from source location line 805, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_nrows`

```rust
unsafe Rf_nrows(x: crate::sexp::SEXP) -> ::std::os::raw::c_int
```

Checked wrapper for `Rf_nrows`. Calls `Rf_nrows_unchecked` and routes through `with_r_thread`.
Generated from source location line 803, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_nrows_unchecked`

```rust
unsafe Rf_nrows_unchecked(x: SEXP) -> ::std::os::raw::c_int
```

Unchecked FFI binding for `Rf_nrows`.
Generated from source location line 803, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_nthcdr`

```rust
unsafe Rf_nthcdr(list: crate::sexp::SEXP, n: ::std::os::raw::c_int) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_nthcdr`. Calls `Rf_nthcdr_unchecked` and routes through `with_r_thread`.
Generated from source location line 861, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_nthcdr_unchecked`

```rust
unsafe Rf_nthcdr_unchecked(list: SEXP, n: ::std::os::raw::c_int) -> SEXP
```

Unchecked FFI binding for `Rf_nthcdr`.
Generated from source location line 861, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_protect`

```rust
unsafe Rf_protect(s: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Add a SEXP to the protect stack, preventing GC collection.

 **Cost: O(1)** — single array write (`R_PPStack[top++] = s`). No allocation.

 Must be balanced by a corresponding `Rf_unprotect`. The protect stack is
 LIFO — nested scopes are safe, but interleaved usage from different scopes
 will cause incorrect unprotection.
Checked wrapper for `Rf_protect`. Calls `Rf_protect_unchecked` and routes through `with_r_thread`.
Generated from source location line 435, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_protect_unchecked`

```rust
unsafe Rf_protect_unchecked(s: SEXP) -> SEXP
```

 Add a SEXP to the protect stack, preventing GC collection.

 **Cost: O(1)** — single array write (`R_PPStack[top++] = s`). No allocation.

 Must be balanced by a corresponding `Rf_unprotect`. The protect stack is
 LIFO — nested scopes are safe, but interleaved usage from different scopes
 will cause incorrect unprotection.
Unchecked FFI binding for `Rf_protect`.
Generated from source location line 435, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_setAttrib`

```rust
unsafe Rf_setAttrib(vec: crate::sexp::SEXP, name: crate::sexp::SEXP, val: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_setAttrib`. Calls `Rf_setAttrib_unchecked` and routes through `with_r_thread`.
Generated from source location line 512, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_setAttrib_unchecked`

```rust
unsafe Rf_setAttrib_unchecked(vec: SEXP, name: SEXP, val: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_setAttrib`.
Generated from source location line 512, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_setVar`

```rust
unsafe Rf_setVar(symbol: crate::sexp::SEXP, value: crate::sexp::SEXP, rho: crate::sexp::SEXP)
```

Checked wrapper for `Rf_setVar`. Calls `Rf_setVar_unchecked` and routes through `with_r_thread`.
Generated from source location line 907, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_setVar_unchecked`

```rust
unsafe Rf_setVar_unchecked(symbol: SEXP, value: SEXP, rho: SEXP)
```

Unchecked FFI binding for `Rf_setVar`.
Generated from source location line 907, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_shallow_duplicate`

```rust
unsafe Rf_shallow_duplicate(s: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `Rf_shallow_duplicate`. Calls `Rf_shallow_duplicate_unchecked` and routes through `with_r_thread`.
Generated from source location line 747, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_shallow_duplicate_unchecked`

```rust
unsafe Rf_shallow_duplicate_unchecked(s: SEXP) -> SEXP
```

Unchecked FFI binding for `Rf_shallow_duplicate`.
Generated from source location line 747, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_substitute`

```rust
unsafe Rf_substitute(lang: crate::sexp::SEXP, rho: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Substitute in an expression.

 Like R's `substitute()` function.
Checked wrapper for `Rf_substitute`. Calls `Rf_substitute_unchecked` and routes through `with_r_thread`.
Generated from source location line 2174, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_substitute_unchecked`

```rust
unsafe Rf_substitute_unchecked(lang: SEXP, rho: SEXP) -> SEXP
```

 Substitute in an expression.

 Like R's `substitute()` function.
Unchecked FFI binding for `Rf_substitute`.
Generated from source location line 2174, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_topenv`

```rust
unsafe Rf_topenv(target: crate::sexp::SEXP, envir: crate::sexp::SEXP) -> crate::sexp::SEXP
```

 Get the top-level environment.

 Walks up enclosing environments until reaching a top-level env
 (global, namespace, or base).
Checked wrapper for `Rf_topenv`. Calls `Rf_topenv_unchecked` and routes through `with_r_thread`.
Generated from source location line 2069, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_topenv_unchecked`

```rust
unsafe Rf_topenv_unchecked(target: SEXP, envir: SEXP) -> SEXP
```

 Get the top-level environment.

 Walks up enclosing environments until reaching a top-level env
 (global, namespace, or base).
Unchecked FFI binding for `Rf_topenv`.
Generated from source location line 2069, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_translateCharUTF8`

```rust
unsafe Rf_translateCharUTF8(x: crate::sexp::SEXP) -> *const ::std::os::raw::c_char
```

Checked wrapper for `Rf_translateCharUTF8`. Calls `Rf_translateCharUTF8_unchecked` and routes through `with_r_thread`.
Generated from source location line 320, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_translateCharUTF8_unchecked`

```rust
unsafe Rf_translateCharUTF8_unchecked(x: SEXP) -> *const ::std::os::raw::c_char
```

Unchecked FFI binding for `Rf_translateCharUTF8`.
Generated from source location line 320, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_type2char`

```rust
unsafe Rf_type2char(sexptype: crate::sexp_types::SEXPTYPE) -> *const ::std::os::raw::c_char
```

 Convert SEXPTYPE to C string name.

 Returns a string like "INTSXP", "REALSXP", etc.
Checked wrapper for `Rf_type2char`. Calls `Rf_type2char_unchecked` and routes through `with_r_thread`.
Generated from source location line 2032, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_type2char_unchecked`

```rust
unsafe Rf_type2char_unchecked(sexptype: SEXPTYPE) -> *const ::std::os::raw::c_char
```

 Convert SEXPTYPE to C string name.

 Returns a string like "INTSXP", "REALSXP", etc.
Unchecked FFI binding for `Rf_type2char`.
Generated from source location line 2032, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_unprotect`

```rust
unsafe Rf_unprotect(l: ::std::os::raw::c_int)
```

 Pop the top `l` entries from the protect stack.

 **Cost: O(1)** — single integer subtract (`R_PPStackTop -= l`). No allocation.

 The popped SEXPs become eligible for GC. Must match the number of
 `Rf_protect` calls in the current scope (LIFO order).
Checked wrapper for `Rf_unprotect`. Calls `Rf_unprotect_unchecked` and routes through `with_r_thread`.
Generated from source location line 445, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_unprotect_ptr`

```rust
unsafe Rf_unprotect_ptr(s: crate::sexp::SEXP)
```

 Remove a specific SEXP from anywhere in the protect stack.

 **Cost: O(k)** — scans backwards from top (k = distance from top), then
 shifts remaining entries down. No allocation. R source comment:
 *"should be among the top few items"*.

 Unlike `Rf_unprotect`, this is order-independent — it finds and removes
 the specific pointer regardless of stack position. Useful when LIFO
 discipline cannot be maintained, but more expensive than `Rf_unprotect`.
Checked wrapper for `Rf_unprotect_ptr`. Calls `Rf_unprotect_ptr_unchecked` and routes through `with_r_thread`.
Generated from source location line 457, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_unprotect_ptr_unchecked`

```rust
unsafe Rf_unprotect_ptr_unchecked(s: SEXP)
```

 Remove a specific SEXP from anywhere in the protect stack.

 **Cost: O(k)** — scans backwards from top (k = distance from top), then
 shifts remaining entries down. No allocation. R source comment:
 *"should be among the top few items"*.

 Unlike `Rf_unprotect`, this is order-independent — it finds and removes
 the specific pointer regardless of stack position. Useful when LIFO
 discipline cannot be maintained, but more expensive than `Rf_unprotect`.
Unchecked FFI binding for `Rf_unprotect_ptr`.
Generated from source location line 457, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_unprotect_unchecked`

```rust
unsafe Rf_unprotect_unchecked(l: ::std::os::raw::c_int)
```

 Pop the top `l` entries from the protect stack.

 **Cost: O(1)** — single integer subtract (`R_PPStackTop -= l`). No allocation.

 The popped SEXPs become eligible for GC. Must match the number of
 `Rf_protect` calls in the current scope (LIFO order).
Unchecked FFI binding for `Rf_unprotect`.
Generated from source location line 445, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_warning`

```rust
unsafe Rf_warning(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char)
```

Checked wrapper for `Rf_warning` - panics if called from non-main thread.

#### Safety

- Must be called from the R main thread
- `fmt` and `arg1` must be valid null-terminated C strings

### `sys::Rf_warning_unchecked`

```rust
unsafe Rf_warning_unchecked(arg1: *const ::std::os::raw::c_char)
```

Unchecked variadic `Rf_warning`; call checked wrapper when possible.

### `sys::Rf_xlength`

```rust
unsafe Rf_xlength(x: crate::sexp::SEXP) -> crate::sexp_types::R_xlen_t
```

Checked wrapper for `Rf_xlength`. Calls `Rf_xlength_unchecked` and routes through `with_r_thread`.
Generated from source location line 318, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_xlength_unchecked`

```rust
unsafe Rf_xlength_unchecked(x: SEXP) -> R_xlen_t
```

Unchecked FFI binding for `Rf_xlength`.
Generated from source location line 318, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_xlengthgets`

```rust
unsafe Rf_xlengthgets(x: crate::sexp::SEXP, newlen: crate::sexp_types::R_xlen_t) -> crate::sexp::SEXP
```

 Set vector length (long vector version).
Checked wrapper for `Rf_xlengthgets`. Calls `Rf_xlengthgets_unchecked` and routes through `with_r_thread`.
Generated from source location line 2184, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rf_xlengthgets_unchecked`

```rust
unsafe Rf_xlengthgets_unchecked(x: SEXP, newlen: R_xlen_t) -> SEXP
```

 Set vector length (long vector version).
Unchecked FFI binding for `Rf_xlengthgets`.
Generated from source location line 2184, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::Rprintf`

```rust
unsafe Rprintf(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char)
```

Checked wrapper for `Rprintf` - panics if called from non-main thread.

#### Safety

- Must be called from the R main thread
- `fmt` and `arg1` must be valid null-terminated C strings

### `sys::Rprintf_unchecked`

```rust
unsafe Rprintf_unchecked(arg1: *const ::std::os::raw::c_char)
```

Unchecked variadic `Rprintf`; call checked wrapper when possible.

### `sys::SETCAD4R`

```rust
unsafe SETCAD4R(e: crate::sexp::SEXP, y: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `SETCAD4R`. Calls `SETCAD4R_unchecked` and routes through `with_r_thread`.
Generated from source location line 578, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCAD4R_unchecked`

```rust
unsafe SETCAD4R_unchecked(e: SEXP, y: SEXP) -> SEXP
```

Unchecked FFI binding for `SETCAD4R`.
Generated from source location line 578, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCADDDR`

```rust
unsafe SETCADDDR(x: crate::sexp::SEXP, y: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `SETCADDDR`. Calls `SETCADDDR_unchecked` and routes through `with_r_thread`.
Generated from source location line 577, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCADDDR_unchecked`

```rust
unsafe SETCADDDR_unchecked(x: SEXP, y: SEXP) -> SEXP
```

Unchecked FFI binding for `SETCADDDR`.
Generated from source location line 577, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCADDR`

```rust
unsafe SETCADDR(x: crate::sexp::SEXP, y: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `SETCADDR`. Calls `SETCADDR_unchecked` and routes through `with_r_thread`.
Generated from source location line 576, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCADDR_unchecked`

```rust
unsafe SETCADDR_unchecked(x: SEXP, y: SEXP) -> SEXP
```

Unchecked FFI binding for `SETCADDR`.
Generated from source location line 576, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCADR`

```rust
unsafe SETCADR(x: crate::sexp::SEXP, y: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `SETCADR`. Calls `SETCADR_unchecked` and routes through `with_r_thread`.
Generated from source location line 575, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCADR_unchecked`

```rust
unsafe SETCADR_unchecked(x: SEXP, y: SEXP) -> SEXP
```

Unchecked FFI binding for `SETCADR`.
Generated from source location line 575, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCAR`

```rust
unsafe SETCAR(x: crate::sexp::SEXP, y: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `SETCAR`. Calls `SETCAR_unchecked` and routes through `with_r_thread`.
Generated from source location line 573, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCAR_unchecked`

```rust
unsafe SETCAR_unchecked(x: SEXP, y: SEXP) -> SEXP
```

Unchecked FFI binding for `SETCAR`.
Generated from source location line 573, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCDR`

```rust
unsafe SETCDR(x: crate::sexp::SEXP, y: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `SETCDR`. Calls `SETCDR_unchecked` and routes through `with_r_thread`.
Generated from source location line 574, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETCDR_unchecked`

```rust
unsafe SETCDR_unchecked(x: SEXP, y: SEXP) -> SEXP
```

Unchecked FFI binding for `SETCDR`.
Generated from source location line 574, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETLEVELS`

```rust
unsafe SETLEVELS(x: crate::sexp::SEXP, v: ::std::os::raw::c_int) -> ::std::os::raw::c_int
```

 Set the LEVELS field (for factors).

 Returns the value that was set.
Checked wrapper for `SETLEVELS`. Calls `SETLEVELS_unchecked` and routes through `with_r_thread`.
Generated from source location line 648, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SETLEVELS_unchecked`

```rust
unsafe SETLEVELS_unchecked(x: SEXP, v: ::std::os::raw::c_int) -> ::std::os::raw::c_int
```

 Set the LEVELS field (for factors).

 Returns the value that was set.
Unchecked FFI binding for `SETLEVELS`.
Generated from source location line 648, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_ATTRIB`

```rust
unsafe SET_ATTRIB(x: crate::sexp::SEXP, v: crate::sexp::SEXP)
```

 Set the attributes pairlist of a SEXP.

 # Safety

 `v` must be a pairlist or R_NilValue
Checked wrapper for `SET_ATTRIB`. Calls `SET_ATTRIB_unchecked` and routes through `with_r_thread`.
Generated from source location line 632, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_ATTRIB_unchecked`

```rust
unsafe SET_ATTRIB_unchecked(x: SEXP, v: SEXP)
```

 Set the attributes pairlist of a SEXP.

 # Safety

 `v` must be a pairlist or R_NilValue
Unchecked FFI binding for `SET_ATTRIB`.
Generated from source location line 632, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_COMPLEX_ELT`

```rust
unsafe SET_COMPLEX_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t, v: crate::sexp_types::Rcomplex)
```

Checked wrapper for `SET_COMPLEX_ELT`. Calls `SET_COMPLEX_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 597, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_COMPLEX_ELT_unchecked`

```rust
unsafe SET_COMPLEX_ELT_unchecked(x: SEXP, i: R_xlen_t, v: Rcomplex)
```

Unchecked FFI binding for `SET_COMPLEX_ELT`.
Generated from source location line 597, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_INTEGER_ELT`

```rust
unsafe SET_INTEGER_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t, v: ::std::os::raw::c_int)
```

Checked wrapper for `SET_INTEGER_ELT`. Calls `SET_INTEGER_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 595, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_INTEGER_ELT_unchecked`

```rust
unsafe SET_INTEGER_ELT_unchecked(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int)
```

Unchecked FFI binding for `SET_INTEGER_ELT`.
Generated from source location line 595, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_LOGICAL_ELT`

```rust
unsafe SET_LOGICAL_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t, v: ::std::os::raw::c_int)
```

Checked wrapper for `SET_LOGICAL_ELT`. Calls `SET_LOGICAL_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 594, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_LOGICAL_ELT_unchecked`

```rust
unsafe SET_LOGICAL_ELT_unchecked(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int)
```

Unchecked FFI binding for `SET_LOGICAL_ELT`.
Generated from source location line 594, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_OBJECT`

```rust
unsafe SET_OBJECT(x: crate::sexp::SEXP, v: ::std::os::raw::c_int)
```

 Set the "object" bit.
Checked wrapper for `SET_OBJECT`. Calls `SET_OBJECT_unchecked` and routes through `with_r_thread`.
Generated from source location line 640, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_OBJECT_unchecked`

```rust
unsafe SET_OBJECT_unchecked(x: SEXP, v: ::std::os::raw::c_int)
```

 Set the "object" bit.
Unchecked FFI binding for `SET_OBJECT`.
Generated from source location line 640, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_RAW_ELT`

```rust
unsafe SET_RAW_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t, v: crate::sexp_types::Rbyte)
```

Checked wrapper for `SET_RAW_ELT`. Calls `SET_RAW_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 598, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_RAW_ELT_unchecked`

```rust
unsafe SET_RAW_ELT_unchecked(x: SEXP, i: R_xlen_t, v: Rbyte)
```

Unchecked FFI binding for `SET_RAW_ELT`.
Generated from source location line 598, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_REAL_ELT`

```rust
unsafe SET_REAL_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t, v: f64)
```

Checked wrapper for `SET_REAL_ELT`. Calls `SET_REAL_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 596, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_REAL_ELT_unchecked`

```rust
unsafe SET_REAL_ELT_unchecked(x: SEXP, i: R_xlen_t, v: f64)
```

Unchecked FFI binding for `SET_REAL_ELT`.
Generated from source location line 596, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_STRING_ELT`

```rust
unsafe SET_STRING_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t, v: crate::sexp::SEXP)
```

Checked wrapper for `SET_STRING_ELT`. Calls `SET_STRING_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 593, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_STRING_ELT_unchecked`

```rust
unsafe SET_STRING_ELT_unchecked(x: SEXP, i: R_xlen_t, v: SEXP)
```

Unchecked FFI binding for `SET_STRING_ELT`.
Generated from source location line 593, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_TAG`

```rust
unsafe SET_TAG(x: crate::sexp::SEXP, y: crate::sexp::SEXP)
```

Checked wrapper for `SET_TAG`. Calls `SET_TAG_unchecked` and routes through `with_r_thread`.
Generated from source location line 572, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_TAG_unchecked`

```rust
unsafe SET_TAG_unchecked(x: SEXP, y: SEXP)
```

Unchecked FFI binding for `SET_TAG`.
Generated from source location line 572, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_VECTOR_ELT`

```rust
unsafe SET_VECTOR_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t, v: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `SET_VECTOR_ELT`. Calls `SET_VECTOR_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 599, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::SET_VECTOR_ELT_unchecked`

```rust
unsafe SET_VECTOR_ELT_unchecked(x: SEXP, i: R_xlen_t, v: SEXP) -> SEXP
```

Unchecked FFI binding for `SET_VECTOR_ELT`.
Generated from source location line 599, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::STRING_ELT`

```rust
unsafe STRING_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t) -> crate::sexp::SEXP
```

Checked wrapper for `STRING_ELT`. Calls `STRING_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 592, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::STRING_ELT_unchecked`

```rust
unsafe STRING_ELT_unchecked(x: SEXP, i: R_xlen_t) -> SEXP
```

Unchecked FFI binding for `STRING_ELT`.
Generated from source location line 592, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::S_alloc`

```rust
unsafe S_alloc(nelem: ::std::os::raw::c_long, eltsize: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char
```

 S compatibility: allocate zeroed memory on R's memory stack.

 # Parameters

 - `nelem`: Number of elements
 - `eltsize`: Size of each element
Checked wrapper for `S_alloc`. Calls `S_alloc_unchecked` and routes through `with_r_thread`.
Generated from source location line 1676, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::S_alloc_unchecked`

```rust
unsafe S_alloc_unchecked(nelem: ::std::os::raw::c_long, eltsize: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char
```

 S compatibility: allocate zeroed memory on R's memory stack.

 # Parameters

 - `nelem`: Number of elements
 - `eltsize`: Size of each element
Unchecked FFI binding for `S_alloc`.
Generated from source location line 1676, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::S_realloc`

```rust
unsafe S_realloc(ptr: *mut ::std::os::raw::c_char, newsize: ::std::os::raw::c_long, oldsize: ::std::os::raw::c_long, eltsize: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char
```

 S compatibility: reallocate memory on R's memory stack.

 # Safety

 `ptr` must have been allocated by `S_alloc`.
Checked wrapper for `S_realloc`. Calls `S_realloc_unchecked` and routes through `with_r_thread`.
Generated from source location line 1686, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::S_realloc_unchecked`

```rust
unsafe S_realloc_unchecked(ptr: *mut ::std::os::raw::c_char, newsize: ::std::os::raw::c_long, oldsize: ::std::os::raw::c_long, eltsize: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char
```

 S compatibility: reallocate memory on R's memory stack.

 # Safety

 `ptr` must have been allocated by `S_alloc`.
Unchecked FFI binding for `S_realloc`.
Generated from source location line 1686, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::StringFalse`

```rust
unsafe StringFalse(s: *const ::std::os::raw::c_char) -> crate::sexp_types::Rboolean
```

 Check if a string represents FALSE in R.

 Recognizes "FALSE", "false", "False", "F", "f", etc.
Checked wrapper for `StringFalse`. Calls `StringFalse_unchecked` and routes through `with_r_thread`.
Generated from source location line 1976, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::StringFalse_unchecked`

```rust
unsafe StringFalse_unchecked(s: *const ::std::os::raw::c_char) -> Rboolean
```

 Check if a string represents FALSE in R.

 Recognizes "FALSE", "false", "False", "F", "f", etc.
Unchecked FFI binding for `StringFalse`.
Generated from source location line 1976, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::StringTrue`

```rust
unsafe StringTrue(s: *const ::std::os::raw::c_char) -> crate::sexp_types::Rboolean
```

 Check if a string represents TRUE in R.

 Recognizes "TRUE", "true", "True", "T", "t", etc.
Checked wrapper for `StringTrue`. Calls `StringTrue_unchecked` and routes through `with_r_thread`.
Generated from source location line 1982, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::StringTrue_unchecked`

```rust
unsafe StringTrue_unchecked(s: *const ::std::os::raw::c_char) -> Rboolean
```

 Check if a string represents TRUE in R.

 Recognizes "TRUE", "true", "True", "T", "t", etc.
Unchecked FFI binding for `StringTrue`.
Generated from source location line 1982, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::TAG`

```rust
unsafe TAG(e: crate::sexp::SEXP) -> crate::sexp::SEXP
```

Checked wrapper for `TAG`. Calls `TAG_unchecked` and routes through `with_r_thread`.
Generated from source location line 571, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::TAG_unchecked`

```rust
unsafe TAG_unchecked(e: SEXP) -> SEXP
```

Unchecked FFI binding for `TAG`.
Generated from source location line 571, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::TRUELENGTH`

```rust
unsafe TRUELENGTH(x: crate::sexp::SEXP) -> crate::sexp_types::R_xlen_t
```

 Get the true length (allocated capacity) of a vector.

 May be larger than LENGTH for vectors with reserved space.
 ALTREP-aware.
Checked wrapper for `TRUELENGTH`. Calls `TRUELENGTH_unchecked` and routes through `with_r_thread`.
Generated from source location line 620, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::TRUELENGTH_unchecked`

```rust
unsafe TRUELENGTH_unchecked(x: SEXP) -> R_xlen_t
```

 Get the true length (allocated capacity) of a vector.

 May be larger than LENGTH for vectors with reserved space.
 ALTREP-aware.
Unchecked FFI binding for `TRUELENGTH`.
Generated from source location line 620, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::TYPEOF`

```rust
unsafe TYPEOF(x: crate::sexp::SEXP) -> crate::sexp_types::SEXPTYPE
```

Checked wrapper for `TYPEOF`. Calls `TYPEOF_unchecked` and routes through `with_r_thread`.
Generated from source location line 716, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::TYPEOF_unchecked`

```rust
unsafe TYPEOF_unchecked(x: SEXP) -> SEXPTYPE
```

Unchecked FFI binding for `TYPEOF`.
Generated from source location line 716, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::VECTOR_ELT`

```rust
unsafe VECTOR_ELT(x: crate::sexp::SEXP, i: crate::sexp_types::R_xlen_t) -> crate::sexp::SEXP
```

Checked wrapper for `VECTOR_ELT`. Calls `VECTOR_ELT_unchecked` and routes through `with_r_thread`.
Generated from source location line 591, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::VECTOR_ELT_unchecked`

```rust
unsafe VECTOR_ELT_unchecked(x: SEXP, i: R_xlen_t) -> SEXP
```

Unchecked FFI binding for `VECTOR_ELT`.
Generated from source location line 591, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::XLENGTH`

```rust
unsafe XLENGTH(x: crate::sexp::SEXP) -> crate::sexp_types::R_xlen_t
```

 Get the length of a SEXP as `R_xlen_t` (supports long vectors).

 ALTREP-aware: will call ALTREP Length method if needed.
Checked wrapper for `XLENGTH`. Calls `XLENGTH_unchecked` and routes through `with_r_thread`.
Generated from source location line 614, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::XLENGTH_unchecked`

```rust
unsafe XLENGTH_unchecked(x: SEXP) -> R_xlen_t
```

 Get the length of a SEXP as `R_xlen_t` (supports long vectors).

 ALTREP-aware: will call ALTREP Length method if needed.
Unchecked FFI binding for `XLENGTH`.
Generated from source location line 614, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::altrep::R_make_altcomplex_class`

```rust
unsafe R_make_altcomplex_class(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut crate::sys::DllInfo) -> R_altrep_class_t
```

Checked wrapper for `R_make_altcomplex_class`. Calls `R_make_altcomplex_class_unchecked` and routes through `with_r_thread`.
Generated from source location line 233, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altcomplex_class_unchecked`

```rust
unsafe R_make_altcomplex_class_unchecked(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut DllInfo) -> R_altrep_class_t
```

Unchecked FFI binding for `R_make_altcomplex_class`.
Generated from source location line 233, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altinteger_class`

```rust
unsafe R_make_altinteger_class(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut crate::sys::DllInfo) -> R_altrep_class_t
```

Checked wrapper for `R_make_altinteger_class`. Calls `R_make_altinteger_class_unchecked` and routes through `with_r_thread`.
Generated from source location line 213, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altinteger_class_unchecked`

```rust
unsafe R_make_altinteger_class_unchecked(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut DllInfo) -> R_altrep_class_t
```

Unchecked FFI binding for `R_make_altinteger_class`.
Generated from source location line 213, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altlist_class`

```rust
unsafe R_make_altlist_class(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut crate::sys::DllInfo) -> R_altrep_class_t
```

Checked wrapper for `R_make_altlist_class`. Calls `R_make_altlist_class_unchecked` and routes through `with_r_thread`.
Generated from source location line 238, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altlist_class_unchecked`

```rust
unsafe R_make_altlist_class_unchecked(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut DllInfo) -> R_altrep_class_t
```

Unchecked FFI binding for `R_make_altlist_class`.
Generated from source location line 238, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altlogical_class`

```rust
unsafe R_make_altlogical_class(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut crate::sys::DllInfo) -> R_altrep_class_t
```

Checked wrapper for `R_make_altlogical_class`. Calls `R_make_altlogical_class_unchecked` and routes through `with_r_thread`.
Generated from source location line 223, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altlogical_class_unchecked`

```rust
unsafe R_make_altlogical_class_unchecked(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut DllInfo) -> R_altrep_class_t
```

Unchecked FFI binding for `R_make_altlogical_class`.
Generated from source location line 223, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altraw_class`

```rust
unsafe R_make_altraw_class(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut crate::sys::DllInfo) -> R_altrep_class_t
```

Checked wrapper for `R_make_altraw_class`. Calls `R_make_altraw_class_unchecked` and routes through `with_r_thread`.
Generated from source location line 228, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altraw_class_unchecked`

```rust
unsafe R_make_altraw_class_unchecked(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut DllInfo) -> R_altrep_class_t
```

Unchecked FFI binding for `R_make_altraw_class`.
Generated from source location line 228, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altreal_class`

```rust
unsafe R_make_altreal_class(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut crate::sys::DllInfo) -> R_altrep_class_t
```

Checked wrapper for `R_make_altreal_class`. Calls `R_make_altreal_class_unchecked` and routes through `with_r_thread`.
Generated from source location line 218, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altreal_class_unchecked`

```rust
unsafe R_make_altreal_class_unchecked(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut DllInfo) -> R_altrep_class_t
```

Unchecked FFI binding for `R_make_altreal_class`.
Generated from source location line 218, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altstring_class`

```rust
unsafe R_make_altstring_class(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut crate::sys::DllInfo) -> R_altrep_class_t
```

Checked wrapper for `R_make_altstring_class`. Calls `R_make_altstring_class_unchecked` and routes through `with_r_thread`.
Generated from source location line 208, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::R_make_altstring_class_unchecked`

```rust
unsafe R_make_altstring_class_unchecked(cname: *const ::std::os::raw::c_char, pname: *const ::std::os::raw::c_char, info: *mut DllInfo) -> R_altrep_class_t
```

Unchecked FFI binding for `R_make_altstring_class`.
Generated from source location line 208, column 12.
Generated from source file `miniextendr-api/src/sys/altrep.rs`.

### `sys::altrep::sexp`

```rust
sexp(class: R_altrep_class_t) -> crate::SEXP
```

Extracts the `ptr` field from `R_altrep_class_t`.

Rust equivalent of the C macro `R_SEXP(x)` which expands to `(x).ptr`.

### `sys::altrep::subtype_init`

```rust
const subtype_init(ptr: crate::SEXP) -> R_altrep_class_t
```

Creates an `R_altrep_class_t` from a SEXP pointer.

Rust equivalent of the C macro `R_SUBTYPE_INIT(x)` which expands to `{ x }`.

### `sys::cPsort`

```rust
unsafe cPsort(x: *mut crate::sexp_types::Rcomplex, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int)
```

 Partial sort complex numbers.

 # Parameters

 - `x`: Pointer to Rcomplex array
 - `n`: Number of elements
 - `k`: Target position (0-indexed)
Checked wrapper for `cPsort`. Calls `cPsort_unchecked` and routes through `with_r_thread`.
Generated from source location line 1800, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::cPsort_unchecked`

```rust
unsafe cPsort_unchecked(x: *mut Rcomplex, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int)
```

 Partial sort complex numbers.

 # Parameters

 - `x`: Pointer to Rcomplex array
 - `n`: Number of elements
 - `k`: Target position (0-indexed)
Unchecked FFI binding for `cPsort`.
Generated from source location line 1800, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::exp_rand`

```rust
unsafe exp_rand() -> f64
```

 Generate an exponential random number with rate 1.

 # Important

 Must call `GetRNGstate()` before and `PutRNGstate()` after.
Checked wrapper for `exp_rand`. Calls `exp_rand_unchecked` and routes through `with_r_thread`.
Generated from source location line 1591, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::exp_rand_unchecked`

```rust
unsafe exp_rand_unchecked() -> f64
```

 Generate an exponential random number with rate 1.

 # Important

 Must call `GetRNGstate()` before and `PutRNGstate()` after.
Unchecked FFI binding for `exp_rand`.
Generated from source location line 1591, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::findInterval`

```rust
unsafe findInterval(xt: *const f64, n: ::std::os::raw::c_int, x: f64, rightmost_closed: crate::sexp_types::Rboolean, all_inside: crate::sexp_types::Rboolean, ilo: ::std::os::raw::c_int, mflag: *mut ::std::os::raw::c_int) -> ::std::os::raw::c_int
```

 Find the interval containing a value (binary search).

 Used for interpolation and binning.

 # Parameters

 - `xt`: Sorted breakpoints array
 - `n`: Number of breakpoints
 - `x`: Value to find
 - `rightmost_closed`: If TRUE, rightmost interval is closed
 - `all_inside`: If TRUE, out-of-bounds values map to endpoints
 - `ilo`: Initial guess for interval (1-indexed)
 - `mflag`: Output flag (see R documentation)

 # Returns

 Interval index (1-indexed).
Checked wrapper for `findInterval`. Calls `findInterval_unchecked` and routes through `with_r_thread`.
Generated from source location line 1932, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::findInterval2`

```rust
unsafe findInterval2(xt: *const f64, n: ::std::os::raw::c_int, x: f64, rightmost_closed: crate::sexp_types::Rboolean, all_inside: crate::sexp_types::Rboolean, left_open: crate::sexp_types::Rboolean, ilo: ::std::os::raw::c_int, mflag: *mut ::std::os::raw::c_int) -> ::std::os::raw::c_int
```

 Extended interval finding with left-open option.
Checked wrapper for `findInterval2`. Calls `findInterval2_unchecked` and routes through `with_r_thread`.
Generated from source location line 1944, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::findInterval2_unchecked`

```rust
unsafe findInterval2_unchecked(xt: *const f64, n: ::std::os::raw::c_int, x: f64, rightmost_closed: Rboolean, all_inside: Rboolean, left_open: Rboolean, ilo: ::std::os::raw::c_int, mflag: *mut ::std::os::raw::c_int) -> ::std::os::raw::c_int
```

 Extended interval finding with left-open option.
Unchecked FFI binding for `findInterval2`.
Generated from source location line 1944, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::findInterval_unchecked`

```rust
unsafe findInterval_unchecked(xt: *const f64, n: ::std::os::raw::c_int, x: f64, rightmost_closed: Rboolean, all_inside: Rboolean, ilo: ::std::os::raw::c_int, mflag: *mut ::std::os::raw::c_int) -> ::std::os::raw::c_int
```

 Find the interval containing a value (binary search).

 Used for interpolation and binning.

 # Parameters

 - `xt`: Sorted breakpoints array
 - `n`: Number of breakpoints
 - `x`: Value to find
 - `rightmost_closed`: If TRUE, rightmost interval is closed
 - `all_inside`: If TRUE, out-of-bounds values map to endpoints
 - `ilo`: Initial guess for interval (1-indexed)
 - `mflag`: Output flag (see R documentation)

 # Returns

 Interval index (1-indexed).
Unchecked FFI binding for `findInterval`.
Generated from source location line 1932, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::iPsort`

```rust
unsafe iPsort(x: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int)
```

 Partial sort integers (moves k-th smallest to position k).

 # Parameters

 - `x`: Pointer to integer array
 - `n`: Number of elements
 - `k`: Target position (0-indexed)
Checked wrapper for `iPsort`. Calls `iPsort_unchecked` and routes through `with_r_thread`.
Generated from source location line 1776, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::iPsort_unchecked`

```rust
unsafe iPsort_unchecked(x: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int)
```

 Partial sort integers (moves k-th smallest to position k).

 # Parameters

 - `x`: Pointer to integer array
 - `n`: Number of elements
 - `k`: Target position (0-indexed)
Unchecked FFI binding for `iPsort`.
Generated from source location line 1776, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::isBlankString`

```rust
unsafe isBlankString(s: *const ::std::os::raw::c_char) -> crate::sexp_types::Rboolean
```

 Check if a string is blank (empty or only whitespace).
Checked wrapper for `isBlankString`. Calls `isBlankString_unchecked` and routes through `with_r_thread`.
Generated from source location line 1986, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::isBlankString_unchecked`

```rust
unsafe isBlankString_unchecked(s: *const ::std::os::raw::c_char) -> Rboolean
```

 Check if a string is blank (empty or only whitespace).
Unchecked FFI binding for `isBlankString`.
Generated from source location line 1986, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::norm_rand`

```rust
unsafe norm_rand() -> f64
```

 Generate a standard normal random number (mean 0, sd 1).

 # Important

 Must call `GetRNGstate()` before and `PutRNGstate()` after.
Checked wrapper for `norm_rand`. Calls `norm_rand_unchecked` and routes through `with_r_thread`.
Generated from source location line 1584, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::norm_rand_unchecked`

```rust
unsafe norm_rand_unchecked() -> f64
```

 Generate a standard normal random number (mean 0, sd 1).

 # Important

 Must call `GetRNGstate()` before and `PutRNGstate()` after.
Unchecked FFI binding for `norm_rand`.
Generated from source location line 1584, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::rPsort`

```rust
unsafe rPsort(x: *mut f64, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int)
```

 Partial sort doubles (moves k-th smallest to position k).

 # Parameters

 - `x`: Pointer to double array
 - `n`: Number of elements
 - `k`: Target position (0-indexed)
Checked wrapper for `rPsort`. Calls `rPsort_unchecked` and routes through `with_r_thread`.
Generated from source location line 1790, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::rPsort_unchecked`

```rust
unsafe rPsort_unchecked(x: *mut f64, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int)
```

 Partial sort doubles (moves k-th smallest to position k).

 # Parameters

 - `x`: Pointer to double array
 - `n`: Number of elements
 - `k`: Target position (0-indexed)
Unchecked FFI binding for `rPsort`.
Generated from source location line 1790, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::revsort`

```rust
unsafe revsort(a: *mut f64, ib: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int)
```

 Sort doubles in descending order, carrying along an index array.

 # Parameters

 - `a`: Pointer to double array (sorted in place, descending)
 - `ib`: Pointer to integer array (permuted alongside `a`)
 - `n`: Number of elements
Checked wrapper for `revsort`. Calls `revsort_unchecked` and routes through `with_r_thread`.
Generated from source location line 1753, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::revsort_unchecked`

```rust
unsafe revsort_unchecked(a: *mut f64, ib: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int)
```

 Sort doubles in descending order, carrying along an index array.

 # Parameters

 - `a`: Pointer to double array (sorted in place, descending)
 - `ib`: Pointer to integer array (permuted alongside `a`)
 - `n`: Number of elements
Unchecked FFI binding for `revsort`.
Generated from source location line 1753, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::rsort_with_index`

```rust
unsafe rsort_with_index(x: *mut f64, indx: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int)
```

 Sort doubles with index array.

 # Parameters

 - `x`: Pointer to double array (sorted in place)
 - `indx`: Pointer to integer array (permuted alongside `x`)
 - `n`: Number of elements
Checked wrapper for `rsort_with_index`. Calls `rsort_with_index_unchecked` and routes through `with_r_thread`.
Generated from source location line 1762, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::rsort_with_index_unchecked`

```rust
unsafe rsort_with_index_unchecked(x: *mut f64, indx: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int)
```

 Sort doubles with index array.

 # Parameters

 - `x`: Pointer to double array (sorted in place)
 - `indx`: Pointer to integer array (permuted alongside `x`)
 - `n`: Number of elements
Unchecked FFI binding for `rsort_with_index`.
Generated from source location line 1762, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::unif_rand`

```rust
unsafe unif_rand() -> f64
```

 Generate a uniform random number in (0, 1).

 # Important

 Must call `GetRNGstate()` before and `PutRNGstate()` after.
Checked wrapper for `unif_rand`. Calls `unif_rand_unchecked` and routes through `with_r_thread`.
Generated from source location line 1577, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::unif_rand_unchecked`

```rust
unsafe unif_rand_unchecked() -> f64
```

 Generate a uniform random number in (0, 1).

 # Important

 Must call `GetRNGstate()` before and `PutRNGstate()` after.
Unchecked FFI binding for `unif_rand`.
Generated from source location line 1577, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::vmaxget`

```rust
unsafe vmaxget() -> *mut ::std::os::raw::c_void
```

 Get the current R memory stack watermark.

 Use with `vmaxset()` to restore memory stack state.
 Memory allocated with `R_alloc()` between `vmaxget()` and `vmaxset()`
 will be freed when `vmaxset()` is called.

 # Example

 ```ignore
 unsafe {
     let watermark = vmaxget();
     let buf = R_alloc(100, 1);
     // ... use buf ...
     vmaxset(watermark); // frees buf
 }
 ```
Checked wrapper for `vmaxget`. Calls `vmaxget_unchecked` and routes through `with_r_thread`.
Generated from source location line 1628, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::vmaxget_unchecked`

```rust
unsafe vmaxget_unchecked() -> *mut ::std::os::raw::c_void
```

 Get the current R memory stack watermark.

 Use with `vmaxset()` to restore memory stack state.
 Memory allocated with `R_alloc()` between `vmaxget()` and `vmaxset()`
 will be freed when `vmaxset()` is called.

 # Example

 ```ignore
 unsafe {
     let watermark = vmaxget();
     let buf = R_alloc(100, 1);
     // ... use buf ...
     vmaxset(watermark); // frees buf
 }
 ```
Unchecked FFI binding for `vmaxget`.
Generated from source location line 1628, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::vmaxset`

```rust
unsafe vmaxset(ovmax: *const ::std::os::raw::c_void)
```

 Set the R memory stack watermark, freeing memory allocated since the mark.

 # Safety

 `ovmax` must be a value returned by `vmaxget()` called earlier in the
 same R evaluation context.
Checked wrapper for `vmaxset`. Calls `vmaxset_unchecked` and routes through `with_r_thread`.
Generated from source location line 1636, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `sys::vmaxset_unchecked`

```rust
unsafe vmaxset_unchecked(ovmax: *const ::std::os::raw::c_void)
```

 Set the R memory stack watermark, freeing memory allocated since the mark.

 # Safety

 `ovmax` must be a value returned by `vmaxget()` called earlier in the
 same R evaluation context.
Unchecked FFI binding for `vmaxset`.
Generated from source location line 1636, column 12.
Generated from source file `miniextendr-api/src/sys.rs`.

### `trait_abi::ccall::mx_get`

```rust
unsafe mx_get(sexp: crate::SEXP) -> *mut crate::abi::mx_erased
```

Extract an erased object pointer from an R external pointer.

Retrieves the `*mut mx_erased` stored in an R `EXTPTRSXP`.

#### Arguments

* `sexp` - R external pointer created by [`mx_wrap`]

#### Returns

Pointer to the erased object, or null if:
- `sexp` is not an external pointer
- The external pointer has been invalidated

#### Safety

- `sexp` must be a valid SEXP
- Must be called on R's main thread
- The returned pointer is only valid while R protects the SEXP

### `trait_abi::ccall::mx_query`

```rust
unsafe mx_query(sexp: crate::SEXP, tag: crate::abi::mx_tag) -> *const std::os::raw::c_void
```

Query an object for an interface vtable by tag.

Looks up whether the object implements the trait identified by `tag`,
and returns a pointer to the vtable if so.

#### Arguments

* `sexp` - R external pointer wrapping an erased object
* `tag` - Tag identifying the requested trait interface

#### Returns

- Non-null pointer to the trait's vtable if implemented
- Null pointer if:
  - `sexp` is not a valid erased object
  - The object does not implement the requested trait

#### Safety

- `sexp` must be a valid SEXP
- Must be called on R's main thread
- The returned pointer must be cast to the correct vtable type

#### Example

```ignore
let vtable = unsafe { mx_query(obj, TAG_FOO) };
if !vtable.is_null() {
    let foo_vtable = vtable.cast::<FooVTable>();
    // Call method through vtable...
}
```

### `trait_abi::ccall::mx_query_as`

```rust
unsafe mx_query_as<V>(sexp: crate::SEXP, tag: crate::abi::mx_tag) -> Option<&'static V>
```

Query an object for an interface and return a typed view.

Looks up `tag` in the object's vtable registry and, if found, returns
`Some(&V)` pointing to the data pointer + vtable pair for the trait.

#### Type Parameters

* `V` - The view type (e.g., `FooView`) containing data pointer and vtable

#### Arguments

* `sexp` - R external pointer wrapping an erased object
* `tag` - Tag identifying the requested trait interface

#### Returns

- `Some(&V)` if the object implements the trait
- `None` if the object does not implement the trait

#### Safety

- `sexp` must be a valid SEXP
- `V` must be the correct view type for `tag`
- Must be called on R's main thread

#### Example

```ignore
if let Some(view) = unsafe { mx_query_as::<FooView>(obj, TAG_FOO) } {
    let result = view.some_method(args);
} else {
    panic!("object does not implement Foo");
}
```

### `trait_abi::ccall::mx_wrap`

```rust
unsafe mx_wrap(ptr: *mut crate::abi::mx_erased) -> crate::SEXP
```

Wrap an erased object pointer in an R external pointer.

Creates an R `EXTPTRSXP` that wraps the given erased object. The external
pointer's finalizer will call the object's `drop` function when garbage
collected.

#### Arguments

* `ptr` - Pointer to erased object (must be heap-allocated)

#### Returns

R external pointer (`EXTPTRSXP`) containing the erased object.

#### Safety

- `ptr` must be a valid pointer to `mx_erased`
- `ptr` must be heap-allocated (will be freed by finalizer)
- Must be called on R's main thread
- `mx_abi_register()` must have been called (via `miniextendr_init!`)

#### Example

```ignore
// In constructor
let obj = Box::into_raw(Box::new(MyErasedWrapper::new(data)));
let sexp = unsafe { mx_wrap(obj.cast::<mx_erased>()) };
```

### `trait_abi::conv::check_arity`

```rust
unsafe check_arity(argc: i32, expected: i32, method_name: &str)
```

Check that the number of arguments matches expected arity.

#### Arguments

* `argc` - Actual number of arguments
* `expected` - Expected number of arguments
* `method_name` - Name of method (for error messages)

#### Safety

Must be called on R's main thread (may call `rf_error`).

#### Errors

Calls [`rf_error`] if `argc != expected`.

### `trait_abi::conv::extract_arg`

```rust
unsafe extract_arg<T>(argc: i32, argv: *const crate::SEXP, index: usize, name: &str) -> T
```

Extract and convert an argument from argv with bounds checking.

Checks that `index < argc` before extracting, and provides a helpful
error message if out of bounds.

#### Type Parameters

* `T` - Target Rust type (must implement [`TryFromSexp`])

#### Arguments

* `argc` - Number of arguments
* `argv` - Pointer to argument array
* `index` - Index of argument to extract
* `name` - Name of argument (for error messages)

#### Returns

The converted argument value.

#### Safety

- `argv` must point to at least `argc` valid SEXPs
- Must be called on R's main thread

#### Errors

Calls [`rf_error`] if:
- `index >= argc` (missing argument)
- Conversion fails

#### Example

```ignore
// In a method shim for fn foo(&self, x: i32, y: String)
let x: i32 = unsafe { extract_arg(argc, argv, 0, "x") };
let y: String = unsafe { extract_arg(argc, argv, 1, "y") };
```

[`TryFromSexp`]: crate::TryFromSexp

### `trait_abi::conv::from_sexp`

```rust
unsafe from_sexp<T>(x: crate::SEXP) -> T
```

Convert an R SEXP to a Rust type, aborting via `Rf_error` on failure.

Attempts [`TryFromSexp::try_from_sexp`](crate::TryFromSexp::try_from_sexp)
and calls [`rf_error`] (R longjmp, never returns) if conversion fails.

#### Type Parameters

* `T` - Target Rust type (must implement [`TryFromSexp`])

#### Arguments

* `x` - R value to convert

#### Returns

The converted Rust value.

#### Safety

- `x` must be a valid SEXP
- Must be called on R's main thread

#### Errors

Calls [`rf_error`] (never returns) if conversion fails. Error messages
are generated by [`TryFromSexp`]'s error type's `Display` impl.

#### Example

```ignore
// In a method shim:
let arg0: i32 = unsafe { from_sexp(argv[0]) };
let arg1: String = unsafe { from_sexp(argv[1]) };
```

[`TryFromSexp`]: crate::TryFromSexp

### `trait_abi::conv::nil`

```rust
unsafe nil() -> crate::SEXP
```

Return R's `NULL` value.

Convenience function for method shims returning nothing.

#### Safety

- Must be called on R's main thread (accesses R_NilValue)

#### Returns

R's `NULL` value (`R_NilValue`)

### `trait_abi::conv::rf_error`

```rust
unsafe rf_error(msg: &str) -> never
```

Raise an R error with the given message.

This is a thin wrapper around `Rf_error` for use in method shims.
It never returns (diverges to R's error handler).

#### Arguments

* `msg` - Error message to display

#### Safety

- Must be called on R's main thread
- Never returns (uses R's `Rf_error` which longjmps)

#### Example

```ignore
if argc != 2 {
    unsafe { rf_error("expected 2 arguments, got {argc}"); }
}
```

### `trait_abi::conv::to_sexp`

```rust
unsafe to_sexp<T>(x: T) -> crate::SEXP
```

Convert a Rust value to an R SEXP.

Calls [`IntoR::into_sexp`](crate::IntoR::into_sexp) to produce an R value.
Used in trait ABI method shims.

#### Type Parameters

* `T` - Source Rust type (must implement [`IntoR`])

#### Arguments

* `x` - Rust value to convert

#### Returns

R SEXP representation of the value.

#### Safety

- Must be called on R's main thread
- The returned SEXP must be protected if used across R allocations

#### Example

```ignore
// In a method shim:
let result: f64 = self_ref.area();
unsafe { to_sexp(result) }
```

[`IntoR`]: crate::IntoR

### `trait_abi::conv::try_from_sexp`

```rust
unsafe try_from_sexp<T>(x: crate::SEXP) -> Result<T, <T as >::Error>
```

Convert an R SEXP to a Rust type, returning a Result.

Unlike [`from_sexp`], this function returns a `Result` instead of
calling `rf_error` on failure. Useful when you want custom error handling.

#### Type Parameters

* `T` - Target Rust type (must implement [`TryFromSexp`])

#### Arguments

* `x` - R value to convert

#### Returns

`Ok(T)` on success, `Err` with the conversion error on failure.

#### Safety

- `x` must be a valid SEXP
- Must be called on R's main thread

#### Example

```ignore
match unsafe { try_from_sexp::<i32>(argv[0]) } {
    Ok(val) => { /* use val */ }
    Err(e) => {
        // Custom error handling
        rf_error(&format!("argument 1: {}", e));
    }
}
```

[`TryFromSexp`]: crate::TryFromSexp

### `typed_list::actual_type_string`

```rust
actual_type_string(sexp: crate::SEXP) -> String
```

Get a human-readable string for the actual type of a SEXP.

Includes class attribute if present.

### `typed_list::sexptype_name`

```rust
sexptype_name(stype: crate::SEXPTYPE) -> String
```

Get a human-readable name for a SEXPTYPE.

### `typed_list::validate_list`

```rust
validate_list(list: crate::list::List, spec: &TypedListSpec) -> Result<TypedList, TypedListError>
```

Validate a list against a specification.

#### Errors

- [`TypedListError::Missing`] if a required field is absent
- [`TypedListError::WrongType`] if a field has the wrong SEXP type
- [`TypedListError::WrongLen`] if a field has the wrong length
- [`TypedListError::ExtraFields`] if `allow_extra = false` and extra named fields exist
- [`TypedListError::DuplicateNames`] if the list has duplicate non-empty names

### `unwind_protect::panic_payload_to_string`

```rust
panic_payload_to_string(payload: &dyn Any + Send) -> std::borrow::Cow<'_, str>
```

Extract a message from a panic payload.

Handles `&str`, `String`, and `&String` payloads consistently. The borrowed
variants are returned as `Cow::Borrowed`, so the common `panic!("literal")`
case avoids the heap allocation that a `String` return would force.
Unrecognised payload types fall back to a `Cow::Borrowed` static string.

Call `.into_owned()` (or `.to_string()`) at sites that need an owned
`String`.

### `unwind_protect::with_r_unwind_protect`

```rust
with_r_unwind_protect<F>(f: F, call: Option<crate::SEXP>) -> crate::SEXP
```

Run a closure under `R_UnwindProtect`, returning a tagged condition SEXP on
Rust panics instead of raising an R error.

This is **the** transport for all `#[miniextendr]` functions and methods.
The returned error/condition SEXP is inspected by the generated R wrapper
which raises a proper R condition past the Rust boundary, with `rust_*`
class layering.

Recognises [`crate::condition::RCondition`] payloads (from `error!()`,
`warning!()`, `message!()`, `condition!()`) before falling through to the
generic panic→string path.

R-origin errors (longjmp) still pass through via `R_ContinueUnwind`.

For guard sites that have no R wrapper to inspect a tagged SEXP (ALTREP
`RUnwind` callbacks, FFI guard tests) see [`with_r_unwind_protect_or_raise`];
for trait-ABI vtable shims see [`with_r_unwind_protect_shim`].

### `unwind_protect::with_r_unwind_protect_or_raise`

```rust
with_r_unwind_protect_or_raise<F, R>(f: F, call: Option<crate::SEXP>) -> R
```

Execute a closure with R unwind protection, raising any Rust panic as an R
error via `Rf_eval(stop(structure(...)))`.

If the closure panics, the panic is caught and converted to an R error
(longjmp) with `rust_*` class layering. If R raises an error (longjmp), all
Rust RAII resources are properly dropped before R continues unwinding.

**This is NOT the user-facing path for `#[miniextendr]` functions.** That
path is [`with_r_unwind_protect`], which returns a tagged SEXP instead of
longjmping (the macro-generated R wrapper raises the structured condition).

This raising-variant exists for guard sites that have no R wrapper between
them and R's runtime:
- ALTREP `RUnwind` guard callbacks (via the crate-private
  `with_r_unwind_protect_sourced`)
- FFI guard tests / benchmarks exercising the raw `R_UnwindProtect` mechanism

In those contexts there is no consumer-side R wrapper to inspect a tagged
SEXP. Panics are routed through `raise_rust_condition_via_stop` so they
still receive `rust_*` class layering (issue #345). Trait-ABI shims use a
separate SEXP-returning variant ([`with_r_unwind_protect_shim`]) that
re-panics at the View boundary.

#### Arguments

* `f` - The closure to execute
* `call` - Optional R call SEXP for better error messages

### `unwind_protect::with_r_unwind_protect_shim`

```rust
with_r_unwind_protect_shim<F>(f: F) -> crate::SEXP
```

Like [`with_r_unwind_protect`], but tailored for trait-ABI vtable shims.

Same tagged-SEXP behaviour as [`with_r_unwind_protect`], but intended for
shim functions that have no R wrapper of their own. The tagged SEXP is
returned to the View method wrapper, which calls
[`crate::condition::repanic_if_rust_error`] to re-panic with the
reconstructed [`crate::condition::RCondition`]. The outer
`with_r_unwind_protect` in the consumer's C entry point then catches the
re-panic and builds the final tagged SEXP for the consumer's R wrapper.

R-origin errors (longjmp) still pass through via `R_ContinueUnwind` — the
outer guard will catch them.

#### PROTECT note

The returned SEXP is unprotected. The View method wrapper must not call any
R API functions between receiving it and passing it to
`repanic_if_rust_error`. `repanic_if_rust_error` reads the message/kind/class
strings immediately and then panics (or returns), so the SEXP does not need
protection beyond that window.

### `vctrs::new_list_of`

```rust
new_list_of(x: crate::list::List, ptype: Option<crate::SEXP>, size: Option<i32>, class: &[&str], attrs: &[(&str, crate::SEXP)]) -> Result<crate::SEXP, VctrsBuildError>
```

Create a new vctrs list_of object.

This mirrors `vctrs::new_list_of()` in R, creating a list where each
element is expected to be of a consistent type (the prototype).

#### Arguments

* `x` - A list of elements
* `ptype` - The prototype (empty vector defining the element type)
* `size` - Optional fixed size for elements
* `class` - User class names (will be prepended to "vctrs_list_of")
* `attrs` - Additional attributes as (name, value) pairs

#### Class Structure

The resulting class vector will be:
- `c(class..., "vctrs_list_of", "vctrs_vctr", "list")`

#### Requirements

- At least one of `ptype` or `size` must be provided
- `size` must be non-negative if provided

#### Example

```ignore
// Create a list_of<integer>
let x = list!(vec![1, 2], vec![3, 4, 5]);
let ptype = integer(0).into_sexp();
let obj = new_list_of(x, Some(ptype), None, &[], &[])?;
// class(obj) == c("vctrs_list_of", "vctrs_vctr", "list")
```

### `vctrs::new_rcrd`

```rust
new_rcrd(fields: crate::list::List, class: &[&str], attrs: &[(&str, crate::SEXP)]) -> Result<crate::SEXP, VctrsBuildError>
```

Create a new vctrs record object.

This mirrors `vctrs::new_rcrd()` in R, creating a record type where
each element is a collection of fields (like a row in a data frame).

#### Arguments

* `fields` - A named list where all elements have the same length
* `class` - User class names (will be prepended to "vctrs_rcrd")
* `attrs` - Additional attributes as (name, value) pairs

#### Class Structure

The resulting class vector will be:
- `c(class..., "vctrs_rcrd", "vctrs_vctr")`

#### Requirements

- `fields` must be a named list
- All fields must have the same length
- Field names must be unique

#### Example

```ignore
// Create a "rational" record with numerator and denominator
let fields = list!(n = vec![1, 2, 3], d = vec![2, 3, 4]);
let obj = new_rcrd(fields, &["vctrs_rational"], &[])?;
// class(obj) == c("vctrs_rational", "vctrs_rcrd", "vctrs_vctr")
```

### `vctrs::new_vctr`

```rust
new_vctr(data: crate::SEXP, class: &[&str], attrs: &[(&str, crate::SEXP)], inherit_base_type: Option<bool>) -> Result<crate::SEXP, VctrsBuildError>
```

Create a new vctrs vector object.

This mirrors `vctrs::new_vctr()` in R, creating an object with the
appropriate class structure for vctrs compatibility.

#### Arguments

* `data` - The underlying data (must be a vector according to vctrs)
* `class` - User class names (will be prepended to "vctrs_vctr")
* `attrs` - Additional attributes as (name, value) pairs
* `inherit_base_type` - Whether to include the base type in the class vector.
  - `None`: Use default (true for lists, false otherwise)
  - `Some(true)`: Include base type (e.g., "double", "list")
  - `Some(false)`: Don't include base type (error for lists)

#### Class Structure

The resulting class vector will be:
- `c(class..., "vctrs_vctr")` if `inherit_base_type` is false
- `c(class..., "vctrs_vctr", typeof(data))` if `inherit_base_type` is true

#### Names Repair

If `data` has a names attribute with NA values, they are replaced with "".

#### Example

```ignore
// Create a "percent" class backed by doubles
let data = vec![0.1, 0.2, 0.3].into_sexp();
let obj = new_vctr(data, &["vctrs_percent"], &[], None)?;
// class(obj) == c("vctrs_percent", "vctrs_vctr", "double")
```

### `wasm_registry_writer::format_wasm_registry`

```rust
format_wasm_registry(call_defs: &[CallDefRow], altrep_regs: &[AltrepRegRow], trait_dispatches: &[TraitDispatchRow]) -> String
```

Format a `wasm_registry.rs` source file from extracted runtime data.

Output structure:
```text
// header (auto-generated marker, generator-version, content-hash)
use ...;
unsafe extern "C-unwind" { fn <wrapper>(...); ... }
unsafe extern "C" { fn <altrep_reg>(); ... static <vtable>: u8; ... }
pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef] = &[ ... ];
pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration] = &[ ... ];
pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry] = &[ ... ];
```

Every fn / static referenced from a slice gets a matching `extern` decl —
the WASM linker resolves them against the user crate's `#[no_mangle]`
exports.

### `wasm_registry_writer::write_wasm_registry_to_file`

```rust
write_wasm_registry_to_file(path: &str)
```

Read the live distributed slices, format `wasm_registry.rs`, and write it
to `path`. No-op when content is unchanged (matches `write_r_wrappers_to_file`).

### `worker::is_r_main_thread`

```rust
is_r_main_thread() -> bool
```

Check if the current thread is R's main thread.

Returns `true` if called from the main R thread, `false` otherwise.
Before `miniextendr_runtime_init()` is called, always returns `false`.

### `worker::with_r_thread`

```rust
with_r_thread<F, R>(f: F) -> R
```

Execute a closure on R's main thread, returning the result.

This function can be called from any thread:
- From the main thread: executes the closure directly (re-entrant)
- From the worker thread (during `run_on_worker`): sends the work to
  the main thread and blocks until completion

The "main thread" the closure runs on is whichever thread called
[`run_on_worker`]. Per the [`run_on_worker`] contract, that must be
the R main thread (the thread that ran `miniextendr_runtime_init()`);
otherwise R API calls inside the closure happen on the wrong thread.

#### Panics

- If `miniextendr_runtime_init()` hasn't been called yet
- If called from a non-main thread without the `worker-thread` feature
- If called from a non-main thread outside of a `run_on_worker` context
  (even with the `worker-thread` feature)

#### Example

```ignore
use miniextendr_api::with_r_thread;

// From worker thread, safely call R APIs:
let sexp = with_r_thread(|| {
    // This runs on R's main thread
    SEXP::nil()
});
```

---

## Macros

### `condition!`

Signal a generic R condition from Rust with `rust_condition` class layering.

Rides the tagged-condition transport that every `#[miniextendr]` function uses.
Unlike `error!`, a bare condition is a silent no-op if there is no handler.
The raised condition has class `c("rust_condition", "simpleCondition", "condition")`.

An optional `class = "name"` form prepends a custom class.

#### See also

- [`crate::error!`] / [`crate::warning!`] / [`crate::message!`] — louder
  condition kinds. Pick `condition!` when "no handler = silent" is the
  right default (progress events, structured logging hooks).
- [`std::panic!`] — escape hatch when the failure cannot be ignored.
- [`crate::error_value`] — tagged-SEXP transport rationale.

**Name-collision note.** Because `pub mod condition` exists at the crate
root, `use miniextendr_api::condition` imports the module rather than this
macro. Invoke via `miniextendr_api::condition!(...)` (fully qualified) or
via `mx::condition!(...)` after `use miniextendr_api as mx;`.

#### Example

```ignore
use miniextendr_api::condition;

#[miniextendr]
fn signal_progress(n: i32) {
    condition!(class = "my_progress", "processed {n} items");
}
```

```r
withCallingHandlers(
  signal_progress(42L),
  my_progress = function(c) cat("progress:", conditionMessage(c), "\n")
)
# progress: processed 42 items
```

### `define_thread_local_arena!`

Macro to define a thread-local arena with a specific map type.

This creates a zero-sized struct implementing [`ThreadLocalArenaOps`],
providing all arena methods via the trait's default implementations.

#### Example

```ignore
define_thread_local_arena!(
    /// My custom thread-local arena.
    pub MyArena,
    BTreeMap<usize, Entry>,
    MY_ARENA_STATE
);
```

### `error!`

Raise an R error from Rust with `rust_error` class layering.

Rides the tagged-condition transport that every `#[miniextendr]` function uses.
The raised condition has class `c("rust_error", "simpleError", "error", "condition")`.

An optional `class = "name"` form prepends a custom class for programmatic catching:
`c("name", "rust_error", "simpleError", "error", "condition")`.

#### See also

- [`crate::warning!`] / [`crate::message!`] / [`crate::condition!`] — the
  non-error sibling kinds (warning continues execution; message is muffled
  by `suppressMessages`; condition is silent without a handler).
- [`std::panic!`] — escape hatch with the same `rust_error` class layering
  but no custom-class slot. Use for true bugs / impossible states; reach for
  `error!` when callers might want to route by class.
- [`AsRError`] — wraps `Result<_, E: std::error::Error>` for value-style
  propagation through Rust code; converts at the boundary.
- [`crate::error_value`] — module-level rationale for the tagged-SEXP
  transport and the `error_in_r` default.

**Name-collision note.** Because `pub mod error` exists at the crate root,
`use miniextendr_api::error` imports the module rather than this macro.
Invoke via `miniextendr_api::error!(...)` (fully qualified) or via
`mx::error!(...)` after `use miniextendr_api as mx;`.

#### Examples

```ignore
use miniextendr_api as mx;

#[miniextendr]
fn fail() {
    mx::error!("something went wrong: {}", 42);
}

// With a custom class for tryCatch:
#[miniextendr]
fn typed_fail(name: &str) {
    mx::error!(class = "my_error", "missing field: {name}");
}
```

```r
tryCatch(fail(), rust_error = function(e) conditionMessage(e))
# [1] "something went wrong: 42"

tryCatch(typed_fail("x"), my_error = function(e) "caught!")
# [1] "caught!"
```

### `impl_altcomplex_from_data!`

Generate ALTREP trait implementations for a type that implements AltComplexData.

Optional features can be enabled by passing additional arguments:
- `dataptr`: Enable `Dataptr` and `Dataptr_or_null` methods (requires `AltrepDataptr<Rcomplex>`)
- `serialize`: Enable serialization support (requires `AltrepSerialize`)
- `subset`: Enable optimized subsetting (requires `AltrepExtractSubset`)

### `impl_altinteger_from_data!`

Generate ALTREP trait implementations for a type that implements AltIntegerData.

This macro generates `impl Altrep`, `impl AltVec`, and `impl AltInteger` for the type,
delegating to the high-level `AltIntegerData` trait methods.

**Requires**: The type must implement `TypedExternal` (use `#[derive(ExternalPtr)]`).

#### Variants

```ignore
// Basic (no dataptr, no serialization):
impl_altinteger_from_data!(MyType);

// With dataptr (type must implement AltrepDataptr<i32>):
impl_altinteger_from_data!(MyType, dataptr);

// With serialization (type must implement AltrepSerialize):
impl_altinteger_from_data!(MyType, serialize);

// With subset optimization (type must implement AltrepExtractSubset):
impl_altinteger_from_data!(MyType, subset);

// Combine multiple options:
impl_altinteger_from_data!(MyType, dataptr, serialize);
impl_altinteger_from_data!(MyType, subset, serialize);
```

### `impl_altlist_from_data!`

Generate ALTREP trait implementations for a type that implements AltListData.

### `impl_altlogical_from_data!`

Generate ALTREP trait implementations for a type that implements AltLogicalData.

#### Variants

```ignore
// Basic (no dataptr, no serialization):
impl_altlogical_from_data!(MyType);

// With dataptr (type must implement AltrepDataptr<i32>):
impl_altlogical_from_data!(MyType, dataptr);

// With serialization (type must implement AltrepSerialize):
impl_altlogical_from_data!(MyType, serialize);

// With subset optimization (type must implement AltrepExtractSubset):
impl_altlogical_from_data!(MyType, subset);

// Combine multiple options:
impl_altlogical_from_data!(MyType, dataptr, serialize);
impl_altlogical_from_data!(MyType, subset, serialize);
```

### `impl_altraw_from_data!`

Generate ALTREP trait implementations for a type that implements AltRawData.

#### Variants

```ignore
// Basic (no dataptr, no serialization):
impl_altraw_from_data!(MyType);

// With dataptr (type must implement AltrepDataptr<u8>):
impl_altraw_from_data!(MyType, dataptr);

// With serialization (type must implement AltrepSerialize):
impl_altraw_from_data!(MyType, serialize);

// With subset optimization (type must implement AltrepExtractSubset):
impl_altraw_from_data!(MyType, subset);

// Combine multiple options:
impl_altraw_from_data!(MyType, dataptr, serialize);
impl_altraw_from_data!(MyType, subset, serialize);
```

### `impl_altreal_from_data!`

Generate ALTREP trait implementations for a type that implements AltRealData.

#### Variants

```ignore
// Basic (no dataptr, no serialization):
impl_altreal_from_data!(MyType);

// With dataptr (type must implement AltrepDataptr<f64>):
impl_altreal_from_data!(MyType, dataptr);

// With serialization (type must implement AltrepSerialize):
impl_altreal_from_data!(MyType, serialize);

// With subset optimization (type must implement AltrepExtractSubset):
impl_altreal_from_data!(MyType, subset);

// Combine multiple options:
impl_altreal_from_data!(MyType, dataptr, serialize);
impl_altreal_from_data!(MyType, subset, serialize);
```

### `impl_altstring_from_data!`

Generate ALTREP trait implementations for a type that implements AltStringData.

#### Variants

```ignore
// Basic (no serialization):
impl_altstring_from_data!(MyType);

// With dataptr (materialized STRSXP):
impl_altstring_from_data!(MyType, dataptr);

// With serialization (type must implement AltrepSerialize):
impl_altstring_from_data!(MyType, serialize);

// With subset optimization (type must implement AltrepExtractSubset):
impl_altstring_from_data!(MyType, subset);

// Combine multiple options:
impl_altstring_from_data!(MyType, dataptr, serialize);
impl_altstring_from_data!(MyType, subset, serialize);
```

### `impl_inferbase_complex!`

Implement `InferBase` for a complex ALTREP data type.

### `impl_inferbase_integer!`

Implement `InferBase` for an integer ALTREP data type.

### `impl_inferbase_list!`

Implement `InferBase` for a list ALTREP data type.

### `impl_inferbase_logical!`

Implement `InferBase` for a logical ALTREP data type.

### `impl_inferbase_raw!`

Implement `InferBase` for a raw ALTREP data type.

### `impl_inferbase_real!`

Implement `InferBase` for a real ALTREP data type.

### `impl_inferbase_string!`

Implement `InferBase` for a string ALTREP data type.

### `impl_option_try_from_sexp!`

Implement `TryFromSexp for Option<T>` where T already implements TryFromSexp.

NULL → None, otherwise delegates to T::try_from_sexp and wraps in Some.

### `impl_vec_option_try_from_sexp_list!`

Implement `TryFromSexp for Vec<Option<T>>` from R list (VECSXP).

NULL elements become None, others are converted via T::try_from_sexp.

### `impl_vec_try_from_sexp_list!`

Implement `TryFromSexp for Vec<T>` from R list (VECSXP).

Each element is converted via T::try_from_sexp.

### `message!`

Emit an R message from Rust with `rust_message` class layering.

Rides the tagged-condition transport that every `#[miniextendr]` function uses.
The raised condition has class `c("rust_message", "simpleMessage", "message", "condition")`.
Muffled by `suppressMessages()` automatically (standard R restart mechanism).

#### See also

- [`crate::warning!`] / [`crate::condition!`] — louder/quieter sibling kinds.
- [`crate::error!`] — for fatal failures.
- [`std::panic!`] — escape hatch.
- [`crate::error_value`] — tagged-SEXP transport rationale.

No name-collision caveat: there is no `pub mod message`, so
`use miniextendr_api::message;` then `message!(...)` works directly.

#### Example

```ignore
use miniextendr_api::message;

#[miniextendr]
fn log_step(step: i32) {
    message!("step {} complete", step);
}
```

```r
log_step(3L)
# step 3 complete

suppressMessages(log_step(3L))  # no output
```

### `r!`

Evaluate R code written as **Rust tokens**, validated at compile time.

`r!` takes a single R expression as a token stream, `stringify!`s it into a
static R source string at build time, and evaluates it via
[`expression::r_eval_str`] (the same protect-safe parse + eval path as
[`r_str!`](crate::r_str)).

#### What you get today

Because the argument is a Rust token tree, the Rust front-end already
rejects **unbalanced delimiters** (`r!(f(1, 2)` won't compile) and
lexically invalid tokens before R ever sees the string — a cheap
compile-time guard over the pure-runtime [`r_str!`](crate::r_str). The
source is lowered to a `&'static str` (`stringify!`), so there is no
`format!` allocation at the call site.

#### What is deferred

True build-time R-grammar validation (embedding R's parser / invoking
`R_ParseVector` from `build.rs`) and direct `Rf_lang*` call-tree lowering
(skipping the runtime parser entirely) are tracked as a follow-up — see the
issue referenced in the crate docs. Until then `r!` parses its static
string at first evaluation, exactly like `r_str!`.

#### Forms

- `r!(R tokens…)` — evaluate in `R_GlobalEnv`.
- `r!(env: e; R tokens…)` — evaluate in the environment SEXP `e`. The
  leading `env: <expr> ;` is consumed as Rust, the rest is R source. (The
  `;` separator is used instead of a trailing `, env =` because R source is
  a free token stream — a trailing keyword can't be reliably split off it.)

Both evaluate to `Result<SEXP, String>`; the `SEXP` is **unprotected**.

For genuinely dynamic code, use [`r_str!`](crate::r_str) instead.

#### Note on `stringify!` spacing

`stringify!` normalises whitespace (`a+b` and `a + b` both stringify to
`a + b`) but preserves token order and literal contents, which is all R's
parser needs. String literals keep their quotes.

#### Safety

Expands to an `unsafe` block; see [`r_str!`](crate::r_str).

#### Example

```ignore
let three = r!(1L + 2L)?;
let rows = r!(getFromNamespace(".theoph_rows", "dataframeflows")())?;
let in_env = r!(env: my_env; x + 1)?;
```

### `r_print!`

Print to R's console (like `print!`).

#### Example

```ignore
use miniextendr_api::r_print;

r_print!("Hello ");
r_print!("value: {}", 42);
```

### `r_println!`

Print to R's console with a newline (like `println!`).

#### Example

```ignore
use miniextendr_api::r_println;

r_println!();  // just a newline
r_println!("Hello, world!");
r_println!("value: {}", 42);
```

### `r_str!`

Parse and evaluate **runtime** R source from Rust.

`r_str!` is sugar around [`expression::r_eval_str`]. It is the right tool
when the R code is genuinely dynamic — built with `format!`, derived from
user input, or otherwise not known at compile time. For code you can write
literally, prefer [`r!`](crate::r), which gives a cheap compile-time
sanity check on the token stream.

The argument is any expression evaluating to something `AsRef<str>`
(`&str`, `String`, `&String`, …). It is parsed with `R_ParseVector` and
evaluated with `Rf_eval`, with every intermediate SEXP protected and the
parse status checked, so a syntax error becomes an `Err`, never a segfault
or silent wrong answer.

#### Forms

- `r_str!(code)` — evaluate in `R_GlobalEnv`.
- `r_str!(code, env = e)` — evaluate in the environment SEXP `e`.

Both forms evaluate to `Result<SEXP, String>`; the `SEXP` is **unprotected**
(protect it before further allocations).

#### Safety

Expands to an `unsafe` block. Must be reached from (or routed to) the R
main thread — the underlying FFI is `#[r_ffi_checked]`, so calls from a
worker thread are serialized onto the R thread.

#### Example

```ignore
let obj = "mtcars";
let code = format!("summary({obj})");
let summary = r_str!(&code)?;          // in R_GlobalEnv
let three = r_str!("1L + 2L")?;
let in_env = r_str!("x + 1", env = my_env)?;
```

### `report_growth!`

Print and reset all growth counters.

When the `growth-debug` feature is enabled, prints all tracked growth events
to stderr and resets the counters. When disabled, compiles to a no-op.

### `track_growth!`

Track a collection growth (reallocation) event.

When the `growth-debug` feature is enabled, increments a thread-local counter
for the named collection. When disabled, compiles to a no-op.

#### Example

```ignore
let old_cap = vec.capacity();
vec.push(item);
if vec.capacity() != old_cap {
    track_growth!("my_vec");
}
```

### `warning!`

Raise an R warning from Rust with `rust_warning` class layering.

Rides the tagged-condition transport that every `#[miniextendr]` function uses.
Unlike `panic!`, execution continues after `warning!` is caught by a handler.
The raised condition has class `c("rust_warning", "simpleWarning", "warning", "condition")`.

An optional `class = "name"` form prepends a custom class.

#### See also

- [`crate::error!`] — fatal sibling; aborts the call instead of continuing.
- [`crate::message!`] / [`crate::condition!`] — softer signal kinds (muffled
  by `suppressMessages` / silent without handler, respectively).
- [`std::panic!`] — escape hatch when "continue after this" is not a sensible
  semantic.
- [`crate::error_value`] — tagged-SEXP transport rationale.

No name-collision caveat: there is no `pub mod warning`, so
`use miniextendr_api::warning;` then `warning!(...)` works directly.

#### Example

```ignore
use miniextendr_api::warning;

#[miniextendr]
fn maybe_warn(x: i32) -> i32 {
    if x > 100 {
        warning!("x is large: {x}");
    }
    x * 2
}
```

```r
withCallingHandlers(
  maybe_warn(200L),
  warning = function(w) { cat("saw:", conditionMessage(w)); invokeRestart("muffleWarning") }
)
# saw: x is large: 200
# [1] 400
```

---

## Constants

### `adapter_traits::TAG_RCLONE: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RClone` trait.
Generated from source location line 368, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RCOPY: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RCopy` trait.
Generated from source location line 454, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RDEBUG: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RDebug` trait.
Generated from source location line 55, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RDEFAULT: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RDefault` trait.
Generated from source location line 407, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RDISPLAY: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RDisplay` trait.
Generated from source location line 97, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RERROR: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RError` trait.
Generated from source location line 264, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_REXTEND: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RExtend` trait.
Generated from source location line 666, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RFROMITER: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RFromIter` trait.
Generated from source location line 733, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RFROMSTR: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RFromStr` trait.
Generated from source location line 328, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RHASH: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RHash` trait.
Generated from source location line 132, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RITERATOR: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RIterator` trait.
Generated from source location line 540, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RMAKEITER: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RMakeIter` trait.
Generated from source location line 882, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RORD: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `ROrd` trait.
Generated from source location line 164, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RPARTIALORD: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RPartialOrd` trait.
Generated from source location line 204, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `adapter_traits::TAG_RTOVEC: ::miniextendr_api::abi::mx_tag`

Type tag for runtime identification of the `RToVec` trait.
Generated from source location line 787, column 11.
Generated from source file `miniextendr-api/src/adapter_traits.rs`.

### `altrep_traits::KNOWN_UNSORTED: i32`

Known to be unsorted (`KNOWN_UNSORTED` in R).

### `altrep_traits::NA_INTEGER: i32`

NA value for integers.

### `altrep_traits::NA_LOGICAL: i32`

NA value for logical (same as integer in R).

### `altrep_traits::NA_REAL: f64`

NA value for reals (IEEE NaN with R's NA payload).

### `altrep_traits::SORTED_DECR: i32`

Sorted in decreasing order, possibly with ties (`SORTED_DECR` in R).

### `altrep_traits::SORTED_DECR_NA_1ST: i32`

Sorted in decreasing order, with NAs first (`SORTED_DECR_NA_1ST` in R).

### `altrep_traits::SORTED_INCR: i32`

Sorted in increasing order, possibly with ties (`SORTED_INCR` in R).

### `altrep_traits::SORTED_INCR_NA_1ST: i32`

Sorted in increasing order, with NAs first (`SORTED_INCR_NA_1ST` in R).

### `altrep_traits::UNKNOWN_SORTEDNESS: i32`

Unknown sortedness value (INT_MIN in R).

### `error_value::kind::CONDITION: &str`

User-raised `condition!(...)` condition.

### `error_value::kind::CONVERSION: &str`

`TryFromSexp` / coerce / strict-mode conversion failed at argument
unmarshalling.

### `error_value::kind::ERROR: &str`

User-raised `error!(...)` condition.

### `error_value::kind::MESSAGE: &str`

User-raised `message!(...)` condition.

### `error_value::kind::NONE_ERR: &str`

`Option<T>::None` reached where a value was required (raised by the
`NoneOnErr` / required-Option return paths).

### `error_value::kind::OTHER_RUST_ERROR: &str`

Fallback kind written by [`super::make_rust_condition_value`] when the
caller's `kind` argument contained an interior NUL and could not be
converted to a `CString`. Should not appear in normal flow; the match
arm in [`crate::condition::RCondition::from_tagged_sexp`] handles it
defensively by degrading to `RCondition::Error`.

### `error_value::kind::PANIC: &str`

Default kind for Rust panics that surface to R via the generic panic
path (no `RCondition` payload). Layered as `rust_error`.

### `error_value::kind::RESULT_ERR: &str`

`Result<_, E>::Err(...)` formatted via `Debug` (raised when the user
returns an `Err` from a `#[miniextendr]` fn/method).

### `error_value::kind::WARNING: &str`

User-raised `warning!(...)` condition.

### `r_coerce::SUPPORTED_AS_GENERICS: &[&str]`

All supported R coercion generics.

This list can be used to validate user input or generate documentation.

### `sys::IDENT_ATTR_BY_ORDER: ::std::os::raw::c_int`

Compare attributes in order (not as a set).

### `sys::IDENT_EXTPTR_AS_REF: ::std::os::raw::c_int`

Compare external pointers as references (not by address).

### `sys::IDENT_NA_AS_BITS: ::std::os::raw::c_int`

Treat all NAs as identical (ignore NA payload differences).

### `sys::IDENT_NUM_AS_BITS: ::std::os::raw::c_int`

Flags for `R_compute_identical` (bitmask, inverted logic: set bit = disable check).

### `sys::IDENT_USE_BYTECODE: ::std::os::raw::c_int`

Include bytecode in comparison.

### `sys::IDENT_USE_CLOENV: ::std::os::raw::c_int`

Include closure environments in comparison.

### `sys::IDENT_USE_SRCREF: ::std::os::raw::c_int`

Include source references in comparison.

### `thread::DEFAULT_R_STACK_SIZE: usize`

Default stack size for R-compatible threads (8 MiB).

R doesn't enforce a specific stack size - it uses whatever the OS provides:
- **Unix**: Typically 8 MiB from `ulimit -s`
- **Windows**: 64 MiB for the main thread (since R 4.2)

Since we disable R's stack checking via `StackCheckGuard`, the size is about
practical needs rather than R enforcement. Deep recursion in R code (especially
recursive functions, `lapply` chains, or complex formulas) can use significant stack.

Rust's default thread stack is only 2 MiB, which may be insufficient for deep R calls.
We default to 8 MiB as a reasonable balance. Increase via [`RThreadBuilder::stack_size`]
if you encounter stack overflows.

---

## Statics

### `registry::MX_ALTREP_REGISTRATIONS: ::linkme::DistributedSlice<[AltrepRegistration]>`

ALTREP class registration entries, called once at package init.

Each ALTREP struct or trait impl emits an entry pairing the registration
function pointer with its `#[no_mangle]` symbol name. The fn is declared
`pub extern "C"` with `#[unsafe(no_mangle)]`, making it externally
addressable from a separate compilation unit (e.g. the WASM snapshot
codegen path). The fn pointer is not `unsafe` — R-thread invariants are a
module-level contract, not encoded in the type (same convention as the
`extern "C-unwind" fn` entries in `MX_CALL_DEFS`). The `symbol` string is
consumed by the host-time WASM snapshot writer to emit
`extern "C" { fn <symbol>(); }` declarations in `wasm_registry.rs`.

### `registry::MX_CALL_DEFS: ::linkme::DistributedSlice<[crate::sys::R_CallMethodDef]>`

R `.Call` method registrations (function + method C wrappers).

Each `#[miniextendr]` function or method emits an entry here.

### `registry::MX_CLASS_NAMES: ::linkme::DistributedSlice<[ClassNameEntry]>`

Class name entries mapping Rust type names to R-visible class names. **Host-only.**

Each `#[miniextendr]` impl block emits an entry. During
`write_r_wrappers_to_file`, `.__MX_CLASS_REF_<RustName>__` placeholders
in generated R wrapper strings are replaced with the registered R class name
(which may differ when `class = "Override"` is set on the impl block).

### `registry::MX_MATCH_ARG_CHOICES: ::linkme::DistributedSlice<[MatchArgChoicesEntry]>`

Match-arg choices entries for R wrapper post-processing. **Host-only.**

Each `#[miniextendr]` function with `match_arg` params emits an entry.
During `write_r_wrappers_to_file`, the placeholder in the R formal default
is replaced with the actual choices from the enum's `MatchArg::CHOICES`.

### `registry::MX_MATCH_ARG_PARAM_DOCS: ::linkme::DistributedSlice<[MatchArgParamDocEntry]>`

Match-arg `@param` doc entries for R wrapper post-processing. **Host-only.**

Each `#[miniextendr]` function with `match_arg` params that has no
user-written `@param` doc emits an entry here. During
`write_r_wrappers_to_file`, the placeholder in the `@param` roxygen tag
is replaced with e.g. `One of "Fast", "Safe", "Debug".`

### `registry::MX_R_WRAPPERS: ::linkme::DistributedSlice<[RWrapperEntry]>`

R wrapper code fragments with priority for ordering. **Host-only.**

Each `#[miniextendr]` function, impl block, or trait impl emits an entry.
Priorities ensure correct evaluation order when R sources the wrapper file
(sidecar helpers must be defined before class definitions that reference them).

### `registry::MX_S7_SIDECAR_PROPS: ::linkme::DistributedSlice<[SidecarPropEntry]>`

S7 sidecar property documentation entries. **Host-only.**

Each `#[derive(ExternalPtr)] #[externalptr(s7)]` struct with `#[r_data]` fields
emits one entry per public field. During `write_r_wrappers_to_file`, the
`.__MX_S7_SIDECAR_PROP_DOCS_<TypeName>__` placeholder in the S7 class wrapper
is replaced with the formatted `#' @prop {field} {doc}` roxygen lines.

### `registry::MX_TRAIT_DISPATCH: ::linkme::DistributedSlice<[TraitDispatchEntry]>`

Trait dispatch entries for [`universal_query`].

Each `#[miniextendr] impl Trait for Type` emits an entry mapping
`(concrete_tag, trait_tag)` to the trait's vtable pointer.

### `sys::R_BaseEnv: SEXP`

Base package namespace environment.

### `sys::R_BaseNamespace: SEXP`

Base package namespace — encapsulated by SEXP::base_namespace()

### `sys::R_BlankString: SEXP`

Empty string CHARSXP — encapsulated by SEXP::blank_string()

### `sys::R_ClassSymbol: SEXP`

### `sys::R_DimNamesSymbol: SEXP`

### `sys::R_DimSymbol: SEXP`

### `sys::R_EmptyEnv: SEXP`

Empty root environment.

### `sys::R_GlobalEnv: SEXP`

Global environment (`.GlobalEnv`).

### `sys::R_LevelsSymbol: SEXP`

### `sys::R_MissingArg: SEXP`

The "missing argument" sentinel value.

When an R function is called without providing a value for a formal
argument, R passes `R_MissingArg` as a placeholder. This is different
from `R_NilValue` (NULL) - a missing argument means "not provided",
while NULL is an explicit value.

In R: `f <- function(x) missing(x); f()` returns `TRUE`.
Encapsulated by SEXP::missing_arg()

### `sys::R_NaString: SEXP`

Missing string singleton — encapsulated by SEXP::na_string()

### `sys::R_NamesSymbol: SEXP`

Symbol for `names` attribute.

### `sys::R_NilValue: SEXP`

The canonical R `NULL` value.

### `sys::R_RowNamesSymbol: SEXP`

### `sys::R_TspSymbol: SEXP`

---

## Type aliases

### `abi::mx_meth`

`type mx_meth = {'function_pointer': {'sig': {'inputs': [['data', {'raw_pointer': {'is_mutable': True, 'type': {'resolved_path': {'path': 'std::os::raw::c_void', 'id': 4026, 'args': None}}}}], ['argc', {'primitive': 'i32'}], ['argv', {'raw_pointer': {'is_mutable': False, 'type': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': False}}}}}`

Method signature for trait vtable entries.

All trait methods are erased to this uniform signature:
- `data`: Pointer to the concrete object data
- `argc`: Number of arguments in `argv`
- `argv`: Array of SEXP arguments from R
- Returns: SEXP result to R

#### Argument Handling

The shim generated by `#[miniextendr]` on a trait is responsible for:
1. Checking `argc` matches expected arity
2. Converting each `argv[i]` via [`TryFromSexp`]
3. Calling the actual method
4. Converting the result via [`IntoR`]
5. Catching panics and converting to R errors

#### Safety

This function pointer is `unsafe` because:
- `data` must point to valid, properly-aligned data of the expected type
- `argv` must point to `argc` valid SEXP values
- Must be called on R's main thread

[`TryFromSexp`]: crate::TryFromSexp
[`IntoR`]: crate::IntoR

### `externalptr::ErasedExternalPtr`

`type ErasedExternalPtr = ExternalPtr<()>`

Type-erased `ExternalPtr` for cases where the concrete `T` is not needed.

### `gc_protect::ProtectIndex`

`type ProtectIndex = ::std::os::raw::c_int`

R's PROTECT_INDEX type (just `c_int` under the hood).

### `into_r::altrep::Lazy`

`type Lazy = Altrep<T>`

Marker type to opt-in to ALTREP representation for types that have both
eager-copy and ALTREP implementations.

#### Motivation

Types like `Vec<i32>` have two possible conversions to R:
1. **Eager copy** (default): copies all data to R immediately
2. **ALTREP**: keeps data in Rust, provides it on-demand to R

The default `IntoR` for `Vec<i32>` does eager copy. To get ALTREP behavior,
wrap your value in `Altrep<T>`.

#### Example

```ignore
use miniextendr_api::{miniextendr, Altrep};

// Returns an ALTREP-backed integer vector (data stays in Rust)
#[miniextendr]
fn altrep_vec() -> Altrep<Vec<i32>> {
    Altrep((0..1_000_000).collect())
}

// Returns a regular R vector (data copied to R)
#[miniextendr]
fn regular_vec() -> Vec<i32> {
    (0..1_000_000).collect()
}
```

#### Supported Types

`Altrep<T>` works with any type that implements both:
- [`RegisterAltrep`](crate::altrep::RegisterAltrep) - for ALTREP class registration
- [`TypedExternal`](crate::externalptr::TypedExternal) - for wrapping in ExternalPtr

Built-in supported types:
- `Vec<i32>`, `Vec<f64>`, `Vec<bool>`, `Vec<u8>`, `Vec<String>`
- `Box<[i32]>`, `Box<[f64]>`, `Box<[bool]>`, `Box<[u8]>`, `Box<[String]>`
- `Range<i32>`, `Range<i64>`, `Range<f64>`

Opt-in lazy materialization via ALTREP.

Wrapping a return type in `Lazy<T>` causes it to be returned as an
ALTREP vector backed by Rust-owned memory. R reads elements on demand;
full materialization only happens if R needs a contiguous pointer.

#### When to use
- Large vectors (>1000 elements)
- Data R may only partially read
- Computed/external data (Arrow, ndarray, nalgebra)

#### When NOT to use
- Small vectors (<100 elements, ALTREP overhead dominates)
- Data R will immediately modify (triggers instant materialization)

#### Example
```rust,ignore
#[miniextendr]
fn big_result() -> Lazy<Vec<f64>> {
    Lazy(vec![0.0; 1_000_000])
}
```

### `optionals::arrow_impl::StringDictionaryArray`

`type StringDictionaryArray = DictionaryArray<Int32Type>`

Type alias for dictionary-encoded string arrays (Arrow equivalent of R factors).

### `optionals::bitvec_impl::RBitVec`

`type RBitVec = BitVec<u8, Lsb0>`

Standard bit vector type for R interop.

Uses `u8` storage with LSB-first ordering for consistent behavior.

### `optionals::nalgebra_impl::RDMatrix`

`type RDMatrix = nalgebra::base::Matrix<T, nalgebra::base::dimension::Dyn, nalgebra::base::dimension::Dyn, RVecStorage<T, nalgebra::base::dimension::Dyn, nalgebra::base::dimension::Dyn>>`

A nalgebra dynamic matrix backed by R memory. Zero-copy.

This is a real nalgebra matrix that operates directly on R's
column-major matrix memory.

#### Example

```rust,ignore
#[miniextendr]
fn matrix_det(m: RDMatrix<f64>) -> f64 {
    m.determinant()  // zero-copy, no allocation
}
```

### `optionals::nalgebra_impl::RDVector`

`type RDVector = nalgebra::base::Matrix<T, nalgebra::base::dimension::Dyn, nalgebra::base::dimension::U1, RVecStorage<T, nalgebra::base::dimension::Dyn, nalgebra::base::dimension::U1>>`

A nalgebra dynamic vector backed by R memory. Zero-copy.

This is a real nalgebra column vector that operates directly on R's
`REALSXP`/`INTSXP` memory.

#### Example

```rust,ignore
#[miniextendr]
fn vector_norm(v: RDVector<f64>) -> f64 {
    v.norm()  // zero-copy, no allocation
}
```

### `pump::WorkerError`

`type WorkerError = Box<dyn Error + Send + Sync>`

Boxed, thread-safe error type used by [`WorkerPump::run`].

Alias for `Box<dyn Error + Send + Sync>`. Compatible with `anyhow::Error`
via `?` and with standard library error types without requiring extra
dependencies.

### `rarray::RArray3D`

`type RArray3D = RArray<T, 3>`

A 3-dimensional R array.

### `rarray::RMatrix`

`type RMatrix = RArray<T, 2>`

A 2-dimensional R matrix.

### `rarray::RVector`

`type RVector = RArray<T, 1>`

A 1-dimensional R vector with explicit dim attribute.

### `refcount_protect::HashMapArena`

`type HashMapArena = Arena<std::collections::HashMap<usize, Entry>>`

HashMap-based arena (faster for large collections).

### `refcount_protect::RefCountedArena`

`type RefCountedArena = Arena<std::collections::BTreeMap<usize, Entry>>`

BTreeMap-based arena (default, good for reference counting).

### `refcount_protect::RefCountedGuard`

`type RefCountedGuard = ArenaGuard<'a, std::collections::BTreeMap<usize, Entry>>`

Legacy type alias for backwards compatibility.

### `serde::dataframe_de::BorrowedRows`

`type BorrowedRows = crate::Protected<'a, Vec<T>>`

RAII handle around a `Vec<T>` deserialised from a data.frame, with the
source SEXP kept GC-rooted for the bundle's lifetime.

Type alias for [`Protected<'a, Vec<T>>`](crate::Protected) — see #681 for
the underlying primitive. Use [`dataframe_to_vec_borrowed`] to construct.

Access rows via `Deref<Target = Vec<T>>`:

```ignore
let rows: BorrowedRows<'_, Row> = dataframe_to_vec_borrowed(sexp)?;
for r in &*rows {
    // …
}
```

### `sexp_types::R_CFinalizer_t`

`type R_CFinalizer_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['s', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': None, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

C finalizer callback signature used by external pointers.

### `sexp_types::R_xlen_t`

`type R_xlen_t = isize`

R's extended vector length type (`R_xlen_t`).

### `sexp_types::Rbyte`

`type Rbyte = ::std::os::raw::c_uchar`

R byte element type used by `RAWSXP`.

### `sys::DL_FUNC`

`type DL_FUNC = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [], 'output': {'raw_pointer': {'is_mutable': True, 'type': {'resolved_path': {'path': '::std::os::raw::c_void', 'id': 4026, 'args': None}}}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Generic dynamic library function pointer.

R defines this as `void *(*)(void)` - a function taking no arguments and
returning `void*`. This is used for method registration and external pointer
functions. The actual function signatures vary; callers cast to the appropriate
concrete function type before calling.

We use `fn() -> *mut c_void` to match R's signature. The function pointer is
stored generically and cast to the appropriate type when called by R.

### `sys::R_ExternalMethodDef`

`type R_ExternalMethodDef = R_CallMethodDef`

Method definition for .External interface routines.

Structurally identical to `R_CallMethodDef`.

### `sys::R_FortranMethodDef`

`type R_FortranMethodDef = R_CMethodDef`

Method definition for .Fortran interface routines.

Structurally identical to `R_CMethodDef`.

### `sys::R_NativePrimitiveArgType`

`type R_NativePrimitiveArgType = ::std::os::raw::c_uint`

Type descriptor for native primitive arguments in .C/.Fortran calls.

This is used in `R_CMethodDef` and `R_FortranMethodDef` to specify
argument types for type checking.

### `sys::altrep::R_altcomplex_Elt_method_t`

`type R_altcomplex_Elt_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::sexp_types::Rcomplex', 'id': 163, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTCOMPLEX `elt` method.

### `sys::altrep::R_altcomplex_Get_region_method_t`

`type R_altcomplex_Get_region_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['sx', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['n', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['buf', {'raw_pointer': {'is_mutable': True, 'type': {'resolved_path': {'path': 'crate::sexp_types::Rcomplex', 'id': 163, 'args': None}}}}]], 'output': {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTCOMPLEX `get_region` method.

### `sys::altrep::R_altinteger_Elt_method_t`

`type R_altinteger_Elt_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTINTEGER `elt` method.

### `sys::altrep::R_altinteger_Get_region_method_t`

`type R_altinteger_Get_region_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['sx', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['n', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['buf', {'raw_pointer': {'is_mutable': True, 'type': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}}}]], 'output': {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTINTEGER `get_region` method.

### `sys::altrep::R_altinteger_Is_sorted_method_t`

`type R_altinteger_Is_sorted_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTINTEGER `is_sorted` method.

### `sys::altrep::R_altinteger_Max_method_t`

`type R_altinteger_Max_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['narm', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTINTEGER `max` method.

### `sys::altrep::R_altinteger_Min_method_t`

`type R_altinteger_Min_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['narm', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTINTEGER `min` method.

### `sys::altrep::R_altinteger_No_NA_method_t`

`type R_altinteger_No_NA_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTINTEGER `no_na` method.

### `sys::altrep::R_altinteger_Sum_method_t`

`type R_altinteger_Sum_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['narm', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTINTEGER `sum` method.

### `sys::altrep::R_altlist_Elt_method_t`

`type R_altlist_Elt_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTLIST `elt` method.

### `sys::altrep::R_altlist_Set_elt_method_t`

`type R_altlist_Set_elt_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['v', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': None, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTLIST `set_elt` method.

### `sys::altrep::R_altlogical_Elt_method_t`

`type R_altlogical_Elt_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTLOGICAL `elt` method.

### `sys::altrep::R_altlogical_Get_region_method_t`

`type R_altlogical_Get_region_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['sx', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['n', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['buf', {'raw_pointer': {'is_mutable': True, 'type': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}}}]], 'output': {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTLOGICAL `get_region` method.

### `sys::altrep::R_altlogical_Is_sorted_method_t`

`type R_altlogical_Is_sorted_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTLOGICAL `is_sorted` method.

### `sys::altrep::R_altlogical_No_NA_method_t`

`type R_altlogical_No_NA_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTLOGICAL `no_na` method.

### `sys::altrep::R_altlogical_Sum_method_t`

`type R_altlogical_Sum_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['narm', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTLOGICAL `sum` method.

### `sys::altrep::R_altraw_Elt_method_t`

`type R_altraw_Elt_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::sexp_types::Rbyte', 'id': 276, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTRAW `elt` method.

### `sys::altrep::R_altraw_Get_region_method_t`

`type R_altraw_Get_region_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['sx', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['n', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['buf', {'raw_pointer': {'is_mutable': True, 'type': {'resolved_path': {'path': 'crate::sexp_types::Rbyte', 'id': 276, 'args': None}}}}]], 'output': {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTRAW `get_region` method.

### `sys::altrep::R_altreal_Elt_method_t`

`type R_altreal_Elt_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}]], 'output': {'primitive': 'f64'}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREAL `elt` method.

### `sys::altrep::R_altreal_Get_region_method_t`

`type R_altreal_Get_region_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['sx', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['n', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['buf', {'raw_pointer': {'is_mutable': True, 'type': {'primitive': 'f64'}}}]], 'output': {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREAL `get_region` method.

### `sys::altrep::R_altreal_Is_sorted_method_t`

`type R_altreal_Is_sorted_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREAL `is_sorted` method.

### `sys::altrep::R_altreal_Max_method_t`

`type R_altreal_Max_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['narm', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREAL `max` method.

### `sys::altrep::R_altreal_Min_method_t`

`type R_altreal_Min_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['narm', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREAL `min` method.

### `sys::altrep::R_altreal_No_NA_method_t`

`type R_altreal_No_NA_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREAL `no_na` method.

### `sys::altrep::R_altreal_Sum_method_t`

`type R_altreal_Sum_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['narm', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREAL `sum` method.

### `sys::altrep::R_altrep_Coerce_method_t`

`type R_altrep_Coerce_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['rtype', {'resolved_path': {'path': 'crate::sexp_types::SEXPTYPE', 'id': 8, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREP `coerce` method.

### `sys::altrep::R_altrep_DuplicateEX_method_t`

`type R_altrep_DuplicateEX_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['deep', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREP extended `duplicate` method.

### `sys::altrep::R_altrep_Duplicate_method_t`

`type R_altrep_Duplicate_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['deep', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREP `duplicate` method.

### `sys::altrep::R_altrep_Inspect_method_t`

`type R_altrep_Inspect_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['pre', {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}], ['deep', {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}], ['pvec', {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}], ['inspect_subtree', {'resolved_path': {'path': '::std::option::Option', 'id': 89, 'args': {'angle_bracketed': {'args': [{'type': {'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['pre', {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}], ['deep', {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}], ['pvec', {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}]], 'output': None, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}}], 'constraints': []}}}}]], 'output': {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREP `inspect` method.

### `sys::altrep::R_altrep_Length_method_t`

`type R_altrep_Length_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREP `length` method.

### `sys::altrep::R_altrep_Serialized_state_method_t`

`type R_altrep_Serialized_state_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREP `serialized_state` method.

### `sys::altrep::R_altrep_UnserializeEX_method_t`

`type R_altrep_UnserializeEX_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['class', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['state', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['attr', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['objf', {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}], ['levs', {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREP extended `unserialize` method.

### `sys::altrep::R_altrep_Unserialize_method_t`

`type R_altrep_Unserialize_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['class', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['state', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTREP `unserialize` method.

### `sys::altrep::R_altstring_Elt_method_t`

`type R_altstring_Elt_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTSTRING `elt` method.

### `sys::altrep::R_altstring_Is_sorted_method_t`

`type R_altstring_Is_sorted_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTSTRING `is_sorted` method.

### `sys::altrep::R_altstring_No_NA_method_t`

`type R_altstring_No_NA_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': '::std::os::raw::c_int', 'id': 246, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTSTRING `no_na` method.

### `sys::altrep::R_altstring_Set_elt_method_t`

`type R_altstring_Set_elt_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['i', {'resolved_path': {'path': 'crate::sexp_types::R_xlen_t', 'id': 237, 'args': None}}], ['v', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': None, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTSTRING `set_elt` method.

### `sys::altrep::R_altvec_Dataptr_method_t`

`type R_altvec_Dataptr_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['writable', {'resolved_path': {'path': 'crate::sexp_types::Rboolean', 'id': 240, 'args': None}}]], 'output': {'raw_pointer': {'is_mutable': True, 'type': {'resolved_path': {'path': '::std::os::raw::c_void', 'id': 4026, 'args': None}}}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTVEC `dataptr` method.

### `sys::altrep::R_altvec_Dataptr_or_null_method_t`

`type R_altvec_Dataptr_or_null_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'raw_pointer': {'is_mutable': False, 'type': {'resolved_path': {'path': '::std::os::raw::c_void', 'id': 4026, 'args': None}}}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTVEC `dataptr_or_null` method.

### `sys::altrep::R_altvec_Extract_subset_method_t`

`type R_altvec_Extract_subset_method_t = ::std::option::Option<{'function_pointer': {'sig': {'inputs': [['x', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['indx', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}], ['call', {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}]], 'output': {'resolved_path': {'path': 'crate::SEXP', 'id': 236, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': True, 'is_async': False, 'abi': {'C': {'unwind': True}}}}}>`

Signature for ALTVEC `extract_subset` method.
