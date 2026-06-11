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
| `refcount-fast-hash` | Use ahash for refcount arenas (opt-in, not DOS-resistant) |

---

## Structs

### `Altrep`

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

### `AltrepRegRow`

Pre-extracted view of one `MX_ALTREP_REGISTRATIONS` entry.

**Fields:**

- `symbol`: `String`

### `AltrepRegistration`

ALTREP class registration entry: fn pointer + `#[no_mangle]` symbol name.

See [`MX_ALTREP_REGISTRATIONS`] for context.

**Fields:**

- `register`: `{'function_pointer': {'sig': {'inputs': [], 'output': None, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': False, 'is_async': False, 'abi': {'C': {'unwind': False}}}}}`
  - Registration function called once at `R_init_*`.
- `symbol`: `&''static str`
  - Symbol name of `register` (e.g. `"__mx_altrep_reg_MyType"`). Consumed by

### `AltrepSexp`

A SEXP known to be ALTREP. `!Send + !Sync` — must be materialized on the
R main thread before data can be accessed or sent to other threads.

This type prevents ALTREP vectors from being accidentally sent to rayon
or other worker threads where `DATAPTR_RO` would invoke R internals
(undefined behavior).

# As a `#[miniextendr]` parameter

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

# Construction

- [`AltrepSexp::try_wrap`] — runtime check, returns `None` if not ALTREP
- [`AltrepSexp::from_raw`] — unsafe, caller asserts `ALTREP(sexp) != 0`

# Materialization

All materialization methods must be called on the R main thread.

- [`AltrepSexp::materialize`] — forces R to materialize, returns plain SEXP
- [`AltrepSexp::materialize_integer`] — materialize INTSXP and return `&[i32]`
- [`AltrepSexp::materialize_real`] — materialize REALSXP and return `&[f64]`
- [`AltrepSexp::materialize_logical`] — materialize LGLSXP and return `&[i32]`
- [`AltrepSexp::materialize_raw`] — materialize RAWSXP and return `&[u8]`
- [`AltrepSexp::materialize_complex`] — materialize CPLXSXP and return `&[Rcomplex]`
- [`AltrepSexp::materialize_strings`] — materialize STRSXP to `Vec<Option<String>>`

# Thread safety

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

# Safety

The returned SEXP is still ALTREP. Do not call `DATAPTR_RO` on it
from a non-R thread.

#### `from_raw`

```rust
unsafe from_raw(sexp: SEXP) -> Self
```

Wrap a SEXP that is known to be ALTREP.

# Safety

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

# Safety

Must be called on the R main thread.

#### `materialize_complex`

```rust
unsafe materialize_complex(self: &Self) -> &[Rcomplex]
```

Materialize and return a typed slice of `Rcomplex` (CPLXSXP).

# Safety

Must be called on the R main thread. The SEXP must be CPLXSXP.

#### `materialize_integer`

```rust
unsafe materialize_integer(self: &Self) -> &[i32]
```

Materialize and return a typed slice of `i32` (INTSXP).

# Safety

Must be called on the R main thread. The SEXP must be INTSXP.

#### `materialize_logical`

```rust
unsafe materialize_logical(self: &Self) -> &[i32]
```

Materialize and return a typed slice of `i32` (LGLSXP, R's internal logical storage).

# Safety

Must be called on the R main thread. The SEXP must be LGLSXP.

#### `materialize_raw`

```rust
unsafe materialize_raw(self: &Self) -> &[u8]
```

Materialize and return a typed slice of `u8` (RAWSXP).

# Safety

Must be called on the R main thread. The SEXP must be RAWSXP.

#### `materialize_real`

```rust
unsafe materialize_real(self: &Self) -> &[f64]
```

Materialize and return a typed slice of `f64` (REALSXP).

# Safety

Must be called on the R main thread. The SEXP must be REALSXP.

#### `materialize_strings`

```rust
unsafe materialize_strings(self: &Self) -> Vec<Option<String>>
```

Materialize strings into owned Rust data.

Each element is `None` for `NA_character_`, or `Some(String)` otherwise.

# Safety

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

### `Arena`

A reference-counted arena for GC protection, generic over map type.

This provides an alternative to R's PROTECT stack that:
- Uses reference counting for each SEXP
- Allows releasing protections in any order
- Has no stack size limit (uses heap allocation)

# Type Aliases

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

# Safety

Must be called from the R main thread.

#### `guard`

```rust
unsafe guard(self: &Self, x: SEXP) -> ArenaGuard<''_, M>
```

Protect a SEXP and return an RAII guard.

# Safety

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

# Safety

Must be called from the R main thread.

#### `protect`

```rust
unsafe protect(self: &Self, x: SEXP) -> SEXP
```

Protect a SEXP, incrementing its reference count.

# Safety

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

# Safety

Must be called from the R main thread.

#### `unprotect`

```rust
unsafe unprotect(self: &Self, x: SEXP)
```

Unprotect a SEXP, decrementing its reference count.

# Safety

Must be called from the R main thread.

# Panics

Panics if `x` was not protected by this arena.

#### `with_capacity`

```rust
unsafe with_capacity(capacity: usize) -> Self
```

Create a new arena with specific initial capacity.

Pre-sizing the arena avoids growth of the backing VECSXP and rehashing
of the internal map. Use this when the expected number of distinct
protected values is known or can be estimated.

# Safety

Must be called from the R main thread.

### `ArenaGuard`

An RAII guard that unprotects a SEXP when dropped.

**Methods:**

#### `get`

```rust
get(self: &Self) -> SEXP
```

Returns the protected SEXP.

#### `new`

```rust
unsafe new(arena: &''a Arena<M>, sexp: SEXP) -> Self
```

Create a new guard that protects the SEXP and unprotects on drop.

# Safety

Must be called from the R main thread. The SEXP must be valid.

### `AsDataFrame`

Wrap a value and convert it to an R `data.frame` via [`IntoDataFrame`](crate::dataframe::IntoDataFrame) when returned.

Use this at a call site to force a single return value into a data.frame without making
that the type's default representation (for the always-a-data.frame default, use
`#[derive(PreferDataFrame)]` / `#[miniextendr(dataframe)]`). The inner `T` is typically a
`Vec<Row>` where `Row` derives [`DataFrameRow`](crate::markers::DataFrameRow).

A failed conversion ([`DataFrameError`](crate::dataframe::DataFrameError)) surfaces in R as
an error condition.

# Example

```ignore
#[derive(DataFrameRow)]
struct Point { x: f64, y: f64 }

#[miniextendr]
fn grid() -> AsDataFrame<Vec<Point>> {
    AsDataFrame(vec![Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }])
}
// In R: grid() returns a data.frame with columns x, y
```

### `AsDisplay`

Wrap a `T: Display` and convert it to an R character scalar.

Any type implementing `std::fmt::Display` can be returned to R as a string
without implementing miniextendr traits.

# Example

```ignore
use std::net::IpAddr;

#[miniextendr]
fn format_ip(ip: &str) -> AsDisplay<IpAddr> {
    AsDisplay(ip.parse().unwrap())
}
// R gets: "192.168.1.1"
```

### `AsDisplayVec`

Wrap a `Vec<T: Display>` and convert it to an R character vector.

# Example

```ignore
#[miniextendr]
fn format_errors(errors: Vec<std::io::Error>) -> AsDisplayVec<std::io::Error> {
    AsDisplayVec(errors)
}
```

### `AsExternalPtr`

Wrap a value and convert it to an R external pointer when returned from Rust.

Use this wrapper when you want to return a Rust value as an opaque pointer
that R code can pass back to Rust functions later.

# Example

```ignore
struct Connection { handle: u64 }

impl IntoExternalPtr for Connection { /* ... */ }

#[miniextendr]
fn open_connection(path: &str) -> AsExternalPtr<Connection> {
    AsExternalPtr(Connection { handle: 42 })
}
// In R: open_connection("foo") returns an external pointer
```

### `AsFromStr`

Wrap a parsed `T: FromStr` from an R character scalar.

Pass an R character scalar and it will be parsed into `T` via `str::parse()`.

# Example

```ignore
use std::net::IpAddr;

#[miniextendr]
fn check_ip(addr: AsFromStr<IpAddr>) -> bool {
    addr.0.is_loopback()
}
// R: check_ip("127.0.0.1") → TRUE
```

### `AsFromStrVec`

Wrap a `Vec<T: FromStr>` parsed from an R character vector.

Each element of the R character vector is parsed into `T`.
All parse errors are collected with their indices.

# Example

```ignore
use std::net::IpAddr;

#[miniextendr]
fn parse_ips(addrs: AsFromStrVec<IpAddr>) -> Vec<bool> {
    addrs.0.into_iter().map(|ip| ip.is_loopback()).collect()
}
// R: parse_ips(c("127.0.0.1", "8.8.8.8")) → c(TRUE, FALSE)
```

### `AsJson`

Serialize `T` to a compact JSON string, return as R character scalar.

# Example

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

### `AsJsonPretty`

Serialize `T` to a pretty-printed JSON string, return as R character scalar.

Same as [`AsJson`] but with indentation for human readability.

### `AsJsonVec`

Serialize each element of a `Vec<T>` to a JSON string, return as R character vector.

# Example

```ignore
#[miniextendr]
fn serialize_points(points: Vec<Point>) -> AsJsonVec<Point> {
    AsJsonVec(points)
}
// R gets: c('{"x":1,"y":2}', '{"x":3,"y":4}')
```

### `AsList`

Wrap a value and convert it to an R list via [`IntoList`] when returned from Rust.

Use this wrapper when you want to convert a single value to an R list without
making that the default behavior for the type.

# Example

```ignore
#[derive(IntoList)]
struct Point { x: f64, y: f64 }

#[miniextendr]
fn make_point() -> AsList<Point> {
    AsList(Point { x: 1.0, y: 2.0 })
}
// In R: make_point() returns list(x = 1.0, y = 2.0)
```

### `AsNamedList`

Wrap a tuple pair collection and convert it to a **named R list** (VECSXP).

Preserves insertion order and allows duplicate names (sequence semantics).

# Supported input types

| Input | Bounds |
|-------|--------|
| `Vec<(K, V)>` | `K: AsRef<str>`, `V: IntoR` |
| `[(K, V); N]` | `K: AsRef<str>`, `V: IntoR` |
| `&[(K, V)]` | `K: AsRef<str>`, `V: Clone + IntoR` |

# Example

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

### `AsNamedVector`

Wrap a tuple pair collection and convert it to a **named atomic R vector**
(INTSXP, REALSXP, LGLSXP, RAWSXP, or STRSXP).

Preserves insertion order and allows duplicate names (sequence semantics).
Values must be homogeneous and implement [`AtomicElement`].

# Supported input types

| Input | Bounds |
|-------|--------|
| `Vec<(K, V)>` | `K: AsRef<str>`, `V: AtomicElement` |
| `[(K, V); N]` | `K: AsRef<str>`, `V: AtomicElement` |
| `&[(K, V)]` | `K: AsRef<str>`, `V: Clone + AtomicElement` |

# Example

```ignore
#[miniextendr]
fn make_scores() -> AsNamedVector<Vec<(&str, f64)>> {
    AsNamedVector(vec![("alice", 95.0), ("bob", 87.5)])
}
// In R: make_scores() returns c(alice = 95.0, bob = 87.5)
```

### `AsRError`

Structured error wrapper that preserves the `std::error::Error` cause chain.

When displayed, formats the error message with its full source chain:
```text
top-level message
  caused by: middle error
  caused by: root cause
```

Implements `From<E>` so it works with `?` and `.map_err(AsRError)`.

# Example

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
rust_type_name(self: &Self) -> &''static str
```

Get the Rust type name of the wrapped error (for programmatic matching).

### `AsRNative`

Wrap a scalar [`RNativeType`] and force native R vector conversion.

This creates a length-1 R vector containing the scalar value. Use this when
you want to ensure a value is converted to its native R representation (e.g.,
`i32` → integer vector, `f64` → numeric vector) rather than another path
like `IntoExternalPtr`.

# Example

```ignore
#[derive(Clone, Copy, RNativeType)]
struct Meters(f64);

#[miniextendr]
fn distance() -> AsRNative<Meters> {
    AsRNative(Meters(42.5))
}
// In R: distance() returns 42.5 (numeric vector of length 1)
```

# Performance

This wrapper directly allocates an R vector and writes the value,
avoiding intermediate Rust allocations.

### `AsSerialize`

Wrapper that converts any `Serialize` type to R via serde_r.

This is the serde analog to `AsList<T: IntoList>`. Use it when you want to
return a `Serialize` type from a `#[miniextendr]` function and have it
automatically converted to an R list.

# Example

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

# Extracting the inner value

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

### `AsVctrs`

Wrap a value and convert it to a **vctrs** S3 vector via [`IntoVctrs`](crate::vctrs::IntoVctrs)
when returned.

Use this at a call site to return a `#[derive(Vctrs)]` type as its R vctrs object without the
manual `value.into_vctrs().map_err(...)` boilerplate. For a type that should *always* convert
this way, use `#[derive(Vctrs, PreferVctrs)]` instead.

A failed build ([`VctrsBuildError`](crate::vctrs::VctrsBuildError)) surfaces in R as an error
condition.

# Example

```ignore
#[derive(Vctrs)]
#[vctrs(class = "percent", base = "double")]
struct Percent { #[vctrs(data)] values: Vec<f64> }

#[miniextendr]
fn percent(x: Vec<f64>) -> AsVctrs<Percent> {
    AsVctrs(Percent { values: x })
}
```

### `Borsh`

Wrapper for borsh-serializable types.

Converts between R raw vectors (RAWSXP) and borsh binary format.
Use `Borsh(value)` to wrap a value for conversion.

### `CallDefRow`

Pre-extracted, cdylib-side view of one `R_CallMethodDef`.

`R_CallMethodDef` carries `name` as a raw `*const c_char`; safely walking
it requires `unsafe`. The formatter takes already-extracted, owned values
so it can be unit-tested without globals.

**Fields:**

- `name`: `String`
- `num_args`: `i32`

### `CaptureGroups`

Wrapper for regex capture groups.

This type wraps `regex::Captures` for access from R.
It holds owned copies of capture group strings for safe access.

**Methods:**

#### `capture`

```rust
capture(re: &Regex, text: &str) -> Option<Self>
```

Create capture groups from a regex and text.

### `ClassNameEntry`

Entry mapping a Rust type name to its R-visible class name and class system.

Emitted by every `#[miniextendr(env|r6|s3|s4|s7|vctrs)]` impl block.
Used by the resolver in `write_r_wrappers_to_file` to replace
`.__MX_CLASS_REF_<RustName>__` placeholders with the actual R class name.

**Fields:**

- `rust_type`: `&''static str`
  - Rust type identifier, e.g. `"S7Shape"`.
- `r_class_name`: `&''static str`
  - R-visible class name. Equals `rust_type` unless `class = "Override"` was
- `class_system`: `&''static str`
  - Class system tag: `"env"` | `"r6"` | `"s3"` | `"s4"` | `"s7"` | `"vctrs"`.

### `Coerced`

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

### `Collect`

Write an `ExactSizeIterator` of native R types directly into an R vector.

Skips the intermediate `Vec` allocation — the R vector is allocated once
and the iterator writes directly into it.

Requires `ExactSizeIterator` because R vectors must know their length
at allocation time.

# Naming

`Collect` is in the representation-forcing wrapper family but does not take the
`As*` prefix used by [`AsList`] / [`AsExternalPtr`] / [`AsRNative`]: those wrap a
finished value `T`, whereas `Collect` wraps an *iterator* and materializes it into
an R vector. The divergence is intentional — see the module docs and #871.

# Example

```ignore
#[miniextendr]
fn sines(n: i32) -> Collect<impl ExactSizeIterator<Item = f64>> {
    Collect((0..n).map(|i| (i as f64).sin()))
}
```

### `CollectNA`

Write an `ExactSizeIterator` of `Option<T>` directly into an R vector with NA support.

`None` values become `NA` in R. Works for `f64` and `i32`.

Like [`Collect`], this is an iterator adapter and is exempt from the `As*`
naming convention (see #871).

# Example

```ignore
#[miniextendr]
fn with_gaps(n: i32) -> CollectNA<impl ExactSizeIterator<Item = Option<f64>>> {
    CollectNA((0..n).map(|i| if i % 3 == 0 { None } else { Some(i as f64) }))
}
```

### `CollectNAInt`

Write an `ExactSizeIterator` of `Option<i32>` directly into an R integer vector with NA.

Like [`Collect`], this is an iterator adapter and is exempt from the `As*`
naming convention (see #871).

### `CollectStrings`

Write an `ExactSizeIterator` of `String` directly into an R character vector.

Strings require per-element CHARSXP allocation (no bulk `copy_from_slice`),
so this is a separate type from [`Collect`]. Like [`Collect`], it is an
iterator adapter and is exempt from the `As*` naming convention (see #871).

# Example

```ignore
#[miniextendr]
fn upper(words: Vec<String>) -> CollectStrings<impl ExactSizeIterator<Item = String>> {
    CollectStrings(words.into_iter().map(|w| w.to_uppercase()))
}
```

### `DataFrame`

An owned, validated R `data.frame`. **The** data-frame type.

Wraps a built VECSXP carrying the `data.frame` class + `row.names`. A single coherent
type for building (Rust → R), reading (R → Rust), and post-assembly editing — replacing
the historical row-buffer / built-SEXP / read-wrapper trio with one coherent type.

# Building

Prefer the [`IntoDataFrame`] trait on your data:

```ignore
let df: DataFrame = rows.into_dataframe()?;
```

or the closure-fill [`DataFrame::builder`] for heterogeneous parallel column fill
(`feature = "rayon"`).

# Reading

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

# Safety

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

# Errors

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

# PROTECT discipline

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

### `DispatchNames`

Custom slot names for [`dispatch_to_dataframes`]'s output list.

Defaults to `ok = "ok"`, `err = "err"`. Override either or both via
`DispatchNames { ok: "results".into(), err: "errors".into() }`.

**Fields:**

- `ok`: `String`
- `err`: `String`

### `DllInfo`

Opaque dynamic library descriptor from R.

### `Dots`

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

# Safety Note

This is safe because the miniextendr macro always wraps `...` in
`list(...)` on the R side. However, if you're receiving a SEXP
from another source, use [`try_list`](Dots::try_list) instead.

# Example
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

# Example
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

# Errors

Returns [`ListFromSexpError`] if:
- The SEXP is not a list type (VECSXP or pairlist)
- The list contains duplicate non-NA names

# Example
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

# Example

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

# Errors

Returns [`TypedListError`] if:
- The dots are not a valid list
- A required field is missing
- A field has the wrong type or length
- Extra fields exist when `allow_extra = false`

### `DuplicateNameError`

Error when a list has duplicate non-NA names.

**Fields:**

- `name`: `String`
  - The duplicate name that was found.

### `ExternalPtr`

An owned pointer stored in R's external pointer SEXP.

This is conceptually similar to `Box<T>`, but with the following differences:
- Memory is freed by R's GC via a registered finalizer (non-deterministic)
- The underlying SEXP is Copy, so aliasing must be manually prevented
- Type checking happens at runtime via `Any::downcast` (Rust `TypeId`)

# Thread Safety

`ExternalPtr` is `Send` to allow returning from worker thread functions.
However, **concurrent access is not allowed** - R's runtime is single-threaded.
All R API calls are serialized through the main thread via `with_r_thread`.

# Safety

The ExternalPtr assumes exclusive ownership of the underlying data.
Cloning the raw SEXP without proper handling will lead to double-free.

# Examples

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

# Warning

The returned SEXP must not be duplicated or the finalizer will double-free.

#### `assume_init`

```rust
assume_init(self: Self) -> ExternalPtr<T>
```

Converts to `ExternalPtr<T>`.

# Safety

The value must have been initialized.

# Implementation Note

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

# Safety

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

# Safety

- `raw` must have been allocated via `Box::into_raw` or equivalent
- `raw` must not be null
- Caller transfers ownership to the ExternalPtr
- Must be called from R's main thread (no debug assertions)

#### `from_sexp`

```rust
unsafe from_sexp(sexp: SEXP) -> Self
```

Create a type-erased ExternalPtr from an EXTPTRSXP without checking the stored type.

# Safety

- `sexp` must be a valid EXTPTRSXP
- Caller must ensure exclusive ownership semantics are upheld

#### `from_sexp_unchecked`

```rust
unsafe from_sexp_unchecked(sexp: SEXP) -> Self
```

Create an ExternalPtr from an SEXP without type checking.

# Safety

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
leak<'a>(this: Self) -> &''a mut T
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

# Panics

Panics if called from a non-main thread outside of a `run_on_worker` context.

Equivalent to `Box::new`.

[`with_r_thread`]: crate::worker::with_r_thread

#### `new_unchecked`

```rust
unsafe new_unchecked(x: T) -> Self
```

Allocates memory on the heap and places `x` into it, without thread checks.

# Safety

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

# Note

Unlike `Box::pin`, this requires `T: Unpin` because `ExternalPtr`
implements `DerefMut` unconditionally. For `!Unpin` types, use
`ExternalPtr::new` and manage pinning guarantees manually.

#### `pin_unchecked`

```rust
pin_unchecked(x: T) -> Pin<Self>
```

Constructs a new `Pin<ExternalPtr<T>>` without requiring `Unpin`.

# Safety

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

# Safety

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

# Safety note

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

# Safety

- `user_prot` must be a valid SEXP or R_NilValue
- Must be called from the R main thread

#### `stored_type_name`

```rust
stored_type_name(self: &Self) -> Option<&''static str>
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

# Safety

Must be called from the R main thread. Only use in ALTREP callbacks
or other contexts where you're certain you're on the main thread.

#### `type_name`

```rust
type_name() -> &''static str
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

# Safety

- `sexp` must be a valid EXTPTRSXP created by this library
- The caller must ensure no other ExternalPtr owns this SEXP

#### `wrap_sexp_unchecked`

```rust
unsafe wrap_sexp_unchecked(sexp: SEXP) -> Option<Self>
```

Attempt to wrap a SEXP as an ExternalPtr (unchecked version).

Skips thread safety checks for performance-critical paths like ALTREP callbacks.

# Safety

- `sexp` must be a valid EXTPTRSXP created by this library
- The caller must ensure exclusive ownership
- Must be called from the R main thread (guaranteed in ALTREP callbacks)

#### `wrap_sexp_with_error`

```rust
unsafe wrap_sexp_with_error(sexp: SEXP) -> Result<Self, TypeMismatchError>
```

Attempt to wrap a SEXP as an ExternalPtr, returning an error with type info on mismatch.

This is used by the [`TryFromSexp`] trait implementation.

# Safety

Same as [`wrap_sexp`](Self::wrap_sexp).

[`TryFromSexp`]: crate::TryFromSexp

#### `write`

```rust
write(self: Self, value: T) -> ExternalPtr<T>
```

Writes a value and converts to initialized.

Creates a new SEXP with `T`'s type information (the original
`MaybeUninit<T>` SEXP becomes an orphaned shell, cleaned up by GC).

### `ExternalSlice`

A slice stored as a standalone struct, suitable for wrapping in ExternalPtr.

This is analogous to the data inside a `Box<[T]>`, but stores capacity
for proper deallocation when created from a `Vec`.

# Usage

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

### `Factor`

A borrowed view into an R factor's integer indices.

Provides `Deref` to `&[i32]` for direct slice access to the factor's
underlying integer data. The indices are 1-based (matching R's convention)
with `NA_INTEGER` for missing values.

# Example

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
level(self: &Self, idx: usize) -> &''a str
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

### `FactorMut`

A mutable borrowed view into an R factor's integer indices.

Provides `DerefMut` to `&mut [i32]` for direct mutable slice access.
The indices are 1-based (matching R's convention) with `NA_INTEGER` for NA.

# Example

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
level(self: &Self, idx: usize) -> &''a str
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

### `FactorOptionVec`

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

### `FactorVec`

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

### `FromJson`

Parse an R character scalar as JSON into `T: Deserialize`.

# Example

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

### `IterComplexData`

Iterator-backed complex number vector.

Wraps an iterator producing `Rcomplex` values and exposes it as an ALTREP complex vector.

# Example

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

### `IterIntCoerceData`

Iterator-backed integer vector with coercion from any integer-like type.

Wraps an iterator producing values that coerce to `i32` (e.g., `u16`, `i8`, etc.).

# Example

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

### `IterIntData`

Iterator-backed integer vector data.

Wraps an iterator producing `i32` values and exposes it as an ALTREP integer vector.

# Example

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

### `IterIntFromBoolData`

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

### `IterListData`

Iterator-backed list vector.

Wraps an iterator producing R `SEXP` values and exposes it as an ALTREP list.

# Safety

The iterator must produce valid, protected SEXP values. Each SEXP must remain
protected for the lifetime of the ALTREP object.

# Example

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

# Safety

The iterator must produce valid, protected SEXP values.

#### `from_iter`

```rust
from_iter(iter: I, len: usize) -> Self
```

Create from an iterator with explicit length.

# Safety

The iterator must produce valid, protected SEXP values.

### `IterLogicalData`

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

### `IterRawData`

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

### `IterRealCoerceData`

Iterator-backed real vector with coercion from any float-like type.

Wraps an iterator producing values that coerce to `f64` (e.g., `f32`, integer types).

# Example

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

### `IterRealData`

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

### `IterState`

Core state for iterator-backed ALTREP vectors.

Provides lazy element generation with caching for random-access semantics.
Iterator elements are cached as they're accessed, enabling repeatable reads.

# Type Parameters

- `I`: The iterator type (must be `ExactSizeIterator` or provide explicit length)
- `T`: The element type produced by the iterator

# Design

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

# Returns

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

# Length Mismatch Handling

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

# Arguments

- `iter`: The iterator to wrap
- `len`: The expected number of elements

# Length Mismatch

If the iterator produces a different number of elements than `len`:
- Fewer elements: Missing indices return `None`/NA/default values
- More elements: Extra elements are ignored (truncated to `len`)

A warning is printed to stderr when a mismatch is detected.

### `IterStringData`

Iterator-backed string vector.

Wraps an iterator producing `String` values and exposes it as an ALTREP character vector.

# Performance Warning

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

# Example

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

### `JiffTimestampVec`

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

### `JiffTimestampVecMut`

Mutable reference wrapper for [`JiffTimestampVec`] ALTREP data. Implements `TryFromSexp`, `Deref`, and `DerefMut`.

### `JiffTimestampVecRef`

Immutable reference wrapper for [`JiffTimestampVec`] ALTREP data. Implements `TryFromSexp` and `Deref<Target = JiffTimestampVec>`.

### `JiffZonedVec`

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

### `JiffZonedVecMut`

Mutable reference wrapper for [`JiffZonedVec`] ALTREP data. Implements `TryFromSexp`, `Deref`, and `DerefMut`.

### `JiffZonedVecRef`

Immutable reference wrapper for [`JiffZonedVec`] ALTREP data. Implements `TryFromSexp` and `Deref<Target = JiffZonedVec>`.

### `JsonOptions`

Options for converting R objects to JSON.

# Example

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

### `List`

Owned handle to an R list (`VECSXP`).

# Examples

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

# Errors

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

# Safety

Caller must ensure `sexp` is a valid list object (typically a `VECSXP` or
a pairlist coerced to `VECSXP`) whose lifetime remains managed by R.

#### `from_raw_pairs`

```rust
from_raw_pairs<N>(pairs: Vec<(N, SEXP)>) -> Self
```

Build a list from `(name, SEXP)` pairs (heterogeneous-friendly).

# Safety Note

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

# Safety Note

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

# Safety Note

The input SEXPs should already be protected or be children of protected
containers.

#### `from_values`

```rust
from_values<T>(values: Vec<T>) -> Self
```

Build an unnamed list from values.

Use this for tuple-like structures where positional access is more natural.

# Example

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

# Example

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

# Safety

- Must be called from the R main thread
- `child` must be a valid SEXP
- `self` must be a valid, protected VECSXP

# Panics

Panics if `idx` is out of bounds.

# Example

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

# Safety

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

# Safety

- Must be called from the R main thread
- `self` must be a valid, protected VECSXP

# Example

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

# Example

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

# Example

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

# Example

```ignore
let list = List::from_pairs(vec![
    ("x", vec![1, 2, 3]),
])
.set_class_str(&["data.frame"])
.set_row_names_str(&["row_a", "row_b", "row_c"]);
```

### `ListAccumulator`

Accumulator for building lists when the length is unknown upfront.

Unlike [`super::ListBuilder`] which requires knowing the size at construction,
`ListAccumulator` supports dynamic growth via [`push`](Self::push). It uses
[`ReprotectSlot`] internally to maintain **O(1) protect stack usage** regardless
of how many elements are pushed.

# When to Use

| Scenario | Recommended Type |
|----------|-----------------|
| Known size | [`super::ListBuilder`] - more efficient, no reallocation |
| Unknown size | `ListAccumulator` - bounded stack, dynamic growth |
| Streaming/iterators | `ListAccumulator` or [`collect_list`] |

# Growth Strategy

The internal list grows exponentially (2x) when capacity is exceeded,
achieving amortized O(1) push. Elements are copied during growth.

# Example

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

# Safety

Must be called from the R main thread.

#### `into_root`

```rust
unsafe into_root(self: Self) -> Root<''a>
```

Finalize the accumulator and return a `Root` pointing to the list.

The returned list is truncated to the actual length (if smaller than capacity).

# Safety

Must be called from the R main thread.

#### `into_sexp`

```rust
unsafe into_sexp(self: Self) -> SEXP
```

Finalize and return the raw SEXP.

# Safety

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
unsafe new(scope: &''a ProtectScope, initial_cap: usize) -> Self
```

Create a new accumulator with the given initial capacity.

A capacity of 0 is valid; the list will grow on first push.

# Safety

Must be called from the R main thread.

#### `push`

```rust
unsafe push<T>(self: &mut Self, value: T)
```

Push a value onto the accumulator.

The value is converted to a SEXP via [`IntoR`] and inserted.
If the internal list is full, it grows automatically.

# Safety

Must be called from the R main thread.

#### `push_if`

```rust
unsafe push_if<T>(self: &mut Self, condition: bool, value: T)
```

Push a value only if the condition is true.

# Safety

Must be called from the R main thread.

#### `push_if_with`

```rust
unsafe push_if_with<T>(self: &mut Self, condition: bool, f: impl FnOnce)
```

Push a lazily-evaluated value only if the condition is true.

The closure is only called if `condition` is true.

# Safety

Must be called from the R main thread.

#### `push_named`

```rust
unsafe push_named<T>(self: &mut Self, name: &str, value: T)
```

Push a named value onto the accumulator.

# Safety

Must be called from the R main thread.

#### `push_sexp`

```rust
unsafe push_sexp(self: &mut Self, sexp: SEXP)
```

Push a raw SEXP onto the accumulator.

# Safety

- Must be called from the R main thread
- `sexp` must be a valid SEXP (it will be temporarily protected)

### `ListBuilder`

Builder for constructing lists with efficient protection management.

`ListBuilder` holds a reference to a [`ProtectScope`], allowing multiple
elements to be inserted without repeatedly protecting/unprotecting each one.
This is more efficient than using [`List::set_elt`] in a loop.

# Example

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
unsafe from_protected(scope: &''a ProtectScope, list: SEXP) -> Self
```

Create a builder wrapping an existing protected list.

# Safety

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
unsafe new(scope: &''a ProtectScope, len: usize) -> Self
```

Create a new list builder with the given length.

The list is allocated and protected using the provided scope.

# Safety

Must be called from the R main thread.

#### `set`

```rust
unsafe set(self: &Self, idx: isize, child: SEXP)
```

Set an element at the given index.

The `child` should be protected by the same scope (or a parent scope).
Use `scope.protect_raw(...)` before calling this method.

# Safety

- `child` must be a valid SEXP
- `child` should be protected (typically via the same scope)

#### `set_protected`

```rust
unsafe set_protected(self: &Self, idx: isize, child: SEXP)
```

Set an element, protecting the child within the builder's scope.

This is a convenience method that protects the child and then inserts it.

# Safety

- `child` must be a valid SEXP

### `ListMut`

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

# Safety

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

### `MapSerializer`

Serializer for maps (HashMap, BTreeMap).

### `MatchArgChoicesEntry`

Entry for replacing match_arg placeholder defaults with actual choices.

**Fields:**

- `placeholder`: `&''static str`
  - Placeholder string in the R formal default, e.g. `".__MX_MATCH_ARG_CHOICES_mode__"`.
- `choices_str`: `{'function_pointer': {'sig': {'inputs': [], 'output': {'resolved_path': {'path': 'String', 'id': 26, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': False, 'is_async': False, 'abi': 'Rust'}}}`
  - Function that returns the choices as a comma-separated quoted string,
- `preferred_default`: `&''static str`
  - User-supplied `default = "..."` value (unquoted, e.g. `"zstd"`), or `""`

### `MatchArgParamDocEntry`

Entry for replacing match_arg `@param` doc placeholders with human-readable
choice descriptions.

**Fields:**

- `placeholder`: `&''static str`
  - Placeholder string in the `@param` roxygen tag, e.g.
- `several_ok`: `bool`
  - `true` for `several_ok` params (emits "One or more of …");
- `choices_str`: `{'function_pointer': {'sig': {'inputs': [], 'output': {'resolved_path': {'path': 'String', 'id': 26, 'args': None}}, 'is_c_variadic': False}, 'generic_params': [], 'header': {'is_const': False, 'is_unsafe': False, 'is_async': False, 'abi': 'Rust'}}}`
  - Function that returns the choices as a comma-separated quoted string,

### `NamedDataFrameListBuilder`

Assemble a named list whose elements are [`DataFrame`]s,
without per-result `OwnedProtect` bookkeeping.

# Why this is distinct from [`DataFrame::builder`]

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

# Example

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

# Safety (caller)

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

### `NamedList`

A named list with O(1) name-based element lookup.

Wraps a [`List`] and builds a `HashMap<String, usize>` index of element names
on construction. Use this when you need to access multiple elements by name
from the same list — each lookup is O(1) instead of O(n).

# When to Use

| Pattern | Type |
|---------|------|
| Single named lookup | [`List::get_named`] is fine |
| Multiple named lookups | `NamedList` (O(n) build + O(1) per lookup) |
| Positional access only | [`List`] — no indexing overhead |

# Name Handling

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

# Errors

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

### `NamedVector`

Wrapper that converts a map to/from a **named atomic R vector** instead of a
named list.

The inner map must have `String` keys and values that implement [`AtomicElement`].

# Supported value types

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

### `NullOnErr`

Marker type for `Result<T, ()>` that converts `Err(())` to NULL.

This type is used internally by the `#[miniextendr]` macro when handling
`Result<T, ()>` return types. When the error type is `()`, there's no
error message to report, so we return NULL instead of raising an error.

# Usage

You typically don't use this directly. When you write:

```ignore
#[miniextendr]
fn maybe_value(x: i32) -> Result<i32, ()> {
    if x > 0 { Ok(x) } else { Err(()) }
}
```

The macro generates code that converts `Err(())` to `Err(NullOnErr)` and
returns `NULL` in R.

# Note

`NullOnErr` intentionally does NOT implement `Display` to avoid conflicting
with the generic `IntoR for Result<T, E: Display>` impl. It has its own
specialized `IntoR` impl that returns NULL on error.

### `OwnedProtect`

A single-object RAII guard: `PROTECT` on create, `UNPROTECT(1)` on drop.

Use this for simple cases where you're protecting a single value and
don't need the batching benefits of [`ProtectScope`].

# Example

```ignore
unsafe fn allocate_and_fill() -> SEXP {
    let guard = OwnedProtect::new(Rf_allocVector(REALSXP, 10));
    fill_vector(guard.get());
    // Return the SEXP - guard drops and unprotects on this line.
    // This is safe because no GC can occur between unprotect and return.
    guard.get()
}
```

# Warning: Stack Ordering

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

# Safety

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

# Safety

- Must be called from the R main thread
- `x` must be a valid SEXP

### `PanicReport`

A structured panic report passed to the telemetry hook.

**Fields:**

- `message`: `&''a str`
  - The panic message (extracted from the panic payload).
- `source`: `PanicSource`
  - Which panic→R-error boundary caught this panic.

### `ProtectKey`

Generational key for a slot in a [`ProtectPool`].

Contains a slot index and a generation counter. If a slot is released and
reused, the old key's generation won't match and operations will safely
return `None` or no-op.

8 bytes: 4-byte slot index + 4-byte generation.

### `ProtectPool`

A VECSXP-backed pool for GC protection with generational keys.

# Example

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

# Safety

Must be called from the R main thread. `sexp` must be a valid SEXP.

# Panics

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

# Safety

Must be called from the R main thread.

#### `release`

```rust
unsafe release(self: &mut Self, key: ProtectKey)
```

Release a previously protected SEXP.

If the key is stale (already released, or from a different pool), this is a no-op.

# Safety

Must be called from the R main thread.

#### `replace`

```rust
unsafe replace(self: &mut Self, key: ProtectKey, sexp: SEXP) -> bool
```

Overwrite the SEXP at an existing key without releasing/reinserting.

Returns `true` if the key was valid and the value was replaced.
Returns `false` if the key was stale (no-op).

This is the pool equivalent of `R_Reprotect` — O(1), no allocation.

# Safety

Must be called from the R main thread. `sexp` must be a valid SEXP.

#### `with_capacity`

```rust
unsafe with_capacity(capacity: usize) -> Self
```

Create a new pool with a specific initial capacity.

# Safety

Must be called from the R main thread.

# Panics

Panics if `capacity` exceeds `R_xlen_t::MAX` or `u32::MAX`.

### `ProtectScope`

A scope that automatically balances `UNPROTECT(n)` on drop.

This is the primary tool for managing GC protection in batch operations.
Each call to [`protect`][Self::protect] or [`protect_with_index`][Self::protect_with_index]
increments an internal counter; when the scope is dropped, `UNPROTECT(n)` is called.

# Example

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

# Nested Scopes

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
unsafe alloc_character<'a>(self: &''a Self, n: usize) -> Root<''a>
```

Allocate a character vector (STRSXP), protected.

# Safety

Must be called from the R main thread.

#### `alloc_complex`

```rust
unsafe alloc_complex<'a>(self: &''a Self, n: usize) -> Root<''a>
```

Allocate a complex vector (CPLXSXP), protected.

# Safety

Must be called from the R main thread.

#### `alloc_integer`

```rust
unsafe alloc_integer<'a>(self: &''a Self, n: usize) -> Root<''a>
```

Allocate an integer vector (INTSXP), protected.

# Safety

Must be called from the R main thread.

#### `alloc_list`

```rust
unsafe alloc_list<'a>(self: &''a Self, n: i32) -> Root<''a>
```

Allocate a list (VECSXP) of the given length and immediately protect it.

# Safety

Same as [`alloc_vector`][Self::alloc_vector].

#### `alloc_logical`

```rust
unsafe alloc_logical<'a>(self: &''a Self, n: usize) -> Root<''a>
```

Allocate a logical vector (LGLSXP), protected.

# Safety

Must be called from the R main thread.

#### `alloc_matrix`

```rust
unsafe alloc_matrix<'a>(self: &''a Self, ty: SEXPTYPE, nrow: i32, ncol: i32) -> Root<''a>
```

Allocate a matrix of the given type and dimensions, and immediately protect it.

# Safety

Same as [`alloc_vector`][Self::alloc_vector].

#### `alloc_raw`

```rust
unsafe alloc_raw<'a>(self: &''a Self, n: usize) -> Root<''a>
```

Allocate a raw vector (RAWSXP), protected.

# Safety

Must be called from the R main thread.

#### `alloc_real`

```rust
unsafe alloc_real<'a>(self: &''a Self, n: usize) -> Root<''a>
```

Allocate a real vector (REALSXP), protected.

# Safety

Must be called from the R main thread.

#### `alloc_strsxp`

```rust
unsafe alloc_strsxp<'a>(self: &''a Self, n: usize) -> Root<''a>
```

Allocate a STRSXP (character vector) of the given length and immediately protect it.

# Safety

Same as [`alloc_vector`][Self::alloc_vector].

#### `alloc_vecsxp`

```rust
unsafe alloc_vecsxp<'a>(self: &''a Self, n: usize) -> Root<''a>
```

Allocate a VECSXP (generic list) of the given length and immediately protect it.

# Safety

Same as [`alloc_vector`][Self::alloc_vector].

#### `alloc_vector`

```rust
unsafe alloc_vector<'a>(self: &''a Self, ty: SEXPTYPE, n: R_xlen_t) -> Root<''a>
```

Allocate a vector of the given type and length, and immediately protect it.

This combines allocation and protection in a single step, eliminating the
GC gap that exists when you separately allocate and then protect.

# Safety

- Must be called from the R main thread
- Only protects the newly allocated object; does not protect other live
  unprotected objects during allocation

# Example

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
unsafe coerce<'a>(self: &''a Self, x: SEXP, target: SEXPTYPE) -> Root<''a>
```

Coerce a SEXP to a different type, protected.

# Safety

Must be called from the R main thread. `x` must be a valid SEXP.

#### `collect`

```rust
unsafe collect<'a, T, I>(self: &''a Self, iter: I) -> Root<''a>
```

Collect an iterator into a typed R vector.

This allocates once, protects, and fills directly - the most efficient pattern
for typed vectors. The element type `T` determines the R vector type via
the [`RNativeType`] trait.

# Type Mapping

| Rust Type | R Vector Type |
|-----------|---------------|
| `i32` | `INTSXP` |
| `f64` | `REALSXP` |
| `u8` | `RAWSXP` |
| [`RLogical`](crate::RLogical) | `LGLSXP` |
| [`Rcomplex`](crate::Rcomplex) | `CPLXSXP` |

# Safety

Must be called from the R main thread.

# Example

```ignore
unsafe fn squares(n: usize) -> SEXP {
    let scope = ProtectScope::new();
    // Type inferred from iterator
    scope.collect((0..n).map(|i| (i * i) as i32)).get()
}
```

# Unknown Length

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

# Safety

You must ensure the protects performed in this scope are correctly
unprotected elsewhere, or you will leak protect stack entries.

#### `duplicate`

```rust
unsafe duplicate<'a>(self: &''a Self, x: SEXP) -> Root<''a>
```

Deep-duplicate a SEXP, protected.

# Safety

Must be called from the R main thread. `x` must be a valid SEXP.

#### `mkchar`

```rust
unsafe mkchar<'a>(self: &''a Self, s: &str) -> Root<''a>
```

Create a CHARSXP from a Rust `&str`, protected.

# Safety

Must be called from the R main thread.

#### `new`

```rust
unsafe new() -> Self
```

Create a new protection scope.

# Safety

Must be called from the R main thread.

#### `new_env`

```rust
unsafe new_env<'a>(self: &''a Self, parent: SEXP, hash: bool, size: i32) -> Root<''a>
```

Create a new environment, protected.

# Safety

Must be called from the R main thread.

#### `protect`

```rust
unsafe protect<'a>(self: &''a Self, x: SEXP) -> Root<''a>
```

Protect `x` and return a rooted handle tied to this scope.

This always calls `Rf_protect`. The protection is released when
the scope is dropped (along with all other protections in this scope).

# Safety

- Must be called from the R main thread
- `x` must be a valid SEXP

#### `protect2`

```rust
unsafe protect2<'a>(self: &''a Self, a: SEXP, b: SEXP) -> (Root<''a>, Root<''a>)
```

Protect two values at once (convenience method).

# Safety

Same as [`protect`][Self::protect].

#### `protect3`

```rust
unsafe protect3<'a>(self: &''a Self, a: SEXP, b: SEXP, c: SEXP) -> (Root<''a>, Root<''a>, Root<''a>)
```

Protect three values at once (convenience method).

# Safety

Same as [`protect`][Self::protect].

#### `protect_raw`

```rust
unsafe protect_raw(self: &Self, x: SEXP) -> SEXP
```

Protect and return the raw `SEXP` (sometimes more convenient).

# Safety

Same as [`protect`][Self::protect].

#### `protect_with_index`

```rust
unsafe protect_with_index<'a>(self: &''a Self, x: SEXP) -> ReprotectSlot<''a>
```

Protect `x` with an index slot so it can be replaced later via [`R_Reprotect`].

Use this when you need to update a protected value in-place without
growing the protection stack.

# Safety

- Must be called from the R main thread
- `x` must be a valid SEXP

# Example

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

# Safety

Only call if you know the scope was disarmed and you want to restore
automatic unprotection. Be careful not to double-unprotect.

#### `rooted`

```rust
unsafe rooted<'a>(self: &''a Self, sexp: SEXP) -> Root<''a>
```

Create a `Root<'a>` for an already-protected SEXP without adding protection.

This is useful when you have a SEXP that is already protected by some other
mechanism (e.g., a `ReprotectSlot`) and want to return it as a `Root` tied
to this scope's lifetime for API consistency.

# Safety

- The caller must ensure `sexp` is already protected and will remain
  protected for at least the lifetime of this scope
- Must be called from the R main thread

#### `scalar_complex`

```rust
unsafe scalar_complex<'a>(self: &''a Self, x: crate::Rcomplex) -> Root<''a>
```

Create a scalar complex (length-1 CPLXSXP), protected.

# Safety

Must be called from the R main thread.

#### `scalar_integer`

```rust
unsafe scalar_integer<'a>(self: &''a Self, x: i32) -> Root<''a>
```

Create a scalar integer (length-1 INTSXP), protected.

# Safety

Must be called from the R main thread.

#### `scalar_logical`

```rust
unsafe scalar_logical<'a>(self: &''a Self, x: bool) -> Root<''a>
```

Create a scalar logical (length-1 LGLSXP), protected.

# Safety

Must be called from the R main thread.

#### `scalar_raw`

```rust
unsafe scalar_raw<'a>(self: &''a Self, x: u8) -> Root<''a>
```

Create a scalar raw (length-1 RAWSXP), protected.

# Safety

Must be called from the R main thread.

#### `scalar_real`

```rust
unsafe scalar_real<'a>(self: &''a Self, x: f64) -> Root<''a>
```

Create a scalar real (length-1 REALSXP), protected.

# Safety

Must be called from the R main thread.

#### `scalar_string`

```rust
unsafe scalar_string<'a>(self: &''a Self, s: &str) -> Root<''a>
```

Create a scalar string (length-1 STRSXP) from a Rust `&str`, protected.

# Safety

Must be called from the R main thread.

#### `shallow_duplicate`

```rust
unsafe shallow_duplicate<'a>(self: &''a Self, x: SEXP) -> Root<''a>
```

Shallow-duplicate a SEXP, protected.

# Safety

Must be called from the R main thread. `x` must be a valid SEXP.

### `Protected`

A Rust value (`T`) bundled with an [`OwnedProtect`] guard on an SEXP
the value borrows from. The protect releases on drop; the lifetime
ties any borrows inside `T` to `&self`, so `T`'s SEXP-internal
references can't outlive the protection.

# When to use `Protected<'a, T>` vs the alternatives

| Pattern | Use | Why |
|---------|-----|-----|
| [`OwnedProtect`] | raw SEXP, no Rust view | one-shot protect/unprotect on a single SEXP |
| [`ProtectScope`] + [`Root`] | several SEXPs in one function body | batched UNPROTECT, no Rust view |
| `Protected<'a, T>` | SEXP + Rust view of its data | hand the bundle to callers; borrows in `T` tied to `&self` |

# Notes on Send/Sync

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

# Safety

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

# Safety

- Must be called from the R main thread.
- `sexp` must be a valid SEXP.
- If `inner` borrows from `sexp`, its lifetime parameter must
  match `'a`.

# Example

```ignore
use miniextendr_api::{Protected, OwnedProtect};
use miniextendr_api::prelude::SEXP;

unsafe fn wrap_view(sexp: SEXP, view: MyView<'_>) -> Protected<'_, MyView<'_>> {
    // Protect the SEXP and bundle it with the view.
    // The view's borrow is tied to the returned Protected.
    Protected::new(sexp, view)
}
```

### `ProtectedStrVec`

GC-protected view over an R character vector (`STRSXP`).

Unlike [`StrVec`] (which is `Copy` and trusts the caller for GC protection),
`ProtectedStrVec` wraps a [`Protected<'static, StrVec>`](crate::gc_protect::Protected) that keeps the
STRSXP alive. All borrowed data (`&str`, iterators) has its lifetime tied to `&self`,
not `'static` — preventing use-after-GC bugs at compile time.

# When to use

- **`StrVec`**: for SEXP arguments to `.Call` (R protects them), or when you
  manage protection yourself. Lightweight, `Copy`.
- **`ProtectedStrVec`**: when you allocate or receive an STRSXP and need to
  keep it alive beyond the immediate scope. Not `Copy`.

# Example

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

# Safety

- `sexp` must be a valid STRSXP.
- `sexp` must remain GC-protected for the lifetime of this struct.
- Must be called from the R main thread.

#### `get_cow`

```rust
get_cow(self: &Self, idx: isize) -> Option<Cow<''_, str>>
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
iter(self: &Self) -> ProtectedStrVecIter<''_>
```

Iterate over elements as `Option<&str>` (lifetime tied to `&self`).

#### `iter_cow`

```rust
iter_cow(self: &Self) -> ProtectedStrVecCowIter<''_>
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

# Safety

- `sexp` must be a valid STRSXP.
- Must be called from the R main thread.

### `ProtectedStrVecCowIter`

Encoding-safe iterator over `ProtectedStrVec`.

### `ProtectedStrVecIter`

Iterator over `ProtectedStrVec` with lifetime tied to the protection guard.

### `RAllocator`

R-backed global allocator.

All allocations are backed by R RAWSXP objects and protected from
garbage collection. The allocator stores metadata before the returned
pointer to enable proper deallocation.

**Note:** This should NOT be used as `#[global_allocator]` in R package
library crates, as it would be invoked during compilation/build time when
R isn't available. Instead, use it explicitly in standalone binaries that
embed R, or use arena-style allocation APIs.

# Thread Safety

This allocator is safe to use from any thread. R API calls are automatically
routed to the main thread via `with_r_thread_or_inline`.

### `RArray`

An N-dimensional R array.

This type wraps an R array SEXP. The dimension count `NDIM` is tracked
at compile time, but dimension sizes are read from the R object.

# Type Parameters

- `T`: The element type, must implement [`RNativeType`]
- `NDIM`: The number of dimensions (compile-time constant)

# Thread Safety

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

# Safety

The SEXP must be protected and valid.

#### `as_slice_mut`

```rust
unsafe as_slice_mut(self: &mut Self) -> &mut [T]
```

Get the data as a mutable slice (column-major order).

# Safety

- The SEXP must be protected and valid
- No other references to the data may exist

#### `column`

```rust
unsafe column(self: &Self, col: usize) -> &[T]
```

Get a column as a slice.

# Safety

The SEXP must be protected and valid.

#### `column_mut`

```rust
unsafe column_mut(self: &mut Self, col: usize) -> &mut [T]
```

Get a mutable column as a slice.

Columns are contiguous in R's column-major layout, so this returns
a proper `&mut [T]` without any striding.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if `col >= ncol`.

#### `dim`

```rust
unsafe dim(self: &Self, dim: usize) -> usize
```

Get a specific dimension size.

# Safety

The SEXP must be valid.

# Panics

Panics if `dim >= NDIM`.

#### `dims`

```rust
unsafe dims(self: &Self) -> [usize; NDIM]
```

Get the dimensions as an array.

# Safety

The SEXP must be valid.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: SEXP) -> Result<Self, SexpError>
```

Create an RArray from a SEXP, validating type and dimensions.

# Safety

The SEXP must be protected from GC for the lifetime of the returned RArray.

# Errors

Returns an error if:
- The SEXP type doesn't match `T::SEXP_TYPE`
- The dim attribute has wrong number of dimensions

#### `from_sexp_unchecked`

```rust
const unsafe from_sexp_unchecked(sexp: SEXP) -> Self
```

Create an RArray from a SEXP without validation.

# Safety

- The SEXP must be protected from GC
- The SEXP must have the correct type for `T`
- The SEXP must have exactly `NDIM` dimensions

#### `get`

```rust
unsafe get(self: &Self, indices: [usize; NDIM]) -> T
```

Get an element by N-dimensional indices.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any index is out of bounds.

#### `get_class`

```rust
unsafe get_class(self: &Self) -> Option<SEXP>
```

Get the `class` attribute if present.

Equivalent to R's `GET_CLASS(x)`.

# Safety

The SEXP must be valid.

#### `get_colnames`

```rust
unsafe get_colnames(self: &Self) -> Option<SEXP>
```

Get column names from the `dimnames` attribute.

Equivalent to R's `GET_COLNAMES(x)` / `Rf_GetColNames(x)`.

# Safety

The SEXP must be valid.

#### `get_dimnames`

```rust
unsafe get_dimnames(self: &Self) -> Option<SEXP>
```

Get the `dimnames` attribute if present.

Equivalent to R's `GET_DIMNAMES(x)`.

# Safety

The SEXP must be valid.

#### `get_names`

```rust
unsafe get_names(self: &Self) -> Option<SEXP>
```

Get the `names` attribute if present.

Equivalent to R's `GET_NAMES(x)`.

# Safety

The SEXP must be valid.

#### `get_rc`

```rust
unsafe get_rc(self: &Self, row: usize, col: usize) -> T
```

Get an element by row and column.

# Safety

The SEXP must be protected and valid.

#### `get_rownames`

```rust
unsafe get_rownames(self: &Self) -> Option<SEXP>
```

Get row names from the `dimnames` attribute.

Equivalent to R's `GET_ROWNAMES(x)` / `Rf_GetRowNames(x)`.

# Safety

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

# Safety

The SEXP must be valid (needed to read dims).

# Panics

Panics if any index is out of bounds.

#### `ncol`

```rust
unsafe ncol(self: &Self) -> usize
```

Get the number of columns.

# Safety

The SEXP must be valid.

#### `new`

```rust
unsafe new<F>(dims: [usize; NDIM], init: F) -> Self
```

Allocate a new R array with the given dimensions.

The array is allocated. The closure receives a mutable slice to
initialize the data.

# Safety

Must be called from the R main thread (or via routed FFI).
The returned RArray holds an unprotected SEXP - caller must protect.

# Example

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

# Safety

The SEXP must be valid.

#### `set`

```rust
unsafe set(self: &mut Self, indices: [usize; NDIM], value: T)
```

Set an element by N-dimensional indices.

# Safety

- The SEXP must be protected and valid
- No other references to the data may exist

# Panics

Panics if any index is out of bounds.

#### `set_class`

```rust
unsafe set_class(self: &mut Self, class: SEXP)
```

Set the `class` attribute.

Equivalent to R's `SET_CLASS(x, n)`.

# Safety

The SEXP must be valid and not shared.

#### `set_dimnames`

```rust
unsafe set_dimnames(self: &mut Self, dimnames: SEXP)
```

Set the `dimnames` attribute.

Equivalent to R's `SET_DIMNAMES(x, n)`.

# Safety

The SEXP must be valid and not shared.

#### `set_names`

```rust
unsafe set_names(self: &mut Self, names: SEXP)
```

Set an arbitrary attribute by symbol (unchecked internal helper).

# Safety

Set the `names` attribute.

Equivalent to R's `SET_NAMES(x, n)`.

# Safety

The SEXP must be valid and not shared.

#### `set_rc`

```rust
unsafe set_rc(self: &mut Self, row: usize, col: usize, value: T)
```

Set an element by row and column.

# Safety

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

# Safety

The SEXP must be protected and valid.

# Example

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

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<i16>
```

Copy array data to an owned `Vec`, coercing from the R native type.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<i64>
```

Copy array data to an owned `Vec`, coercing from the R native type.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<isize>
```

Copy array data to an owned `Vec`, coercing from the R native type.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<u16>
```

Copy array data to an owned `Vec`, coercing from the R native type.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<u32>
```

Copy array data to an owned `Vec`, coercing from the R native type.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<u64>
```

Copy array data to an owned `Vec`, coercing from the R native type.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<usize>
```

Copy array data to an owned `Vec`, coercing from the R native type.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<f32>
```

Copy array data to an owned `Vec`, coercing from the R native type.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `to_vec_coerced`

```rust
unsafe to_vec_coerced(self: &Self) -> Vec<bool>
```

Copy array data to an owned `Vec`, coercing from the R native type.

# Safety

The SEXP must be protected and valid.

# Panics

Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).

#### `zeros`

```rust
unsafe zeros(dims: [usize; NDIM]) -> Self
```

Allocate a new R array filled with zeros.

# Safety

Must be called from the R main thread (or via routed FFI).
The returned RArray holds an unprotected SEXP - caller must protect.

### `RBorrow`

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

### `RCall`

Builder for constructing and evaluating R function calls.

`RCall` constructs a LANGSXP (R language object) from a function name or
SEXP and a sequence of arguments (optionally named). It handles GC
protection during construction and evaluation.

# Example

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

# Safety

Must be called from the R main thread. All argument SEXPs must still
be valid (protected or otherwise reachable by R's GC).

#### `eval`

```rust
unsafe eval(self: &Self, env: SEXP) -> Result<SEXP, String>
```

Evaluate the call in the given environment.

Uses `R_tryEvalSilent` so that R errors are captured as `Err(String)`
rather than causing a longjmp through Rust frames.

# Safety

- Must be called from the R main thread.
- `env` must be a valid ENVSXP.
- All argument SEXPs must still be valid.

# Returns

- `Ok(SEXP)` with the result (unprotected — caller should protect if needed)
- `Err(String)` with the R error message on failure

#### `eval_base`

```rust
unsafe eval_base(self: &Self) -> Result<SEXP, String>
```

Evaluate in `R_BaseEnv`.

# Safety

Same as [`eval`](Self::eval).

#### `from_cstr`

```rust
unsafe from_cstr(fun_name: &CStr) -> Self
```

Start building a call to a function given as a C string literal.

More efficient than [`new`](Self::new) when a `&CStr` is available.

# Safety

Must be called from the R main thread.

#### `from_sexp`

```rust
unsafe from_sexp(fun: SEXP) -> Self
```

Start building a call with a function SEXP (closure, builtin, etc.).

# Safety

`fun` must be a valid SEXP representing a callable R object.

#### `named_arg`

```rust
named_arg(self: Self, name: &str, value: SEXP) -> Self
```

Add a named argument.

# Panics

Panics if `name` contains a null byte.

#### `new`

```rust
unsafe new(fun_name: &str) -> Self
```

Start building a call to a named R function.

The function is looked up via `Rf_install`, which returns an interned symbol.

# Safety

Must be called from the R main thread.

# Panics

Panics if `fun_name` contains a null byte.

### `RCloneView`

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

# Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RCopyView`

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

# Safety

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

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RDataFrameBuilder`

Builder for assembling an R `data.frame` whose columns are filled in parallel.

This is the heterogeneous-column analogue of [`with_r_matrix`]: instead of one
homogeneous matrix, you declare a set of typed columns (each with its own
element type and fill closure) and the builder fills them all in **one flat
parallel pass** over `(column, row-range)` work-items.

# Two axes of parallelism, one work-stealing pass

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

# Phases

1. Allocate each column's backing storage **serially on the R/worker thread**
   (native columns get a protected R vector; character columns get an owned
   `Vec<Option<String>>`). Strict PROTECT discipline — the dangerous part.
2. Fill all columns in **one flat parallel pass**. No R API calls happen
   inside the parallel region.
3. Set character `CHARSXP`s serially on the R thread (CHARSXP allocation is
   forbidden on rayon threads), then assemble the `VECSXP`, `names`, compact
   `row.names` (`c(NA_integer_, -nrow)`), and `class = "data.frame"`.

# Column kinds

- [`column::<T>`][RDataFrameBuilder::column] — a native-typed column
  (`f64`/`i32`/`RLogical`/`u8`/`Rcomplex`). The fill closure receives a
  mutable chunk and its offset, exactly like [`with_r_vec`]. The buffer is R
  memory, filled directly with zero intermediate allocation.
- [`column_str`][RDataFrameBuilder::column_str] — a character (`STRSXP`)
  column. The per-row `Option<String>` values are computed **in parallel**
  (contributing chunks to the same flat work-list as native columns), but the
  `CHARSXP`s are set **serially** afterward. `None` becomes `NA_character_`.

# Example

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

# Safety argument (disjoint mutation, no aliasing)

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

# Protection

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

### `RDebugView`

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

# Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RDefaultView`

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

# Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RDeserializer`

Deserializer that converts R SEXP to Rust values.

# Type Mappings

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

# Example

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

### `RDisplayView`

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

# Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `REncodingInfo`

Cached snapshot of R's encoding / locale state at init time.

### `REnv`

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

# Safety

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

# Safety

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

# Safety

Must be called from the R main thread.

#### `empty`

```rust
unsafe empty() -> Self
```

The empty environment (`R_EmptyEnv`).

# Safety

Must be called from the R main thread.

#### `from_sexp`

```rust
unsafe from_sexp(sexp: SEXP) -> Self
```

Wrap an arbitrary environment SEXP.

# Safety

`sexp` must be a valid ENVSXP.

#### `global`

```rust
unsafe global() -> Self
```

The global environment (`R_GlobalEnv`).

# Safety

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

# Safety

Must be called from the R main thread.

# Errors

Returns `Err` if the package namespace is not found (package not loaded).

### `RErrorView`

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

# Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RFlags`

Wrapper for bitflags types that implements R conversions.

`RFlags<T>` wraps any type `T` that implements `bitflags::Flags` and provides
`TryFromSexp` and `IntoR` implementations for R interop.

# Example

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

### `RFromStrView`

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

# Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RHashView`

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

# Safety

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

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RIteratorView`

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

# Safety

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

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RLogical`

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

### `ROrdView`

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

# Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RPartialOrdView`

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

# Safety

Same as `try_from_sexp`.

#### `try_from_sexp`

```rust
unsafe try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self>
```

Try to create a view from an R SEXP.

Returns `Some(Self)` if the object implements this trait,
`None` otherwise.

# Safety

- `sexp` must be a valid R external pointer (EXTPTRSXP)
- Must be called on R's main thread

### `RPrimitive`

A primitive Arrow array that may be backed by R memory.

`RPrimitive<T>` wraps a [`PrimitiveArray<T>`](arrow_array::PrimitiveArray) and optionally carries the
source R SEXP. When the array came from R (via `TryFromSexp`), converting
back to R is zero-copy — the original SEXP is returned directly.

All Arrow APIs work transparently via `Deref<Target = PrimitiveArray<T>>`.

# Example

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

# Safety

The caller must ensure `sexp` is a valid R vector whose data buffer
backs `array` (i.e., the array was created via `sexp_to_arrow_buffer`).

#### `into_inner`

```rust
into_inner(self: Self) -> arrow_array::PrimitiveArray<T>
```

Get the inner Arrow array, discarding provenance.

### `RRng`

A wrapper around R's random number generator that implements `rand::Rng`.

This allows using R's RNG with any `rand`-compatible code, ensuring
reproducibility when seeds are set via `set.seed()` in R.

# Requirements

R's RNG state must be initialized before using this type. Either:
- Use `#[miniextendr(rng)]` attribute on the function
- Create an [`RngGuard`][crate::RngGuard] before using `RRng`

# Example

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

# Safety Requirements

R's RNG state must have been initialized via `GetRNGstate()` before
calling any methods on this type. Use `#[miniextendr(rng)]` or
[`RngGuard`][crate::RngGuard] to ensure this.

### `RSerializer`

Serializer that converts Rust values directly to R SEXP.

# Type Mappings

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

# Example

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

### `RSidecar`

Marker type for enabling R sidecar accessors in an `ExternalPtr` struct.

When used with `#[derive(ExternalPtr)]` and `#[r_data]`, this field acts as
a selector that enables R-facing accessors for sibling `#[r_data]` fields.

# Supported Field Types

- **`SEXP`** - Raw SEXP access, no conversion
- **`i32`, `f64`, `bool`, `u8`** - Zero-overhead scalars (stored directly in R)
- **Any `IntoR` type** - Automatic conversion (e.g., `String`, `Vec<T>`)

# Example

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

# Design Notes

- `RSidecar` is a ZST (zero-sized type) - no runtime cost
- Only `pub` `#[r_data]` fields get R wrapper functions generated
- Multiple `RSidecar` fields in one struct is a compile error

### `RStringArray`

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

# Safety

The caller must ensure `sexp` is a valid STRSXP that was the source
for the StringArray's data (i.e., the array was built from this STRSXP).

#### `into_inner`

```rust
into_inner(self: Self) -> StringArray
```

Get the inner StringArray, discarding provenance.

### `RSymbol`

A safe wrapper around R symbols (SYMSXP).

R symbols are interned strings used as variable and function names.
They are never garbage collected, so `RSymbol` does not need GC protection.

# Example

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

# Safety

Must be called from the R main thread.

#### `new`

```rust
unsafe new(name: &str) -> Self
```

Create or retrieve an interned R symbol.

# Safety

Must be called from the R main thread.

# Panics

Panics if `name` contains a null byte.

### `RThreadBuilder`

Builder for spawning threads with R-appropriate stack sizes.

This builder is always available and configures threads with stack sizes
suitable for R workloads (8 MiB default, vs Rust's 2 MiB default).

When the `nonapi` feature is enabled, spawned threads also automatically
disable R's stack checking via `StackCheckGuard`, allowing R API calls
from the thread.

# Example

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

# Example

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

### `RVecStorage`

Column-major R-backed storage for nalgebra matrices.

This type wraps an R SEXP (REALSXP, INTSXP, or RAWSXP) and implements
nalgebra's storage traits. The underlying data is R-allocated memory,
protected from garbage collection via `R_PreserveObject`.

# Zero-Copy Guarantee

- **Input**: `TryFromSexp` wraps the R vector directly (no copy)
- **Output**: `IntoR` returns the underlying SEXP (no copy)
- **Compute**: all nalgebra operations read/write R memory directly

# Thread Safety

`RVecStorage` is `!Send` and `!Sync` because accessing R memory requires
being on R's main thread. Functions using `RDVector`/`RDMatrix` automatically
route to the main thread via the `#[miniextendr]` macro.

# ALTREP

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

# Safety

Must be called on R's main thread.

#### `from_sexp_vector`

```rust
unsafe from_sexp_vector(sexp: SEXP) -> Result<Self, SexpError>
```

Wrap an existing R SEXP as a column vector storage (zero-copy).

# Safety

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

# Safety

The returned SEXP is **unprotected**. The caller must either return it
directly to R (`.Call` return) or protect it immediately.

#### `new_matrix`

```rust
unsafe new_matrix(nrows: usize, ncols: usize, init: impl FnOnce) -> Self
```

Allocate a new R matrix and initialize it.

# Safety

Must be called on R's main thread.

#### `new_vector`

```rust
unsafe new_vector(len: usize, init: impl FnOnce) -> Self
```

Allocate a new R vector and initialize it.

# Safety

Must be called on R's main thread.

### `RWrapperEntry`

R wrapper code with priority for ordering.

**Fields:**

- `priority`: `RWrapperPriority`
  - Ordering priority (lower = earlier in output file).
- `content`: `&''static str`
  - R source code fragment.
- `source_file`: `&''static str`
  - Source file path (from `file!()`). Used to derive a default `@rdname`

### `R_CMethodDef`

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

### `R_CallMethodDef`

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

### `R_altrep_class_t`

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

# Safety
Must be called on R's main thread. `x` must be a valid SEXP.

#### `new_altrep`

```rust
unsafe new_altrep(self: Self, data1: SEXP, data2: SEXP) -> SEXP
```

Create a new ALTREP instance with data1 and data2 slots.

# Safety
Must be called on R's main thread. `data1` and `data2` must be valid SEXPs.

#### `new_altrep_unchecked`

```rust
unsafe new_altrep_unchecked(self: Self, data1: SEXP, data2: SEXP) -> SEXP
```

Create a new ALTREP instance (no thread check).

# Safety
Must be called on R's main thread.

#### `set_coerce_method`

```rust
unsafe set_coerce_method(self: Self, fun: R_altrep_Coerce_method_t)
```

Set the Coerce method.
# Safety
Must be called during R initialization.

#### `set_complex_elt_method`

```rust
unsafe set_complex_elt_method(self: Self, fun: R_altcomplex_Elt_method_t)
```

Set the complex Elt method.
# Safety
Must be called during R initialization.

#### `set_complex_get_region_method`

```rust
unsafe set_complex_get_region_method(self: Self, fun: R_altcomplex_Get_region_method_t)
```

Set the complex Get_region method.
# Safety
Must be called during R initialization.

#### `set_dataptr_method`

```rust
unsafe set_dataptr_method(self: Self, fun: R_altvec_Dataptr_method_t)
```

Set the Dataptr method.
# Safety
Must be called during R initialization.

#### `set_dataptr_or_null_method`

```rust
unsafe set_dataptr_or_null_method(self: Self, fun: R_altvec_Dataptr_or_null_method_t)
```

Set the Dataptr_or_null method.
# Safety
Must be called during R initialization.

#### `set_duplicate_ex_method`

```rust
unsafe set_duplicate_ex_method(self: Self, fun: R_altrep_DuplicateEX_method_t)
```

Set the DuplicateEX method.
# Safety
Must be called during R initialization.

#### `set_duplicate_method`

```rust
unsafe set_duplicate_method(self: Self, fun: R_altrep_Duplicate_method_t)
```

Set the Duplicate method.
# Safety
Must be called during R initialization.

#### `set_extract_subset_method`

```rust
unsafe set_extract_subset_method(self: Self, fun: R_altvec_Extract_subset_method_t)
```

Set the Extract_subset method.
# Safety
Must be called during R initialization.

#### `set_inspect_method`

```rust
unsafe set_inspect_method(self: Self, fun: R_altrep_Inspect_method_t)
```

Set the Inspect method.
# Safety
Must be called during R initialization.

#### `set_integer_elt_method`

```rust
unsafe set_integer_elt_method(self: Self, fun: R_altinteger_Elt_method_t)
```

Set the integer Elt method.
# Safety
Must be called during R initialization.

#### `set_integer_get_region_method`

```rust
unsafe set_integer_get_region_method(self: Self, fun: R_altinteger_Get_region_method_t)
```

Set the integer Get_region method.
# Safety
Must be called during R initialization.

#### `set_integer_is_sorted_method`

```rust
unsafe set_integer_is_sorted_method(self: Self, fun: R_altinteger_Is_sorted_method_t)
```

Set the integer Is_sorted method.
# Safety
Must be called during R initialization.

#### `set_integer_max_method`

```rust
unsafe set_integer_max_method(self: Self, fun: R_altinteger_Max_method_t)
```

Set the integer Max method.
# Safety
Must be called during R initialization.

#### `set_integer_min_method`

```rust
unsafe set_integer_min_method(self: Self, fun: R_altinteger_Min_method_t)
```

Set the integer Min method.
# Safety
Must be called during R initialization.

#### `set_integer_no_na_method`

```rust
unsafe set_integer_no_na_method(self: Self, fun: R_altinteger_No_NA_method_t)
```

Set the integer No_NA method.
# Safety
Must be called during R initialization.

#### `set_integer_sum_method`

```rust
unsafe set_integer_sum_method(self: Self, fun: R_altinteger_Sum_method_t)
```

Set the integer Sum method.
# Safety
Must be called during R initialization.

#### `set_length_method`

```rust
unsafe set_length_method(self: Self, fun: R_altrep_Length_method_t)
```

Set the Length method.
# Safety
Must be called during R initialization.

#### `set_list_elt_method`

```rust
unsafe set_list_elt_method(self: Self, fun: R_altlist_Elt_method_t)
```

Set the list Elt method.
# Safety
Must be called during R initialization.

#### `set_list_set_elt_method`

```rust
unsafe set_list_set_elt_method(self: Self, fun: R_altlist_Set_elt_method_t)
```

Set the list Set_elt method.
# Safety
Must be called during R initialization.

#### `set_logical_elt_method`

```rust
unsafe set_logical_elt_method(self: Self, fun: R_altlogical_Elt_method_t)
```

Set the logical Elt method.
# Safety
Must be called during R initialization.

#### `set_logical_get_region_method`

```rust
unsafe set_logical_get_region_method(self: Self, fun: R_altlogical_Get_region_method_t)
```

Set the logical Get_region method.
# Safety
Must be called during R initialization.

#### `set_logical_is_sorted_method`

```rust
unsafe set_logical_is_sorted_method(self: Self, fun: R_altlogical_Is_sorted_method_t)
```

Set the logical Is_sorted method.
# Safety
Must be called during R initialization.

#### `set_logical_no_na_method`

```rust
unsafe set_logical_no_na_method(self: Self, fun: R_altlogical_No_NA_method_t)
```

Set the logical No_NA method.
# Safety
Must be called during R initialization.

#### `set_logical_sum_method`

```rust
unsafe set_logical_sum_method(self: Self, fun: R_altlogical_Sum_method_t)
```

Set the logical Sum method.
# Safety
Must be called during R initialization.

#### `set_raw_elt_method`

```rust
unsafe set_raw_elt_method(self: Self, fun: R_altraw_Elt_method_t)
```

Set the raw Elt method.
# Safety
Must be called during R initialization.

#### `set_raw_get_region_method`

```rust
unsafe set_raw_get_region_method(self: Self, fun: R_altraw_Get_region_method_t)
```

Set the raw Get_region method.
# Safety
Must be called during R initialization.

#### `set_real_elt_method`

```rust
unsafe set_real_elt_method(self: Self, fun: R_altreal_Elt_method_t)
```

Set the real Elt method.
# Safety
Must be called during R initialization.

#### `set_real_get_region_method`

```rust
unsafe set_real_get_region_method(self: Self, fun: R_altreal_Get_region_method_t)
```

Set the real Get_region method.
# Safety
Must be called during R initialization.

#### `set_real_is_sorted_method`

```rust
unsafe set_real_is_sorted_method(self: Self, fun: R_altreal_Is_sorted_method_t)
```

Set the real Is_sorted method.
# Safety
Must be called during R initialization.

#### `set_real_max_method`

```rust
unsafe set_real_max_method(self: Self, fun: R_altreal_Max_method_t)
```

Set the real Max method.
# Safety
Must be called during R initialization.

#### `set_real_min_method`

```rust
unsafe set_real_min_method(self: Self, fun: R_altreal_Min_method_t)
```

Set the real Min method.
# Safety
Must be called during R initialization.

#### `set_real_no_na_method`

```rust
unsafe set_real_no_na_method(self: Self, fun: R_altreal_No_NA_method_t)
```

Set the real No_NA method.
# Safety
Must be called during R initialization.

#### `set_real_sum_method`

```rust
unsafe set_real_sum_method(self: Self, fun: R_altreal_Sum_method_t)
```

Set the real Sum method.
# Safety
Must be called during R initialization.

#### `set_serialized_state_method`

```rust
unsafe set_serialized_state_method(self: Self, fun: R_altrep_Serialized_state_method_t)
```

Set the Serialized_state method.
# Safety
Must be called during R initialization.

#### `set_string_elt_method`

```rust
unsafe set_string_elt_method(self: Self, fun: R_altstring_Elt_method_t)
```

Set the string Elt method.
# Safety
Must be called during R initialization.

#### `set_string_is_sorted_method`

```rust
unsafe set_string_is_sorted_method(self: Self, fun: R_altstring_Is_sorted_method_t)
```

Set the string Is_sorted method.
# Safety
Must be called during R initialization.

#### `set_string_no_na_method`

```rust
unsafe set_string_no_na_method(self: Self, fun: R_altstring_No_NA_method_t)
```

Set the string No_NA method.
# Safety
Must be called during R initialization.

#### `set_string_set_elt_method`

```rust
unsafe set_string_set_elt_method(self: Self, fun: R_altstring_Set_elt_method_t)
```

Set the string Set_elt method.
# Safety
Must be called during R initialization.

#### `set_unserialize_ex_method`

```rust
unsafe set_unserialize_ex_method(self: Self, fun: R_altrep_UnserializeEX_method_t)
```

Set the UnserializeEX method.
# Safety
Must be called during R initialization.

#### `set_unserialize_method`

```rust
unsafe set_unserialize_method(self: Self, fun: R_altrep_Unserialize_method_t)
```

Set the Unserialize method.
# Safety
Must be called during R initialization.

### `Raw`

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

### `RawHeader`

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

### `RawSlice`

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

### `RawSliceTagged`

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

### `RawTagged`

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

### `Rcomplex`

R's complex scalar layout (`Rcomplex`).

**Fields:**

- `r`: `f64`
  - Real part.
- `i`: `f64`
  - Imaginary part.

### `ReprotectSlot`

A protected slot created with `R_ProtectWithIndex` and updated with `R_Reprotect`.

This allows updating a protected value in-place without growing the protection
stack. Useful for loops that repeatedly allocate and update a value.

The slot is valid only while the creating [`ProtectScope`] is alive.

# When to Use `ReprotectSlot`

Use `ReprotectSlot` when you need to **reassign a protected value** multiple times:

| Pattern | Use | Why |
|---------|-----|-----|
| Accumulator loop | `ReprotectSlot` | Repeatedly replace result without stack growth |
| Single allocation | `ProtectScope::protect` | Simpler, no reassignment needed |
| Child insertion | `List::set_elt` | Container handles child protection |

# Warning: RAII Assignment Pitfall

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

# Examples

## Accumulator Pattern

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

## Starting with Empty Slot

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

## Multiple Slots

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

# Safety

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

# Safety

Must be called from the R main thread (accesses R's `R_NilValue`).

#### `replace`

```rust
unsafe replace(self: &Self, x: SEXP) -> SEXP
```

Replace the slot's value with `x` and return the old value.

This provides `Option::replace`-like semantics. The slot now protects
`x`, and the old value is returned **unprotected**.

# Safety

- Must be called from the R main thread
- `x` must be a valid SEXP
- The returned SEXP is **unprotected**. If it needs to survive further
  allocations, you must protect it explicitly.

# Example

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

# Safety

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

# Safety

- Must be called from the R main thread
- The closure must return a valid SEXP

# Example

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

# Safety

- Must be called from the R main thread
- The returned SEXP is **unprotected**. If it needs to survive further
  allocations, you must protect it explicitly.

# Example

```ignore
let slot = scope.protect_with_index(some_value);
// ... work with slot.get() ...
let old = slot.take();  // slot now holds R_NilValue
// old is unprotected - protect it if needed
let guard = OwnedProtect::new(old);
```

### `RndMat`

An R matrix with zero-copy ndarray 2D view access.

`RndMat<T>` wraps an R matrix SEXP and provides `.view()` / `.view_mut()`
methods returning Fortran-order (column-major) `ArrayView2` / `ArrayViewMut2`.

# Example

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

# Safety

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

# Safety

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

# Safety

Must be called on R's main thread.

#### `nrow`

```rust
nrow(self: &Self) -> usize
```

Number of rows.

#### `view`

```rust
view(self: &Self) -> ArrayView2<''_, T>
```

Zero-copy Fortran-order (column-major) 2D view.

#### `view_mut`

```rust
view_mut(self: &mut Self) -> ArrayViewMut2<''_, T>
```

Zero-copy mutable Fortran-order 2D view.

#### `zeros`

```rust
unsafe zeros(nrow: usize, ncol: usize) -> Self
```

Allocate a zero-filled R matrix.

# Safety

Must be called on R's main thread.

### `RndVec`

An R vector with zero-copy ndarray view access.

`RndVec<T>` wraps an R SEXP and provides `.view()` / `.view_mut()` methods
that return ndarray `ArrayView1` / `ArrayViewMut1` views directly over
R's memory. No data is copied on input or output.

# GC Protection

Uses `R_PreserveObject` for GC protection — safe across `.Call` boundaries.

# Thread Safety

`!Send` and `!Sync` — must be used on R's main thread.

# Example

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

# Safety

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

# Safety

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

# Safety

Must be called on R's main thread.

#### `view`

```rust
view(self: &Self) -> ArrayView1<''_, T>
```

Zero-copy read-only ndarray view.

#### `view_mut`

```rust
view_mut(self: &mut Self) -> ArrayViewMut1<''_, T>
```

Zero-copy mutable ndarray view.

#### `zeros`

```rust
unsafe zeros(len: usize) -> Self
```

Allocate a zero-filled R vector.

# Safety

Must be called on R's main thread.

### `RngGuard`

RAII guard for R's RNG state.

Calls `GetRNGstate()` on creation and `PutRNGstate()` on drop.
This ensures RNG state is properly saved even if the function panics
or returns early.

# Example

```ignore
use miniextendr_api::rng::RngGuard;
use miniextendr_api::sys::unif_rand;

fn generate_uniform() -> f64 {
    let _guard = RngGuard::new();
    unsafe { unif_rand() }
}
```

# Warning: R Longjumps

This guard relies on Rust's drop semantics. If R triggers a longjmp
(via `Rf_error` etc.), the destructor will NOT run unless the code
is wrapped in `with_r_unwind_protect`. For functions exposed to R,
prefer using `#[miniextendr(rng)]` which handles this correctly.

# Safety

Must be used on R's main thread. The guard assumes it has exclusive
access to R's RNG state while alive.

**Methods:**

#### `new`

```rust
new() -> Self
```

Create a new RNG guard, loading the current RNG state.

Calls `GetRNGstate()` to load R's `.Random.seed` into the RNG.

# Safety

Must be called from R's main thread.

### `Root`

A rooted SEXP tied to the lifetime of a [`ProtectScope`].

This type has **no `Drop`**. The scope owns unprotection responsibility.
This makes `Root` cheap to move and copy (it's just a pointer + lifetime).

# Lifetime

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

### `SEXP`

R's pointer type for S-expressions.

This is a newtype wrapper around `*mut SEXPREC` that implements Send and Sync.
SEXP is just a handle (pointer) - the actual data it points to is managed by R's
garbage collector and should only be accessed on R's main thread.

# Safety

While SEXP is Send+Sync (allowing it to be passed between threads), the data
it points to must only be accessed on R's main thread. The miniextendr runtime
enforces this through the worker thread pattern.

# Equality Semantics

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

# Safety

Must be called from the R main thread. The returned SEXP is unprotected;
any subsequent allocation may collect it.

#### `alloc_list`

```rust
unsafe alloc_list(n: R_xlen_t) -> SEXP
```

Allocate an R list (VECSXP) of length `n`. Unprotected.

Equivalent to `Rf_allocVector(VECSXP, n)`. Elements are initialised to `R_NilValue`.

# Safety

Must be called from the R main thread. The returned SEXP is unprotected —
wrap it in [`OwnedProtect`](crate::gc_protect::OwnedProtect) before any
other allocation that could trigger GC.

#### `alloc_strsxp`

```rust
unsafe alloc_strsxp(n: R_xlen_t) -> SEXP
```

Allocate an R character vector (STRSXP) of length `n`. Unprotected.

Equivalent to `Rf_allocVector(STRSXP, n)`. Elements are initialised to `R_BlankString`.

# Safety

Must be called from the R main thread. The returned SEXP is unprotected —
wrap it in [`OwnedProtect`](crate::gc_protect::OwnedProtect) before any
other allocation that could trigger GC.

#### `altrep_data1_raw`

```rust
unsafe altrep_data1_raw(self: Self) -> SEXP
```

Get the raw SEXP in the ALTREP data1 slot.

# Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `altrep_data1_raw_unchecked`

```rust
unsafe altrep_data1_raw_unchecked(self: Self) -> SEXP
```

Get the raw SEXP in the ALTREP data1 slot (unchecked — no thread routing).

# Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `altrep_data2_raw`

```rust
unsafe altrep_data2_raw(self: Self) -> SEXP
```

Get the raw SEXP in the ALTREP data2 slot.

# Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `altrep_data2_raw_unchecked`

```rust
unsafe altrep_data2_raw_unchecked(self: Self) -> SEXP
```

Get the raw SEXP in the ALTREP data2 slot (unchecked — no thread routing).

# Safety

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

# Safety

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

# Safety

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

# Safety

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

# Safety

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

# Safety

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

# Safety

Must be called from the R main thread.

#### `set_altrep_data1`

```rust
unsafe set_altrep_data1(self: Self, v: SEXP)
```

Set the ALTREP data1 slot.

# Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `set_altrep_data2`

```rust
unsafe set_altrep_data2(self: Self, v: SEXP)
```

Set the ALTREP data2 slot.

# Safety

- `self` must be a valid ALTREP SEXP
- Must be called from the R main thread

#### `set_altrep_data2_unchecked`

```rust
unsafe set_altrep_data2_unchecked(self: Self, v: SEXP)
```

Set the ALTREP data2 slot (unchecked — no thread routing).

# Safety

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

### `SEXPREC`

Opaque underlying S-expression header type.

### `SeqSerializer`

Serializer for sequences (Vec, tuples).

Uses smart dispatch: if all elements are homogeneous scalars of the same
primitive type, coalesces into an R atomic vector. Otherwise creates a list.

### `SerdeRowBuilder`

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

# Examples

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

# Examples

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

# Examples

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

### `SerdeRows`

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

### `SexpLengthError`

Error describing an unexpected R object length.

**Fields:**

- `expected`: `usize`
  - Required length.
- `actual`: `usize`
  - Actual length encountered.

### `SexpNaError`

Error for NA values in conversions that require non-missing values.

**Fields:**

- `sexp_type`: `crate::SEXPTYPE`
  - R type where an NA was found.

### `SexpTypeError`

Error describing an unexpected R `SEXPTYPE`.

**Fields:**

- `expected`: `crate::SEXPTYPE`
  - Expected R type.
- `actual`: `crate::SEXPTYPE`
  - Actual R type encountered.

### `SidecarPropEntry`

Entry documenting a sidecar (`#[r_data]`) property on an S7 ExternalPtr type.

Emitted by `#[derive(ExternalPtr)] #[externalptr(s7)]` for each public `#[r_data]`
field. Used by `write_r_wrappers_to_file` to substitute the
`.__MX_S7_SIDECAR_PROP_DOCS_<TypeName>__` placeholder with `#' @prop` lines.

**Fields:**

- `rust_type`: `&''static str`
  - Rust type name, e.g. `"SidecarS7"`.
- `field_name`: `&''static str`
  - Field name, e.g. `"prop_int"`.
- `prop_doc`: `&''static str`
  - Documentation string for this property.

### `SparseIterComplexData`

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

### `SparseIterIntData`

Sparse iterator-backed integer vector data.

Uses `Iterator::nth()` to skip directly to requested indices.
Only accessed elements are cached; skipped elements return `NA_INTEGER`.

# Example

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

### `SparseIterLogicalData`

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

### `SparseIterRawData`

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

### `SparseIterRealData`

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

### `SparseIterState`

Core state for sparse iterator-backed ALTREP vectors.

Unlike [`super::IterState`], this variant uses `Iterator::nth()` to skip elements
efficiently, only caching the elements that are actually accessed.

# Type Parameters

- `I`: The iterator type
- `T`: The element type produced by the iterator

# Design

- **Sparse:** Only accessed elements are cached (uses `BTreeMap`)
- **Skipping:** Uses `nth()` to skip directly to requested indices
- **Trade-off:** Skipped elements are gone forever (iterator is consumed)
- **Best for:** Large iterators where only a small subset of elements are accessed

# Comparison with `IterState`

| Feature | `IterState` | `SparseIterState` |
|---------|-------------|-------------------|
| Cache storage | Contiguous `Vec<T>` | Sparse `BTreeMap<usize, T>` |
| Access pattern | Prefix (0..=i) cached | Only accessed indices cached |
| Skipped elements | All cached | Gone forever (return NA) |
| Memory for sparse access | O(max_index) | O(num_accessed) |
| `as_slice()` support | Yes (after full materialization) | No (sparse) |

# Example

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

# Returns

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

# Arguments

- `iter`: The iterator to wrap
- `len`: The expected number of elements

### `StrVec`

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

# Safety

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
get_cow(self: Self, idx: isize) -> Option<Cow<''static, str>>
```

Get the string at the given index as `Cow<str>` (encoding-safe).

Returns `Cow::Borrowed` for UTF-8 strings (zero-copy), `Cow::Owned` for
non-UTF-8 strings (translated via `Rf_translateCharUTF8`).
Returns `None` if out of bounds or `NA_character_`.

#### `get_str`

```rust
get_str(self: Self, idx: isize) -> Option<&''static str>
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

# Safety

- Must be called from the R main thread
- `charsxp` must be a valid CHARSXP (from `Rf_mkChar*` or `STRING_ELT`)
- `self` must be a valid, protected STRSXP

# Panics

Panics if `idx` is out of bounds.

#### `set_charsxp_unchecked`

```rust
unsafe set_charsxp_unchecked(self: Self, idx: isize, charsxp: SEXP)
```

Set a CHARSXP without protecting it.

# Safety

In addition to the safety requirements of [`set_charsxp`](Self::set_charsxp):
- The caller must ensure `charsxp` is already protected or from the
  global CHARSXP cache.

#### `set_na`

```rust
unsafe set_na(self: Self, idx: isize)
```

Set an element to `NA_character_`.

# Safety

- Must be called from the R main thread
- `self` must be a valid, protected STRSXP

# Panics

Panics if `idx` is out of bounds.

#### `set_opt_str`

```rust
unsafe set_opt_str(self: Self, idx: isize, s: Option<&str>)
```

Set an element from an optional string.

`None` becomes `NA_character_`.

# Safety

- Must be called from the R main thread
- `self` must be a valid, protected STRSXP

# Panics

Panics if `idx` is out of bounds.

#### `set_str`

```rust
unsafe set_str(self: Self, idx: isize, s: &str)
```

Set an element from a Rust string.

Creates a CHARSXP from the string and inserts it safely.

# Safety

- Must be called from the R main thread
- `self` must be a valid, protected STRSXP

# Panics

Panics if `idx` is out of bounds.

### `StrVecBuilder`

Builder for constructing string vectors with efficient protection management.

# Example

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
unsafe new(scope: &''a ProtectScope, len: usize) -> Self
```

Create a new string vector builder with the given length.

# Safety

Must be called from the R main thread.

#### `set_na`

```rust
unsafe set_na(self: &Self, idx: isize)
```

Set an element to `NA_character_`.

# Safety

Must be called from the R main thread.

#### `set_opt_str`

```rust
unsafe set_opt_str(self: &Self, idx: isize, s: Option<&str>)
```

Set an element from an optional string.

# Safety

Must be called from the R main thread.

#### `set_str`

```rust
unsafe set_str(self: &Self, idx: isize, s: &str)
```

Set an element from a Rust string.

# Safety

Must be called from the R main thread.

### `StrVecCowIter`

Iterator over `StrVec` elements as `Option<Cow<'static, str>>`.

Like [`StrVecIter`] but handles non-UTF-8 CHARSXPs via `Rf_translateCharUTF8`.

### `StrVecIter`

Iterator over `StrVec` elements as `Option<&str>`.

Yields `None` for `NA_character_`, `Some(&str)` for valid strings.
Zero-copy — each `&str` borrows directly from R's CHARSXP.

### `StreamingIntData`

Streaming ALTREP for integer (i32) vectors.

Elements are loaded on-demand via a reader closure in fixed-size chunks.
Chunks are cached in a `BTreeMap` for repeated access.

# Reader Contract

The reader `F(start, buf) -> count` fills `buf` with elements starting
at index `start` and returns the number of elements actually written.

# Example

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

### `StreamingRealData`

Streaming ALTREP for real (f64) vectors.

Elements are loaded on-demand via a reader closure in fixed-size chunks.
Chunks are cached in a `BTreeMap` for repeated access.

# Reader Contract

The reader `F(start, buf) -> count` fills `buf` with elements starting
at index `start` and returns the number of elements actually written.

# Example

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

### `StructSerializer`

Serializer for structs.

### `StructVariantSerializer`

Serializer for struct variants: `Enum::Variant { a, b }` -> `list(Variant = list(a=..., b=...))`

### `ThreadLocalArena`

Thread-local BTreeMap-based arena.

This provides the lowest overhead for protection operations by
eliminating RefCell borrow checking.

### `ThreadLocalHashArena`

Thread-local HashMap-based arena.

Combines HashMap's performance for large collections with
thread-local storage's low overhead.

### `TlsRoot`

A rooted SEXP from TLS protection.

This is similar to [`super::Root`] but without a compile-time lifetime tie to
the scope. The protection is valid as long as the enclosing
[`with_protect_scope`] block hasn't exited.

# Warning

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

### `TraitDispatchEntry`

Trait dispatch entry mapping (concrete_tag, trait_tag) → vtable.

**Fields:**

- `concrete_tag`: `crate::abi::mx_tag`
  - Tag identifying the concrete type.
- `trait_tag`: `crate::abi::mx_tag`
  - Tag identifying the trait interface.
- `vtable`: `*const std::os::raw::c_void`
  - Pointer to the trait's vtable (cast from `&'static SomeVTable`).
- `vtable_symbol`: `&''static str`
  - Symbol name of the `#[no_mangle]` vtable static

### `TraitDispatchRow`

Pre-extracted view of one `MX_TRAIT_DISPATCH` entry.

**Fields:**

- `concrete_tag`: `crate::abi::mx_tag`
- `trait_tag`: `crate::abi::mx_tag`
- `vtable_symbol`: `String`

### `TupleVariantSerializer`

Serializer for tuple variants: `Enum::Variant(a, b)` -> `list(Variant = list(a, b))`

### `TypedEntry`

A single entry specification in a typed list.

**Fields:**

- `name`: `&''static str`
  - The expected name of this entry.
- `spec`: `TypeSpec`
  - The expected type of this entry.
- `optional`: `bool`
  - If `true`, the entry is optional (missing allowed).

**Methods:**

#### `any`

```rust
const any(name: &''static str) -> Self
```

Create a required entry that accepts any type.

#### `any_optional`

```rust
const any_optional(name: &''static str) -> Self
```

Create an optional entry that accepts any type.

#### `optional`

```rust
const optional(name: &''static str, spec: TypeSpec) -> Self
```

Create an optional entry with the given name and type.

#### `required`

```rust
const required(name: &''static str, spec: TypeSpec) -> Self
```

Create a required entry with the given name and type.

### `TypedList`

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

### `TypedListSpec`

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

### `WindowedIterIntData`

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

### `WindowedIterRealData`

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

### `WindowedIterState`

Core state for windowed iterator-backed ALTREP vectors.

Like [`super::IterState`], but only keeps a sliding window of elements in memory.
Sequential access within the window is O(1). Access outside the window
materializes the entire vector (falling back to full caching).

This is useful for large iterators where only a small region is accessed
at a time (e.g., streaming data processed in order).

# Type Parameters

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

### `WorkerPump`

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

# Panics

If the worker thread panics, `run` returns
`Err("WorkerPump worker panicked")`.

If the pump closure panics, the panic propagates normally through
`thread::scope`'s `Drop` (which joins the worker), and then out of
`run`.  When called from inside an `#[miniextendr]` body the outer
`R_UnwindProtect` catches it and converts it to an R error.

### `WorkerUnprotectGuard`

A `Send`-safe guard that calls `Rf_unprotect(n)` on drop via `with_r_thread`.

Use this when you `Rf_protect` on the R main thread, then need the unprotect
to happen when a guard drops on a **worker thread** (e.g., rayon parallel code).

[`OwnedProtect`] and [`ProtectScope`] are `!Send` — they can only be used on
the R main thread. `WorkerUnprotectGuard` fills the gap for cross-thread patterns
where allocation + protect happen on the R thread but the guard lives on a worker.

# Example

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

### `mx_base_vtable`

Base vtable present in all erased objects.

This vtable provides the minimal operations needed for any erased object:
- Destructor for cleanup when R garbage collects the wrapper
- Concrete type tag for type-safe downcasts
- Query function to retrieve interface vtables

## Layout Guarantee

This type is `#[repr(C)]` and its layout is frozen. Fields will never
be reordered, and new fields will only be appended at the end.

## Generated By

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

### `mx_erased`

Type-erased object header.

This is the common prefix of all erased objects, providing access to
the base vtable. The actual data follows this header in memory.

## Memory Layout

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

## Layout Guarantee

This type is `#[repr(C)]` and its layout is frozen. The `base` field
will always be at offset 0, and new fields will only be appended.

## Generated By

`#[derive(ExternalPtr)]` generates wrapper structs that place `mx_erased`
as the first field for proper layout.

**Fields:**

- `base`: `*const mx_base_vtable`
  - Pointer to the base vtable.

### `mx_tag`

Type tag for runtime type identification.

A 128-bit identifier split into two 64-bit halves for C compatibility.
Used to identify concrete types and trait interfaces at runtime.

## Generation

Tags should be generated as compile-time constants, typically using
a hash of the fully-qualified type/trait path. The `#[miniextendr]`
attribute macro handles this automatically.

## Comparison

Tags are compared by value equality of both `lo` and `hi` fields.

## Layout Guarantee

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

# Arguments

* `lo` - Lower 64 bits
* `hi` - Upper 64 bits

# Example

```ignore
const MY_TAG: mx_tag = mx_tag::new(0x1234_5678_9abc_def0, 0xfed_cba9_8765_4321);
```

---

## Enums

### `AltrepGuard`

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

### `CoerceError`

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

### `DataFrameError`

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

### `DataFrameShape`

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

### `FactorHandling`

How to serialize R factors to JSON.

**Variants:**

- `Label`
  - Use the factor level label as a string (default).
- `Index`
  - Use the factor level index as an integer (1-based, matching R).

### `GuardMode`

FFI guard mode controlling how panics are caught at Rust-R boundaries.

**Variants:**

- `CatchUnwind`
  - `catch_unwind` only. On panic: fire telemetry, then `Rf_error` (diverges).
- `RUnwind`
  - `R_UnwindProtect`. Catches both Rust panics and R longjmps.

### `IntoRError`

Error returned by [`IntoR::try_into_sexp`](crate::into_r::IntoR::try_into_sexp)
for types whose conversion to R can fail.

# Variants

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

### `ListFromSexpError`

Error when converting SEXP to List fails.

**Variants:**

- `Type(crate::from_r::SexpTypeError)`
  - Wrong SEXP type.
- `DuplicateName(DuplicateNameError)`
  - Duplicate non-NA name found.

### `Logical`

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

### `LogicalCoerceError`

Error type for logical coercion failures.

**Variants:**

- `NAValue`
  - R's NA_LOGICAL cannot be represented as Rust bool
- `InvalidValue(i32)`
  - Value is not 0 or 1

### `MatchArgError`

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

### `Missing`

Wrapper type that detects if an R argument was not passed (missing).

This corresponds to R's `missing()` function. When a function parameter
has type `Missing<T>`, it will be `Missing::Absent` if the caller didn't
provide that argument, or `Missing::Present(value)` if they did.

# Example

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

# Panics

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

# Panics

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

### `N01type`

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

### `NaHandling`

How to handle NA values when converting R to JSON.

**Variants:**

- `Null`
  - Convert NA to JSON null (default).
- `Error`
  - Return an error when NA is encountered.
- `String(String)`
  - Convert NA to a custom string value.

### `PanicSource`

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

### `ParseStatus`

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

### `RBase`

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

### `RCoerceError`

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

### `RCow`

An R-aware copy-on-write slice — the safe, zero-copy-round-trip alternative
to [`std::borrow::Cow<[T]>`](std::borrow::Cow).

See the [module docs](self) for why this exists and how it closes the #880
hazard. In brief: the [`Borrowed`](RCow::Borrowed) arm carries its source
SEXP, so returning it to R is a direct hand-back rather than a speculative
pointer recovery.

# Example

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

- `Borrowed(RBorrow<''a, T>)`
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

### `RNGtype`

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

### `RSerdeError`

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

### `RWrapperPriority`

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

### `RawError`

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

### `Rboolean`

Binary boolean used by many R C APIs.

**Variants:**

- `FALSE`
  - False.
- `TRUE`
  - True.

### `ResultShape`

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

### `SEXPTYPE`

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
type_name(self: Self) -> &''static str
```

Get R's name for this SEXPTYPE (e.g. `"double"`, `"integer"`, `"list"`).

Returns the same string as R's `typeof()` function.

### `Sampletype`

Discrete uniform sample method enum from R_ext/Random.h

**Variants:**

- `ROUNDING`
  - Rounding method for integer sampling.
- `REJECTION`
  - Rejection sampling method.

### `SexpError`

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

### `Sortedness`

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

### `SpecialFloatHandling`

How to handle special float values (NaN, Inf) when converting R to JSON.

**Variants:**

- `Error`
  - Return an error (default) - JSON has no representation for these.
- `Null`
  - Convert to JSON null.
- `String`
  - Convert to a string representation ("NaN", "Infinity", "-Infinity").

### `SplitResults`

Result partition for [`DataFrameShape::Split`].

Used to distinguish "no Ok rows at all" (which lets the caller supply
a sentinel value such as `NULL`, `NA`, `FALSE`, …) from a real
zero-row data.frame.

**Variants:**

- `Some(crate::dataframe::DataFrame)`
  - At least one `Ok` row — partition has a concrete data.frame.
- `None(crate::SEXP)`
  - No `Ok` rows — sentinel SEXP supplied by the caller via

### `SplitShape`

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

### `StorageCoerceError`

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

### `TypeMismatchError`

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

### `TypeSpec`

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
- `Class(&''static str)`
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

### `TypeSpec`

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

### `TypedListError`

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

### `VctrsBuildError`

Error type for vctrs object construction.

# Examples

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

### `VctrsKind`

The kind of vctrs class being created.

This corresponds to the different vctrs constructors:
- [`Vctr`](VctrsKind::Vctr): Simple vector backed by a base type (`vctrs::new_vctr`)
- [`Rcrd`](VctrsKind::Rcrd): Record type with named fields (`vctrs::new_rcrd`)
- [`ListOf`](VctrsKind::ListOf): Homogeneous list with prototype (`vctrs::new_list_of`)

# Examples

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

### `cetype_t`

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
