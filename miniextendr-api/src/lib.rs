//! miniextendr-api: core runtime, FFI, ALTREP, and macros
//!
//! Note: ALTREP trait methods receive raw SEXP pointers from R's runtime.
//! These are safe to dereference because R guarantees valid SEXPs in ALTREP callbacks.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

// Export a rust function to R
/// Derive macro for implementing `TypedExternal` on a type.
/// This enables the type to be stored in an `ExternalPtr<T>`.
pub use miniextendr_macros::ExternalPtr;
/// Derive macro for implementing `RNative` on newtype wrappers.
/// This enables the newtype to be used with `Coerce<R>` traits.
///
/// Supports both tuple structs and single-field named structs:
///
/// ```ignore
/// #[derive(Clone, Copy, miniextendr_api::RNative)]
/// struct UserId(i32);  // tuple struct
///
/// #[derive(Clone, Copy, miniextendr_api::RNative)]
/// struct Temperature { celsius: f64 }  // named field
/// ```
///
pub use miniextendr_macros::RNative;
///
/// ```
/// use miniextendr_api::miniextendr;
///
/// #[miniextendr]
/// fn foo() {}
/// ```
///
/// produces a C wrapper named `C_foo`, and an R wrapper called `foo`.
///
/// In case of function arguments beginning with `_*`, then the R wrapper renames the argument
/// to `unused_*`, as it is not allowed for a variable to begin with `_` in R.
///
/// ## `extern "C-unwind"` and the `unsafe_` prefix
///
/// When providing a raw C ABI function, the function IS the C wrapper (no safe Rust
/// wrapper is generated). Since these functions bypass miniextendr's safety features
/// (worker thread, panic handling, type conversion), the R wrapper is prefixed with
/// `unsafe_` to signal this to R users.
///
/// ```
/// use miniextendr_api::miniextendr;
/// use miniextendr_api::ffi::{SEXP, R_NilValue};
///
/// #[miniextendr]
/// #[unsafe(no_mangle)]
/// extern "C-unwind" fn C_my_function() -> SEXP { unsafe { R_NilValue } }
/// ```
///
/// This generates:
/// - C symbol: `C_my_function` (the function you wrote)
/// - R wrapper: `unsafe_C_my_function()` (prefixed with `unsafe_`)
///
/// The `unsafe_` prefix indicates that:
/// 1. The function runs directly on R's main thread (no worker thread isolation)
/// 2. Panics may not be properly caught (depends on your implementation)
/// 3. No automatic type conversion from R to Rust types
///
/// Use `extern "C-unwind"` when you need direct FFI control, such as for ALTREP
/// callbacks or performance-critical code that handles its own safety.
///
///
/// ## Variadic support: [`Dots`] / DotDotDot / `...`
///
/// It is possible to provide `...` as the last argument in an `miniextendr`-`fn`.
/// The corresponding R wrapper will then provide this argument as an evaluated arguments `list(...)`.
///
/// Since Rust does not have variadic support, the provided `fn`'s `...` is overwritten with [`&Dots`].
/// While R can handle unnamed, variadic arguments i.e. `...`, regular Rust `fn` cannot, therefore
/// when `...` is provided, the Rust function has its last argument renamed to `_dots`. Normally,
/// the R wrapper would have its `_*` arguments renamed to `unused_*`, but this is unnecessary in this case.
///
/// It is necessary to add register these functions using [`miniextendr_module`] in order for them to
/// be available in the surrounding R package.
///
/// ## Attributes
///
/// The macro supports the following attributes:
///
/// - `#[miniextendr(unsafe(main_thread))]` - Force the function to run on the main R thread.
///   Use this for functions that call R APIs internally. This is marked `unsafe(...)` because
///   bypassing the worker-thread pattern can allow R errors (longjmp) to skip Rust destructors.
///
/// - `#[miniextendr(invisible)]` - Force the R wrapper to return invisibly.
///   Normally, functions returning `()`, `Option<()>`, or `Result<(), _>` return invisibly.
///
/// - `#[miniextendr(visible)]` - Force the R wrapper to return visibly.
///   Overrides the default invisible behavior for unit-returning functions.
///
/// - `#[miniextendr(check_interrupt)]` - Check for user interrupts (Ctrl+C) before executing.
///   Calls `R_CheckUserInterrupt()` at the start of the function. Implies `unsafe(main_thread)`.
///
/// - `#[miniextendr(coerce)]` - Enable type coercion for ALL non-R-native parameter types.
///   Allows using types like `u16`, `i16`, `i8`, `f32`, `Vec<u16>`, etc. as parameters.
///   R values are extracted as native types (i32, f64) and coerced using [`TryCoerce`].
///
/// - Per-parameter `#[miniextendr(coerce)]` - Add to individual parameters for selective coercion:
///   ```ignore
///   #[miniextendr]
///   fn foo(#[miniextendr(coerce)] x: u16, y: i32) { ... }
///   ```
///
/// See [`COERCE.md`] in the repository for details on supported coercions.
///
/// Multiple attributes can be combined: `#[miniextendr(coerce, invisible)]`
///
/// ## R wrappers
///
/// The generated R wrapper calls the C wrapper via `.Call()`. By default:
/// - Functions returning `()`, `Option<()>`, or `Result<(), _>` return invisibly
/// - All other return types are visible
///
/// [`&Dots`]: dots::Dots
/// [`Dots`]: dots::Dots
pub use miniextendr_macros::miniextendr;
pub use miniextendr_macros::miniextendr_module;
pub use miniextendr_macros::r_ffi_checked;

pub mod altrep;
pub mod altrep_bridge;
pub mod altrep_data;
pub mod altrep_impl;
pub mod altrep_registration;
pub mod altrep_traits;
pub mod ffi;

// Re-export high-level ALTREP data traits
pub use altrep_data::{
    AltComplexData, AltIntegerData, AltListData, AltLogicalData, AltRawData, AltRealData,
    AltStringData, AltrepDataptr, AltrepLen, Logical, Sortedness,
};
// Re-export AltrepBase for base type inference
pub use altrep::{AltrepBase, RBase};
// Note: SexpExt is pub(crate), imported directly in modules that need it
pub mod from_r;
pub mod into_r;
pub use into_r::IntoR;
pub mod unwind_protect;
pub mod worker;

// Rayon integration (parallel computation with R interop)
#[cfg(feature = "rayon")]
pub mod rayon_bridge;
pub use worker::*;

// Thread safety utilities for calling R from non-main threads
pub mod thread;

// Stack size constants and builder (always available)
#[cfg(windows)]
pub use thread::WINDOWS_R_STACK_SIZE;
pub use thread::{DEFAULT_R_STACK_SIZE, RThreadBuilder};

// Stack checking control (requires nonapi feature)
#[cfg(feature = "nonapi")]
pub use thread::{StackCheckGuard, scope_with_r, spawn_with_r, with_stack_checking_disabled};

// Error handling helpers (r_stop, r_warning, r_print, r_println, r_error! macro)
pub mod error;
pub use error::{r_print, r_println, r_stop, r_warning};

// Re-export from_r
pub use from_r::{SexpError, SexpLengthError, SexpNaError, SexpTypeError, TryFromSexp};

// Encoding / locale probing (mainly for debugging; some parts require `nonapi`)
// TODO: Fix encoding module - it references non-exported non-API symbols
// #[cfg(feature = "nonapi")]
// pub mod encoding;

// Note: RNativeType is pub(crate), imported directly in modules that need it

pub mod backtrace;

pub mod coerce;
pub use coerce::{
    // Trait bounds (for where clauses)
    CanCoerceToInteger,
    CanCoerceToLogical,
    CanCoerceToRaw,
    CanCoerceToReal,
    // Traits
    Coerce,
    CoerceError,
    RNative,
    TryCoerce,
};

pub mod dots;

// External pointer module - Box-like owned pointer wrapping R's EXTPTRSXP
pub mod externalptr;
pub use externalptr::{
    ErasedExternalPtr, ExternalPtr, ExternalSlice, SendableSexp, TypedExternal,
    altrep_data1_as, altrep_data1_mut, altrep_data2_as,
};

// TypedExternal implementations for std types
pub mod externalptr_std;

// R object preservation and allocator
pub(crate) mod preserve;

pub mod allocator;
pub use allocator::RAllocator;

/// This is used to ensure the macros of `miniextendr-macros` treat this crate as a "user crate"
/// atleast in the `macro_coverage`
#[doc(hidden)]
extern crate self as miniextendr_api;

#[doc(hidden)]
pub mod macro_coverage;
