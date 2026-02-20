//! # miniextendr-macros - Procedural macros for Rust <-> R interop
//!
//! This crate provides the procedural macros that power miniextendr's code
//! generation. Most users should depend on `miniextendr-api` and use its
//! re-exports, but this crate can be used directly when you only need macros.
//!
//! Primary macros and derives:
//! - `#[miniextendr]` on functions, impl blocks, trait defs, and trait impls.
//! - `miniextendr_module!` for registration and wrapper aggregation.
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
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                         miniextendr_module! { ... }                      │
//! │                                                                          │
//! │  1. Parse module contents:                                               │
//! │     ├── mod <name>;          → package name                              │
//! │     ├── fn <name>;           → function registration                     │
//! │     ├── struct <name>;       → ALTREP registration                       │
//! │     ├── impl <Type>;         → class method registration                 │
//! │     └── impl Trait for Type; → trait ABI registration                    │
//! │                                                                          │
//! │  2. Generate:                                                            │
//! │     ├── R_CallMethodDef array from all registered items                  │
//! │     ├── R_init_<name>_miniextendr() initialization function              │
//! │     ├── Combined R wrapper code (all functions + classes)                │
//! │     └── Trait ABI vtable/view types if traits present                    │
//! └──────────────────────────────────────────────────────────────────────────┘
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
//! | `miniextendr_module` | Module macro parsing and codegen |
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
mod altrep_module;
mod c_wrapper_builder;
mod list_macro;
mod miniextendr_fn;
mod typed_list;
use crate::miniextendr_fn::{MiniextendrFnAttrs, MiniextendrFunctionParsed};
mod miniextendr_impl;
use crate::miniextendr_module::MiniextendrModule;
pub(crate) use miniextendr_macros_core::miniextendr_module;
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

// vctrs support
#[cfg(feature = "vctrs")]
mod vctrs_derive;

pub(crate) use miniextendr_macros_core::{
    call_method_def_ident_for, match_arg_call_defs_ident_for, r_wrapper_const_ident_for,
};

// Feature default mutual exclusivity guards
#[cfg(all(feature = "default-r6", feature = "default-s7"))]
compile_error!("`default-r6` and `default-s7` are mutually exclusive");
#[cfg(all(feature = "default-worker", feature = "default-main-thread"))]
compile_error!("`default-worker` and `default-main-thread` are mutually exclusive");

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

/// Returns the first generic type argument from a path segment.
fn first_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    nth_type_argument(seg, 0)
}

/// Returns the second generic type argument from a path segment.
fn second_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    nth_type_argument(seg, 1)
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
/// use miniextendr_api::{miniextendr, miniextendr_module};
///
/// #[miniextendr]
/// fn add(a: i32, b: i32) -> i32 { a + b }
///
/// miniextendr_module! {
///     mod mypkg;
///     fn add;
/// }
/// ```
///
/// This produces a C wrapper `C_add` and an R wrapper `add()`.
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
/// - `#[miniextendr(unsafe(main_thread))]`
/// - `#[miniextendr(invisible)]` / `#[miniextendr(visible)]`
/// - `#[miniextendr(check_interrupt)]`
/// - `#[miniextendr(coerce)]` (also usable per-parameter)
/// - `#[miniextendr(unwrap_in_r)]` (return `Result<T, E>` to R without unwrapping)
/// - `#[miniextendr(dots = typed_list!(...))]` - validate dots, create `dots_typed`
///
/// # Impl blocks (class systems)
///
/// Apply `#[miniextendr(env|r6|s7|s3|s4)]` to an `impl Type` block and list
/// `impl Type;` in `miniextendr_module!`.
///
/// ## R6 Active Bindings
///
/// For R6 classes, use `#[miniextendr(r6(active))]` on methods to create
/// active bindings (computed properties accessed without parentheses):
///
/// ```ignore
/// use miniextendr_api::{miniextendr, miniextendr_module};
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
///
/// miniextendr_module! {
///     mod mypkg;
///     impl Rectangle;
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
/// use miniextendr_api::{miniextendr, miniextendr_module, ExternalPtr};
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
/// use miniextendr_api::{miniextendr, miniextendr_module, ExternalPtr};
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
/// `#[miniextendr] impl Trait for Type` and list `impl Trait for Type;` in
/// `miniextendr_module!`.
///
/// # ALTREP
///
/// Apply `#[miniextendr(class = "...", pkg = "...", base = "...")]` to a
/// one-field wrapper struct and list `struct Type;` in `miniextendr_module!`.
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
        // Delegate to ALTREP path (structs/enums)
        return altrep::expand_altrep_struct(attr, item);
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
        doc,
        error_in_r,
        c_symbol,
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
    let match_arg_call_defs = parsed.match_arg_call_defs_ident();
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
    func_ptr_def.extend(
        (0..inputs.len())
            .map(|_| syn::parse_quote!(::miniextendr_api::ffi::SEXP))
            .collect::<Vec<_>>(),
    );

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
    if coerce_all {
        conversion_builder = conversion_builder.with_coerce_all();
    }
    for input in inputs.iter() {
        if let syn::FnArg::Typed(pt) = input
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
        {
            let param_name = pat_ident.ident.to_string();
            if parsed.has_coerce_attr(&param_name) {
                conversion_builder = conversion_builder.with_coerce_param(param_name);
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
    // 1. **Main Thread Strategy** (with_r_unwind_protect)
    //    - All code runs on R's main thread
    //    - Required when SEXP types are involved (not Send)
    //    - Required for R API calls (Rf_*, R_*)
    //    - Panic handling via R_UnwindProtect (Rust destructors run correctly)
    //    - Use cases:
    //      * Functions returning SEXP
    //      * Functions taking SEXP parameters
    //      * Functions using Dots (variadic args)
    //      * Functions with #[miniextendr(unsafe(main_thread))]
    //      * Functions with check_interrupt (R_CheckUserInterrupt)
    //
    // 2. **Worker Thread Strategy** (run_on_worker + catch_unwind)
    //    - Argument conversion on main thread (SEXP → Rust types)
    //    - Function execution on dedicated worker thread (clean panic isolation)
    //    - Result conversion on main thread (Rust types → SEXP)
    //    - Panic handling via catch_unwind (prevents unwinding across FFI boundary)
    //    - Benefits:
    //      * Clean panic handling with proper destructors
    //      * Isolates user code from R's execution context
    //      * Catches panics from both user code and conversions
    //    - ExternalPtr<T> is Send: can be returned from worker thread functions
    //    - R API calls from worker use with_r_thread (serialized to main thread)
    //
    // Default: Worker thread (safer, cleaner panic handling)
    // Override: Use main thread when SEXP types are in the signature
    //
    // Note: Functions that call R API directly (not through with_r_thread) and
    // don't have SEXP in their signature need #[miniextendr(unsafe(main_thread))].
    //
    // Thread strategy:
    // - force_worker overrides the default, but cannot override hard requirements
    // - Hard requirements for main thread: returns_sexp, has_sexp_inputs, has_dots, check_interrupt
    // - force_main_thread is a user request (can be combined with force_worker, but main_thread wins)
    let requires_main_thread = returns_sexp || has_sexp_inputs || has_dots || check_interrupt;
    let use_main_thread = requires_main_thread || (force_main_thread && !force_worker);

    // Suppress unused variable warning when force_worker doesn't affect strategy
    let _ = force_worker;
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
            // RNG variant: wrap in catch_unwind so we can call PutRNGstate before error handling
            let rng_panic_handler = if error_in_r {
                quote::quote! {
                    ::miniextendr_api::error_value::make_rust_error_value(
                        &::miniextendr_api::worker::panic_payload_to_string(&payload),
                        "panic",
                    )
                }
            } else {
                quote::quote! { ::std::panic::resume_unwind(payload) }
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
        // Pure Rust functions: use worker thread strategy
        let c_wrapper_doc = format!(
            "C wrapper for [`{}`] (worker thread). See [`{}`] for R wrapper.",
            rust_ident, r_wrapper_generator
        );
        let worker_panic_handler = if error_in_r {
            quote::quote! {
                ::miniextendr_api::error_value::make_rust_error_value(
                    &::miniextendr_api::worker::panic_payload_to_string(&payload),
                    "panic",
                )
            }
        } else {
            quote::quote! {
                ::miniextendr_api::worker::panic_message_to_r_error(
                    ::miniextendr_api::worker::panic_payload_to_string(&payload)
                )
            }
        };
        if error_in_r {
            // error_in_r: use run_on_worker_result to get Result<T, String> instead of diverging
            quote::quote! {
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

                        match ::miniextendr_api::worker::run_on_worker_result(move || {
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
                                    &__panic_msg, "panic",
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
            quote::quote! {
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

                        let #rust_result_ident = ::miniextendr_api::worker::run_on_worker(move || {
                            #(#in_closure_stmts)*
                            let #rust_result_ident = #rust_ident(#(#rust_inputs),*);
                            #(#post_call_statements)*
                            #rust_result_ident
                        });

                        #unwind_protect_fn(
                            move || #return_expression,
                            None,
                        )
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
    // Add user-specified parameter defaults, merged with auto-defaults for Missing<T> params
    let mut merged_defaults =
        r_wrapper_builder::merge_missing_defaults(inputs, parsed.param_defaults());
    // Add NULL default for match_arg params that don't already have an explicit default
    for match_arg_param in parsed.match_arg_params() {
        let r_name = r_wrapper_builder::normalize_r_arg_ident(
            &syn::Ident::new(match_arg_param, proc_macro2::Span::call_site()),
        )
        .to_string();
        merged_defaults.entry(r_name).or_insert_with(|| "NULL".to_string());
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
    let export_comment = if matches!(vis, syn::Visibility::Public(_))
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
    // Collect (r_param_name, rust_type) for each match_arg param
    let match_arg_param_info: Vec<(String, &syn::Type)> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let rust_name = pat_ident.ident.to_string();
                if parsed.has_match_arg_attr(&rust_name) {
                    let r_name =
                        r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();
                    return Some((r_name, pt.ty.as_ref()));
                }
            }
            None
        })
        .collect();

    let match_arg_prelude = if match_arg_param_info.is_empty() {
        String::new()
    } else {
        let mut lines = Vec::new();
        for (r_param, _) in &match_arg_param_info {
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
            // match.arg validation
            lines.push(format!(
                "{param} <- base::match.arg({param}, .__mx_choices_{param})",
                param = r_param,
            ));
        }
        lines.join("\n  ")
    };

    // Generate lifecycle prelude if needed
    let lifecycle_prelude = lifecycle_spec
        .as_ref()
        .and_then(|spec| spec.r_prelude(&r_wrapper_ident_str));

    // Generate R-side precondition checks (stopifnot + fallback precheck calls)
    let precondition_output =
        r_preconditions::build_precondition_checks(inputs, parsed.match_arg_params());
    let precondition_prelude = if precondition_output.static_checks.is_empty() {
        String::new()
    } else {
        precondition_output.static_checks.join("\n  ")
    };

    // Combine all preludes: lifecycle, static preconditions, match.arg
    let combined_prelude = {
        let mut parts = Vec::new();
        if let Some(ref lc) = lifecycle_prelude {
            parts.push(lc.as_str());
        }
        if !precondition_prelude.is_empty() {
            parts.push(&precondition_prelude);
        }
        if !match_arg_prelude.is_empty() {
            parts.push(&match_arg_prelude);
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
    let r_wrapper_str: proc_macro2::TokenStream = {
        use std::str::FromStr;
        // Indent each line by 2 spaces for nicer formatting
        let indented = r_wrapper_string.replace('\n', "\n  ");
        let raw = format!("r#\"\n  {}\n\"#", indented);
        proc_macro2::TokenStream::from_str(&raw).expect("valid raw string literal")
    };

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
        .map(|(r_param, param_ty)| {
            let helper_fn_name = format!(
                "C_{}__match_arg_choices__{}",
                c_ident.to_string().trim_start_matches("C_"),
                r_param
            );
            let helper_fn_ident =
                syn::Ident::new(&helper_fn_name, proc_macro2::Span::call_site());
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
                    ::miniextendr_api::choices_sexp::<#param_ty>()
                }

                #(#cfg_attrs)*
                #[allow(non_upper_case_globals)]
                #[allow(non_snake_case)]
                const #helper_def_ident: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
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
    let match_arg_count = match_arg_helper_def_idents.len();

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

        // R wrapper
        #(#cfg_attrs)*
        #[doc = #r_wrapper_doc]
        #[doc = concat!("Wraps Rust function `", stringify!(#rust_ident), "`.")]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        const #r_wrapper_generator: &str =
            concat!(
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
            );

        // registration of C wrapper in R
        #(#cfg_attrs)*
        #[doc = #call_method_def_doc]
        #[doc = #call_method_def_example]
        #[doc = concat!("Wraps Rust function `", stringify!(#rust_ident), "`.")]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        const #call_method_def: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
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
        #(#match_arg_helpers)*

        // match_arg call defs array (collected by miniextendr_module!)
        #(#cfg_attrs)*
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        const #match_arg_call_defs: [::miniextendr_api::ffi::R_CallMethodDef; #match_arg_count] =
            [#(#match_arg_helper_def_idents),*];

        // doc-lint warnings (if any)
        #doc_lint_warnings
    }
    .into();

    expanded
}

/// Register functions and ALTREP types with R's dynamic symbol registration.
///
/// This macro generates the `R_init_<module>_miniextendr` entrypoint that R calls
/// when loading the shared library.
///
/// # Syntax
///
/// ```ignore
/// miniextendr_module! {
///     mod mymodule;
///
///     // Functions annotated with #[miniextendr]
///     fn my_function;
///     extern "C-unwind" fn C_my_raw_function;
///
///     // ALTREP types (registers the class with R)
///     struct MyAltrepClass;
///
///     // Impl blocks (class systems)
///     impl MyType;
///
///     // Trait impls (ABI dispatch)
///     impl Counter for MyType;
///
///     // Re-export from submodules
///     use submodule;
/// }
/// ```
///
/// # Function Registration
///
/// Functions listed here must be defined with the `#[miniextendr]` attribute.
/// The macro looks up the generated `CALL_METHOD_DEF_<name>` constant that
/// `#[miniextendr]` creates for each function.
///
/// The distinction between Rust ABI and C ABI functions is handled by
/// `#[miniextendr]` at the function definition site, not in this module declaration:
///
/// - **Rust ABI** (`fn foo(...)`): `#[miniextendr]` generates a `C_foo` wrapper
/// - **C ABI** (`extern "C-unwind" fn foo(...)`): `#[miniextendr]` uses the function directly,
///   and the R wrapper is prefixed with `unsafe_`
///
/// Both are listed the same way in `miniextendr_module!`:
///
/// ```ignore
/// miniextendr_module! {
///     mod mypackage;
///     fn rust_function;    // refers to #[miniextendr] fn rust_function
///     fn c_function;       // refers to #[miniextendr] extern "C-unwind" fn c_function
/// }
/// ```
///
/// # ALTREP Registration
///
/// Structs listed are registered as ALTREP classes during `R_init_*`.
/// The struct must implement the appropriate ALTREP traits.
///
/// # Impl and trait impl registration
///
/// - `impl Type;` registers a `#[miniextendr(...)] impl Type` block (env/S3/S4/S7/R6).
/// - `impl Trait for Type;` registers ABI wrappers for cross-package trait dispatch.
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn add(a: i32, b: i32) -> i32 { a + b }
///
/// #[miniextendr]
/// #[unsafe(no_mangle)]
/// extern "C-unwind" fn C_fast_add(a: SEXP, b: SEXP) -> SEXP { /* ... */ }
///
/// miniextendr_module! {
///     mod mypackage;
///     fn add;
///     extern "C-unwind" fn C_fast_add;
/// }
/// ```
#[proc_macro]
pub fn miniextendr_module(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_module = syn::parse_macro_input!(item as MiniextendrModule);

    let module = &parsed_module.module_name.ident;
    let module_entrypoint_ident = quote::format_ident!("R_init_{module}_miniextendr");
    let module_source_loc_doc = source_location_doc(module.span());
    // Build call entries with their cfg attributes preserved
    let call_entries_with_attrs: Vec<(Vec<syn::Attribute>, syn::Expr)> = parsed_module
        .functions
        .iter()
        .map(|f| {
            let call_method_def = f.call_method_def_ident();
            let cfg_attrs = extract_cfg_attrs(&f.attrs);
            (cfg_attrs, syn::parse_quote!(#call_method_def))
        })
        .collect();
    let _call_entries: Vec<syn::Expr> = call_entries_with_attrs
        .iter()
        .map(|(_, expr)| expr.clone())
        .collect();

    // Count entries without cfg attributes (always included)
    let unconfigured_entries_len = call_entries_with_attrs
        .iter()
        .filter(|(cfg_attrs, _)| cfg_attrs.is_empty())
        .count();

    // Generate conditional length constants for entries with cfg attributes
    let cfg_len_consts: Vec<proc_macro2::TokenStream> = call_entries_with_attrs
        .iter()
        .enumerate()
        .filter(|(_, (cfg_attrs, _))| !cfg_attrs.is_empty())
        .map(|(i, (cfg_attrs, _))| {
            let const_name = quote::format_ident!("__CFG_FN_LEN_{}", i);
            // Generate both cfg and not(cfg) variants
            let negated_attrs: Vec<proc_macro2::TokenStream> = cfg_attrs
                .iter()
                .map(|attr| {
                    // Extract the meta from the cfg attribute and negate it
                    let meta = &attr.meta;
                    quote::quote!(#[cfg(not #meta)])
                })
                .collect();
            quote::quote! {
                #(#cfg_attrs)*
                const #const_name: usize = 1;
                #(#negated_attrs)*
                const #const_name: usize = 0;
            }
        })
        .collect();

    // Build the length expression: base + sum of conditional constants
    let cfg_len_idents: Vec<syn::Ident> = call_entries_with_attrs
        .iter()
        .enumerate()
        .filter(|(_, (cfg_attrs, _))| !cfg_attrs.is_empty())
        .map(|(i, _)| quote::format_ident!("__CFG_FN_LEN_{}", i))
        .collect();

    let call_entries_len = unconfigured_entries_len;

    // Generate ALTREP registrations for struct items (if they implement RegisterAltrep)
    let altrep_regs: Vec<syn::Expr> = parsed_module
        .structs
        .iter()
        .map(|s| {
            let ty = &s.ident;
            syn::parse_quote!(<#ty as ::miniextendr_api::altrep_registration::RegisterAltrep>::get_or_init_class())
        })
        .collect();

    // Generate impl block call defs for registration with cfg attributes
    let impl_call_defs_with_attrs: Vec<(Vec<syn::Attribute>, syn::Expr)> = parsed_module
        .impls
        .iter()
        .map(|i| {
            let call_defs_static = i.call_defs_const_ident();
            let cfg_attrs = extract_cfg_attrs(&i.attrs);
            (cfg_attrs, syn::parse_quote!(#call_defs_static))
        })
        .collect();
    let impl_call_defs: Vec<syn::Expr> = impl_call_defs_with_attrs
        .iter()
        .map(|(_, expr)| expr.clone())
        .collect();

    // Generate sidecar call defs for registration (from #[derive(ExternalPtr)] with #[r_data])
    // These are generated even if empty - the constants exist but may be zero-length arrays
    let rdata_call_defs: Vec<syn::Expr> = parsed_module
        .impls
        .iter()
        .map(|i| {
            let call_defs_static = i.rdata_call_defs_const_ident();
            syn::parse_quote!(#call_defs_static)
        })
        .collect();

    // Generate match_arg choices helper call defs for registration (per-function)
    // Every #[miniextendr] function generates a MATCH_ARG_CALL_DEFS_* array (empty if no match_arg params)
    let match_arg_call_defs: Vec<syn::Expr> = parsed_module
        .functions
        .iter()
        .map(|f| {
            let call_defs_static = f.match_arg_call_defs_ident();
            syn::parse_quote!(#call_defs_static)
        })
        .collect();

    // Generate impl R wrapper refs with cfg attributes
    let impl_r_wrappers_with_cfg: Vec<(Vec<syn::Attribute>, syn::Expr)> = parsed_module
        .impls
        .iter()
        .map(|i| {
            let r_wrapper_const = i.r_wrappers_const_ident();
            let cfg_attrs = extract_cfg_attrs(&i.attrs);
            (cfg_attrs, syn::parse_quote!(#r_wrapper_const))
        })
        .collect();

    // Generate sidecar R wrapper refs (from #[derive(ExternalPtr)] with #[r_data])
    let rdata_r_wrappers: Vec<syn::Expr> = parsed_module
        .impls
        .iter()
        .map(|i| {
            let r_wrapper_const = i.rdata_r_wrappers_const_ident();
            syn::parse_quote!(#r_wrapper_const)
        })
        .collect();

    // Generate vctrs R wrapper refs (from #[derive(Vctrs)])
    #[cfg(feature = "vctrs")]
    let vctrs_r_wrappers_with_cfg: Vec<(Vec<syn::Attribute>, syn::Expr)> = parsed_module
        .vctrs
        .iter()
        .map(|v| {
            let r_wrapper_const = v.r_wrappers_const_ident();
            let cfg_attrs = extract_cfg_attrs(&v.attrs);
            (cfg_attrs, syn::parse_quote!(#r_wrapper_const))
        })
        .collect();
    #[cfg(not(feature = "vctrs"))]
    let vctrs_r_wrappers_with_cfg: Vec<(Vec<syn::Attribute>, syn::Expr)> = Vec::new();

    // Separate ALTREP trait impls from regular cross-package trait impls
    let (altrep_impls, regular_trait_impls) =
        altrep_module::extract_altrep_impls(&parsed_module.trait_impls);

    // Generate ALTREP code (low-level traits, registration, IntoR)
    let (altrep_generated_code, altrep_registration_exprs) =
        altrep_module::generate_altrep_code(&altrep_impls);

    // Generate trait impl call defs for registration with cfg attributes
    // (only for regular trait impls, not ALTREP)
    let trait_impl_call_defs_with_attrs: Vec<(Vec<syn::Attribute>, syn::Expr)> =
        regular_trait_impls
            .iter()
            .map(|ti| {
                let call_defs_static = ti.call_defs_const_ident();
                let cfg_attrs = extract_cfg_attrs(&ti.attrs);
                (cfg_attrs, syn::parse_quote!(#call_defs_static))
            })
            .collect();
    let trait_impl_call_defs: Vec<syn::Expr> = trait_impl_call_defs_with_attrs
        .iter()
        .map(|(_, expr)| expr.clone())
        .collect();

    // Generate trait impl R wrapper refs with cfg attributes
    // (only for regular trait impls, not ALTREP)
    let trait_impl_r_wrappers_with_cfg: Vec<(Vec<syn::Attribute>, syn::Expr)> = regular_trait_impls
        .iter()
        .map(|ti| {
            let r_wrapper_const = ti.r_wrappers_const_ident();
            let cfg_attrs = extract_cfg_attrs(&ti.attrs);
            (cfg_attrs, syn::parse_quote!(#r_wrapper_const))
        })
        .collect();

    // Get CALL_ENTRIES const arrays from child modules (via `use`)
    let use_module_call_entries_with_cfg: Vec<(Vec<syn::Attribute>, syn::Expr)> = parsed_module
        .uses
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_ident_upper = use_module_ident.to_string().to_uppercase();
            let call_entries_const = quote::format_ident!("CALL_ENTRIES_{use_module_ident_upper}");
            let cfg_attrs = extract_cfg_attrs(&x.attrs);
            (cfg_attrs, syn::parse_quote!(#use_module_ident::#call_entries_const))
        })
        .collect();

    // Call ALTREP registration from child modules (via `use`)
    let use_module_altrep_regs: Vec<proc_macro2::TokenStream> = parsed_module
        .uses
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let altrep_reg_fn = quote::format_ident!("{use_module_ident}_register_altrep");
            let cfg_attrs = extract_cfg_attrs(&x.attrs);
            quote::quote!(#(#cfg_attrs)* #use_module_ident::#altrep_reg_fn())
        })
        .collect();

    // region: r wrapper generation in `mod`

    // R wrapper generators with cfg attributes preserved
    let r_wrapper_generators_with_cfg: Vec<(Vec<syn::Attribute>, syn::Expr)> = parsed_module
        .functions
        .iter()
        .map(|x| {
            let r_wrapper_const = x.r_wrapper_const_ident();
            let cfg_attrs = extract_cfg_attrs(&x.attrs);
            (cfg_attrs, syn::parse_quote!(#r_wrapper_const))
        })
        .collect();

    // Generate conditional R wrapper array elements
    // For functions without cfg: just include the const reference
    // For functions with cfg: prefix with cfg attributes (Rust allows cfg on array elements)
    let r_wrapper_generators: Vec<proc_macro2::TokenStream> = r_wrapper_generators_with_cfg
        .iter()
        .map(|(cfg_attrs, expr)| {
            if cfg_attrs.is_empty() {
                quote::quote!(#expr)
            } else {
                // Cfg attributes on array elements work in Rust:
                // const ARR: &[&str] = &[FOO, #[cfg(feature = "x")] BAR];
                quote::quote!(#(#cfg_attrs)* #expr)
            }
        })
        .collect();
    // Collect child modules' function wrappers (PARTS)
    let r_wrappers_use_other_modules: Vec<proc_macro2::TokenStream> = parsed_module
        .uses
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_ident_upper = use_module_ident.to_string().to_uppercase();
            let r_wrappers_use_module =
                quote::format_ident!("R_WRAPPERS_PARTS_{use_module_ident_upper}");
            let cfg_attrs = extract_cfg_attrs(&x.attrs);
            quote::quote!(#(#cfg_attrs)* #use_module_ident::#r_wrappers_use_module)
        })
        .collect();

    // Collect child modules' impl wrappers (IMPLS)
    let r_wrappers_impl_use_other_modules: Vec<proc_macro2::TokenStream> = parsed_module
        .uses
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_ident_upper = use_module_ident.to_string().to_uppercase();
            let r_wrappers_use_module =
                quote::format_ident!("R_WRAPPERS_IMPLS_{use_module_ident_upper}");
            let cfg_attrs = extract_cfg_attrs(&x.attrs);
            quote::quote!(#(#cfg_attrs)* #use_module_ident::#r_wrappers_use_module)
        })
        .collect();

    let module_upper = module.to_string().to_uppercase();
    let r_wrappers_parts_ident = quote::format_ident!("R_WRAPPERS_PARTS_{module_upper}");
    let r_wrappers_deps_ident = quote::format_ident!("R_WRAPPERS_DEPS_{module_upper}");
    let r_wrappers_impl_deps_ident = quote::format_ident!("R_WRAPPERS_IMPL_DEPS_{module_upper}");

    // Generate doc string listing all registered functions
    let fn_links: Vec<String> = parsed_module
        .functions
        .iter()
        .map(|f| format!("[`{}`]", f.ident))
        .collect();
    let struct_links: Vec<String> = parsed_module
        .structs
        .iter()
        .map(|s| format!("[`{}`]", s.ident))
        .collect();
    let impl_links: Vec<String> = parsed_module
        .impls
        .iter()
        .map(|i| format!("[`{}`]", i.ident))
        .collect();
    let module_doc = if fn_links.is_empty() && struct_links.is_empty() && impl_links.is_empty() {
        format!("R entrypoint for module `{}`.", module)
    } else {
        let mut doc = format!(
            "R entrypoint for module `{}`.\n\n# Registered items\n",
            module
        );
        if !fn_links.is_empty() {
            doc.push_str(&format!("- Functions: {}\n", fn_links.join(", ")));
        }
        if !struct_links.is_empty() {
            doc.push_str(&format!("- ALTREP types: {}\n", struct_links.join(", ")));
        }
        if !impl_links.is_empty() {
            doc.push_str(&format!("- Impl blocks: {}\n", impl_links.join(", ")));
        }
        doc
    };

    // endregion

    // Check if we have impl blocks to register (affects wrapper lists)
    let _has_impls = !parsed_module.impls.is_empty();

    // Generate trait ABI wrapper infrastructure grouped by concrete type.
    // Only for regular trait impls (not ALTREP). Generic types are skipped since
    // cross-package trait dispatch requires named types with ExternalPtr.
    let mut trait_impl_groups: Vec<(syn::Ident, Vec<syn::Path>)> = Vec::new();
    for ti in regular_trait_impls.iter() {
        // Only process simple types (non-generic)
        if let Some(type_ident) = ti.simple_type_ident() {
            if let Some((_, traits)) = trait_impl_groups
                .iter_mut()
                .find(|(ty, _)| ty == type_ident)
            {
                traits.push(ti.trait_path.clone());
            } else {
                trait_impl_groups.push((type_ident.clone(), vec![ti.trait_path.clone()]));
            }
        }
    }
    let trait_impl_wrappers: Vec<proc_macro2::TokenStream> = trait_impl_groups
        .iter()
        .map(|(type_ident, trait_paths)| generate_trait_impl_wrapper(trait_paths, type_ident))
        .collect();

    // R wrapper parts const (includes both functions and impl wrappers)
    let r_wrappers_impls_ident = quote::format_ident!("R_WRAPPERS_IMPLS_{module_upper}");

    // Generate CALL_ENTRIES constant name
    let call_entries_const_ident = quote::format_ident!("CALL_ENTRIES_{module_upper}");

    // Generate call_entries accessor function name (returns &[R_CallMethodDef])
    let call_entries_fn_ident = quote::format_ident!("{module}_call_entries");

    // Generate ALTREP registration function name
    let altrep_reg_fn_ident = quote::format_ident!("{module}_register_altrep");

    // Build const call entries array, including impl call defs and trait impl call defs.
    // Use explicit usize suffix to avoid type inference issues with many additions
    let call_entries_len_lit = syn::LitInt::new(
        &format!("{}usize", call_entries_len),
        proc_macro2::Span::call_site(),
    );
    // Use UFCS to call slice's inherent len() instead of RToVec::len() which returns i64
    let impl_call_defs_len_exprs: Vec<proc_macro2::TokenStream> = parsed_module
        .impls
        .iter()
        .map(|i| {
            let call_defs_static = i.call_defs_const_ident();
            quote::quote!(<[_]>::len(&#call_defs_static))
        })
        .collect();
    // Only include regular trait impls (not ALTREP) in call defs length
    let trait_impl_call_defs_len_exprs: Vec<proc_macro2::TokenStream> = regular_trait_impls
        .iter()
        .map(|ti| {
            let call_defs_static = ti.call_defs_const_ident();
            quote::quote!(<[_]>::len(&#call_defs_static))
        })
        .collect();

    // Sidecar call defs length (from #[derive(ExternalPtr)] with #[r_data])
    let rdata_call_defs_len_exprs: Vec<proc_macro2::TokenStream> = parsed_module
        .impls
        .iter()
        .map(|i| {
            let call_defs_static = i.rdata_call_defs_const_ident();
            quote::quote!(<[_]>::len(&#call_defs_static))
        })
        .collect();

    // Match-arg choices helper call defs length (per-function)
    let match_arg_call_defs_len_exprs: Vec<proc_macro2::TokenStream> = parsed_module
        .functions
        .iter()
        .map(|f| {
            let call_defs_static = f.match_arg_call_defs_ident();
            quote::quote!(<[_]>::len(&#call_defs_static))
        })
        .collect();

    // Precheck fallback helper call defs length (per-function)
    // Calculate total length expression, including conditional cfg lengths
    let cfg_len_exprs: Vec<proc_macro2::TokenStream> = cfg_len_idents
        .iter()
        .map(|ident| quote::quote!(#ident))
        .collect();
    let all_len_exprs: Vec<proc_macro2::TokenStream> =
        std::iter::once(quote::quote!(#call_entries_len_lit))
            .chain(cfg_len_exprs.iter().cloned())
            .chain(impl_call_defs_len_exprs.iter().cloned())
            .chain(trait_impl_call_defs_len_exprs.iter().cloned())
            .chain(rdata_call_defs_len_exprs.iter().cloned())
            .chain(match_arg_call_defs_len_exprs.iter().cloned())
            .collect();
    let total_len_expr = if all_len_exprs.is_empty()
        || (call_entries_len == 0
            && cfg_len_idents.is_empty()
            && impl_call_defs_len_exprs.is_empty()
            && trait_impl_call_defs_len_exprs.is_empty()
            && rdata_call_defs_len_exprs.is_empty()
            && match_arg_call_defs_len_exprs.is_empty())
    {
        quote::quote!(0usize)
    } else {
        quote::quote!(#(#all_len_exprs)+*)
    };

    // Generate conditional call entry assignment statements
    let call_entry_assignments: Vec<proc_macro2::TokenStream> = call_entries_with_attrs
        .iter()
        .map(|(cfg_attrs, expr)| {
            quote::quote! {
                #(#cfg_attrs)*
                {
                    entries[idx] = #expr;
                    idx += 1;
                }
            }
        })
        .collect();

    let call_entries_storage = quote::quote! {
        /// This module's call entries (excluding children).
        #[doc(hidden)]
        pub(crate) const #call_entries_const_ident: [::miniextendr_api::ffi::R_CallMethodDef; #total_len_expr] = {
            const EMPTY: ::miniextendr_api::ffi::R_CallMethodDef = ::miniextendr_api::ffi::R_CallMethodDef {
                name: std::ptr::null(),
                fun: None,
                numArgs: 0,
            };
            let mut entries = [EMPTY; #total_len_expr];
            let mut idx: usize = 0;
            #(#call_entry_assignments)*
            #(
                let mut j: usize = 0;
                let slice = &#impl_call_defs;
                while j < <[_]>::len(slice) {
                    entries[idx] = slice[j];
                    idx += 1;
                    j += 1;
                }
            )*
            #(
                let mut j: usize = 0;
                let slice = &#trait_impl_call_defs;
                while j < <[_]>::len(slice) {
                    entries[idx] = slice[j];
                    idx += 1;
                    j += 1;
                }
            )*
            #(
                let mut j: usize = 0;
                let slice = &#rdata_call_defs;
                while j < <[_]>::len(slice) {
                    entries[idx] = slice[j];
                    idx += 1;
                    j += 1;
                }
            )*
            #(
                let mut j: usize = 0;
                let slice = &#match_arg_call_defs;
                while j < <[_]>::len(slice) {
                    entries[idx] = slice[j];
                    idx += 1;
                    j += 1;
                }
            )*
            entries
        };

        /// Returns this module's call entries as a slice.
        fn #call_entries_fn_ident() -> &'static [::miniextendr_api::ffi::R_CallMethodDef] {
            &#call_entries_const_ident
        }
    };

    // Build a combined const array including child modules and a sentinel.
    //
    // For cfg-gated use entries, generate paired const declarations (like __CFG_FN_LEN_*)
    // so the array size is always valid regardless of which features are enabled.
    let cfg_use_len_consts: Vec<proc_macro2::TokenStream> = use_module_call_entries_with_cfg
        .iter()
        .enumerate()
        .filter(|(_, (cfg_attrs, _))| !cfg_attrs.is_empty())
        .map(|(i, (cfg_attrs, expr))| {
            let const_name = quote::format_ident!("__CFG_USE_LEN_{}", i);
            let negated_attrs: Vec<proc_macro2::TokenStream> = cfg_attrs
                .iter()
                .map(|attr| {
                    let meta = &attr.meta;
                    quote::quote!(#[cfg(not #meta)])
                })
                .collect();
            quote::quote! {
                #(#cfg_attrs)*
                const #const_name: usize = <[_]>::len(&#expr);
                #(#negated_attrs)*
                const #const_name: usize = 0;
            }
        })
        .collect();

    // Length expressions for unconfigured use entries (always present)
    let uncfg_use_len_exprs: Vec<proc_macro2::TokenStream> = use_module_call_entries_with_cfg
        .iter()
        .filter(|(cfg_attrs, _)| cfg_attrs.is_empty())
        .map(|(_, expr)| quote::quote!(<[_]>::len(&#expr)))
        .collect();

    // Length idents for cfg-gated use entries
    let cfg_use_len_idents: Vec<syn::Ident> = use_module_call_entries_with_cfg
        .iter()
        .enumerate()
        .filter(|(_, (cfg_attrs, _))| !cfg_attrs.is_empty())
        .map(|(i, _)| quote::format_ident!("__CFG_USE_LEN_{}", i))
        .collect();

    // Combine all use-module length addends
    let all_use_len_exprs: Vec<proc_macro2::TokenStream> = uncfg_use_len_exprs
        .into_iter()
        .chain(cfg_use_len_idents.iter().map(|ident| quote::quote!(#ident)))
        .collect();

    let all_call_entries_const_ident = quote::format_ident!("ALL_CALL_ENTRIES_{module_upper}");
    let all_entries_len_expr = if all_use_len_exprs.is_empty() {
        quote::quote!(#total_len_expr + 1usize)
    } else {
        quote::quote!(#total_len_expr + #(#all_use_len_exprs)+* + 1usize)
    };

    // Generate child-module copy blocks, each wrapped with cfg attrs if needed
    let use_module_copy_blocks: Vec<proc_macro2::TokenStream> = use_module_call_entries_with_cfg
        .iter()
        .map(|(cfg_attrs, expr)| {
            let copy_block = quote::quote! {
                let mut j: usize = 0;
                let slice = &#expr;
                while j < <[_]>::len(slice) {
                    entries[idx] = slice[j];
                    idx += 1;
                    j += 1;
                }
            };
            if cfg_attrs.is_empty() {
                copy_block
            } else {
                quote::quote! {
                    #(#cfg_attrs)*
                    {
                        #copy_block
                    }
                }
            }
        })
        .collect();

    let all_call_entries_storage = quote::quote! {
        // Conditional length constants for feature-gated child module entries
        #(#cfg_use_len_consts)*

        /// This module's call entries including children, with sentinel.
        #[doc(hidden)]
        const #all_call_entries_const_ident: [::miniextendr_api::ffi::R_CallMethodDef; #all_entries_len_expr] = {
            const EMPTY: ::miniextendr_api::ffi::R_CallMethodDef = ::miniextendr_api::ffi::R_CallMethodDef {
                name: std::ptr::null(),
                fun: None,
                numArgs: 0,
            };
            let mut entries = [EMPTY; #all_entries_len_expr];
            let mut idx: usize = 0;

            // Local entries
            let mut j: usize = 0;
            let slice = &#call_entries_const_ident;
            while j < <[_]>::len(slice) {
                entries[idx] = slice[j];
                idx += 1;
                j += 1;
            }

            // Child module entries
            #(#use_module_copy_blocks)*

            // Sentinel
            entries[idx] = ::miniextendr_api::ffi::R_CallMethodDef {
                name: std::ptr::null(),
                fun: None,
                numArgs: 0,
            };

            entries
        };
    };

    // Check if we have trait impl blocks to register
    let _has_trait_impls = !parsed_module.trait_impls.is_empty();

    // Generate R wrapper impls constant - includes impl, trait impl, sidecar, and vctrs wrappers
    // Combine all with cfg info and generate conditional array elements
    let mut all_impl_r_wrappers_with_cfg: Vec<(Vec<syn::Attribute>, syn::Expr)> = Vec::new();
    // Sidecar R wrappers FIRST (from #[derive(ExternalPtr)] with #[r_data])
    // These must come before impl wrappers because R6/S7 class definitions
    // may reference .rdata_active_bindings_* / .rdata_properties_* helpers.
    all_impl_r_wrappers_with_cfg.extend(
        rdata_r_wrappers
            .iter()
            .map(|expr| (Vec::new(), expr.clone())),
    );
    all_impl_r_wrappers_with_cfg.extend(impl_r_wrappers_with_cfg.iter().cloned());
    all_impl_r_wrappers_with_cfg.extend(trait_impl_r_wrappers_with_cfg.iter().cloned());
    // Add vctrs R wrappers (from #[derive(Vctrs)])
    all_impl_r_wrappers_with_cfg.extend(vctrs_r_wrappers_with_cfg.iter().cloned());

    // Generate array elements with cfg attributes where needed
    let all_impl_r_wrapper_elements: Vec<proc_macro2::TokenStream> = all_impl_r_wrappers_with_cfg
        .iter()
        .map(|(cfg_attrs, expr)| {
            if cfg_attrs.is_empty() {
                quote::quote!(#expr)
            } else {
                quote::quote!(#(#cfg_attrs)* #expr)
            }
        })
        .collect();

    let r_wrappers_impls_const = if all_impl_r_wrapper_elements.is_empty() {
        quote::quote! {
            #[doc(hidden)]
            #[doc = #module_source_loc_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            pub const #r_wrappers_impls_ident: &[&str] = &[];
        }
    } else {
        quote::quote! {
            #[doc(hidden)]
            #[doc = #module_source_loc_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            pub const #r_wrappers_impls_ident: &[&str] = &[#(#all_impl_r_wrapper_elements),*];
        }
    };

    // Generate the module - common structure for both cases
    quote::quote! {
        #[doc(hidden)]
        #[doc = #module_source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        pub const #r_wrappers_parts_ident: &[&str] = &[#(#r_wrapper_generators),*];
        #r_wrappers_impls_const
        #[doc(hidden)]
        #[doc = #module_source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        pub const #r_wrappers_deps_ident: &[&[&str]] = &[#(#r_wrappers_use_other_modules),*];
        #[doc(hidden)]
        #[doc = #module_source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        pub const #r_wrappers_impl_deps_ident: &[&[&str]] = &[#(#r_wrappers_impl_use_other_modules),*];

        // Conditional length constants for feature-gated function entries
        #(#cfg_len_consts)*

        #call_entries_storage
        #all_call_entries_storage

        // Trait ABI wrapper infrastructure
        #(#trait_impl_wrappers)*

        // ALTREP trait implementations generated from `impl AltXxxData for Type;`
        #altrep_generated_code

        /// Register ALTREP classes declared in this module.
        #[doc = #module_source_loc_doc]
        pub(crate) fn #altrep_reg_fn_ident() {
            // From `struct Type;` declarations (old style)
            #(#altrep_regs;)*
            // From `impl AltXxxData for Type;` declarations (new style)
            #(#altrep_registration_exprs;)*
        }

        #[doc = #module_doc]
        #[doc = #module_source_loc_doc]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        extern "C-unwind" fn #module_entrypoint_ident(dll: *mut ::miniextendr_api::ffi::DllInfo) {
            // Register ALTREP classes from this module
            #altrep_reg_fn_ident();

            // Register ALTREP classes from child modules
            #(#use_module_altrep_regs;)*

            unsafe {
                ::miniextendr_api::ffi::R_registerRoutines_unchecked(
                    dll,
                    std::ptr::null(),
                    #all_call_entries_const_ident.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null()
                );
                // R_useDynamicSymbols and R_forceSymbols are called in entrypoint.c
            }
        }
    }
    .into()
}

/// Generate wrapper infrastructure for a trait implementation.
///
/// For `impl Counter for SimpleCounter;` generates:
/// - `__MxWrapperSimpleCounter` - Type-erased wrapper struct
/// - `__MX_BASE_VTABLE_SIMPLECOUNTER` - Base vtable
/// - `__mx_wrap_simplecounter()` - Constructor
///
/// If multiple `impl Trait for Type;` entries exist for the same concrete type,
/// the generated query function includes all listed traits.
fn generate_trait_impl_wrapper(
    trait_paths: &[syn::Path],
    type_ident: &syn::Ident,
) -> proc_macro2::TokenStream {
    let type_upper = type_ident.to_string().to_uppercase();
    let type_lower = type_ident.to_string().to_lowercase();

    // Generate identifiers
    let wrapper_name = quote::format_ident!("__MxWrapper{}", type_ident);
    let base_vtable_name = quote::format_ident!("__MX_BASE_VTABLE_{}", type_upper);
    let concrete_tag_name = quote::format_ident!("__MX_TAG_{}", type_upper);
    let drop_fn_name = quote::format_ident!("__mx_drop_{}", type_lower);
    let query_fn_name = quote::format_ident!("__mx_query_{}", type_lower);
    let wrap_fn_name = quote::format_ident!("__mx_wrap_{}", type_lower);
    let source_loc_doc = source_location_doc(type_ident.span());

    // Generate tag path string for hashing
    let tag_path = format!("::{}", type_ident);

    // Generate query branches for each trait
    let query_branches: Vec<proc_macro2::TokenStream> = trait_paths
        .iter()
        .map(|trait_path| {
            let trait_name = trait_path
                .segments
                .last()
                .map(|s| &s.ident)
                .expect("trait path has at least one segment");
            let trait_name_upper = trait_name.to_string().to_uppercase();

            // Build TAG path - same module as trait
            // Strip generic args (e.g., RExtend<i32> → TAG_REXTEND, not TAG_REXTEND<i32>)
            let mut trait_tag_path = trait_path.clone();
            if let Some(last) = trait_tag_path.segments.last_mut() {
                last.ident = quote::format_ident!("TAG_{}", trait_name_upper);
                last.arguments = syn::PathArguments::None;
            }

            // Build vtable static name: __VTABLE_{TRAIT}_FOR_{TYPE}
            let vtable_name =
                quote::format_ident!("__VTABLE_{}_FOR_{}", trait_name_upper, type_upper);

            quote::quote! {
                if trait_tag == #trait_tag_path {
                    return &#vtable_name as *const _ as *const ::std::os::raw::c_void;
                }
            }
        })
        .collect();

    quote::quote! {
        #[doc = concat!(
            "Type-erased wrapper for `",
            stringify!(#type_ident),
            "` with trait dispatch support."
        )]
        #[doc = "Generated by `miniextendr_module!`."]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[repr(C)]
        #[doc(hidden)]
        struct #wrapper_name {
            /// Type-erased header. Must be first field for `mx_erased` compatibility.
            pub erased: ::miniextendr_api::abi::mx_erased,
            /// The actual data.
            pub data: #type_ident,
        }

        #[doc = concat!(
            "Concrete type tag for `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        const #concrete_tag_name: ::miniextendr_api::abi::mx_tag =
            ::miniextendr_api::abi::mx_tag_from_path(concat!(module_path!(), #tag_path));

        #[doc = concat!(
            "Drop function for `",
            stringify!(#type_ident),
            "` wrapper."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        unsafe extern "C" fn #drop_fn_name(ptr: *mut ::miniextendr_api::abi::mx_erased) {
            if ptr.is_null() {
                return;
            }
            let wrapper = ptr as *mut #wrapper_name;
            unsafe { drop(Box::from_raw(wrapper)); }
        }

        #[doc = concat!(
            "Query function for `",
            stringify!(#type_ident),
            "` trait dispatch."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        unsafe extern "C" fn #query_fn_name(
            _ptr: *mut ::miniextendr_api::abi::mx_erased,
            trait_tag: ::miniextendr_api::abi::mx_tag,
        ) -> *const ::std::os::raw::c_void {
            #(#query_branches)*
            ::std::ptr::null()
        }

        #[doc = concat!(
            "Base vtable for `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        static #base_vtable_name: ::miniextendr_api::abi::mx_base_vtable =
            ::miniextendr_api::abi::mx_base_vtable {
                drop: #drop_fn_name,
                concrete_tag: #concrete_tag_name,
                query: #query_fn_name,
                data_offset: ::std::mem::offset_of!(#wrapper_name, data),
            };

        #[doc = concat!(
            "Create a type-erased wrapper for `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        fn #wrap_fn_name(data: #type_ident) -> *mut ::miniextendr_api::abi::mx_erased {
            let wrapper = Box::new(#wrapper_name {
                erased: ::miniextendr_api::abi::mx_erased {
                    base: &#base_vtable_name,
                },
                data,
            });
            Box::into_raw(wrapper) as *mut ::miniextendr_api::abi::mx_erased
        }
    }
}

/// Generate thread-safe wrappers for R FFI functions.
///
/// Apply this to an `extern "C-unwind"` block to generate wrappers that ensure
/// R API calls happen on R's main thread.
///
/// # Behavior by Return Type
///
/// The wrapper behavior depends on the return type:
///
/// ## Value-returning functions
/// Functions returning values (SEXP, i32, etc.) are automatically routed to
/// the main thread via `with_r_thread` when called from a worker thread.
///
/// ## Pointer-returning functions
/// Functions returning raw pointers (`*const T`, `*mut T`) **cannot be routed**
/// and will **panic** if called from a non-main thread. This is because the
/// pointer could become invalid when R's garbage collector runs on the main thread.
///
/// For pointer-returning APIs (like `INTEGER`, `REAL`), you must:
/// - Call them from the main thread, OR
/// - Use `with_r_thread(|| { ... })` to execute pointer operations on main thread
///   and process results before returning
///
/// # Initialization Requirement
///
/// `miniextendr_worker_init()` must be called before using any wrapped function.
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
///     // Value-returning: automatically routed from worker threads
///     pub fn Rf_ScalarInteger(arg1: i32) -> SEXP;
///
///     // Pointer-returning: panics if called from worker thread
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

    // Extract inner type - must be a newtype (single field)
    let inner_ty: syn::Type = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                fields.unnamed.first().unwrap().ty.clone()
            }
            syn::Fields::Named(fields) if fields.named.len() == 1 => {
                fields.named.first().unwrap().ty.clone()
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
/// To enable trait dispatch wrappers, list trait impls in `miniextendr_module!`:
///
/// ```ignore
/// use miniextendr_api::{miniextendr, miniextendr_module};
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
///
/// miniextendr_module! {
///     mod mypkg;
///     impl Counter for MyCounter;
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
/// #[miniextendr(class = "ConstantInt", pkg = "mypkg")]
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
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
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
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
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
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
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
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
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
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
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
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
#[proc_macro_derive(AltrepList, attributes(altrep))]
pub fn derive_altrep_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_list(input)
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

/// Derive `PrefersList`: opt into list-first IntoR by implementing the marker and IntoR wrapper.
#[proc_macro_derive(PreferList)]
pub fn derive_prefer_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferDataFrame`: opt into data.frame-first IntoR by implementing the marker and IntoR wrapper.
#[proc_macro_derive(PreferDataFrame)]
pub fn derive_prefer_data_frame(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_data_frame(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferExternalPtr`: marks a type as preferring ExternalPtr conversion.
#[proc_macro_derive(PreferExternalPtr)]
pub fn derive_prefer_externalptr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_externalptr(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferRNativeType`: marks a type as preferring native SEXP conversion.
#[proc_macro_derive(PreferRNativeType)]
pub fn derive_prefer_rnative(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_rnative(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `DataFrameRow`: generates a companion DataFrame type with collection fields.
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
/// // And all conversion impls
/// ```
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

#[cfg(test)]
mod tests;
