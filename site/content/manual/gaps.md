+++
title = "miniextendr: Known Gaps and Limitations"
weight = 65
description = "This document catalogs known gaps, limitations, and undocumented behaviors in miniextendr. It serves as both user documentation and a roadmap for future improvements."
+++

This document catalogs known gaps, limitations, and undocumented behaviors in miniextendr. It serves as both user documentation and a roadmap for future improvements.

---

## Table of Contents

1. [Macro Limitations](#1-macro-limitations)
2. [Type Conversion Gaps](#2-type-conversion-gaps)
3. [Class System Gaps](#3-class-system-gaps)
4. [Incomplete Features](#4-incomplete-features)
5. [Undocumented Behavior](#5-undocumented-behavior)
6. [Testing Gaps](#6-testing-gaps)
7. [Documentation Status](#7-documentation-status)

---

## 1. Macro Limitations

### ~~1.1 RFactor Types Cannot Be Function Parameters~~ RESOLVED

**Status:** Works as expected
**Resolution:** RFactor types can be used directly as function parameters.

The `#[derive(RFactor)]` macro generates `TryFromSexp` and `IntoR` implementations, and the `#[miniextendr]` macro's fallback conversion path handles them correctly.

**Working examples:**
```rust
#[derive(RFactor)]
enum Color { Red, Green, Blue }

// Single enum - works directly
#[miniextendr]
pub fn describe_color(color: Color) -> &'static str {
    match color {
        Color::Red => "red",
        // ...
    }
}

// Return enum - works directly
#[miniextendr]
pub fn get_color(name: &str) -> Color {
    match name {
        "red" => Color::Red,
        // ...
    }
}

// Vector of enums - use FactorVec wrapper
#[miniextendr]
pub fn count_colors(colors: FactorVec<Color>) -> Vec<i32> { ... }

// Vector with NAs - use FactorOptionVec wrapper
#[miniextendr]
pub fn colors_with_na(colors: FactorOptionVec<Color>) -> Vec<&'static str> { ... }
```

The wrapper types (`FactorVec<T>`, `FactorOptionVec<T>`) are needed for vectors due to Rust's orphan rules preventing `impl IntoR for Vec<T: RFactor>`.

---

### 1.2 Dots Must Be Last Parameter

**Status:** By design (R semantics)
**Impact:** Low - matches R convention
**Location:** `miniextendr-macros/src/r_wrapper_builder.rs:133`

Variadic arguments (`...`) can only appear as the final parameter.

**Current behavior:**
```rust
// WORKS
#[miniextendr]
pub fn my_func(x: i32, ...) { }

// DOES NOT WORK
#[miniextendr]
pub fn my_func(..., x: i32) { }
```

**Contract:** R's argument matching algorithm requires `...` to be final. When R encounters `...`, it captures all remaining unmatched arguments. Placing named parameters after `...` creates ambiguity in R's dispatch mechanism. miniextendr enforces this at compile time.

The generated R wrapper:
```r
my_func <- function(x, ...) {
    .Call(C_my_func, .call = match.call(), x, list(...))
}
```

See [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md) for the full dots guide, including `typed_list!` validation.

---

### 1.3 Feature-Gated Modules

**Status:** By design
**Impact:** Low

Feature-gated modules use path-based module switching with `#[cfg]` on `mod` declarations:

```rust
// In lib.rs
#[cfg(feature = "rayon")]
#[path = "rayon_tests.rs"]
mod rayon_tests;

#[cfg(not(feature = "rayon"))]
#[path = "rayon_tests_disabled.rs"]
mod rayon_tests;
```

Create a stub for the disabled case:
```rust
// rayon_tests_disabled.rs
// Empty when feature disabled
```

This pattern is used throughout the rpkg example package (e.g., `rayon_tests.rs` / `rayon_tests_disabled.rs`) and is documented in CLAUDE.md.

See [FEATURES.md](FEATURES.md) for the full feature flags reference.

---

### ~~1.4 No Documentation Override Attributes~~ RESOLVED

**Status:** All implemented
**Resolution:** `#[miniextendr(internal)]` injects `@keywords internal` and suppresses `@export`.
`#[miniextendr(noexport)]` suppresses `@export` only. Both work on standalone functions
and all 6 class system impl blocks via `ClassDocBuilder::with_export_control()`.
`#[miniextendr(doc = "...")]` provides custom roxygen override (replaces auto-generated docs).

**Working example:**
```rust
#[miniextendr(doc = "@title Custom Title\n@description Custom description.")]
pub fn my_func(x: i32) -> i32 { x }
```

See `rpkg/src/rust/doc_attr_tests.rs` for test coverage.

---

## 2. Type Conversion Gaps

### 2.1 Mutable Slice Parameters Rejected at Compile Time

**Status:** Enforced (compile error)
**Impact:** Medium
**Location:** `miniextendr-macros/src/rust_conversion_builder.rs` (rejection)

**Contract:** `&mut [T]` at the `#[miniextendr]` boundary is rejected with a compile error because R's GC can invalidate the slice pointer mid-use. Use `Vec<T>` instead (copies data on input via `TryFromSexp`, copies back on output via `IntoR`).

```rust
// Accept immutable slice, return modified copy
#[miniextendr]
pub fn modify_vec(data: &[i32]) -> Vec<i32> {
    let mut result = data.to_vec();
    result[0] = 42;
    result
}

// Accept owned Vec (copies data)
#[miniextendr]
pub fn modify_vec_owned(mut data: Vec<i32>) -> Vec<i32> {
    data[0] = 42;
    data
}

// For mutable state, use ExternalPtr
#[miniextendr]
pub fn modify_state(state: &mut MyState) {
    state.value = 42;
}
```

See [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) for slice lifetime details and [SAFETY.md](SAFETY.md) for the full safety model.

---

### 2.2 String Matrix/Array Support

**Status:** Not implemented
**Impact:** Low
**Location:** `miniextendr-api/src/into_r.rs`

`ndarray::Array<String, Ix2>` and similar string arrays are not directly convertible.

**Workaround:**
```rust
// Convert manually via nested vectors
let string_matrix: Vec<Vec<String>> = array.outer_iter()
    .map(|row| row.iter().cloned().collect())
    .collect();
```

**Why:** R's STRSXP is a vector of CHARSXP pointers, not contiguous memory. Direct ndarray integration would require special handling because ndarray assumes a contiguous backing buffer.

See [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) for supported matrix types and [FEATURES.md](FEATURES.md#ndarray) for the `ndarray` feature flag.

---

### 2.3 Nested Collection Conversions

**Status:** Complete
**Impact:** Low

All common nested collection types are supported:

| Type | Status |
|------|--------|
| `Vec<Vec<T>>` | Works |
| `Vec<Option<T>>` | Works (all scalar types) |
| `Vec<HashMap<String, V>>` | Works (converts to/from R list of named lists) |
| `Vec<BTreeMap<String, V>>` | Works (converts to/from R list of named lists) |
| `HashMap<K, Vec<V>>` | Works via nested conversion |
| `NamedVector<HashMap<String, V>>` | Works (converts to/from named atomic vector) |
| `NamedVector<BTreeMap<String, V>>` | Works (converts to/from named atomic vector) |

---

## 3. Class System Gaps

### ~~3.1 S7 Features~~ RESOLVED

**Status:** All features implemented
**Location:** `miniextendr-macros/src/miniextendr_impl.rs`

All S7 features are implemented:

| Feature | Status |
|---------|--------|
| Basic class definition | Implemented |
| Constructor (`new`) | Implemented |
| Instance methods | Implemented |
| Static methods | Implemented |
| External generics | Implemented |
| Computed properties (`s7(getter)`) | Implemented |
| Dynamic properties (`s7(getter)` + `s7(setter)`) | Implemented |
| Property defaults, required, deprecated | Implemented |
| Generic dispatch control (`no_dots`, `fallback`) | Implemented |
| `convert_from` / `convert_to` (type coercion) | Implemented |
| Abstract classes (`s7(abstract)`) | Implemented |
| Single inheritance (`s7(parent = "...")`) | Implemented |
| Multi-level inheritance (3+ level chains) | Implemented |
| Property validation (`s7(validate)`) | Implemented |

Method combination (before/after) is not applicable — S7 is single-dispatch by design.

**Multi-level inheritance example:** `S7Animal` (abstract) -> `S7Dog` -> `S7GoldenRetriever`
demonstrates a 3-level chain with methods and properties inherited through S7 generic dispatch.
See `rpkg/src/rust/s7_tests.rs` and `rpkg/tests/testthat/test-class-systems.R`.

---

### ~~3.2 R6 Active Bindings Not Implemented~~ RESOLVED

**Status:** Implemented
**Resolution:** The `#[miniextendr(r6(active))]` attribute now generates R6 active bindings.

Active bindings provide property-like access (`obj$area` instead of `obj$area()`).

**Working example:**
```rust
#[derive(ExternalPtr)]
pub struct Rectangle { width: f64, height: f64 }

#[miniextendr(r6)]
impl Rectangle {
    pub fn new(width: f64, height: f64) -> Self {
        Rectangle { width, height }
    }

    // Regular method - called with ()
    pub fn get_width(&self) -> f64 { self.width }

    // Active binding - accessed like property
    #[miniextendr(r6(active))]
    pub fn area(&self) -> f64 { self.width * self.height }
}
```

**In R:**
```r
r <- Rectangle$new(3, 4)
r$get_width()   # Regular method: 3
r$area          # Active binding: 12 (no parentheses!)
```

---

### ~~3.3 No Direct Field Access~~ RESOLVED

**Status:** Solved via sidecar pattern
**Resolution:** The `#[r_data]` attribute + `RSidecar` + `r_data_accessors` macro provides
automatic field access for R6 and Env class systems.

**Working example (R6):**
```rust
use miniextendr_api::{r_data_accessors, RSidecar};

#[derive(ExternalPtr)]
pub struct Config {
    // Rust-only fields (not exposed to R)
    internal_cache: Vec<u8>,
}

/// Sidecar: fields accessible from R as active bindings.
#[r_data]
pub struct ConfigData {
    pub name: String,
    pub score: f64,
}

r_data_accessors!(Config, ConfigData);

#[miniextendr(r6)]
impl Config {
    pub fn new(name: String, score: f64) -> (Self, ConfigData) {
        (Config { internal_cache: vec![] }, ConfigData { name, score })
    }
}
```

**In R:**
```r
cfg <- Config$new("test", 0.95)
cfg$name         # "test" (active binding, no parentheses)
cfg$name <- "x"  # Sets the field
cfg$score        # 0.95
```

**Class system support for sidecar field access:**
- **R6**: Active bindings (`obj$field` for get, `obj$field <- value` for set)
- **Env**: Standalone functions (`Type_get_field()` / `Type_set_field()`)
- **S3, S4, S7**: Sidecar also supported via generated accessor generics/methods

For non-sidecar structs, manual getters are the recommended approach and work well
across all class systems.

---

### 3.4 S4 Limitations

**Status:** Core features implemented; advanced S4 features mostly not applicable
**Impact:** Low
**Location:** `miniextendr-macros/src/miniextendr_impl.rs:1701-1826`

| Feature | Status |
|---------|--------|
| `setClass` with `.ptr` slot | Implemented |
| `setGeneric`/`setMethod` | Implemented |
| Virtual classes | N/A (use S7 abstract classes instead) |
| Multiple dispatch | N/A (miniextendr is single-dispatch; Rust types are opaque `.ptr`) |
| Method combination | N/A (not meaningful for `.Call`-based methods) |
| Slot validation | N/A (`.ptr` is always externalptr, validation happens in Rust) |
| Class inheritance | Not implemented (use S7 for inheritance chains) |

Most "missing" S4 features are not applicable because miniextendr wraps Rust types as opaque
external pointers -- S4's slot system, multiple dispatch, and validation operate on R-native
data structures that don't exist here. **S7 is the recommended class system for advanced OOP.**

**Recommended pattern:** Use `#[miniextendr(s7)]` for inheritance chains, computed properties,
and generic dispatch. Fall back to S4 only when integrating with Bioconductor or other S4-based
ecosystems.

See [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md) for the full class system comparison and decision flowchart.

---

## 4. Incomplete Features

### 4.1 R Connections API (Experimental)

**Status:** Partial, unstable
**Feature flag:** `connections`
**Location:** `miniextendr-api/src/connection.rs`

The connections API wraps R's internal connection system but is marked unstable because R explicitly reserves the right to change the connection ABI without compatibility.

**What's implemented:**
- `RConnectionImpl` trait for custom connections
- `RCustomConnection` builder
- Callback trampolines for all connection operations
- `std::io` adapters (`IoRead`, `IoWrite`, `IoReadWrite`, etc.)
- `RConnectionIo` builder with capability detection

**What's missing:**
- No wide character support
- Limited binary mode handling
- No statistics/introspection

**Safety mechanism:**
```rust
// Runtime version check - will panic if R's connection ABI changed
check_connections_version();  // Expects R_CONNECTIONS_VERSION == 1
```

**Warning from R source:**
> "We do not expect future connection APIs to be backward-compatible so if you use this, you *must* check the version and proceed only if it matches what you expect."

**Recommended pattern:** Gate connection usage behind feature detection and always check the ABI version:
```rust
if check_connections_version().is_ok() {
    // Safe to use connection API
}
```

See [FEATURES.md](FEATURES.md#connections) for the `connections` feature flag.

---

### ~~4.2 vctrs Integration (Partial)~~ MOSTLY RESOLVED

**Status:** Comprehensive — derive macros, impl blocks, coercion chains all implemented
**Feature flag:** `vctrs`
**Location:** `miniextendr-api/src/vctrs.rs`, `miniextendr-macros/src/vctrs_derive.rs`

**What's implemented:**
- `#[derive(Vctrs)]` macro with `Vctr`, `Rcrd`, `ListOf` kinds
- `#[miniextendr(vctrs)]` impl block support for methods
- `coerce = "type"` attribute generates `vec_ptype2`/`vec_cast` methods
- `vec_proxy`, `vec_restore`, `format` protocol methods
- Advanced features: `proxy_equal`, `proxy_compare`, `proxy_order`, `arith`, `math`
- C API wrappers: `init_vctrs()`, `obj_is_vector()`, `short_vec_size()`, etc.
- Construction helpers: `new_vctr()`, `new_rcrd()`, `new_list_of()`

**What's still missing:**
- Cross-package vctrs type export (no mechanism to share class defs across packages)
- vctrs inheritance (`extends = "parent_type"` pattern)

---

### 4.3 Async/Await Support

**Status:** Not planned (by design)
**Impact:** Low

There is no async/await or Tokio integration. R's C API is single-threaded and synchronous —
async would require a runtime that conflicts with R's execution model. The worker thread
pattern already provides the key benefit (non-blocking Rust execution):

```rust
#[miniextendr]
pub fn fetch_data(url: String) -> String {
    // I/O happens on worker thread (doesn't block R)
    // Use blocking HTTP client like ureq
    ureq::get(&url).call()?.into_string()?
}
```

For true async I/O needs, users should use R-level parallelism (mirai, callr) with
miniextendr handling the per-request Rust work synchronously.

See [THREADS.md](THREADS.md) for the worker thread model and [FEATURES.md](FEATURES.md#rayon) for parallel iteration via Rayon.

---

### 4.4 Lazy Evaluation / Promises

**Status:** Not planned (not applicable)
**Impact:** None

**Contract:** R evaluates all `.Call()` arguments before entering C/Rust code. By the time miniextendr receives a value, promises have already been forced. This is an R-level guarantee, not a miniextendr limitation.

Implementing promise accessors would only be useful for manipulating promises passed inside list/environment containers, which is a niche use case better handled with raw SEXP manipulation.

**Recommended pattern:** For lazy evaluation needs, use R-level wrappers (e.g., `delayedAssign`, `substitute`) and pass the resulting values to Rust. If you need unevaluated expressions, work with `LANGSXP` and call `Rf_eval()` explicitly via the FFI.

---

## 5. Undocumented Behavior

### 5.1 Coercion Precedence

**Location:** `miniextendr-api/src/coerce.rs`

When multiple type conversions are available, the system uses trait-based precedence:

**Precedence order (highest to lowest):**
1. **`Coerce<R>`** - Infallible conversions (identity, widening)
2. **`TryCoerce<R>`** - Fallible conversions (narrowing, overflow-possible)

```rust
// Coerce takes precedence when both exist
impl Coerce<f64> for i32 { ... }      // i32 -> f64: infallible widening
impl TryCoerce<i32> for f64 { ... }   // f64 -> i32: fallible narrowing
```

The blanket impl ensures `Coerce` always wins:
```rust
impl<T, R> TryCoerce<R> for T where T: Coerce<R> {
    fn try_coerce(self) -> Result<R, Infallible> {
        Ok(self.coerce())
    }
}
```

See [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md#coercion-system) for the user-facing coercion guide and [COERCE.md](COERCE.md) for the full coercion trait design.

---

### 5.2 NA Value Handling

**Location:** `miniextendr-api/src/from_r.rs:28-36`

**NA constants:**
| Type | NA Value | Notes |
|------|----------|-------|
| Integer | `i32::MIN` (-2147483648) | Same as `NA_INTEGER` in R |
| Logical | `i32::MIN` | Same representation as integer |
| Real | `0x7FF00000000007A2` (bits) | Specific IEEE 754 NaN payload |

**Critical distinction - NA_REAL vs NaN:**
```rust
// These are DIFFERENT values
let na = NA_REAL;           // R's NA (specific bit pattern)
let nan = f64::NAN;         // Regular IEEE NaN

// Detection requires bit comparison
fn is_na_real(value: f64) -> bool {
    value.to_bits() == NA_REAL.to_bits()
}
```

Regular `f64::NAN` values are preserved as valid data. Only the specific `NA_REAL` bit pattern is treated as NA.

**Option-to-NA coercion:**
```rust
let x: Option<f64> = None;
let r: f64 = x.coerce();  // Returns NA_REAL

let x: Option<i32> = None;
let r: i32 = x.coerce();  // Returns NA_INTEGER
```

See [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md#na-value-representation) for the full NA handling guide with examples for all types.

---

### 5.3 SEXP Lifetime Assumptions

**Location:** `miniextendr-api/src/ffi.rs:215-228`

**Contract:** SEXP lifetimes are tied to R's GC protection, not Rust's borrow checker. miniextendr uses `'static` as a convenience but the actual lifetime is the protection scope.

**The `'static` lifetime is a lie:**
```rust
/// # Safety
/// The returned slice has `'static` lifetime for API convenience, but this
/// is a lie - the actual lifetime is tied to the SEXP's protection status.
unsafe fn as_slice<T: RNativeType>(&self) -> &'static [T];
```

**Safe SEXP storage patterns:**

| Pattern | Safe? | Notes |
|---------|-------|-------|
| Return SEXP from `.Call` | Yes | R receives and protects it |
| Store in `ExternalPtr` | Yes | R owns and GC's it |
| Store in `Preserve` wrapper | Yes | Added to preserve list |
| Store raw SEXP in Rust struct | **No** | Can be GC'd without warning |
| Store in `OwnedProtect` | Yes | Protected for duration |

**String lifetime exception:**
```rust
// CHARSXP strings are interned - truly 'static for session
unsafe fn charsxp_to_str(charsxp: SEXP) -> &'static str
```

**Recommended pattern:** Use `OwnedProtect` or `ProtectScope` for RAII-based SEXP protection. Never store raw SEXPs in long-lived Rust structures without protection.

See [GC_PROTECT.md](GC_PROTECT.md) for the full GC protection toolkit and [SAFETY.md](SAFETY.md) for safety invariants.

---

### 5.4 Mutable Receiver Semantics on ExternalPtr

**Location:** `miniextendr-api/src/externalptr.rs:667-683`

**Contract:** `ExternalPtr` wraps a heap-allocated Rust value behind an R external pointer. Mutation via `&mut self` affects the shared heap allocation directly -- both R and Rust see the same data. This is reference semantics, not copy-on-write.

```rust
impl MyStruct {
    #[miniextendr]
    pub fn increment(&mut self) {
        self.value += 1;  // Mutates Rust data
    }
}
```

**What actually happens:**
1. R's SEXP is `Copy` - the pointer value is copied
2. Rust gets exclusive mutable access to the pointed-to data
3. Mutation affects the underlying allocation
4. R's reference still points to the same allocation
5. Mutation IS visible to R (they share the allocation)

**The confusion:** It DOES work, but only because `ExternalPtr` stores a pointer to heap-allocated Rust data. The mutation is visible because both R and Rust point to the same heap location.

**What would NOT work:**
```rust
// DON'T DO THIS - trying to replace the ExternalPtr itself
pub fn replace(&mut self, new: Self) {
    *self = new;  // This replaces the Rust wrapper, not R's SEXP
}
```

See [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md#externalptr-semantics) for the ExternalPtr ownership model.

---

### 5.5 Thread Safety Debug Assertions

**Location:** `miniextendr-api/src/from_r.rs`

**Contract:** R's C API is single-threaded. miniextendr enforces this via two layers: debug-only assertions on SEXP access, and runtime checks on all checked FFI wrappers.

Debug-only SEXP thread assertions:

```rust
#[cfg(debug_assertions)]
fn assert_main_thread() { ... }
```

**Implication:** Release builds could have SEXP-access thread safety violations that go undetected. Use `#[miniextendr(unsafe(main_thread))]` for explicit main-thread-only functions.

**Runtime thread checks (always active):** The checked FFI wrappers (`Rf_error`, `Rprintf`, etc.) check `is_r_main_thread()` at runtime in all build modes and panic with a clear message like "Rf_error called from non-main thread".

**Recommended pattern:** Rely on the worker thread model for safe R API access. For explicit thread control, use `spawn_with_r()` or `StackCheckGuard`.

See [THREADS.md](THREADS.md) for the thread safety model, [SAFETY.md](SAFETY.md) for invariants, and [ERROR_HANDLING.md](ERROR_HANDLING.md#thread-safety) for thread-related error patterns.

---

### 5.6 Thread Panic Propagation Limitation

**Status:** Known limitation
**Impact:** Low (edge case)
**Location:** `rpkg/src/rust/panic_tests.rs`

Panics from spawned threads cannot be cleanly propagated through `extern "C-unwind"` functions.

**The problem:**
```rust
#[miniextendr]
pub extern "C-unwind" fn problematic() -> SEXP {
    let result = std::thread::spawn(|| {
        panic!("error on spawned thread");
    }).join();

    // Trying to propagate the panic causes:
    // "fatal runtime error: failed to initiate panic, error 5"
    if let Err(e) = result {
        std::panic::resume_unwind(e);  // CRASHES
    }
    // ...
}
```

**Why:** The Rust panic runtime interacts poorly with miniextendr's panic handling when panics cross thread boundaries through FFI functions.

**Workaround:** Handle thread errors explicitly rather than propagating panics:
```rust
#[miniextendr]
pub fn safe_threaded_work() -> Result<i32, String> {
    let result = std::thread::spawn(|| {
        // work that might fail
        42
    }).join();

    match result {
        Ok(v) => Ok(v),
        Err(_) => Err("thread panicked".to_string()),
    }
}
```

**Tests:** See `test-errors-more.R` for skipped tests demonstrating this behavior.

See [ERROR_HANDLING.md](ERROR_HANDLING.md) for the full error handling model and [THREADS.md](THREADS.md) for the worker thread architecture.

---

## 6. Testing Gaps

### ~~6.1 No Property-Based Testing~~ RESOLVED

**Status:** Implemented (24 tests)
**Location:** `miniextendr-api/tests/roundtrip_properties.rs`

Property-based roundtrip tests using `proptest` verify `val → SEXP → val` preservation for:
- Scalar types: i32, f64, bool, String, u8
- Option types: Option<i32>, Option<f64>, Option<bool>, Option<String>
- Vector types: Vec<i32>, Vec<f64>, Vec<bool>, Vec<String>, Vec<u8>
- NA-aware vectors: Vec<Option<i32>>, Vec<Option<f64>>, Vec<Option<bool>>, Vec<Option<String>>
- i64 safe-range ([-2^53, 2^53])
- Edge cases: i32 boundaries, f64 special values, empty vectors, all-NA vectors, unicode strings

Run with: `cargo test -p miniextendr-api --test roundtrip_properties`

---

### ~~6.2 Trybuild Tests Not Utilized~~ RESOLVED

**Status:** Already implemented (23 tests)
**Location:** `miniextendr-macros/tests/ui/`

The trybuild test suite contains 23 compile-fail tests covering:
- Invalid `#[miniextendr]` attribute options
- Type mismatches (non-IntoR returns, non-RNative args)
- Module declaration errors
- Invalid attribute combinations (active on non-R6, etc.)
- Derive macro errors (RNativeType on enum, etc.)
- Trait definition errors (async methods, generic methods, etc.)

Run with: `cargo test --test ui -p miniextendr-macros`

---

### 6.3 Snapshot Testing

**Status:** Not implemented
**Impact:** Low

`miniextendr-macros/tests/snapshots.rs` does not exist. R wrapper output is tested
indirectly via trybuild UI tests (6.2) and R-level integration tests, but there are
no inline snapshot tests for generated R code.

---

## 7. Documentation Status

### 7.1 User Guides

| Guide | Status |
|-------|--------|
| Getting Started | [docs/GETTING_STARTED.md](GETTING_STARTED.md) |
| Choosing a Class System | [docs/CLASS_SYSTEMS.md](CLASS_SYSTEMS.md) |
| Type Conversions | [docs/TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) |
| ALTREP Tutorial | [docs/ALTREP.md](ALTREP.md) |
| Error Handling Best Practices | [docs/ERROR_HANDLING.md](ERROR_HANDLING.md) |
| Thread Safety Guide | Covered in Error Handling |
| Building a Package from Scratch | Covered in Getting Started |

### 7.2 API Documentation

| Topic | Status |
|-------|--------|
| Coercion precedence rules | [docs/TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) |
| NA handling for each type | [docs/TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) |
| SEXP lifetime rules | [docs/TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) |
| Feature flag effects | [docs/FEATURES.md](FEATURES.md) |

### 7.3 Example Coverage

| Feature | Example Status | Location |
|---------|----------------|----------|
| R Connections | Complete | `rpkg/src/rust/connection_tests.rs` |
| vctrs custom types | Complete (`percent` class) | `rpkg/src/rust/vctrs_class_example.rs` |
| Cross-package traits | Complete | `tests/cross-package/` (producer.pkg + consumer.pkg) |
| ALTREP lazy vectors | Basic | `rpkg/src/rust/lib.rs` (altrep section) |

---

## Appendix: File Locations

### Macro Implementation
- `miniextendr-macros/src/miniextendr_fn.rs` - Function macro
- `miniextendr-macros/src/miniextendr_impl.rs` - Impl block macro
- `miniextendr-macros/src/factor_derive.rs` - RFactor derive
- `miniextendr-macros/src/rust_conversion_builder.rs` - Parameter conversion
- `miniextendr-macros/src/r_wrapper_builder.rs` - R wrapper generation
- `miniextendr-macros/src/return_type_analysis.rs` - Return type handling

### Core API
- `miniextendr-api/src/from_r.rs` - TryFromSexp implementations
- `miniextendr-api/src/into_r.rs` - IntoR implementations
- `miniextendr-api/src/coerce.rs` - Coercion traits
- `miniextendr-api/src/error.rs` - Error types
- `miniextendr-api/src/externalptr.rs` - ExternalPtr
- `miniextendr-api/src/factor.rs` - RFactor support
- `miniextendr-api/src/dots.rs` - Dots type

### Optional Features
- `miniextendr-api/src/connection.rs` - R connections (feature: `connections`)
- `miniextendr-api/src/vctrs.rs` - vctrs integration (feature: `vctrs`)

### Tests
- `miniextendr-api/tests/` - Rust integration tests
- `miniextendr-macros/tests/` - Macro tests
- `rpkg/tests/testthat/` - R package tests
- `tests/cross-package/` - Cross-package ABI tests
