//! # miniextendr-macros - Procedural macros for Rust <-> R interop
//!
//! This crate provides the procedural macros that power miniextendr's code
//! generation. Most users should depend on `miniextendr-api` and use its
//! re-exports, but this crate can be used directly when you only need macros.
//!
//! Primary macros and derives:
//! - `#[miniextendr]` on functions, impl blocks, trait defs, and trait impls.
//! - `#[r_ffi_checked]` for main-thread routing of C-ABI wrappers.
//! - Derives: `ExternalPtr`, `RNativeType`, ALTREP derives, `RFactor`.
//! - Helpers: `typed_list` for typed list builders.
//!
//! R wrapper generation is driven by Rust doc comments (roxygen tags are
//! extracted). The `document` binary collects these wrappers and writes
//! `R/miniextendr_wrappers.R` during package build.
//!
//! ## Quick start
//!
//! ```ignore
//! use miniextendr_api::miniextendr;
//!
//! #[miniextendr]
//! fn add(a: i32, b: i32) -> i32 {
//!     a + b
//! }
//! ```
//!
//! ## Macro expansion pipeline
//!
//! ### Overview
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                         #[miniextendr] on fn                             │
//! │                                                                          │
//! │  1. Parse: syn::ItemFn → MiniextendrFunctionParsed                       │
//! │  2. Analyze return type (Result<T>, Option<T>, raw SEXP, etc.)           │
//! │  3. Generate:                                                            │
//! │     ├── C wrapper: extern "C-unwind" fn C_<name>(call: SEXP, ...) → SEXP │
//! │     ├── R wrapper: const R_WRAPPER_<NAME>: &str = "..."                  │
//! │     └── Registration: const call_method_def_<name>: R_CallMethodDef      │
//! │  4. Original function preserved (with added attributes)                  │
//! └──────────────────────────────────────────────────────────────────────────┘
//!
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                    #[miniextendr(env|r6|s3|s4|s7)] on impl               │
//! │                                                                          │
//! │  1. Parse: syn::ItemImpl → extract methods                               │
//! │  2. For each method:                                                     │
//! │     ├── Generate C wrapper (handles self parameter)                      │
//! │     ├── Generate R method wrapper string                                 │
//! │     └── Generate registration entry                                      │
//! │  3. Generate class definition code per class system:                     │
//! │     ├── env: new.env() + method assignment                               │
//! │     ├── r6: R6Class() definition                                         │
//! │     ├── s3: S3 generics + methods                                        │
//! │     ├── s4: setClass() + setMethod()                                     │
//! │     └── s7: new_class() definition                                       │
//! │  4. Emit const with combined R code                                      │
//! └──────────────────────────────────────────────────────────────────────────┘
//!
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                         #[miniextendr] on trait                          │
//! │                                                                          │
//! │  1. Parse: syn::ItemTrait → extract method signatures                    │
//! │  2. Generate:                                                            │
//! │     ├── Trait tag constant: const TAG_<TRAIT>: mx_tag = ...              │
//! │     ├── Vtable struct: struct __vtable_<Trait> { ... }                   │
//! │     └── CCalls table: static MX_CCALL_<TRAIT>: [...] = ...               │
//! │  3. Original trait preserved                                             │
//! └──────────────────────────────────────────────────────────────────────────┘
//!
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                    #[miniextendr] impl Trait for Type                    │
//! │                                                                          │
//! │  1. Parse: syn::ItemImpl (trait impl)                                    │
//! │  2. Generate:                                                            │
//! │     ├── Vtable instance: static __VTABLE_<TRAIT>_FOR_<TYPE>: ...         │
//! │     ├── Wrapper struct: struct __MxWrapper<Type> { erased, data }        │
//! │     ├── Query function: fn __mx_query_<type>(tag) → vtable ptr           │
//! │     └── Base vtable: static __MX_BASE_VTABLE_<TYPE>: ...                 │
//! │  3. Original impl preserved                                              │
//! └──────────────────────────────────────────────────────────────────────────┘
//!
//! ```
//!
//! ### Key Modules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | `miniextendr_fn` | Function parsing and attribute handling |
//! | `c_wrapper_builder` | C wrapper generation (`extern "C-unwind"`) |
//! | `r_wrapper_builder` | R wrapper code generation |
//! | `rust_conversion_builder` | Rust→SEXP return value conversion |
//! | `miniextendr_impl` | `impl Type` block processing |
//! | `r_class_formatter` | Class system code generation (env/r6/s3/s4/s7) |
//! | `miniextendr_trait` | Trait ABI metadata generation |
//! | `miniextendr_impl_trait` | `impl Trait for Type` vtable generation |
//! | `altrep` / `altrep_derive` | ALTREP struct derivation |
//! | `externalptr_derive` | `#[derive(ExternalPtr)]` |
//! | `roxygen` | Roxygen doc comment handling |
//!
//! ### Generated Symbol Naming
//!
//! For a function `my_func`:
//! - C wrapper: `C_my_func`
//! - R wrapper const: `R_WRAPPER_MY_FUNC`
//! - Registration: `call_method_def_my_func`
//!
//! For a type `MyType` with trait `Counter`:
//! - Vtable: `__VTABLE_COUNTER_FOR_MYTYPE`
//! - Wrapper: `__MxWrapperMyType`
//! - Query: `__mx_query_mytype`
//!
//! ## Return Type Handling
//!
//! The `return_type_analysis` module determines how to convert Rust returns to SEXP:
//!
//! | Rust Type | Strategy | R Result |
//! |-----------|----------|----------|
//! | `T: IntoR` | `.into_sexp()` | Converted value |
//! | `Result<T, E>` | Unwrap or R error | Value or error |
//! | `Option<T>` | `Some` → value, `None` → `NULL` | Value or NULL |
//! | `SEXP` | Pass through | Raw SEXP |
//! | `()` | Invisible NULL | `invisible(NULL)` |
//!
//! Use `#[miniextendr(unwrap_in_r)]` to return `Result<T, E>` to R without unwrapping.
//!
//! ## Thread Strategy
//!
//! By default, `#[miniextendr]` functions run on a **worker thread** for clean panic handling.
//! The macro automatically switches to **main thread** when it detects:
//!
//! - Function takes or returns `SEXP`
//! - Function uses variadic dots (`...`)
//! - `check_interrupt` attribute is set
//!
//! ### When to use `#[miniextendr(unsafe(main_thread))]`
//!
//! Only use this attribute when your function calls R API **directly** using
//! `_unchecked` variants (bypassing `with_r_thread`) and the macro can't auto-detect it.
//! This is rare:
//!
//! ```rust,ignore
//! // NEEDED: Calls _unchecked R API, but signature doesn't show it
//! #[miniextendr(unsafe(main_thread))]
//! fn call_r_api_internally() -> i32 {
//!     // _unchecked variants assume we're on main thread
//!     unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(42); }
//!     42
//! }
//!
//! // NOT NEEDED: Macro auto-detects SEXP return
//! #[miniextendr]
//! fn returns_sexp() -> SEXP { /* ... */ }
//!
//! // NOT NEEDED: ExternalPtr is Send, can cross thread boundary
//! #[miniextendr]
//! fn returns_extptr() -> ExternalPtr<MyType> { /* ... */ }
//! ```
//!
//! **Note**: `ExternalPtr<T>` is `Send` - it can be returned from worker thread functions.
//! All R API operations on ExternalPtr are serialized through `with_r_thread`.
//!
//! ## Class Systems
//!
//! The `r_class_formatter` module generates R code for different class systems:
//!
//! | System | Generated R Code | Self Parameter |
//! |--------|------------------|----------------|
//! | `env` | `new.env()` with methods | `self` environment |
//! | `r6` | `R6Class()` | `self` environment |
//! | `s3` | `structure()` + generics | First argument |
//! | `s4` | `setClass()` + `setMethod()` | First argument |
//! | `s7` | `new_class()` | `self` property |

// miniextendr-macros procedural macros

mod altrep;
mod c_wrapper_builder;
mod list_macro;
mod miniextendr_fn;
mod typed_list;
use crate::miniextendr_fn::{MiniextendrFnAttrs, MiniextendrFunctionParsed};
mod miniextendr_impl;
mod r_wrapper_builder;
/// Builder utilities for formatting R wrapper arguments and calls.
pub(crate) use r_wrapper_builder::RArgumentBuilder;
mod rust_conversion_builder;
/// Helper for generating Rust→R conversion code for return values.
pub(crate) use rust_conversion_builder::RustConversionBuilder;
mod method_return_builder;
/// Helpers for shaping method return handling (R vs Rust wrapper code).
pub(crate) use method_return_builder::{MethodReturnBuilder, ReturnStrategy};
mod altrep_derive;
mod dataframe_derive;
mod lifecycle;
mod list_derive;
mod r_class_formatter;
mod r_preconditions;
mod return_type_analysis;
mod roxygen;

// Trait ABI support modules
mod externalptr_derive;
mod miniextendr_impl_trait;
mod miniextendr_trait;
mod typed_external_macro;

// Factor support
mod factor_derive;
mod match_arg_derive;

// Struct/enum dispatch for #[miniextendr] on structs and enums
mod struct_enum_dispatch;

// vctrs support
#[cfg(feature = "vctrs")]
mod vctrs_derive;

pub(crate) use miniextendr_macros_core::{call_method_def_ident_for, r_wrapper_const_ident_for};

// Feature default mutual exclusivity guards
#[cfg(all(feature = "default-r6", feature = "default-s7"))]
compile_error!("`default-r6` and `default-s7` are mutually exclusive");
// Note: default-main-thread was removed — main thread is now the hardcoded default.
// default-worker still opts into worker thread execution.

// normalize_r_arg_ident is now provided by r_wrapper_builder module

/// Extract `#[cfg(...)]` attributes from a list of attributes.
///
/// These should be propagated to generated items so they are conditionally
/// compiled along with the original function.
fn extract_cfg_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .cloned()
        .collect()
}

/// Format a human-readable source location note from a syntax span.
///
/// Column is reported as 1-based for consistency with editor displays.
pub(crate) fn source_location_doc(span: proc_macro2::Span) -> String {
    let start = span.start();
    format!(
        "Generated from source location line {}, column {}.",
        start.line,
        start.column + 1
    )
}

/// Build a `TokenStream` containing a raw string literal from an R wrapper string.
pub(crate) fn r_wrapper_raw_literal(s: &str) -> proc_macro2::TokenStream {
    use std::str::FromStr;
    let raw = format!("r#\"\n{}\n\"#", s);
    proc_macro2::TokenStream::from_str(&raw).expect("valid raw string literal")
}

/// Returns the first generic type argument from a path segment.
pub(crate) fn first_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    nth_type_argument(seg, 0)
}

/// Returns the second generic type argument from a path segment.
fn second_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    nth_type_argument(seg, 1)
}

/// Extract the inner type `T` from `Vec<T>`. Returns `None` if not a `Vec`.
pub(crate) fn extract_vec_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(tp) = ty {
        let seg = tp.path.segments.last()?;
        if seg.ident == "Vec" {
            return first_type_argument(seg);
        }
    }
    None
}

/// Returns the `n`-th generic type argument from a path segment.
fn nth_type_argument(seg: &syn::PathSegment, n: usize) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        let mut count = 0;
        for arg in ab.args.iter() {
            if let syn::GenericArgument::Type(ty) = arg {
                if count == n {
                    return Some(ty);
                }
                count += 1;
            }
        }
    }
    None
}

#[inline]
/// Returns true if `ty` is syntactically `SEXP`.
fn is_sexp_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| s.ident == "SEXP")
        .unwrap_or(false))
}

/// Known vctrs S3 generics that need `@importFrom vctrs` for roxygen registration.
///
/// This list includes all generics exported by vctrs that users might implement
/// S3 methods for when creating custom vector types.
const VCTRS_GENERICS: &[&str] = &[
    // Core proxy/restore (required for most custom types)
    "vec_proxy",
    "vec_restore",
    // Type coercion (required for vec_c, vec_rbind, etc.)
    "vec_ptype2",
    "vec_cast",
    // Equality/comparison/ordering proxies
    "vec_proxy_equal",
    "vec_proxy_compare",
    "vec_proxy_order",
    // Printing/formatting
    "vec_ptype_abbr",
    "vec_ptype_full",
    "obj_print_data",
    "obj_print_footer",
    "obj_print_header",
    // str() output
    "obj_str_data",
    "obj_str_footer",
    "obj_str_header",
    // Arithmetic (for numeric-like types)
    "vec_arith",
    "vec_math",
    // Other
    "vec_ptype_finalise",
    "vec_cbind_frame_ptype",
    // List-of conversion
    "as_list_of",
];

/// Check if a generic name is a vctrs generic that needs `@importFrom vctrs`.
#[inline]
fn is_vctrs_generic(generic: &str) -> bool {
    VCTRS_GENERICS.contains(&generic)
}

/// Export Rust items to R.
///
/// `#[miniextendr]` can be applied to:
/// - `fn` items (generate C + R wrappers)
/// - `impl` blocks (generate R class methods)
/// - `trait` items (generate trait ABI metadata)
/// - ALTREP wrapper structs (generate `RegisterAltrep` impls)
///
/// # Functions
///
/// ```ignore
/// use miniextendr_api::miniextendr;
///
/// #[miniextendr]
/// fn add(a: i32, b: i32) -> i32 { a + b }
/// ```
///
/// This produces a C wrapper `C_add` and an R wrapper `add()`.
/// Registration is automatic via linkme distributed slices.
///
/// ## `extern "C-unwind"`
///
/// If the function is declared `extern "C-unwind"` and exported with
/// `#[no_mangle]` (2021), `#[unsafe(no_mangle)]` (2024), or `#[export_name = "..."]`,
/// the function itself is the C symbol and the R wrapper is prefixed with
/// `unsafe_` to signal bypassed safety (no worker isolation or conversion).
///
/// ## Variadics (`...`)
///
/// Use `...` as the last argument. The Rust parameter becomes `_dots: &Dots`.
/// Use `name @ ...` to give it a custom name (e.g., `args @ ...` → `args: &Dots`).
///
/// ### Typed Dots Validation
///
/// Use `#[miniextendr(dots = typed_list!(...))]` to automatically validate dots
/// and create a `dots_typed` variable with typed accessors:
///
/// ```ignore
/// #[miniextendr(dots = typed_list!(x => numeric(), y => integer(), z? => character()))]
/// pub fn my_func(...) -> String {
///     let x: f64 = dots_typed.get("x").expect("x");
///     let y: i32 = dots_typed.get("y").expect("y");
///     let z: Option<String> = dots_typed.get_opt("z").expect("z");
///     format!("x={}, y={}", x, y)
/// }
/// ```
///
/// Type specs: `numeric()`, `integer()`, `logical()`, `character()`, `list()`,
/// `raw()`, `complex()`, or `"class_name"` for class inheritance checks.
/// Add `(n)` for exact length: `numeric(4)`. Use `?` suffix for optional fields.
/// Use `@exact;` prefix for strict mode (reject extra fields).
///
/// ## Attributes
///
/// - `#[miniextendr(unsafe(main_thread))]` — run on R's main thread (bypass worker)
/// - `#[miniextendr(invisible)]` / `#[miniextendr(visible)]` — control return visibility
/// - `#[miniextendr(check_interrupt)]` — check for user interrupt after call
/// - `#[miniextendr(coerce)]` — coerce R type before conversion (also usable per-parameter)
/// - `#[miniextendr(strict)]` — reject lossy conversions for i64/u64/isize/usize
/// - `#[miniextendr(unwrap_in_r)]` — return `Result<T, E>` to R without unwrapping
/// - `#[miniextendr(dots = typed_list!(...))]` — validate dots, create `dots_typed`
/// - `#[miniextendr(internal)]` — adds `@keywords internal` to R wrapper
/// - `#[miniextendr(noexport)]` — suppresses `@export` from R wrapper
///
/// # Impl blocks (class systems)
///
/// Apply `#[miniextendr(env|r6|s7|s3|s4)]` to an `impl Type` block.
/// Use `#[miniextendr(label = "...")]` to disambiguate multiple impl blocks
/// on the same type.
/// Registration is automatic.
///
/// ## R6 Active Bindings
///
/// For R6 classes, use `#[miniextendr(r6(active))]` on methods to create
/// active bindings (computed properties accessed without parentheses):
///
/// ```ignore
/// use miniextendr_api::miniextendr;
///
/// pub struct Rectangle {
///     width: f64,
///     height: f64,
/// }
///
/// #[miniextendr(r6)]
/// impl Rectangle {
///     pub fn new(width: f64, height: f64) -> Self {
///         Self { width, height }
///     }
///
///     /// Returns the area (width * height).
///     #[miniextendr(r6(active))]
///     pub fn area(&self) -> f64 {
///         self.width * self.height
///     }
///
///     /// Regular method (requires parentheses).
///     pub fn scale(&mut self, factor: f64) {
///         self.width *= factor;
///         self.height *= factor;
///     }
/// }
/// ```
///
/// In R:
/// ```r
/// r <- Rectangle$new(3, 4)
/// r$area        # 12 (active binding - no parentheses!)
/// r$scale(2)    # Regular method call
/// r$area        # 24
/// ```
///
/// Active bindings must be getter-only methods taking only `&self`.
///
/// ## S7 Properties
///
/// For S7 classes, use `#[miniextendr(s7(getter))]` and `#[miniextendr(s7(setter))]`
/// to create computed properties accessed via `@`:
///
/// ```ignore
/// use miniextendr_api::{miniextendr, ExternalPtr};
///
/// #[derive(ExternalPtr)]
/// pub struct Range {
///     start: f64,
///     end: f64,
/// }
///
/// #[miniextendr(s7)]
/// impl Range {
///     pub fn new(start: f64, end: f64) -> Self {
///         Self { start, end }
///     }
///
///     /// Computed property (read-only): length of the range.
///     #[miniextendr(s7(getter))]
///     pub fn length(&self) -> f64 {
///         self.end - self.start
///     }
///
///     /// Dynamic property getter.
///     #[miniextendr(s7(getter, prop = "midpoint"))]
///     pub fn get_midpoint(&self) -> f64 {
///         (self.start + self.end) / 2.0
///     }
///
///     /// Dynamic property setter.
///     #[miniextendr(s7(setter, prop = "midpoint"))]
///     pub fn set_midpoint(&mut self, value: f64) {
///         let half = self.length() / 2.0;
///         self.start = value - half;
///         self.end = value + half;
///     }
/// }
/// ```
///
/// In R:
/// ```r
/// r <- Range(0, 10)
/// r@length     # 10 (computed, read-only)
/// r@midpoint   # 5 (dynamic property)
/// r@midpoint <- 20  # Adjusts start/end to center at 20
/// ```
///
/// ### Property Attributes
///
/// - `#[miniextendr(s7(getter))]` - Read-only computed property
/// - `#[miniextendr(s7(getter, prop = "name"))]` - Named property getter
/// - `#[miniextendr(s7(setter, prop = "name"))]` - Named property setter
/// - `#[miniextendr(s7(getter, default = "0.0"))]` - Property with default value
/// - `#[miniextendr(s7(getter, required))]` - Required property (error if not provided)
/// - `#[miniextendr(s7(getter, frozen))]` - Property that can only be set once
/// - `#[miniextendr(s7(getter, deprecated = "Use X instead"))]` - Deprecated property
/// - `#[miniextendr(s7(validate))]` - Validator function for property
///
/// ## S7 Generic Dispatch Control
///
/// Control how S7 generics are created:
///
/// - `#[miniextendr(s7(no_dots))]` - Create strict generic without `...`
/// - `#[miniextendr(s7(dispatch = "x,y"))]` - Multi-dispatch on multiple arguments
/// - `#[miniextendr(s7(fallback))]` - Register method for `class_any` (catch-all).
///   The generated R wrapper uses `tryCatch(x@.ptr, error = function(e) x)` to
///   safely extract the self argument, so non-miniextendr objects won't crash with
///   a slot-access error. Instead, incompatible objects produce a Rust type-conversion
///   error when the method tries to interpret the argument as `&Self`.
///
/// ```ignore
/// #[miniextendr(s7)]
/// impl MyClass {
///     /// Strict generic: function(x) instead of function(x, ...)
///     #[miniextendr(s7(no_dots))]
///     pub fn strict_method(&self) -> i32 { 42 }
///
///     /// Fallback method dispatched on class_any.
///     /// Calling this on a non-MyClass object produces a type-conversion error,
///     /// not a slot-access crash.
///     #[miniextendr(s7(fallback))]
///     pub fn describe(&self) -> String { "generic description".into() }
/// }
/// ```
///
/// ## S7 Type Conversion (`convert`)
///
/// Use `convert_from` and `convert_to` to enable S7's `convert()` for type coercion:
///
/// ```ignore
/// use miniextendr_api::{miniextendr, ExternalPtr};
///
/// #[derive(ExternalPtr)]
/// pub struct Celsius { value: f64 }
///
/// #[derive(ExternalPtr)]
/// pub struct Fahrenheit { value: f64 }
///
/// #[miniextendr(s7)]
/// impl Fahrenheit {
///     pub fn new(value: f64) -> Self { Self { value } }
///
///     /// Convert FROM Celsius TO Fahrenheit.
///     /// Usage: S7::convert(celsius_obj, Fahrenheit)
///     #[miniextendr(s7(convert_from = "Celsius"))]
///     pub fn from_celsius(c: ExternalPtr<Celsius>) -> Self {
///         Fahrenheit { value: c.value * 9.0 / 5.0 + 32.0 }
///     }
///
///     /// Convert FROM Fahrenheit TO Celsius.
///     /// Usage: S7::convert(fahrenheit_obj, Celsius)
///     #[miniextendr(s7(convert_to = "Celsius"))]
///     pub fn to_celsius(&self) -> Celsius {
///         Celsius { value: (self.value - 32.0) * 5.0 / 9.0 }
///     }
/// }
/// ```
///
/// In R:
/// ```r
/// c <- Celsius(100)
/// f <- S7::convert(c, Fahrenheit)  # Uses convert_from
/// c2 <- S7::convert(f, Celsius)    # Uses convert_to
/// ```
///
/// **Note:** Classes must be defined before they can be referenced in convert methods.
/// Define the "from" class before the "to" class to avoid forward reference issues.
///
/// # Traits (ABI)
///
/// Apply `#[miniextendr]` to a trait to generate ABI metadata, then use
/// `#[miniextendr] impl Trait for Type`. Registration is automatic.
///
/// # ALTREP
///
/// Apply `#[miniextendr(class = "...", base = "...")]` to a one-field
/// wrapper struct. Registration is automatic.
#[proc_macro_attribute]
pub fn miniextendr(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Try to parse as function first
    if syn::parse::<syn::ItemFn>(item.clone()).is_ok() {
        // Continue with function handling below
    } else if syn::parse::<syn::ItemImpl>(item.clone()).is_ok() {
        // Delegate to impl block parser
        return miniextendr_impl::expand_impl(attr, item);
    } else if syn::parse::<syn::ItemTrait>(item.clone()).is_ok() {
        // Delegate to trait ABI generator
        return miniextendr_trait::expand_trait(attr, item);
    } else {
        // Delegate to struct/enum dispatch (handles ALTREP, ExternalPtr, list, dataframe, factor, match_arg)
        return struct_enum_dispatch::expand_struct_or_enum(attr, item);
    }

    let MiniextendrFnAttrs {
        force_main_thread,
        force_worker,
        force_invisible,
        check_interrupt,
        coerce_all,
        rng,
        unwrap_in_r,
        return_pref,
        s3_generic,
        s3_class,
        dots_spec,
        dots_span,
        lifecycle,
        strict,
        internal,
        noexport,
        export,
        doc,
        error_in_r,
        c_symbol,
        r_name: fn_r_name,
        r_entry,
        r_post_checks,
        r_on_exit,
    } = syn::parse_macro_input!(attr as MiniextendrFnAttrs);

    let mut parsed = syn::parse_macro_input!(item as MiniextendrFunctionParsed);

    // Reject async functions
    if let Some(asyncness) = &parsed.item().sig.asyncness {
        return syn::Error::new_spanned(
            asyncness,
            "async functions are not supported by #[miniextendr]; \
             R's C API is synchronous and incompatible with async executors",
        )
        .into_compile_error()
        .into();
    }

    // Validate: reject generic functions (extern "C-unwind" + #[no_mangle] incompatible with generics)
    if !parsed.item().sig.generics.params.is_empty() {
        let err = syn::Error::new_spanned(
            &parsed.item().sig.generics,
            "#[miniextendr] functions cannot have generic type parameters. \
             Generic functions are incompatible with `extern \"C-unwind\"` and `#[no_mangle]` \
             required for R FFI. Consider using trait objects or monomorphization instead.",
        );
        return err.into_compile_error().into();
    }

    parsed.add_track_caller_if_needed();
    parsed.add_inline_never_if_needed();

    // Extract commonly used values
    let uses_internal_c_wrapper = parsed.uses_internal_c_wrapper();
    let call_method_def = parsed.call_method_def_ident();
    let c_ident = if let Some(ref sym) = c_symbol {
        syn::Ident::new(sym, parsed.c_wrapper_ident().span())
    } else {
        parsed.c_wrapper_ident()
    };
    let r_wrapper_generator = parsed.r_wrapper_const_ident();

    use syn::spanned::Spanned;

    // Extract references to parsed components
    let rust_ident = parsed.ident();
    let inputs = parsed.inputs();
    let output = parsed.output();
    let abi = parsed.abi();
    let attrs = parsed.attrs();
    let vis = parsed.vis();
    let generics = parsed.generics();
    let has_dots = parsed.has_dots();
    let named_dots = parsed.named_dots().cloned();

    // Check for @title/@description conflicts with implicit values (doc-lint feature)
    // Skip when `doc` attribute overrides the roxygen — implicit docs are irrelevant then.
    let doc_lint_warnings = if doc.is_some() {
        proc_macro2::TokenStream::new()
    } else {
        crate::roxygen::doc_conflict_warnings(attrs, rust_ident.span())
    };

    let rust_arg_count = inputs.len();
    let registered_arg_count = if uses_internal_c_wrapper {
        rust_arg_count + 1
    } else {
        rust_arg_count
    };
    let num_args = syn::LitInt::new(&registered_arg_count.to_string(), inputs.span());

    // name of the C-wrapper
    let c_ident_name = syn::LitCStr::new(
        std::ffi::CString::new(c_ident.to_string())
            .expect("couldn't create a C-string for the C wrapper name")
            .as_c_str(),
        rust_ident.span(),
    );
    // registration of the C-wrapper
    // these are needed to transmute fn-item to fn-pointer correctly.
    let mut func_ptr_def: Vec<syn::Type> = Vec::new();
    if uses_internal_c_wrapper {
        func_ptr_def.push(syn::parse_quote!(::miniextendr_api::ffi::SEXP));
    }
    func_ptr_def.extend((0..inputs.len()).map(|_| syn::parse_quote!(::miniextendr_api::ffi::SEXP)));

    // calling the rust function with
    let rust_inputs: Vec<syn::Ident> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(p) = pt.pat.as_ref()
            {
                return Some(p.ident.clone());
            }
            None
        })
        .collect();
    // dbg!(&rust_inputs);

    // Hygiene: Use call_site() for internal variable names that should be visible to
    // procedural macro machinery but not create unhygienic references to user code.
    // call_site() = span of the macro invocation (#[miniextendr])
    let call_param_ident = syn::Ident::new("__miniextendr_call", proc_macro2::Span::call_site());
    let mut c_wrapper_inputs: Vec<syn::FnArg> = Vec::new();
    if uses_internal_c_wrapper {
        c_wrapper_inputs.push(syn::parse_quote!(#call_param_ident: ::miniextendr_api::ffi::SEXP));
    }
    for arg in inputs.iter() {
        match arg {
            syn::FnArg::Receiver(receiver) => {
                let err = syn::Error::new_spanned(
                    receiver,
                    "self parameter not allowed in standalone functions; \
                     use #[miniextendr(env|r6|s3|s4|s7)] on impl blocks instead",
                );
                return err.into_compile_error().into();
            }
            syn::FnArg::Typed(pt) => {
                let pat = &pt.pat;
                match pat.as_ref() {
                    syn::Pat::Ident(pat_ident) => {
                        let mut pat_ident = pat_ident.clone();
                        pat_ident.mutability = None;
                        pat_ident.by_ref = None;
                        let ident = pat_ident;
                        c_wrapper_inputs
                            .push(syn::parse_quote!(#ident: ::miniextendr_api::ffi::SEXP));
                    }
                    syn::Pat::Wild(_) => {
                        unreachable!(
                            "wildcard patterns should have been transformed to synthetic identifiers"
                        )
                    }
                    _ => {
                        let err = syn::Error::new_spanned(
                            pat,
                            "unsupported pattern in function argument; only simple identifiers are supported",
                        );
                        return err.into_compile_error().into();
                    }
                }
            }
        }
    }
    // dbg!(&wrapper_inputs);
    let mut pre_call_statements: Vec<proc_macro2::TokenStream> = Vec::new();
    if check_interrupt {
        pre_call_statements.push(quote::quote! {
            unsafe { ::miniextendr_api::ffi::R_CheckUserInterrupt(); }
        });
    }

    // Validate dots_spec usage (actual injection happens later in the function body)
    if dots_spec.is_some() && !has_dots {
        let err = syn::Error::new(
            dots_span.unwrap_or_else(proc_macro2::Span::call_site),
            "#[miniextendr(dots = typed_list!(...))] requires a `...` parameter in the function signature",
        );
        return err.into_compile_error().into();
    }

    // Build conversion builder with coercion settings
    let mut conversion_builder = RustConversionBuilder::new();
    if strict {
        conversion_builder = conversion_builder.with_strict();
    }
    if error_in_r {
        conversion_builder = conversion_builder.with_error_in_r();
    }
    if coerce_all {
        conversion_builder = conversion_builder.with_coerce_all();
    }
    for input in inputs.iter() {
        if let syn::FnArg::Typed(pt) = input
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
        {
            let param_name = pat_ident.ident.to_string();
            if parsed.has_coerce_attr(&param_name) {
                conversion_builder = conversion_builder.with_coerce_param(param_name.clone());
            }
            if parsed.has_match_arg_attr(&param_name) && parsed.has_several_ok(&param_name) {
                conversion_builder = conversion_builder.with_match_arg_several_ok(param_name);
            }
        }
    }

    // Generate conversion statements (split for worker thread compatibility)
    // pre_closure: runs on main thread, produces owned values to move
    // in_closure: runs inside worker closure, creates borrows from moved storage
    let (pre_closure_stmts, in_closure_stmts): (Vec<_>, Vec<_>) = inputs
        .iter()
        .zip(rust_inputs.iter())
        .filter_map(|(arg, sexp_ident)| {
            if let syn::FnArg::Typed(pat_type) = arg {
                Some(conversion_builder.build_conversion_split(pat_type, sexp_ident))
            } else {
                None
            }
        })
        .fold(
            (Vec::new(), Vec::new()),
            |(mut pre, mut in_c), (owned, borrowed)| {
                pre.extend(owned);
                in_c.extend(borrowed);
                (pre, in_c)
            },
        );
    // For main thread paths (no split needed), flatten both into closure_statements
    let closure_statements: Vec<_> = pre_closure_stmts
        .iter()
        .chain(in_closure_stmts.iter())
        .cloned()
        .collect();

    // Hygiene: Use mixed_site() for internal variables that need to reference both
    // macro-generated items (quote!) and user-provided items from the original function.
    // mixed_site() = allows capturing both hygiene contexts for cross-context references.
    let rust_result_ident =
        syn::Ident::new("__miniextendr_rust_result", proc_macro2::Span::mixed_site());

    // Analyze return type to determine:
    // - Whether it returns SEXP (affects thread strategy)
    // - Whether result should be invisible
    // - How to convert Rust → SEXP
    // - Post-call processing (unwrap Option/Result)
    let return_analysis = return_type_analysis::analyze_return_type(
        output,
        &rust_result_ident,
        rust_ident,
        return_pref,
        unwrap_in_r,
        strict,
        error_in_r,
    );

    let returns_sexp = return_analysis.returns_sexp;
    let is_invisible_return_type = return_analysis.is_invisible;
    let return_expression = return_analysis.return_expression;
    let post_call_statements = return_analysis.post_call_statements;

    // Apply explicit visibility override from #[miniextendr(invisible)] or #[miniextendr(visible)]
    let is_invisible_return_type = force_invisible.unwrap_or(is_invisible_return_type);

    // Check if any input parameter is SEXP (not Send, must stay on main thread)
    let has_sexp_inputs = inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            is_sexp_type(pat_type.ty.as_ref())
        } else {
            false
        }
    });

    // ═══════════════════════════════════════════════════════════════════════════
    // Thread Strategy Selection
    // ═══════════════════════════════════════════════════════════════════════════
    //
    // miniextendr supports two execution strategies:
    //
    // 1. **Main Thread Strategy** (with_r_unwind_protect) — DEFAULT
    //    - All code runs on R's main thread
    //    - Required when SEXP types are involved (not Send)
    //    - Required for R API calls (Rf_*, R_*)
    //    - Panic handling via R_UnwindProtect (Rust destructors run correctly)
    //    - Combined with error_in_r: errors returned as tagged SEXP values,
    //      R wrapper raises structured condition objects
    //    - Simpler execution model, better R integration
    //
    // 2. **Worker Thread Strategy** (run_on_worker + catch_unwind) — OPT-IN
    //    - Argument conversion on main thread (SEXP → Rust types)
    //    - Function execution on dedicated worker thread (clean panic isolation)
    //    - Result conversion on main thread (Rust types → SEXP)
    //    - Panic handling via catch_unwind (prevents unwinding across FFI boundary)
    //    - Opt in with #[miniextendr(worker)]
    //    - ExternalPtr<T> is Send: can be returned from worker thread functions
    //    - R API calls from worker use with_r_thread (serialized to main thread)
    //
    // Default: Main thread (safer with error_in_r, simpler execution model)
    // Override: Use worker thread with #[miniextendr(worker)]
    //
    // Thread strategy:
    // - Main thread is always used unless force_worker is set
    // - force_worker cannot override hard requirements for main thread
    // - Hard requirements: returns_sexp, has_sexp_inputs, has_dots, check_interrupt
    let requires_main_thread = returns_sexp || has_sexp_inputs || has_dots || check_interrupt;
    let use_main_thread = !force_worker || requires_main_thread;

    // Suppress unused variable warnings — force_main_thread is now the default,
    // and force_worker is consumed in use_main_thread above.
    let _ = (force_main_thread, force_worker);
    // RNG state management tokens
    let (rng_get, rng_put) = if rng {
        (
            quote::quote! { unsafe { ::miniextendr_api::ffi::GetRNGstate(); } },
            quote::quote! { unsafe { ::miniextendr_api::ffi::PutRNGstate(); } },
        )
    } else {
        (
            proc_macro2::TokenStream::new(),
            proc_macro2::TokenStream::new(),
        )
    };
    let source_loc_doc = source_location_doc(rust_ident.span());

    // Select unwind protection function: error_in_r returns tagged error values on panic,
    // standard mode raises R errors via Rf_errorcall.
    let unwind_protect_fn = if error_in_r {
        quote::quote! { ::miniextendr_api::unwind_protect::with_r_unwind_protect_error_in_r }
    } else {
        quote::quote! { ::miniextendr_api::unwind_protect::with_r_unwind_protect }
    };

    let c_wrapper = if abi.is_some() {
        proc_macro2::TokenStream::new()
    } else if use_main_thread {
        // SEXP-returning or Dots-taking functions: use with_r_unwind_protect on main thread
        let c_wrapper_doc = format!(
            "C wrapper for [`{}`] (main thread). See [`{}`] for R wrapper.",
            rust_ident, r_wrapper_generator
        );
        if rng {
            // RNG variant: wrap in catch_unwind so we can call PutRNGstate before error handling.
            // rng always implies error_in_r (validated at parse time), so we always return
            // a tagged error value on panic instead of resume_unwind.
            let rng_panic_handler = quote::quote! {
                ::miniextendr_api::error_value::make_rust_error_value(
                    &::miniextendr_api::unwind_protect::panic_payload_to_string(&*payload),
                    "panic",
                    Some(#call_param_ident),
                )
            };
            quote::quote! {
                #[doc = #c_wrapper_doc]
                #[doc = concat!("Wraps Rust function `", stringify!(#rust_ident), "`.")]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                #vis extern "C-unwind" fn #c_ident #generics(#(#c_wrapper_inputs),*) -> ::miniextendr_api::ffi::SEXP {
                    #rng_get
                    let __result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                        #(#pre_call_statements)*
                        #unwind_protect_fn(
                            || {
                                #(#closure_statements)*
                                let #rust_result_ident = #rust_ident(#(#rust_inputs),*);
                                #(#post_call_statements)*
                                #return_expression
                            },
                            Some(#call_param_ident),
                        )
                    }));
                    #rng_put
                    match __result {
                        Ok(sexp) => sexp,
                        Err(payload) => { #rng_panic_handler },
                    }
                }
            }
        } else {
            // Non-RNG variant: direct call to with_r_unwind_protect
            quote::quote! {
                #[doc = #c_wrapper_doc]
                #[doc = concat!("Wraps Rust function `", stringify!(#rust_ident), "`.")]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                #vis extern "C-unwind" fn #c_ident #generics(#(#c_wrapper_inputs),*) -> ::miniextendr_api::ffi::SEXP {
                    #(#pre_call_statements)*

                    #unwind_protect_fn(
                        || {
                            #(#closure_statements)*
                            let #rust_result_ident = #rust_ident(#(#rust_inputs),*);
                            #(#post_call_statements)*
                            #return_expression
                        },
                        Some(#call_param_ident),
                    )
                }
            }
        }
    } else {
        // Pure Rust functions: use worker thread strategy.
        // Emit a compile-time check that the `worker-thread` feature is enabled.
        // Without it, `run_on_worker` is a stub that runs inline (silent degradation).
        // Check both `worker-thread` (direct) and `default-worker` (implies worker-thread
        // via miniextendr-api, but the user crate may only have the latter in its features).
        let worker_feature_check = {
            let fn_name = rust_ident.to_string();
            let msg = format!(
                "`#[miniextendr(worker)]` on `{fn_name}` requires the `worker-thread` cargo feature. \
                 Add `worker-thread = [\"miniextendr-api/worker-thread\"]` to your [features] in Cargo.toml."
            );
            quote::quote! {
                #[cfg(not(any(feature = "worker-thread", feature = "default-worker")))]
                compile_error!(#msg);
            }
        };
        let c_wrapper_doc = format!(
            "C wrapper for [`{}`] (worker thread). See [`{}`] for R wrapper.",
            rust_ident, r_wrapper_generator
        );
        let worker_panic_handler = if error_in_r {
            quote::quote! {
                ::miniextendr_api::error_value::make_rust_error_value(
                    &::miniextendr_api::unwind_protect::panic_payload_to_string(&*payload),
                    "panic",
                    Some(#call_param_ident),
                )
            }
        } else {
            quote::quote! {
                ::miniextendr_api::worker::panic_message_to_r_error(
                    ::miniextendr_api::unwind_protect::panic_payload_to_string(&*payload),
                    None,
                )
            }
        };
        if error_in_r {
            // error_in_r: run_on_worker returns Result; Err → tagged error value
            quote::quote! {
                #worker_feature_check

                #[doc = #c_wrapper_doc]
                #[doc = concat!("Wraps Rust function `", stringify!(#rust_ident), "`.")]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                #vis extern "C-unwind" fn #c_ident #generics(#(#c_wrapper_inputs),*) -> ::miniextendr_api::ffi::SEXP {
                    #rng_get
                    let __miniextendr_panic_result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(move || {
                        #(#pre_call_statements)*
                        #(#pre_closure_stmts)*

                        match ::miniextendr_api::worker::run_on_worker(move || {
                            #(#in_closure_stmts)*
                            let #rust_result_ident = #rust_ident(#(#rust_inputs),*);
                            #(#post_call_statements)*
                            #rust_result_ident
                        }) {
                            Ok(#rust_result_ident) => {
                                #unwind_protect_fn(
                                    move || #return_expression,
                                    None,
                                )
                            }
                            Err(__panic_msg) => {
                                ::miniextendr_api::error_value::make_rust_error_value(
                                    &__panic_msg, "panic", Some(#call_param_ident),
                                )
                            }
                        }
                    }));
                    #rng_put
                    match __miniextendr_panic_result {
                        Ok(sexp) => sexp,
                        Err(payload) => { #worker_panic_handler },
                    }
                }
            }
        } else {
            // run_on_worker returns Result; Err → R error via Rf_error
            quote::quote! {
                #worker_feature_check

                #[doc = #c_wrapper_doc]
                #[doc = concat!("Wraps Rust function `", stringify!(#rust_ident), "`.")]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                #vis extern "C-unwind" fn #c_ident #generics(#(#c_wrapper_inputs),*) -> ::miniextendr_api::ffi::SEXP {
                    #rng_get
                    let __miniextendr_panic_result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(move || {
                        #(#pre_call_statements)*
                        #(#pre_closure_stmts)*

                        match ::miniextendr_api::worker::run_on_worker(move || {
                            #(#in_closure_stmts)*
                            let #rust_result_ident = #rust_ident(#(#rust_inputs),*);
                            #(#post_call_statements)*
                            #rust_result_ident
                        }) {
                            Ok(#rust_result_ident) => {
                                #unwind_protect_fn(
                                    move || #return_expression,
                                    None,
                                )
                            }
                            Err(__panic_msg) => {
                                ::miniextendr_api::worker::panic_message_to_r_error(__panic_msg, None)
                            }
                        }
                    }));
                    #rng_put
                    match __miniextendr_panic_result {
                        Ok(sexp) => sexp,
                        Err(payload) => { #worker_panic_handler },
                    }
                }
            }
        }
    };

    // check the validity of the provided C-function!
    if abi.is_some() {
        // check that #[no_mangle] or #[unsafe(no_mangle)] or #[export_name] is present!
        let has_no_mangle = attrs.iter().any(|attr| {
            attr.path().is_ident("no_mangle")
                || attr
                    .parse_nested_meta(|meta| {
                        if meta.path.is_ident("no_mangle") {
                            Err(meta.error("found #[no_mangle]"))
                        } else {
                            Ok(())
                        }
                    })
                    .is_err()
        });

        let has_export_name = attrs.iter().any(|attr| attr.path().is_ident("export_name"));

        if !has_no_mangle && !has_export_name {
            return syn::Error::new(
                attrs
                    .first()
                    .map(|attr| attr.span())
                    .unwrap_or_else(|| abi.span()),
                "extern \"C-unwind\" functions need a visible C symbol for R's .Call interface. \
                 Add one of:\n  \
                 - `#[unsafe(no_mangle)]` (Rust 2024 edition)\n  \
                 - `#[no_mangle]` (Rust 2021 edition)\n  \
                 - `#[export_name = \"my_symbol\"]` (custom symbol name)",
            )
            .into_compile_error()
            .into();
        }

        // Validate return type is SEXP for extern "C-unwind" functions
        match output {
            non_return_type @ syn::ReturnType::Default => {
                return syn::Error::new(
                    non_return_type.span(),
                    "extern \"C-unwind\" functions used with #[miniextendr] must return SEXP. \
                     Add `-> miniextendr_api::ffi::SEXP` as the return type. \
                     If you want automatic type conversion, remove `extern \"C-unwind\"` and let \
                     the macro generate the C wrapper.",
                )
                .into_compile_error()
                .into();
            }
            syn::ReturnType::Type(_rarrow, output_type) => match output_type.as_ref() {
                syn::Type::Path(type_path) => {
                    if let Some(path_to_sexp) = type_path.path.segments.last().map(|x| &x.ident)
                        && path_to_sexp != "SEXP"
                    {
                        return syn::Error::new(
                            path_to_sexp.span(),
                            format!(
                                "extern \"C-unwind\" functions must return SEXP, found `{}`. \
                                 R's .Call interface expects SEXP return values. \
                                 Change the return type to `miniextendr_api::ffi::SEXP`, or remove \
                                 `extern \"C-unwind\"` to let the macro handle type conversion.",
                                path_to_sexp,
                            ),
                        )
                        .into_compile_error()
                        .into();
                    }
                }
                _ => {
                    return syn::Error::new(
                        output_type.span(),
                        "extern \"C-unwind\" functions must return SEXP. \
                         R's .Call interface expects SEXP return values. \
                         Change the return type to `miniextendr_api::ffi::SEXP`, or remove \
                         `extern \"C-unwind\"` to let the macro handle type conversion.",
                    )
                    .into_compile_error()
                    .into();
                }
            },
        }

        // Validate all input types are SEXP for extern "C-unwind" functions.
        // R's .Call interface passes all arguments as SEXP, so accepting other types is UB.
        // Also reject variadic (...) signatures which are not valid for .Call.
        for input in inputs.iter() {
            match input {
                syn::FnArg::Receiver(recv) => {
                    return syn::Error::new_spanned(
                        recv,
                        "extern \"C-unwind\" functions cannot have a `self` parameter. \
                         R's .Call interface only accepts SEXP arguments. \
                         Use `#[miniextendr(env|r6|s3|s4|s7)]` on an impl block for methods.",
                    )
                    .into_compile_error()
                    .into();
                }
                syn::FnArg::Typed(pat_type) => {
                    // Check if this is a variadic pattern (...)
                    if let syn::Pat::Rest(_) = pat_type.pat.as_ref() {
                        return syn::Error::new_spanned(
                            pat_type,
                            "extern functions cannot use variadic (...) - .Call passes fixed arguments",
                        )
                        .into_compile_error()
                        .into();
                    }

                    // Validate type is SEXP
                    let is_sexp = match pat_type.ty.as_ref() {
                        syn::Type::Path(type_path) => type_path
                            .path
                            .segments
                            .last()
                            .is_some_and(|seg| seg.ident == "SEXP"),
                        _ => false,
                    };

                    if !is_sexp {
                        // Check if the type looks like &Dots or Dots
                        let is_dots_type =
                            if let syn::Type::Reference(type_ref) = pat_type.ty.as_ref() {
                                if let syn::Type::Path(inner) = type_ref.elem.as_ref() {
                                    inner
                                        .path
                                        .segments
                                        .last()
                                        .is_some_and(|seg| seg.ident == "Dots")
                                } else {
                                    false
                                }
                            } else if let syn::Type::Path(type_path) = pat_type.ty.as_ref() {
                                type_path
                                    .path
                                    .segments
                                    .last()
                                    .is_some_and(|seg| seg.ident == "Dots")
                            } else {
                                false
                            };

                        let msg = if is_dots_type {
                            "extern functions cannot use Dots; use `...` syntax in non-extern #[miniextendr] functions instead"
                        } else {
                            "extern function parameters must be SEXP - .Call passes all arguments as SEXP"
                        };
                        return syn::Error::new_spanned(&pat_type.ty, msg)
                            .into_compile_error()
                            .into();
                    }
                }
            }
        }
    }

    // region: R wrappers generation in `fn`
    // Build R formal parameters and call arguments using shared builder
    let mut arg_builder = RArgumentBuilder::new(inputs);
    if has_dots {
        arg_builder = arg_builder.with_dots(named_dots.clone().map(|id| id.to_string()));
    }
    // Add user-specified parameter defaults (Missing<T> defaults handled via body prelude)
    let mut merged_defaults = parsed.param_defaults().clone();
    // Add default for match_arg params that don't already have an explicit default.
    // Use a placeholder that gets replaced at write time with the actual choices
    // from the enum's MatchArg::CHOICES (resolved when the cdylib is loaded).
    let mut match_arg_placeholders: Vec<(String, String)> = Vec::new(); // (placeholder, rust_param)
    // (r_name, rust_param) pairs — used later to build @param doc placeholders
    let mut match_arg_r_names: Vec<(String, String)> = Vec::new();
    for match_arg_param in parsed.match_arg_params() {
        let r_name = r_wrapper_builder::normalize_r_arg_ident(&syn::Ident::new(
            match_arg_param,
            proc_macro2::Span::call_site(),
        ))
        .to_string();
        match_arg_r_names.push((r_name.clone(), match_arg_param.clone()));
        if !merged_defaults.contains_key(&r_name) {
            let placeholder = format!(
                ".__MX_MATCH_ARG_CHOICES_{}_{}__",
                c_ident.to_string().trim_start_matches("C_"),
                r_name
            );
            merged_defaults.insert(r_name.clone(), placeholder.clone());
            match_arg_placeholders.push((placeholder, match_arg_param.clone()));
        }
    }
    // Add c("a", "b", "c") default for choices params (idiomatic R match.arg pattern)
    for (param_name, choices) in parsed.choices_params() {
        let r_name = r_wrapper_builder::normalize_r_arg_ident(&syn::Ident::new(
            param_name,
            proc_macro2::Span::call_site(),
        ))
        .to_string();
        let quoted: Vec<String> = choices.iter().map(|c| format!("\"{}\"", c)).collect();
        merged_defaults
            .entry(r_name)
            .or_insert_with(|| format!("c({})", quoted.join(", ")));
    }
    arg_builder = arg_builder.with_defaults(merged_defaults);

    let r_formals = arg_builder.build_formals();
    let mut r_call_args_strs = arg_builder.build_call_args_vec();

    // Prepend .call parameter if using internal C wrapper
    if uses_internal_c_wrapper {
        r_call_args_strs.insert(0, ".call = match.call()".to_string());
    }

    // Build the R body string consistently
    let c_ident_str = c_ident.to_string();
    let call_args_joined = r_call_args_strs.join(", ");
    let call_expr = if r_call_args_strs.is_empty() {
        format!(".Call({})", c_ident_str)
    } else {
        format!(".Call({}, {})", c_ident_str, call_args_joined)
    };
    let r_wrapper_return_str = if error_in_r {
        // error_in_r mode: capture result, check for error value, raise R condition
        let final_return = if is_invisible_return_type {
            "invisible(.val)"
        } else {
            ".val"
        };
        crate::method_return_builder::error_in_r_standalone_body(&call_expr, final_return)
    } else if !is_invisible_return_type {
        call_expr
    } else {
        format!("invisible({})", call_expr)
    };
    // Determine R function name and S3-specific comments
    let is_s3_method = s3_generic.is_some() || s3_class.is_some();
    let r_wrapper_ident_str: String;
    let s3_method_comment: String;

    if is_s3_method {
        // For S3 methods, function name is generic.class
        // generic defaults to Rust function name if not specified
        let generic = s3_generic.clone().unwrap_or_else(|| rust_ident.to_string());
        // s3_class is guaranteed to be Some here because MiniextendrFnAttrs::parse
        // validates that s3(...) always has class specified
        let class = s3_class.as_ref().expect("s3_class validated at parse time");
        r_wrapper_ident_str = format!("{}.{}", generic, class);
        // Add @importFrom for vctrs generics so roxygen registers the dependency
        let import_comment = if is_vctrs_generic(&generic) {
            format!("#' @importFrom vctrs {}\n", generic)
        } else {
            String::new()
        };
        s3_method_comment = format!("{}#' @method {} {}\n", import_comment, generic, class);
    } else if let Some(ref custom_name) = fn_r_name {
        r_wrapper_ident_str = custom_name.clone();
        s3_method_comment = String::new();
    } else if abi.is_some() {
        r_wrapper_ident_str = format!("unsafe_{}", rust_ident);
        s3_method_comment = String::new();
    } else {
        r_wrapper_ident_str = rust_ident.to_string();
        s3_method_comment = String::new();
    };

    // Stable, consistent R formatting style: brace on same line, body indented, closing brace on its own line
    // r_formals is already a joined string from build_formals()
    let formals_joined = r_formals;
    let mut roxygen_tags = if let Some(ref doc_text) = doc {
        // Custom doc override: each line becomes a separate roxygen tag entry
        doc_text.lines().map(|l| l.to_string()).collect()
    } else {
        crate::roxygen::roxygen_tags_from_attrs(attrs)
    };

    // Determine lifecycle: explicit attr > #[deprecated] extraction
    let lifecycle_spec = lifecycle.or_else(|| {
        attrs
            .iter()
            .find_map(crate::lifecycle::parse_rust_deprecated)
    });

    // Inject lifecycle badge into roxygen tags if present
    if let Some(ref spec) = lifecycle_spec {
        crate::lifecycle::inject_lifecycle_badge(&mut roxygen_tags, spec);
    }

    // Auto-generate @param tags for choices params (unless user already wrote one)
    for arg in inputs.iter() {
        if let syn::FnArg::Typed(pt) = arg
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
        {
            let rust_name = pat_ident.ident.to_string();
            if let Some(choices) = parsed.choices_for_param(&rust_name) {
                let r_name = r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();
                // Only inject if user didn't already write @param for this param
                let has_user_param = roxygen_tags
                    .iter()
                    .any(|t| t.trim_start().starts_with(&format!("@param {}", r_name)));
                if !has_user_param {
                    let quoted: Vec<String> =
                        choices.iter().map(|c| format!("\"{}\"", c)).collect();
                    let prefix = if parsed.has_several_ok(&rust_name) {
                        "One or more of"
                    } else {
                        "One of"
                    };
                    roxygen_tags.push(format!(
                        "@param {} {} {}.",
                        r_name,
                        prefix,
                        quoted.join(", ")
                    ));
                }
            }
        }
    }

    // Auto-generate @param tags for match_arg params (unless user already wrote one).
    // Emits a placeholder that gets replaced at write time with the actual choices,
    // e.g. `One of "Fast", "Safe", "Debug".`
    // Collect (doc_placeholder, rust_param) for MX_MATCH_ARG_PARAM_DOCS entries.
    let mut match_arg_param_doc_placeholders: Vec<(String, String)> = Vec::new();
    for (r_name, rust_param) in &match_arg_r_names {
        let has_user_param = roxygen_tags
            .iter()
            .any(|t| t.trim_start().starts_with(&format!("@param {r_name}")));
        if !has_user_param {
            let doc_placeholder = format!(
                ".__MX_MATCH_ARG_PARAM_DOC_{}_{}__",
                c_ident.to_string().trim_start_matches("C_"),
                r_name
            );
            roxygen_tags.push(format!("@param {r_name} {doc_placeholder}"));
            match_arg_param_doc_placeholders.push((doc_placeholder, rust_param.clone()));
        }
    }

    // Auto-generate @param for any function parameter that doesn't have one yet.
    // This prevents R CMD check warnings about undocumented arguments.
    // Skip dots params (they become `...` in R formals, not a named param).
    for arg in inputs.iter() {
        if let syn::FnArg::Typed(pt) = arg
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
        {
            // Skip dots parameters — they map to `...` in R, which can't have @param
            if parsed.is_dots_param(&pat_ident.ident) {
                continue;
            }
            let r_name = r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();
            let has_param = roxygen_tags
                .iter()
                .any(|t| t.trim_start().starts_with(&format!("@param {r_name}")));
            if !has_param {
                roxygen_tags.push(format!("@param {r_name} (no documentation available)"));
            }
        }
    }

    // Ensure a @title exists when we have auto-generated tags (e.g., @param from choices)
    // but the auto-title logic in roxygen_tags_from_attrs didn't fire (because has_any_tags
    // was false at that point — choices @param tags are added after extraction).
    // Prefer the implicit title from doc comments; fall back to the function name.
    if !roxygen_tags.is_empty() && !crate::roxygen::has_roxygen_tag(&roxygen_tags, "title") {
        let title = crate::roxygen::implicit_title_from_attrs(attrs)
            .unwrap_or_else(|| rust_ident.to_string().replace('_', " "));
        roxygen_tags.insert(0, format!("@title {}", title));
    }

    let roxygen_tags_str = crate::roxygen::format_roxygen_tags(&roxygen_tags);
    let has_export_tag = crate::roxygen::has_roxygen_tag(&roxygen_tags, "export");
    let has_no_rd_tag = crate::roxygen::has_roxygen_tag(&roxygen_tags, "noRd");
    let has_internal_tag = crate::roxygen::has_roxygen_tag(&roxygen_tags, "keywords internal");
    // Add roxygen comments: @source for traceability, @export if public
    let source_comment = format!(
        "#' @source Generated by miniextendr from Rust fn `{}`\n",
        rust_ident
    );
    // Inject @keywords internal if #[miniextendr(internal)] and not already present
    let internal_comment = if internal && !has_internal_tag {
        "#' @keywords internal\n"
    } else {
        ""
    };
    // S3 methods need both @method (for registration) AND @export (for NAMESPACE)
    // Don't auto-export functions marked with @noRd, @keywords internal, or attr flags
    // #[miniextendr(export)] forces @export even on non-pub functions
    let export_comment = if (matches!(vis, syn::Visibility::Public(_)) || export)
        && !has_export_tag
        && !has_no_rd_tag
        && !has_internal_tag
        && !internal
        && !noexport
    {
        "#' @export\n".to_string()
    } else {
        String::new()
    };
    // Generate match.arg prelude for parameters with #[miniextendr(match_arg)]
    // Collect (r_param_name, rust_name, rust_type) for each match_arg param
    let match_arg_param_info: Vec<(String, String, &syn::Type)> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let rust_name = pat_ident.ident.to_string();
                if parsed.has_match_arg_attr(&rust_name) {
                    let r_name =
                        r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();
                    return Some((r_name, rust_name, pt.ty.as_ref()));
                }
            }
            None
        })
        .collect();

    let match_arg_prelude = if match_arg_param_info.is_empty() {
        String::new()
    } else {
        let mut lines = Vec::new();
        for (r_param, rust_name, _) in &match_arg_param_info {
            // choices helper call
            let choices_c_name = format!(
                "C_{}__match_arg_choices__{}",
                c_ident.to_string().trim_start_matches("C_"),
                r_param
            );
            lines.push(format!(
                ".__mx_choices_{param} <- .Call({choices_c}, .call = match.call())",
                param = r_param,
                choices_c = choices_c_name,
            ));
            // factor → character normalization
            lines.push(format!(
                "{param} <- if (is.factor({param})) as.character({param}) else {param}",
                param = r_param,
            ));
            // match.arg validation — with several.ok when requested
            if parsed.has_several_ok(rust_name) {
                lines.push(format!(
                    "{param} <- base::match.arg({param}, .__mx_choices_{param}, several.ok = TRUE)",
                    param = r_param,
                ));
            } else {
                lines.push(format!(
                    "{param} <- base::match.arg({param}, .__mx_choices_{param})",
                    param = r_param,
                ));
            }
        }
        lines.join("\n  ")
    };

    // Generate idiomatic match.arg prelude for choices params
    // These use the simpler pattern: `param <- match.arg(param)` (no C helper call needed)
    // With `several_ok`, emit `match.arg(param, several.ok = TRUE)` for multi-value selection
    let choices_prelude = {
        let mut lines = Vec::new();
        for arg in inputs.iter() {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let rust_name = pat_ident.ident.to_string();
                if parsed.choices_for_param(&rust_name).is_some() {
                    let r_name =
                        r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();
                    if parsed.has_several_ok(&rust_name) {
                        lines.push(format!(
                            "{r_name} <- match.arg({r_name}, several.ok = TRUE)"
                        ));
                    } else {
                        lines.push(format!("{r_name} <- match.arg({r_name})"));
                    }
                }
            }
        }
        if lines.is_empty() {
            String::new()
        } else {
            lines.join("\n  ")
        }
    };

    // Generate lifecycle prelude if needed
    let lifecycle_prelude = lifecycle_spec
        .as_ref()
        .and_then(|spec| spec.r_prelude(&r_wrapper_ident_str));

    // Generate R-side precondition checks (stopifnot + fallback precheck calls)
    // Skip both match_arg and choices params (already validated by match.arg)
    let mut skip_params = parsed.match_arg_params().clone();
    for param_name in parsed.choices_params().keys() {
        let r_name = r_wrapper_builder::normalize_r_arg_ident(&syn::Ident::new(
            param_name,
            proc_macro2::Span::call_site(),
        ))
        .to_string();
        skip_params.insert(r_name);
    }
    let precondition_output = r_preconditions::build_precondition_checks(inputs, &skip_params);
    let precondition_prelude = if precondition_output.static_checks.is_empty() {
        String::new()
    } else {
        precondition_output.static_checks.join("\n  ")
    };

    // Generate Missing<T> prelude: `if (missing(param)) param <- quote(expr=)`
    let missing_prelude = {
        let lines = r_wrapper_builder::build_missing_prelude(inputs, parsed.param_defaults());
        if lines.is_empty() {
            String::new()
        } else {
            lines.join("\n  ")
        }
    };

    // Combine all preludes: r_entry, on.exit, missing defaults, lifecycle, static preconditions, match.arg, choices, r_post_checks
    let on_exit_str = r_on_exit.as_ref().map(|oe| oe.to_r_code());
    let combined_prelude = {
        let mut parts = Vec::new();
        if let Some(ref entry) = r_entry {
            parts.push(entry.as_str());
        }
        if let Some(ref s) = on_exit_str {
            parts.push(s.as_str());
        }
        if !missing_prelude.is_empty() {
            parts.push(missing_prelude.as_str());
        }
        if let Some(ref lc) = lifecycle_prelude {
            parts.push(lc.as_str());
        }
        if !precondition_prelude.is_empty() {
            parts.push(&precondition_prelude);
        }
        if !match_arg_prelude.is_empty() {
            parts.push(&match_arg_prelude);
        }
        if !choices_prelude.is_empty() {
            parts.push(&choices_prelude);
        }
        if let Some(ref post) = r_post_checks {
            parts.push(post.as_str());
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n  "))
        }
    };

    let r_wrapper_string = if let Some(prelude) = combined_prelude {
        format!(
            "{}{}{}{}{}{} <- function({}) {{\n  {}\n  {}\n}}",
            roxygen_tags_str,
            source_comment,
            s3_method_comment,
            internal_comment,
            export_comment,
            r_wrapper_ident_str,
            formals_joined,
            prelude,
            r_wrapper_return_str
        )
    } else {
        format!(
            "{}{}{}{}{}{} <- function({}) {{\n  {}\n}}",
            roxygen_tags_str,
            source_comment,
            s3_method_comment,
            internal_comment,
            export_comment,
            r_wrapper_ident_str,
            formals_joined,
            r_wrapper_return_str
        )
    };
    // Use a raw string literal for better readability in macro expansion
    let r_wrapper_str = r_wrapper_raw_literal(&r_wrapper_string);

    // endregion

    let abi = abi
        .cloned()
        .unwrap_or_else(|| syn::parse_quote!(extern "C-unwind"));

    // Extract cfg attributes to apply to generated items
    let cfg_attrs = extract_cfg_attrs(parsed.attrs());

    // Generate doc strings with links
    let r_wrapper_doc = format!(
        "R wrapper code for [`{}`], calls [`{}`].",
        rust_ident, c_ident
    );
    let call_method_def_doc = format!(
        "R call method definition for [`{}`] (C wrapper: [`{}`]).",
        rust_ident, c_ident
    );
    let call_method_def_example = format!(
        "Value: `R_CallMethodDef {{ name: \"{}\", numArgs: {}, fun: <DL_FUNC> }}`",
        c_ident, num_args
    );
    let source_start = rust_ident.span().start();
    let source_line_lit = syn::LitInt::new(&source_start.line.to_string(), rust_ident.span());
    let source_col_lit =
        syn::LitInt::new(&(source_start.column + 1).to_string(), rust_ident.span());

    // Get the normalized item for output, with roxygen tags stripped from docs.
    // Roxygen tags are for R documentation and shouldn't appear in rustdoc.
    let mut original_item = parsed.item_without_roxygen();
    // Strip only the miniextendr attributes; keep everything else.
    original_item
        .attrs
        .retain(|attr| !attr.path().is_ident("miniextendr"));

    // Inject dots_typed binding into function body if dots = typed_list!(...) was specified
    if let Some(ref spec_tokens) = dots_spec {
        let dots_param = named_dots.clone().unwrap_or_else(|| {
            syn::Ident::new("__miniextendr_dots", proc_macro2::Span::call_site())
        });
        let validation_stmt: syn::Stmt = syn::parse_quote! {
            let dots_typed = #dots_param.typed(#spec_tokens)
                .expect("dots validation failed");
        };
        original_item.block.stmts.insert(0, validation_stmt);
    }

    let original_item = original_item;

    // Generate match_arg choices helper C wrappers and R_CallMethodDef entries
    let mut match_arg_helper_def_idents: Vec<syn::Ident> = Vec::new();
    let match_arg_helpers: Vec<proc_macro2::TokenStream> = match_arg_param_info
        .iter()
        .map(|(r_param, rust_name, param_ty)| {
            // For several_ok, the param type is Vec<Mode> — extract inner Mode for choices_sexp
            let choices_ty: &syn::Type = if parsed.has_several_ok(rust_name) {
                extract_vec_inner_type(param_ty).unwrap_or(param_ty)
            } else {
                param_ty
            };
            let helper_fn_name = format!(
                "C_{}__match_arg_choices__{}",
                c_ident.to_string().trim_start_matches("C_"),
                r_param
            );
            let helper_fn_ident = syn::Ident::new(&helper_fn_name, proc_macro2::Span::call_site());
            let helper_def_ident = syn::Ident::new(
                &format!("call_method_def_{}", helper_fn_name),
                proc_macro2::Span::call_site(),
            );
            match_arg_helper_def_idents.push(helper_def_ident.clone());
            let helper_c_name = syn::LitCStr::new(
                std::ffi::CString::new(helper_fn_name.clone())
                    .expect("valid C string")
                    .as_c_str(),
                proc_macro2::Span::call_site(),
            );
            quote::quote! {
                #(#cfg_attrs)*
                #[allow(non_snake_case)]
                #[unsafe(no_mangle)]
                pub extern "C-unwind" fn #helper_fn_ident(
                    __miniextendr_call: ::miniextendr_api::ffi::SEXP,
                ) -> ::miniextendr_api::ffi::SEXP {
                    ::miniextendr_api::choices_sexp::<#choices_ty>()
                }

                #(#cfg_attrs)*
                #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS)]
                #[linkme(crate = ::miniextendr_api::linkme)]
                #[allow(non_upper_case_globals)]
                #[allow(non_snake_case)]
                static #helper_def_ident: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
                    ::miniextendr_api::ffi::R_CallMethodDef {
                        name: #helper_c_name.as_ptr(),
                        fun: Some(std::mem::transmute::<
                            unsafe extern "C-unwind" fn(
                                ::miniextendr_api::ffi::SEXP,
                            ) -> ::miniextendr_api::ffi::SEXP,
                            unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void,
                        >(#helper_fn_ident)),
                        numArgs: 1i32,
                    }
                };
            }
        })
        .collect();

    // Generate MX_MATCH_ARG_CHOICES entries for placeholder → choices replacement
    let match_arg_choices_entries: Vec<proc_macro2::TokenStream> = match_arg_placeholders
        .iter()
        .filter_map(|(placeholder, rust_param)| {
            // Find the type for this param from match_arg_param_info
            let (_, _, param_ty) = match_arg_param_info
                .iter()
                .find(|(_, rn, _)| rn == rust_param)?;
            // For several_ok, extract inner type from Vec<Mode>
            let choices_ty: &syn::Type = if parsed.has_several_ok(rust_param) {
                extract_vec_inner_type(param_ty).unwrap_or(param_ty)
            } else {
                param_ty
            };
            let entry_ident = syn::Ident::new(
                &format!(
                    "match_arg_choices_entry_{}",
                    placeholder.trim_matches('_').replace('.', "_")
                ),
                proc_macro2::Span::call_site(),
            );
            Some(quote::quote! {
                #(#cfg_attrs)*
                #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_MATCH_ARG_CHOICES)]
                #[linkme(crate = ::miniextendr_api::linkme)]
                #[allow(non_upper_case_globals)]
                #[allow(non_snake_case)]
                static #entry_ident: ::miniextendr_api::registry::MatchArgChoicesEntry =
                    ::miniextendr_api::registry::MatchArgChoicesEntry {
                        placeholder: #placeholder,
                        choices_str: || {
                            <#choices_ty as ::miniextendr_api::match_arg::MatchArg>::CHOICES
                                .iter()
                                .map(|c| format!(
                                    "\"{}\"",
                                    ::miniextendr_api::match_arg::escape_r_string(c)
                                ))
                                .collect::<Vec<_>>()
                                .join(", ")
                        },
                    };
            })
        })
        .collect();

    // Generate MX_MATCH_ARG_PARAM_DOCS entries for @param doc placeholder → choice description
    let match_arg_param_doc_entries: Vec<proc_macro2::TokenStream> =
        match_arg_param_doc_placeholders
            .iter()
            .filter_map(|(doc_placeholder, rust_param)| {
                // Find the type for this param from match_arg_param_info
                let (_, _, param_ty) = match_arg_param_info
                    .iter()
                    .find(|(_, rn, _)| rn == rust_param)?;
                // For several_ok, extract inner type from Vec<Mode>
                let choices_ty: &syn::Type = if parsed.has_several_ok(rust_param) {
                    extract_vec_inner_type(param_ty).unwrap_or(param_ty)
                } else {
                    param_ty
                };
                let several_ok_lit = parsed.has_several_ok(rust_param);
                let entry_ident = syn::Ident::new(
                    &format!(
                        "match_arg_param_doc_entry_{}",
                        doc_placeholder.trim_matches('_').replace('.', "_")
                    ),
                    proc_macro2::Span::call_site(),
                );
                Some(quote::quote! {
                    #(#cfg_attrs)*
                    #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_MATCH_ARG_PARAM_DOCS)]
                    #[linkme(crate = ::miniextendr_api::linkme)]
                    #[allow(non_upper_case_globals)]
                    #[allow(non_snake_case)]
                    static #entry_ident: ::miniextendr_api::registry::MatchArgParamDocEntry =
                        ::miniextendr_api::registry::MatchArgParamDocEntry {
                            placeholder: #doc_placeholder,
                            several_ok: #several_ok_lit,
                            choices_str: || {
                                <#choices_ty as ::miniextendr_api::match_arg::MatchArg>::CHOICES
                                    .iter()
                                    .map(|c| format!("\"{}\"", c))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            },
                        };
                })
            })
            .collect();

    // Generate doc comment linking to C wrapper and R wrapper constant
    let fn_r_wrapper_doc = format!(
        "See [`{}`] for C wrapper, [`{}`] for R wrapper.",
        c_ident, r_wrapper_generator
    );

    let expanded: proc_macro::TokenStream = quote::quote! {
        // rust function with doc link to R wrapper
        #[doc = #fn_r_wrapper_doc]
        #original_item

        // C wrapper
        #(#cfg_attrs)*
        #c_wrapper

        // R wrapper (self-registers via distributed_slice)
        #(#cfg_attrs)*
        #[doc = #r_wrapper_doc]
        #[doc = concat!("Wraps Rust function `", stringify!(#rust_ident), "`.")]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS)]
                #[linkme(crate = ::miniextendr_api::linkme)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        static #r_wrapper_generator: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::Function,
                source_file: file!(),
                content: concat!(
                    "# Generated from Rust fn `",
                    stringify!(#rust_ident),
                    "` (",
                    file!(),
                    ":",
                    #source_line_lit,
                    ":",
                    #source_col_lit,
                    ")",
                    #r_wrapper_str
                ),
            };

        // registration of C wrapper in R (self-registers via distributed_slice)
        #(#cfg_attrs)*
        #[doc = #call_method_def_doc]
        #[doc = #call_method_def_example]
        #[doc = concat!("Wraps Rust function `", stringify!(#rust_ident), "`.")]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS)]
                #[linkme(crate = ::miniextendr_api::linkme)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        static #call_method_def: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
            ::miniextendr_api::ffi::R_CallMethodDef {
                name: #c_ident_name.as_ptr(),
                // Cast to DL_FUNC (generic function pointer) for storage in R's registration table.
                // R will cast back to the appropriate signature when calling.
                fun: Some(std::mem::transmute::<
                    unsafe #abi fn(#(#func_ptr_def),*) -> ::miniextendr_api::ffi::SEXP,
                    unsafe #abi fn() -> *mut ::std::os::raw::c_void
                >(#c_ident)),
                numArgs: #num_args,
            }
        };

        // match_arg choices helpers (C wrappers + R_CallMethodDef entries)
        // Each helper's call_method_def self-registers via distributed_slice
        #(#match_arg_helpers)*

        // match_arg choices entries for R wrapper default replacement
        #(#match_arg_choices_entries)*

        // match_arg @param doc entries for R wrapper roxygen doc replacement
        #(#match_arg_param_doc_entries)*

        // doc-lint warnings (if any)
        #doc_lint_warnings
    }
    .into();

    expanded
}

/// Generate thread-safe wrappers for R FFI functions.
///
/// Apply this to an `extern "C-unwind"` block to generate wrappers that ensure
/// R API calls happen on R's main thread.
///
/// # Behavior
///
/// All non-variadic functions are routed to the main thread via `with_r_thread`
/// when called from a worker thread. The return value is wrapped in `Sendable`
/// and sent back to the caller. This applies to both value-returning functions
/// (SEXP, i32, etc.) and pointer-returning functions (`*const T`, `*mut T`).
///
/// Pointer-returning functions (like `INTEGER`, `REAL`) are safe to route because
/// the underlying SEXP must be GC-protected by the caller, and R's GC only runs
/// during R API calls which are serialized through `with_r_thread`.
///
/// # Initialization Requirement
///
/// `miniextendr_runtime_init()` must be called before using any wrapped function.
/// Calling before initialization will panic with a descriptive error message.
///
/// # Limitations
///
/// - Variadic functions are passed through unchanged (no wrapper)
/// - Statics are passed through unchanged
/// - Functions with `#[link_name]` are passed through unchanged
///
/// # Example
///
/// ```ignore
/// #[r_ffi_checked]
/// unsafe extern "C-unwind" {
///     // Routed to main thread via with_r_thread when called from worker
///     pub fn Rf_ScalarInteger(arg1: i32) -> SEXP;
///     pub fn INTEGER(x: SEXP) -> *mut i32;
/// }
/// ```
#[proc_macro_attribute]
pub fn r_ffi_checked(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let foreign_mod = syn::parse_macro_input!(item as syn::ItemForeignMod);

    let foreign_mod_attrs = &foreign_mod.attrs;
    let abi = &foreign_mod.abi;
    let mut unchecked_items = Vec::new();
    let mut checked_wrappers = Vec::new();

    for item in &foreign_mod.items {
        match item {
            syn::ForeignItem::Fn(fn_item) => {
                let is_variadic = fn_item.sig.variadic.is_some();

                // Check if function already has #[link_name] - if so, pass through unchanged
                let has_link_name = fn_item
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("link_name"));

                if is_variadic || has_link_name {
                    // Pass through variadic functions and functions with explicit link_name unchanged
                    unchecked_items.push(item.clone());
                } else {
                    // Generate checked wrapper for non-variadic functions
                    let vis = &fn_item.vis;
                    let fn_name = &fn_item.sig.ident;
                    let fn_name_str = fn_name.to_string();
                    let unchecked_name = quote::format_ident!("{}_unchecked", fn_name);
                    let unchecked_name_str = unchecked_name.to_string();
                    let inputs = &fn_item.sig.inputs;
                    let output = &fn_item.sig.output;
                    // Filter out link_name attributes (already checked above, but be safe)
                    let attrs: Vec<_> = fn_item
                        .attrs
                        .iter()
                        .filter(|attr| !attr.path().is_ident("link_name"))
                        .collect();
                    let checked_doc = format!(
                        "Checked wrapper for `{}`. Calls `{}` and routes through `with_r_thread`.",
                        fn_name_str, unchecked_name_str
                    );
                    let checked_doc_lit = syn::LitStr::new(&checked_doc, fn_name.span());
                    let source_loc_doc = crate::source_location_doc(fn_name.span());
                    let source_loc_doc_lit = syn::LitStr::new(&source_loc_doc, fn_name.span());

                    // Generate the unchecked FFI binding with #[link_name]
                    // Same visibility as the checked variant
                    let link_name = syn::LitStr::new(&fn_name_str, fn_name.span());
                    let unchecked_fn: syn::ForeignItem = syn::parse_quote! {
                        #(#attrs)*
                        #[doc = concat!("Unchecked FFI binding for `", stringify!(#fn_name), "`.")]
                        #[doc = #source_loc_doc_lit]
                        #[doc = concat!("Generated from source file `", file!(), "`.")]
                        #[link_name = #link_name]
                        #vis fn #unchecked_name(#inputs) #output;
                    };
                    unchecked_items.push(unchecked_fn);

                    // Generate a checked wrapper function
                    let arg_names: Vec<_> = inputs
                        .iter()
                        .filter_map(|arg| {
                            if let syn::FnArg::Typed(pat_type) = arg
                                && let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                            {
                                Some(pat_ident.ident.clone())
                            } else {
                                None
                            }
                        })
                        .collect();

                    let is_never = matches!(output, syn::ReturnType::Type(_, ty) if matches!(**ty, syn::Type::Never(_)));

                    let wrapper = if is_never {
                        // Never-returning functions (like Rf_error)
                        quote::quote! {
                            #(#attrs)*
                            #[doc = #checked_doc_lit]
                            #[doc = #source_loc_doc_lit]
                            #[doc = concat!("Generated from source file `", file!(), "`.")]
                            #[inline(always)]
                            #[allow(non_snake_case)]
                            #vis unsafe fn #fn_name(#inputs) #output {
                                ::miniextendr_api::worker::with_r_thread(move || unsafe {
                                    #unchecked_name(#(#arg_names),*)
                                })
                            }
                        }
                    } else {
                        // Normal functions - route via with_r_thread
                        quote::quote! {
                            #(#attrs)*
                            #[doc = #checked_doc_lit]
                            #[doc = #source_loc_doc_lit]
                            #[doc = concat!("Generated from source file `", file!(), "`.")]
                            #[inline(always)]
                            #[allow(non_snake_case)]
                            #vis unsafe fn #fn_name(#inputs) #output {
                                let result = ::miniextendr_api::worker::with_r_thread(move || {
                                    ::miniextendr_api::worker::Sendable(unsafe {
                                        #unchecked_name(#(#arg_names),*)
                                    })
                                });
                                result.0
                            }
                        }
                    };
                    checked_wrappers.push(wrapper);
                }
            }
            _ => {
                // Pass through statics and other items unchanged
                unchecked_items.push(item.clone());
            }
        }
    }

    let expanded = quote::quote! {
        #(#foreign_mod_attrs)*
        unsafe #abi {
            #(#unchecked_items)*
        }

        #(#checked_wrappers)*
    };

    expanded.into()
}

/// Derive macro for implementing `RNativeType` on a newtype wrapper.
///
/// This allows newtype wrappers around R native types to work with `Vec<T>`,
/// `&[T]` conversions and the `Coerce<R>` traits.
/// The inner type must implement `RNativeType`.
///
/// # Supported Struct Forms
///
/// Both tuple structs and single-field named structs are supported:
///
/// ```ignore
/// use miniextendr_api::RNativeType;
///
/// // Tuple struct (most common)
/// #[derive(Clone, Copy, RNativeType)]
/// struct UserId(i32);
///
/// // Named single-field struct
/// #[derive(Clone, Copy, RNativeType)]
/// struct Temperature { celsius: f64 }
/// ```
///
/// # Generated Code
///
/// For `struct UserId(i32)`, this generates:
///
/// ```ignore
/// impl RNativeType for UserId {
///     const SEXP_TYPE: SEXPTYPE = <i32 as RNativeType>::SEXP_TYPE;
///
///     unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
///         <i32 as RNativeType>::dataptr_mut(sexp).cast()
///     }
/// }
/// ```
///
/// # Using the Newtype with Coerce
///
/// Once `RNativeType` is derived, you can implement `Coerce` to/from the newtype:
///
/// ```ignore
/// impl Coerce<UserId> for i32 {
///     fn coerce(self) -> UserId { UserId(self) }
/// }
///
/// let id: UserId = 42.coerce();
/// ```
///
/// # Requirements
///
/// - Must be a newtype struct (exactly one field, tuple or named)
/// - The inner type must implement `RNativeType` (`i32`, `f64`, `RLogical`, `u8`, `Rcomplex`)
/// - Should also derive `Copy` (required by `RNativeType: Copy`)
#[proc_macro_derive(RNativeType)]
pub fn derive_rnative_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract inner type and constructor — must be a newtype (single field)
    let (inner_ty, elt_ctor): (syn::Type, proc_macro2::TokenStream) = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = fields.unnamed.first().unwrap().ty.clone();
                let ctor = quote::quote! { Self(val) };
                (ty, ctor)
            }
            syn::Fields::Named(fields) if fields.named.len() == 1 => {
                let field = fields.named.first().unwrap();
                let ty = field.ty.clone();
                let field_name = field.ident.as_ref().unwrap();
                let ctor = quote::quote! { Self { #field_name: val } };
                (ty, ctor)
            }
            _ => {
                return syn::Error::new_spanned(
                    name,
                    "#[derive(RNativeType)] requires a newtype struct with exactly one field",
                )
                .into_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "#[derive(RNativeType)] only works on structs")
                .into_compile_error()
                .into();
        }
    };

    let expanded = quote::quote! {
        impl #impl_generics ::miniextendr_api::ffi::RNativeType for #name #ty_generics #where_clause {
            const SEXP_TYPE: ::miniextendr_api::ffi::SEXPTYPE =
                <#inner_ty as ::miniextendr_api::ffi::RNativeType>::SEXP_TYPE;

            #[inline]
            unsafe fn dataptr_mut(sexp: ::miniextendr_api::ffi::SEXP) -> *mut Self {
                // Newtype is repr(transparent), so we can cast the pointer
                unsafe {
                    <#inner_ty as ::miniextendr_api::ffi::RNativeType>::dataptr_mut(sexp).cast()
                }
            }

            #[inline]
            fn elt(sexp: ::miniextendr_api::ffi::SEXP, i: isize) -> Self {
                let val = <#inner_ty as ::miniextendr_api::ffi::RNativeType>::elt(sexp, i);
                #elt_ctor
            }
        }

    };

    expanded.into()
}

/// Derive macro for implementing `TypedExternal` on a type.
///
/// This makes the type compatible with `ExternalPtr<T>` for storing in R's external pointers.
///
/// # Basic Usage
///
/// ```ignore
/// use miniextendr_api::TypedExternal;
///
/// #[derive(ExternalPtr)]
/// struct MyData {
///     value: i32,
/// }
///
/// // Now you can use ExternalPtr<MyData>
/// let ptr = ExternalPtr::new(MyData { value: 42 });
/// ```
///
/// # Trait ABI
///
/// Trait dispatch wrappers are automatically generated:
///
/// ```ignore
/// use miniextendr_api::miniextendr;
///
/// #[derive(ExternalPtr)]
/// struct MyCounter {
///     value: i32,
/// }
///
/// #[miniextendr]
/// impl Counter for MyCounter {
///     fn value(&self) -> i32 { self.value }
///     fn increment(&mut self) { self.value += 1; }
/// }
/// ```
///
/// This generates additional infrastructure for type-erased trait dispatch:
/// - `__MxWrapperMyCounter` - Type-erased wrapper struct
/// - `__MX_BASE_VTABLE_MYCOUNTER` - Base vtable with drop/query
/// - `__mx_wrap_mycounter()` - Constructor returning `*mut mx_erased`
///
/// # Generated Code (Basic)
///
/// For a type `MyData` without traits:
///
/// ```ignore
/// impl TypedExternal for MyData {
///     const TYPE_NAME: &'static str = "MyData";
///     const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
/// }
/// ```
#[proc_macro_derive(ExternalPtr, attributes(externalptr, r_data))]
pub fn derive_external_ptr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    externalptr_derive::derive_external_ptr(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP integer vector data types.
///
/// Auto-implements `AltrepLen`, `AltIntegerData`, and calls `impl_altinteger_from_data!`.
///
/// # Attributes
///
/// - `#[altrep(len = "field_name")]` - Specify length field (auto-detects "len" or "length")
/// - `#[altrep(elt = "field_name")]` - For constant vectors, specify which field provides elements
/// - `#[altrep(dataptr)]` - Pass `dataptr` option to low-level macro
/// - `#[altrep(serialize)]` - Pass `serialize` option to low-level macro
/// - `#[altrep(subset)]` - Pass `subset` option to low-level macro
/// - `#[altrep(no_lowlevel)]` - Skip automatic `impl_altinteger_from_data!` call
///
/// # Example (Constant Vector - Zero Boilerplate!)
///
/// ```ignore
/// #[derive(ExternalPtr, AltrepInteger)]
/// #[altrep(elt = "value")]  // All elements return this field
/// pub struct ConstantIntData {
///     value: i32,
///     len: usize,
/// }
///
/// // That's it! 3 lines instead of 30!
/// // AltrepLen, AltIntegerData, and low-level impls are auto-generated
///
/// #[miniextendr(class = "ConstantInt")]
/// pub struct ConstantIntClass(pub ConstantIntData);
/// ```
///
/// # Example (Custom elt() - Override One Method)
///
/// ```ignore
/// #[derive(ExternalPtr, AltrepInteger)]
/// pub struct ArithSeqData {
///     start: i32,
///     step: i32,
///     len: usize,
/// }
///
/// // Auto-generates AltrepLen and stub AltIntegerData
/// // Just override elt() for custom logic:
/// impl AltIntegerData for ArithSeqData {
///     fn elt(&self, i: usize) -> i32 {
///         self.start + (i as i32) * self.step
///     }
/// }
/// ```
#[proc_macro_derive(AltrepInteger, attributes(altrep))]
pub fn derive_altrep_integer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_integer(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP real vector data types.
///
/// Auto-implements `AltrepLen` and `AltRealData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepReal, attributes(altrep))]
pub fn derive_altrep_real(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_real(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP logical vector data types.
///
/// Auto-implements `AltrepLen` and `AltLogicalData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepLogical, attributes(altrep))]
pub fn derive_altrep_logical(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_logical(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP raw vector data types.
///
/// Auto-implements `AltrepLen` and `AltRawData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepRaw, attributes(altrep))]
pub fn derive_altrep_raw(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_raw(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP string vector data types.
///
/// Auto-implements `AltrepLen` and `AltStringData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepString, attributes(altrep))]
pub fn derive_altrep_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_string(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP complex vector data types.
///
/// Auto-implements `AltrepLen` and `AltComplexData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepComplex, attributes(altrep))]
pub fn derive_altrep_complex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_complex(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP list vector data types.
///
/// Auto-implements `AltrepLen` and `AltListData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger),
/// except `dataptr` and `subset` which are not supported for list ALTREP.
#[proc_macro_derive(AltrepList, attributes(altrep))]
pub fn derive_altrep_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive ALTREP registration for a data struct.
///
/// Generates `TypedExternal`, `AltrepClass`, `RegisterAltrep`, `IntoR`,
/// linkme registration entry, and `Ref`/`Mut` accessor types.
///
/// The struct must already have low-level ALTREP traits implemented.
/// For most use cases, prefer a family-specific derive:
/// `#[derive(AltrepInteger)]`, `#[derive(AltrepReal)]`, etc.
/// Use `#[altrep(manual)]` on a family derive to skip data trait generation
/// when you provide your own `AltrepLen` + `Alt*Data` impls.
///
/// # Attributes
///
/// - `#[altrep(class = "Name")]` — custom ALTREP class name (defaults to struct name)
///
/// # Example
///
/// ```ignore
/// // Prefer family derives with manual:
/// #[derive(AltrepInteger)]
/// #[altrep(manual, class = "MyCustom", serialize)]
/// struct MyData { ... }
///
/// impl AltrepLen for MyData { ... }
/// impl AltIntegerData for MyData { ... }
/// ```
#[proc_macro_derive(Altrep, attributes(altrep))]
pub fn derive_altrep(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep::derive_altrep(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `IntoList` for a struct (Rust → R list).
///
/// - Named structs → named R list: `list(x = 1L, y = 2L)`
/// - Tuple structs → unnamed R list: `list(1L, 2L)`
/// - Fields annotated `#[into_list(ignore)]` are skipped
#[proc_macro_derive(IntoList, attributes(into_list))]
pub fn derive_into_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_into_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `TryFromList` for a struct (R list → Rust).
///
/// - Named structs: extract by field name
/// - Tuple structs: extract by position (0, 1, 2, ...)
/// - Fields annotated `#[into_list(ignore)]` are not read and are initialized with `Default::default()`
#[proc_macro_derive(TryFromList, attributes(into_list))]
pub fn derive_try_from_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_try_from_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PrefersList`: when a type implements both `IntoList` and `ExternalPtr`,
/// this selects list as the default `IntoR` conversion.
///
/// Without a Prefer* derive, types that implement multiple conversion paths
/// will get a compile error due to conflicting `IntoR` impls.
///
/// # Example
///
/// ```ignore
/// #[derive(IntoList, PreferList)]
/// struct Config { verbose: bool, threads: i32 }
/// // IntoR produces list(verbose = TRUE, threads = 4L)
/// ```
#[proc_macro_derive(PreferList)]
pub fn derive_prefer_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferDataFrame`: when a type implements both `IntoDataFrame` (via `DataFrameRow`)
/// and other conversion paths, this selects data.frame as the default `IntoR` conversion.
///
/// # Example
///
/// ```ignore
/// #[derive(DataFrameRow, PreferDataFrame)]
/// struct Obs { time: f64, value: f64 }
/// // IntoR produces data.frame(time = ..., value = ...)
/// ```
#[proc_macro_derive(PreferDataFrame)]
pub fn derive_prefer_data_frame(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_data_frame(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferExternalPtr`: when a type implements both `ExternalPtr` and
/// other conversion paths (e.g., `IntoList`), this selects `ExternalPtr` wrapping
/// as the default `IntoR` conversion.
///
/// # Example
///
/// ```ignore
/// #[derive(ExternalPtr, IntoList, PreferExternalPtr)]
/// struct Model { weights: Vec<f64> }
/// // IntoR wraps as ExternalPtr (opaque R object), not list
/// ```
#[proc_macro_derive(PreferExternalPtr)]
pub fn derive_prefer_externalptr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_externalptr(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferRNativeType`: when a newtype wraps an `RNativeType` and also
/// implements other conversions, this selects the native R vector conversion
/// as the default `IntoR` path.
///
/// # Example
///
/// ```ignore
/// #[derive(Copy, Clone, RNativeType, PreferRNativeType)]
/// struct Meters(f64);
/// // IntoR produces a numeric scalar, not an ExternalPtr
/// ```
#[proc_macro_derive(PreferRNativeType)]
pub fn derive_prefer_rnative(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_rnative(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `DataFrameRow`: generates a companion `*DataFrame` type with collection fields,
/// plus `IntoR` / `TryFromSexp` / `IntoDataFrame` impls for seamless R data.frame conversion.
///
/// # Example
///
/// ```ignore
/// #[derive(DataFrameRow)]
/// struct Measurement {
///     time: f64,
///     value: f64,
/// }
///
/// // Generates MeasurementDataFrame { time: Vec<f64>, value: Vec<f64> }
/// // plus conversion impls
/// ```
///
/// # Struct-level attributes
///
/// - `#[dataframe(name = "CustomDf")]` — custom name for the generated DataFrame type
/// - `#[dataframe(align)]` — pad shorter columns with NA to match longest
/// - `#[dataframe(tag = "my_tag")]` — attach a tag attribute to the data.frame
/// - `#[dataframe(conflicts = "string")]` — resolve conflicting column types as strings
///
/// # Field-level attributes
///
/// - `#[dataframe(skip)]` — omit this field from the DataFrame
/// - `#[dataframe(rename = "col")]` — custom column name
/// - `#[dataframe(as_list)]` — keep collection as single list column (no expansion)
/// - `#[dataframe(expand)]` / `#[dataframe(unnest)]` — expand collection into suffixed columns
/// - `#[dataframe(width = N)]` — pin expansion width (shorter rows get NA)
#[proc_macro_derive(DataFrameRow, attributes(dataframe))]
pub fn derive_dataframe_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    dataframe_derive::derive_dataframe_row(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `RFactor`: enables conversion between Rust enums and R factors.
///
/// # Usage
///
/// ```ignore
/// #[derive(Copy, Clone, RFactor)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
/// ```
///
/// # Attributes
///
/// - `#[r_factor(rename = "name")]` - Rename a variant's level string
/// - `#[r_factor(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)
#[proc_macro_derive(RFactor, attributes(r_factor))]
pub fn derive_r_factor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    factor_derive::derive_r_factor(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `MatchArg`: enables conversion between Rust enums and R character strings
/// with `match.arg` semantics (partial matching, informative errors).
///
/// # Usage
///
/// ```ignore
/// #[derive(Copy, Clone, MatchArg)]
/// enum Mode {
///     Fast,
///     Safe,
///     Debug,
/// }
/// ```
///
/// # Attributes
///
/// - `#[match_arg(rename = "name")]` - Rename a variant's choice string
/// - `#[match_arg(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)
///
/// # Generated Implementations
///
/// - `MatchArg` - Choice metadata and bidirectional conversion
/// - `TryFromSexp` - Convert R STRSXP/factor to enum (with partial matching)
/// - `IntoR` - Convert enum to R character scalar
#[proc_macro_derive(MatchArg, attributes(match_arg))]
pub fn derive_match_arg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match_arg_derive::derive_match_arg(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `Vctrs`: enables creating vctrs-compatible S3 vector classes from Rust structs.
///
/// # Usage
///
/// ```ignore
/// #[derive(Vctrs)]
/// #[vctrs(class = "percent", base = "double")]
/// pub struct Percent {
///     data: Vec<f64>,
/// }
/// ```
///
/// # Attributes
///
/// - `#[vctrs(class = "name")]` - R class name (required)
/// - `#[vctrs(base = "type")]` - Base type: double, integer, logical, character, raw, list, record
/// - `#[vctrs(abbr = "abbr")]` - Abbreviation for `vec_ptype_abbr`
/// - `#[vctrs(inherit_base = true|false)]` - Whether to include base type in class vector
///
/// # Generated Implementations
///
/// - `VctrsClass` - Metadata trait for vctrs class information
/// - `VctrsRecord` (for `base = "record"`) - Field names for record types
#[cfg(feature = "vctrs")]
#[proc_macro_derive(Vctrs, attributes(vctrs))]
pub fn derive_vctrs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    vctrs_derive::derive_vctrs(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Create a `TypedListSpec` for validating `...` arguments or lists.
///
/// This macro provides ergonomic syntax for defining typed list specifications
/// that can be used with `Dots::typed()` to validate the structure of
/// `...` arguments passed from R.
///
/// # Syntax
///
/// ```text
/// typed_list!(
///     name => type_spec,    // required field with type
///     name? => type_spec,   // optional field with type
///     name,                 // required field, any type
///     name?,                // optional field, any type
/// )
/// ```
///
/// For strict mode (no extra fields allowed):
/// ```text
/// typed_list!(@exact; name => type_spec, ...)
/// ```
///
/// # Type Specifications
///
/// ## Base types (with optional length)
/// - `numeric()` / `numeric(4)` - Real/double vector
/// - `integer()` / `integer(4)` - Integer vector
/// - `logical()` / `logical(4)` - Logical vector
/// - `character()` / `character(4)` - Character vector
/// - `raw()` / `raw(4)` - Raw vector
/// - `complex()` / `complex(4)` - Complex vector
/// - `list()` / `list(4)` - List (VECSXP)
///
/// ## Special types
/// - `data_frame()` - Data frame
/// - `factor()` - Factor
/// - `matrix()` - Matrix
/// - `array()` - Array
/// - `function()` - Function
/// - `environment()` - Environment
/// - `null()` - NULL only
/// - `any()` - Any type
///
/// ## String literals
/// - `"numeric"`, `"integer"`, etc. - Same as call syntax
/// - `"data.frame"` - Data frame (alias)
/// - `"MyClass"` - Any other string is treated as a class name (uses `Rf_inherits`)
///
/// # Examples
///
/// ## Basic usage
///
/// ```ignore
/// use miniextendr_api::{miniextendr, typed_list, Dots};
///
/// #[miniextendr]
/// pub fn process_args(dots: ...) -> Result<i32, String> {
///     let args = dots.typed(typed_list!(
///         alpha => numeric(4),
///         beta => list(),
///         gamma? => "character",
///     )).map_err(|e| e.to_string())?;
///
///     let alpha: Vec<f64> = args.get("alpha").map_err(|e| e.to_string())?;
///     Ok(alpha.len() as i32)
/// }
/// ```
///
/// ## Strict mode
///
/// ```ignore
/// // Reject any extra named fields
/// let args = dots.typed(typed_list!(@exact;
///     x => numeric(),
///     y => numeric(),
/// ))?;
/// ```
///
/// ## Class checking
///
/// ```ignore
/// // Check for specific R class (uses Rf_inherits semantics)
/// let args = dots.typed(typed_list!(
///     data => "data.frame",
///     model => "lm",
/// ))?;
/// ```
///
/// ## Attribute sugar
///
/// Instead of calling `.typed()` manually, you can use `typed_list!` directly in the
/// `#[miniextendr]` attribute for automatic validation:
///
/// ```ignore
/// #[miniextendr(dots = typed_list!(x => numeric(), y => numeric()))]
/// pub fn my_func(...) -> String {
///     // `dots_typed` is automatically created and validated
///     let x: f64 = dots_typed.get("x").expect("x");
///     let y: f64 = dots_typed.get("y").expect("y");
///     format!("x={}, y={}", x, y)
/// }
/// ```
///
/// This injects validation at the start of the function body:
/// ```ignore
/// let dots_typed = _dots.typed(typed_list!(...)).expect("dots validation failed");
/// ```
///
/// See the [`#[miniextendr]`](macro@miniextendr) attribute documentation for more details.
///
#[proc_macro]
pub fn typed_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as typed_list::TypedListInput);
    typed_list::expand_typed_list(parsed).into()
}

/// Construct an R list from Rust values.
///
/// This macro provides a convenient way to create R lists in Rust code,
/// using R-like syntax. Values are converted to R objects via the [`IntoR`] trait.
///
/// # Syntax
///
/// ```ignore
/// // Named entries (like R's list())
/// list!(
///     alpha = 1,
///     beta = "hello",
///     "my-name" = vec![1, 2, 3],
/// )
///
/// // Unnamed entries
/// list!(1, "hello", vec![1, 2, 3])
///
/// // Mixed (unnamed entries get empty string names)
/// list!(alpha = 1, 2, beta = "hello")
///
/// // Empty list
/// list!()
/// ```
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::{list, IntoR};
///
/// // Create a named list
/// let my_list = list!(
///     x = 42,
///     y = "hello world",
///     z = vec![1.0, 2.0, 3.0],
/// );
///
/// // In R this is equivalent to:
/// // list(x = 42L, y = "hello world", z = c(1, 2, 3))
/// ```
///
/// [`IntoR`]: https://docs.rs/miniextendr-api/latest/miniextendr_api/into_r/trait.IntoR.html
#[proc_macro]
pub fn list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as list_macro::ListInput);
    list_macro::expand_list(parsed).into()
}

/// Internal proc macro used by TPIE (Trait-Provided Impl Expansion).
///
/// Called by `__mx_impl_<Trait>!` macro_rules macros generated by `#[miniextendr]` on traits.
/// Do not call directly.
#[proc_macro]
#[doc(hidden)]
pub fn __mx_trait_impl_expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    miniextendr_impl_trait::expand_tpie(input)
}

/// Generate `TypedExternal` and `IntoExternalPtr` impls for a concrete monomorphization
/// of a generic type.
///
/// Since `#[derive(ExternalPtr)]` rejects generic types, use this macro to generate
/// the necessary impls for a specific type instantiation.
///
/// # Example
///
/// ```ignore
/// struct Wrapper<T> { inner: T }
///
/// impl_typed_external!(Wrapper<i32>);
/// impl_typed_external!(Wrapper<String>);
/// ```
#[proc_macro]
pub fn impl_typed_external(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match typed_external_macro::impl_typed_external(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

/// Generate the `R_init_*` entry point for a miniextendr R package.
///
/// This macro consolidates all package initialization into a single line.
/// It generates an `extern "C-unwind"` function that R calls when loading
/// the shared library.
///
/// # Usage
///
/// ```ignore
/// // Auto-detects package name from CARGO_CRATE_NAME (recommended):
/// miniextendr_api::miniextendr_init!();
///
/// // Or specify explicitly (for edge cases):
/// miniextendr_api::miniextendr_init!(mypkg);
/// ```
///
/// The generated function calls `miniextendr_api::init::package_init` which
/// handles panic hooks, runtime init, locale assertion, ALTREP setup, trait ABI
/// registration, routine registration, and symbol locking.
#[proc_macro]
pub fn miniextendr_init(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pkg_name: syn::Ident = if input.is_empty() {
        // Auto-detect from CARGO_CRATE_NAME (set by cargo during compilation)
        let name = std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| {
            panic!(
                "CARGO_CRATE_NAME not set. Either pass the package name explicitly: \
                 miniextendr_init!(mypkg), or ensure you're building with cargo."
            )
        });
        syn::Ident::new(&name, proc_macro2::Span::call_site())
    } else {
        syn::parse_macro_input!(input as syn::Ident)
    };
    let fn_name = syn::Ident::new(&format!("R_init_{}", pkg_name), pkg_name.span());

    // Build a byte string literal with NUL terminator for the package name.
    let mut name_bytes = pkg_name.to_string().into_bytes();
    name_bytes.push(0);
    let byte_lit = syn::LitByteStr::new(&name_bytes, pkg_name.span());

    let expanded = quote::quote! {
        #[unsafe(no_mangle)]
        pub unsafe extern "C-unwind" fn #fn_name(
            dll: *mut ::miniextendr_api::ffi::DllInfo,
        ) {
            unsafe {
                // SAFETY: byte literal is a valid NUL-terminated string produced by the macro.
                let pkg_name = ::std::ffi::CStr::from_bytes_with_nul_unchecked(#byte_lit);
                ::miniextendr_api::init::package_init(dll, pkg_name);
            }
        }

        /// Linker anchor: stub.c references this symbol to force the linker to pull
        /// in the user crate's archive member from the staticlib. With codegen-units = 1,
        /// this single member contains all linkme distributed_slice entries.
        /// The name is package-independent so stub.c doesn't need configure substitution.
        #[unsafe(no_mangle)]
        pub static miniextendr_force_link: ::std::ffi::c_char = 0;
    };

    expanded.into()
}

#[cfg(test)]
mod tests;
