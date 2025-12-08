# miniextendr Documentation

A Rust-R interoperability framework for building R packages with Rust backends.

## Build Commands

**Always use the justfile** for building, testing, and checking the project. Run `just` to see all available commands.

| Command | Description |
|---------|-------------|
| `just check` | Check all crates compile |
| `just build` | Build all crates |
| `just clippy` | Run clippy lints |
| `just fmt` | Format all code |
| `just test` | Run Rust tests |
| `just configure` | Vendor deps and run ./configure |
| `just devtools-test` | Run R tests via devtools |
| `just devtools-load` | Load rpkg with devtools::load_all |
| `just r-cmd-install` | Install rpkg via R CMD INSTALL |
| `just r-cmd-check` | Run R CMD check |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         R Package (rpkg)                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐  │
│  │ R/wrappers.R    │  │ src/entrypoint.c│  │ src/rust/lib.rs     │  │
│  │ (auto-generated)│  │ (R_init_*)      │  │ (#[miniextendr] fns)│  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      miniextendr-macros                             │
│  • #[miniextendr] - generates C wrappers + R wrappers               │
│  • miniextendr_module! - registers functions with R                 │
│  • #[r_ffi_checked] - thread-safe R FFI wrappers                    │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       miniextendr-api                               │
│  ┌──────────┐ ┌──────────┐ ┌─────────────┐ ┌────────────────────┐   │
│  │ ffi.rs   │ │worker.rs │ │externalptr.rs│ │ altrep*.rs (4)    │   │
│  │ (R FFI)  │ │(threading)│ │(Box for R)  │ │ (lazy vectors)    │   │
│  └──────────┘ └──────────┘ └─────────────┘ └────────────────────┘   │
│  ┌──────────────┐ ┌──────────────┐ ┌─────────────┐ ┌─────────────┐  │
│  │unwind_protect│ │ from_r.rs    │ │ into_r.rs   │ │ error.rs    │  │
│  │(R error safe)│ │ (SEXP→Rust)  │ │ (Rust→SEXP) │ │ (r_error!)  │  │
│  └──────────────┘ └──────────────┘ └─────────────┘ └─────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Module Documentation

### 1. ffi.rs - R FFI Definitions (510 LOC)

Raw FFI bindings to R's C API.

#### Types

| Type | Description |
|------|-------------|
| `SEXP` | R's generic pointer type (`*mut SEXPREC`) |
| `SEXPTYPE` | Enum of R object types (INTSXP, REALSXP, etc.) |
| `R_xlen_t` | R's length type (`isize`) |
| `Rboolean` | R's boolean (`FALSE=0`, `TRUE=1`) |
| `Rcomplex` | Complex number `{ r: f64, i: f64 }` |

#### Key Functions

```rust
// Memory allocation
Rf_allocVector(type: SEXPTYPE, len: R_xlen_t) -> SEXP
Rf_protect(sexp: SEXP) -> SEXP
Rf_unprotect(count: c_int)

// Scalar creation
Rf_ScalarInteger(x: i32) -> SEXP
Rf_ScalarReal(x: f64) -> SEXP
Rf_ScalarLogical(x: i32) -> SEXP
Rf_ScalarString(charsxp: SEXP) -> SEXP

// Data access
DATAPTR(x: SEXP) -> *mut c_void       // mutable pointer
DATAPTR_RO(x: SEXP) -> *const c_void  // read-only pointer
INTEGER(x: SEXP) -> *mut i32
REAL(x: SEXP) -> *mut f64
STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP

// External pointers
R_MakeExternalPtr(ptr, tag, prot) -> SEXP
R_ExternalPtrAddr(s: SEXP) -> *mut c_void
R_RegisterCFinalizerEx(s, finalizer, onexit)

// Error handling
Rf_error(fmt, ...) -> !    // Raises R error (never returns)
Rf_warning(fmt, ...)       // Raises R warning
R_UnwindProtect(...)       // Catch R errors with cleanup
```

#### Thread Safety

All R FFI functions have checked wrappers that panic if called from non-main thread:

```rust
// Checked (panics if wrong thread)
Rf_error(fmt, arg)

// Unchecked (internal use)
Rf_error_unchecked(fmt, arg)
```

#### Internal Traits

```rust
// Extension methods for SEXP
trait SexpExt {
    fn type_of(&self) -> SEXPTYPE;
    fn is_null_or_nil(&self) -> bool;
    fn len(&self) -> usize;
    fn as_slice<T: RNativeType>(&self) -> &'static [T];
}

// Marker for R-native element types
trait RNativeType {
    const SEXP_TYPE: SEXPTYPE;
}
// Implemented for: i32, f64, u8, Rboolean
```

---

### 2. worker.rs - Worker Thread Pattern (352 LOC)

Execute Rust code on a separate thread with proper panic handling.

#### Problem Solved

R uses `longjmp` for error handling, which skips Rust destructors. The worker thread pattern:
1. Runs Rust code on a worker thread where `catch_unwind` works
2. Catches panics and converts them to R errors
3. Allows calling R APIs from worker via message passing

#### Key Functions

```rust
/// Check if on R's main thread
pub fn is_r_main_thread() -> bool

/// Run closure on worker thread (called by macro-generated code)
pub fn run_on_worker<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static;

/// Call R APIs from worker thread
pub fn with_r_thread<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static;

/// Initialize the worker (called from entrypoint.c)
pub extern "C-unwind" fn miniextendr_worker_init()
```

#### Example

```rust
#[miniextendr]
fn compute_heavy(n: i32) -> i32 {
    // This runs on worker thread automatically
    let result = expensive_computation(n);

    // Need to call R? Use with_r_thread:
    let rng = with_r_thread(|| {
        unsafe { call_r_runif() }
    });

    result + rng
}
```

#### Execution Flow

```
R calls C_compute_heavy(sexp)
    │
    ▼
run_on_worker(|| { ... })
    │
    ├──► Worker thread: catch_unwind(closure)
    │        │
    │        ├─ with_r_thread(work) ──► Main thread executes work
    │        │                              │
    │        │◄────────────────────────────┘
    │        │
    │        ▼
    │    Done(Result<T, String>)
    │
    ▼
Main thread: convert to SEXP or R error
```

---

### 3. unwind_protect.rs - R Error Protection (162 LOC)

Safe wrapper for `R_UnwindProtect` to run Rust destructors on R errors.

#### Problem Solved

When R's `Rf_error()` is called, it uses `longjmp` which bypasses Rust's drop semantics. `with_r_unwind_protect` ensures destructors run.

#### Key Function

```rust
/// Execute closure with R unwind protection
pub fn with_r_unwind_protect<F, R>(f: F, call: Option<SEXP>) -> R
where
    F: FnOnce() -> R;
```

#### Example

```rust
#[miniextendr]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_risky_operation() -> SEXP {
    with_r_unwind_protect(|| {
        let resource = acquire_resource();  // Will be dropped!

        // This might call Rf_error internally
        unsafe { some_r_api_call() }
    }, None)
}
```

#### How It Works

1. `R_UnwindProtect` calls our trampoline
2. If R error occurs, cleanup handler triggers Rust panic
3. `catch_unwind` catches panic, drops are run
4. `R_ContinueUnwind` resumes R's error flow

---

### 4. externalptr.rs - Box-like Owned Pointer (1,244 LOC)

Store Rust objects in R's external pointer SEXP.

#### Core Type

```rust
/// Owned pointer stored in R's EXTPTRSXP
pub struct ExternalPtr<T: TypedExternal> {
    sexp: SEXP,
    // !Send, !Sync (R is single-threaded)
}
```

#### TypedExternal Trait

```rust
pub trait TypedExternal: 'static {
    const TYPE_NAME: &'static str;
    const TYPE_NAME_CSTR: &'static [u8];  // Null-terminated
}

// Derive macro available:
#[derive(ExternalPtr)]
struct MyData { ... }

// Or manual:
impl_typed_external!(MyData);
```

#### API (Box-equivalent)

```rust
// Construction
ExternalPtr::new(x: T) -> Self              // Thread-safe (uses with_r_thread)
ExternalPtr::new_unchecked(x: T) -> Self    // Main thread only
ExternalPtr::from_raw(ptr: *mut T) -> Self

// Destruction
ExternalPtr::into_raw(this) -> *mut T
ExternalPtr::into_inner(this) -> T
ExternalPtr::leak(this) -> &'a mut T

// Access
impl Deref<Target = T> for ExternalPtr<T>
impl DerefMut for ExternalPtr<T>
fn as_ref(&self) -> Option<&T>
fn as_mut(&mut self) -> Option<&mut T>

// Type checking
fn try_from_sexp(sexp: SEXP) -> Option<Self>  // Runtime type check
fn is_type_match(sexp: SEXP) -> bool

// R interop
fn as_sexp(&self) -> SEXP
fn protect_r_object(&mut self, obj: SEXP)     // Keep R object alive
```

#### Type Identification via R Symbols

Type identification uses R's interned symbol system (via `Rf_install`). This provides efficient pointer comparison for type checking while leveraging R's existing string deduplication:

```rust
// Internal: get or create the R symbol for a type
unsafe fn type_symbol<T: TypedExternal>() -> SEXP {
    Rf_install(T::TYPE_NAME_CSTR.as_ptr().cast())
}

// Type checking is a simple pointer comparison
fn is<T: TypedExternal>(&self) -> bool {
    stored_symbol == type_symbol::<T>()
}
```

#### Example

```rust
#[derive(ExternalPtr)]
struct Counter { value: i32 }

#[miniextendr]
fn counter_new() -> ExternalPtr<Counter> {
    ExternalPtr::new(Counter { value: 0 })
}

#[miniextendr]
fn counter_increment(ptr: ExternalPtr<Counter>) -> i32 {
    ptr.value += 1;
    ptr.value
}
```

---

### 5. ALTREP Modules (~700 LOC total)

Alternative Representations for lazy/compact R vectors.

Files: `altrep.rs` (46 LOC), `altrep_traits.rs`, `altrep_bridge.rs`, `altrep_registration.rs`

#### Proc-Macro Approach

Define custom ALTREP classes with full method control using the `#[miniextendr]` proc-macro:

```rust
// Define custom class with full ALTREP method control
#[miniextendr(class = "ConstantInt", pkg = "rpkg", base = "Int")]
struct ConstantIntClass;

impl Altrep for ConstantIntClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t { 10 }  // Always length 10
}

impl AltVec for ConstantIntClass {
    const HAS_DATAPTR: bool = false;  // No materialization
}

impl AltInteger for ConstantIntClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 { 42 }  // Always returns 42

    const HAS_SUM: bool = true;
    fn sum(x: SEXP, narm: bool) -> SEXP {
        // O(1) sum: 42 * 10 = 420
        unsafe { Rf_ScalarReal(420.0) }
    }
}
```

#### Method Traits

| Trait | Methods | Purpose |
|-------|---------|---------|
| `Altrep` | `length`, `duplicate`, `coerce`, `serialize` | Base ALTREP methods |
| `AltVec` | `dataptr`, `dataptr_or_null`, `extract_subset` | Vector access |
| `AltInteger` | `elt`, `get_region`, `is_sorted`, `no_na`, `sum`, `min`, `max` | Integer-specific |
| `AltReal` | (same as AltInteger) | Real-specific |
| `AltLogical` | `elt`, `get_region`, `is_sorted`, `no_na`, `sum` | Logical-specific |
| `AltRaw` | `elt`, `get_region` | Raw-specific |
| `AltString` | `elt`, `set_elt`, `is_sorted`, `no_na` | String-specific |
| `AltList` | `elt`, `set_elt` | List-specific |

Each method has a corresponding `HAS_*` constant. Set to `true` to enable the method; the proc-macro only registers methods where `HAS_*` is true.

#### Class Registration

Classes are registered lazily on first use via `OnceLock`:

```rust
// Generated by proc-macro
impl RegisterAltrep for ConstantIntClass {
    fn get_or_init_class() -> R_altrep_class_t {
        static CLASS: OnceLock<R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            // Create class and install methods
            ...
        })
    }
}
```

---

### 6. from_r.rs - SEXP to Rust (88 LOC)

Convert R objects to Rust types.

```rust
pub trait TryFromSexp: Sized {
    type Error;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error>;
}

// Implemented for scalars
impl TryFromSexp for i32 { ... }   // From INTSXP length-1
impl TryFromSexp for f64 { ... }   // From REALSXP length-1
impl TryFromSexp for u8  { ... }   // From RAWSXP length-1

// Implemented for slices
impl TryFromSexp for &'static [i32] { ... }
impl TryFromSexp for &'static [f64] { ... }
```

#### Errors

```rust
pub enum SexpError {
    Type(SexpTypeError),    // Wrong SEXPTYPE
    Length(SexpLengthError), // Wrong length (expected 1)
}
```

---

### 7. into_r.rs - Rust to SEXP (97 LOC)

Convert Rust types to R objects.

```rust
pub trait IntoR {
    fn into_sexp(self) -> SEXP;
}

impl IntoR for ()   { ... }  // R_NilValue
impl IntoR for i32  { ... }  // Rf_ScalarInteger
impl IntoR for f64  { ... }  // Rf_ScalarReal
impl IntoR for u8   { ... }  // Rf_ScalarRaw
impl IntoR for bool { ... }  // Rf_ScalarLogical
impl IntoR for &str { ... }  // Rf_ScalarString(Rf_mkCharLenCE)
impl IntoR for String { ... }
impl IntoR for SEXP { ... }  // identity
impl<T: TypedExternal> IntoR for ExternalPtr<T> { ... }
```

---

### 8. error.rs - Error Helpers (96 LOC)

Convenient R error/warning/print functions.

```rust
/// Raise R error (does not return)
pub fn r_stop(msg: &str) -> !

/// Raise R error with formatting
#[macro_export]
macro_rules! r_error {
    ($($arg:tt)*) => { r_stop(&format!($($arg)*)) }
}

/// Raise R warning (returns normally)
pub fn r_warning(msg: &str)

/// Print to R console
pub fn r_print(msg: &str)
pub fn r_println(msg: &str)
```

#### Example

```rust
#[miniextendr]
fn validate(x: i32) -> i32 {
    if x < 0 {
        r_error!("x must be non-negative, got {}", x);
    }
    x * 2
}
```

---

### 9. dots.rs - Variadic Arguments (12 LOC)

Support for R's `...` arguments.

```rust
pub struct Dots {
    pub inner: SEXP,  // The evaluated list(...)
}
```

The macro handles the conversion; users write:

```rust
#[miniextendr]
fn my_sum(dots: ...) -> f64 {
    // dots.inner is VECSXP containing evaluated args
    ...
}

// Or unnamed:
#[miniextendr]
fn my_func(x: i32, ...) {
    // _dots available as &Dots
}
```

---

### 10. backtrace.rs - Panic Hook (25 LOC)

Configurable panic backtrace via environment variable.

```rust
/// Register panic hook that respects MINIEXTENDR_BACKTRACE env var
pub extern "C-unwind" fn miniextendr_panic_hook()
```

Set `MINIEXTENDR_BACKTRACE=1` or `MINIEXTENDR_BACKTRACE=true` to see full backtraces.

---

### 11. macro_coverage.rs - Test Infrastructure (168 LOC)

Internal module that instantiates every macro variation for testing. Contains example functions covering:

- No return / unit return
- `Option<()>` / `Option<T>`
- `Result<(), E>` / `Result<T, E>`
- Mutable arguments
- `()` arguments (NULL in R)
- Leading underscore args
- Dots (named, unnamed, with other args)
- Invisible returns
- Panic paths
- `extern "C-unwind"` functions

---

## miniextendr-macros

### #[miniextendr] on Functions

Generates:
1. The original Rust function
2. A C wrapper (`C_<name>`)
3. An R wrapper string (`R_WRAPPER_<NAME>`)
4. A registration constant (`call_method_def_<name>`)

#### Attributes

```rust
#[miniextendr]                    // Basic usage
#[miniextendr(invisible)]         // Force invisible return
#[miniextendr(visible)]           // Force visible return
#[miniextendr(main_thread)]       // Force main thread execution
#[miniextendr(check_interrupt)]   // Check Ctrl+C before running
```

#### Execution Strategy Selection

| Condition | Strategy |
|-----------|----------|
| Returns `SEXP` | Main thread |
| Returns `ExternalPtr<T>` | Main thread |
| Takes `Dots` (`...`) | Main thread |
| `#[miniextendr(main_thread)]` | Main thread |
| `extern "C-unwind"` ABI | Main thread (direct C wrapper) |
| Everything else | Worker thread |

### #[miniextendr] on Structs

For ALTREP class registration. Generates registration code for method trait implementations.

### miniextendr_module!

Registers functions and structs with R's dynamic loading.

```rust
miniextendr_module! {
    mod mymodule;

    use other_module;  // Include another module's registrations

    fn my_function;
    fn another_function;

    extern "C-unwind" fn C_direct_wrapper;

    struct MyAltrepClass;
}
```

Generates:
- `R_init_<module>` entry point
- Call method registration array
- R wrapper file content

### #[r_ffi_checked]

Applied to extern blocks to generate thread-checked wrappers:

```rust
#[r_ffi_checked]
unsafe extern "C-unwind" {
    pub fn Rf_allocVector(t: SEXPTYPE, n: R_xlen_t) -> SEXP;
}

// Generates (in debug builds):
pub unsafe fn Rf_allocVector(t: SEXPTYPE, n: R_xlen_t) -> SEXP {
    debug_assert!(is_r_main_thread(), "...");
    Rf_allocVector_unchecked(t, n)
}
```

---

## rpkg - Example R Package

Demonstrates all features with test functions:

| Category | Functions |
|----------|-----------|
| Basic | `add`, `add2`, `add3`, `add4` |
| Drops | `drop_message_on_success`, `drop_on_panic` |
| Errors | `add_panic`, `add_r_error`, `add_panic_heap` |
| Unwind | `C_unwind_protect_normal`, `C_unwind_protect_r_error` |
| ExternalPtr | Type safety tests |
| ALTREP | `ConstantIntClass`, `VecIntAltrepClass` |
| Dots | Various `...` patterns |

---

## Simplification History

### Completed Simplifications

1. **ExternalPtr type identification** - Replaced `StableTypeId` (hash + len + ptr stored in RAWSXP) with R's interned symbols via `Rf_install()`. Type checking is now a simple pointer comparison. Removed ~143 LOC including `const_hash_str` FNV-1a hashing and serialization code.

2. **ALTREP backend traits removed** - Deleted `altrep.rs` (1,702 LOC) and `altrep_std_impls.rs` (855 LOC). The proc-macro approach in `altrep_traits.rs`/`altrep_bridge.rs`/`altrep_registration.rs` provides full flexibility. Total: ~2,557 LOC removed.

3. **`ErasedExternalPtr` simplified** - Converted to type alias (commit `87c9f7c`)

4. **Previous experiments removed** - Commit `b4d126b`, -219 lines

### Confirmed NOT Redundant

1. **SendableSexp/SendablePtr** - Required for worker thread communication
2. **macro_coverage.rs** - Intentional test infrastructure
3. **`// TODO: finish the dots module`** - Module appears complete despite TODO
