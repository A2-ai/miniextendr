# miniextendr: Known Gaps and Limitations

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

**Why:** R's argument matching algorithm requires `...` to be final. When R encounters `...`, it captures all remaining unmatched arguments. Placing named parameters after `...` creates ambiguity in R's dispatch mechanism.

The generated R wrapper:
```r
my_func <- function(x, ...) {
    .Call(C_my_func, .call = match.call(), x, list(...))
}
```

---

### 1.3 Feature-Gated Module Entries Don't Work

**Status:** Broken
**Impact:** Medium
**Location:** `miniextendr-macros/src/lib.rs:1207`

Although the macro parses `#[cfg(...)]` attributes on module entries, they don't function correctly at runtime.

**Current behavior:**
```rust
// THIS DOES NOT WORK CORRECTLY
miniextendr_module! {
    mod mymod;

    #[cfg(feature = "rayon")]
    fn rayon_function;  // Won't be conditionally compiled properly
}
```

**Workaround - use path-based module switching:**
```rust
// In lib.rs
#[cfg(feature = "rayon")]
#[path = "rayon_tests.rs"]
mod rayon_tests;

#[cfg(not(feature = "rayon"))]
#[path = "rayon_tests_disabled.rs"]
mod rayon_tests;

miniextendr_module! {
    mod mymod;
    use rayon_tests;  // Always present, contents vary
}
```

Create a stub for the disabled case:
```rust
// rayon_tests_disabled.rs
use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod rayon_tests;
    // Empty when feature disabled
}
```

---

### ~~1.4 No Documentation Override Attributes~~ PARTIALLY RESOLVED

**Status:** `internal` and `noexport` implemented; `doc` override not implemented
**Resolution:** `#[miniextendr(internal)]` injects `@keywords internal` and suppresses `@export`.
`#[miniextendr(noexport)]` suppresses `@export` only. Both work on standalone functions
and all 6 class system impl blocks via `ClassDocBuilder::with_export_control()`.

**Still missing:**
- `#[miniextendr(doc = "Custom documentation")]` — custom roxygen override

---

## 2. Type Conversion Gaps

### 2.1 Mutable Slice Parameters Not Supported

**Status:** By design (safety)
**Impact:** Medium
**Location:** `miniextendr-api/src/from_r.rs:1464-1494`

While `TryFromSexp` is implemented for `&'static mut [T]`, using mutable slices as `#[miniextendr]` function parameters is unsafe and not supported.

**Why it's unsafe:**
1. R vectors use copy-on-write semantics - mutation violates R's invariants
2. The `'static` lifetime on slices is a "lie" - actual lifetime is tied to GC protection
3. Multiple R references to the same vector would create aliased mutable references

**Workarounds:**
```rust
// Option 1: Accept immutable slice, return modified copy
#[miniextendr]
pub fn modify_vec(data: &[i32]) -> Vec<i32> {
    let mut result = data.to_vec();
    result[0] = 42;
    result
}

// Option 2: Accept owned Vec (copies data)
#[miniextendr]
pub fn modify_vec(mut data: Vec<i32>) -> Vec<i32> {
    data[0] = 42;
    data
}

// Option 3: Use ExternalPtr for mutable state
#[miniextendr]
pub fn modify_state(state: &mut MyState) {
    state.value = 42;
}
```

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

**Why:** R's STRSXP is a vector of CHARSXP pointers, not contiguous memory. Direct ndarray integration would require special handling.

---

### 2.3 Nested Collection Conversions

**Status:** Partial
**Impact:** Low

Some nested collection types lack direct conversions:

| Type | Status |
|------|--------|
| `Vec<Vec<T>>` | Works |
| `Vec<Option<T>>` | Works (all scalar types) |
| `Vec<HashMap<K, V>>` | Not directly convertible |
| `HashMap<K, Vec<V>>` | Works via nested conversion |

**Workaround:** Decompose to parallel vectors or use `ExternalPtr` for complex structures.

---

## 3. Class System Gaps

### 3.1 S7 Incomplete Features

**Status:** Partial implementation
**Impact:** Medium
**Location:** `miniextendr-macros/src/miniextendr_impl.rs:1546-1693`

S7 support covers constructors and methods but lacks advanced features:

| Feature | Status |
|---------|--------|
| Basic class definition | Implemented |
| Constructor (`new`) | Implemented |
| Instance methods | Implemented |
| Static methods | Implemented |
| External generics | Implemented |
| Property validation | Not implemented |
| Method combination (before/after) | Not implemented |
| Inheritance chains | Implemented (`parent = "..."`, `abstract`) |
| `@prop_validator` | Not implemented |

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

For users who prefer manual getters on non-sidecar structs, that pattern remains
straightforward and well-understood.

---

### 3.4 S4 Limitations

**Status:** Basic implementation only
**Impact:** Low
**Location:** `miniextendr-macros/src/miniextendr_impl.rs:1701-1826`

| Feature | Status |
|---------|--------|
| `setClass` with `.ptr` slot | Implemented |
| `setGeneric`/`setMethod` | Implemented |
| Virtual classes | Not implemented |
| Multiple dispatch | Not implemented |
| Method combination | Not implemented |
| Slot validation | Not implemented |
| Class inheritance | Not implemented |

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
- No example implementations
- No tests beyond compile-time checks
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

**Status:** Not implemented
**Impact:** Low (workarounds exist)

There is no async/await or Tokio integration. The worker thread pattern handles most use cases.

**Current architecture:**
- Worker thread for CPU-bound Rust code
- `with_r_thread()` for R API calls from worker
- Synchronous execution model

**Workaround for I/O-bound operations:**
```rust
#[miniextendr]
pub fn fetch_data(url: String) -> String {
    // I/O happens on worker thread (doesn't block R)
    // Use blocking HTTP client like ureq
    ureq::get(&url).call()?.into_string()?
}
```

---

### 4.4 Lazy Evaluation / Promises

**Status:** Not implemented
**Impact:** Low

No support for R's promise mechanism (`PROMSXP`). Cannot implement lazy function arguments.

**What would be needed:**
- `PRVALUE()`, `PRCODE()`, `PRENV()` accessors
- Promise state tracking (unevaluated/evaluated/errored)
- Integration with R's evaluation system

**Workaround:** Work with unevaluated expressions (`LANGSXP`) and call `eval()` explicitly.

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

---

### 5.3 SEXP Lifetime Assumptions

**Location:** `miniextendr-api/src/ffi.rs:215-228`

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

---

### 5.4 Mutable Receiver Semantics on ExternalPtr

**Location:** `miniextendr-api/src/externalptr.rs:667-683`

**The pitfall:** `&mut self` methods on `ExternalPtr` mutate the Rust data but this doesn't "update" R's copy.

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

---

### 5.5 Thread Safety Debug Assertions

**Location:** `miniextendr-api/src/from_r.rs`

Thread safety checks only run in debug builds:

```rust
#[cfg(debug_assertions)]
fn assert_main_thread() { ... }
```

**Implication:** Release builds could have thread safety violations that go undetected. Use `#[miniextendr(unsafe(main_thread))]` for explicit main-thread-only functions.

**Runtime thread checks:** The checked FFI wrappers (`Rf_error`, `Rprintf`, etc.) DO check `is_r_main_thread()` at runtime and panic with a clear message like "Rf_error called from non-main thread".

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

---

## 6. Testing Gaps

### 6.1 No Property-Based Testing

**Status:** Not present
**Impact:** Medium

No quickcheck or proptest usage. All tests are deterministic.

**Recommendation:** Add property-based tests for:
- Type conversions (roundtrip properties)
- ALTREP implementations (length invariants)
- Coercion chains (transitivity)

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

### 6.3 Snapshot Testing Underutilized

**Status:** Minimal usage
**Location:** `miniextendr-macros/tests/snapshots.rs`

`expect-test` is used but only for a few cases.

**Recommendation:** Add snapshot tests for:
- Generated R wrapper functions
- Generated roxygen documentation
- NAMESPACE entries
- Class system boilerplate (R6, S3, S4, S7)

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
- `miniextendr-macros/src/miniextendr_module.rs` - Module macro
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
