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
//! extracted). During package build, the wrapper-gen pass loads the installed
//! shared object into R and calls `miniextendr_write_wrappers`, which walks the
//! linkme `#[distributed_slice]` tables and writes `R/<pkg>-wrappers.R`.
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
//! │     ├── C wrapper: extern "C-unwind" fn C_<crate>_<name>(...) → SEXP     │
//! │     ├── R wrapper: static R_WRAPPER_<name>: RWrapperEntry                  │
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
//! │     ├── Vtable instance: static __VTABLE_<crate>_<Trait>_FOR_<Type>     │
//! │     ├── Shared runtime storage: ErasedObject<Type>                     │
//! │     ├── Shared query: registry::universal_query                        │
//! │     └── Base vtable: static __MX_BASE_VTABLE_<Type>: ...                 │
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
//! Every `#[no_mangle]` symbol is prefixed with the consuming crate's name
//! (from `CARGO_CRATE_NAME`) so two packages loaded into one webR session
//! can't collide on a C symbol (#1273; helpers in `naming.rs`). For a
//! function `my_func` in a crate `mypkg`:
//! - C wrapper: `C_mypkg_my_func`
//! - R wrapper registration: `R_WRAPPER_my_func`
//! - Registration: `call_method_def_my_func`
//!
//! For a type `MyType` with trait `Counter` in a crate `mypkg`:
//! - Vtable: `__VTABLE_mypkg_Counter_FOR_MyType`
//! - Storage: runtime `ErasedObject<MyType>`
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
//! By default, `#[miniextendr]` functions run on R's main thread. Opt into
//! worker-thread execution with `#[miniextendr(worker)]` (requires the
//! `worker-thread` feature on `miniextendr-api`). A worker opt-in is ignored
//! when the signature requires main-thread execution (returns/takes `SEXP`,
//! uses variadic dots, or sets `check_interrupt`).
//!
//! **Note**: `ExternalPtr<T>` is `Send` — it can be returned from worker
//! thread functions. All R API operations on `ExternalPtr` are serialized
//! through `with_r_thread`.
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
mod call_method_def;
mod cfg_gates;
mod deprecation;
mod dots_policy;
mod externalptr_args;
mod list_macro;
mod match_arg_keys;
mod miniextendr_fn;
mod receiver_body;
mod return_wrap;
mod s7_conversion;
mod trait_receiver;
mod trait_self_ref;
mod trait_signature;
mod type_inspect;
mod typed_dataframe;
mod typed_list;
mod util;
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
mod condition_derive;
mod dataframe_derive;
mod lifecycle;
mod list_derive;
mod r_class_formatter;
mod r_preconditions;
mod r_shadowing;
mod return_type_analysis;
mod roxygen;

// Trait ABI support modules
mod all_equal_derive;
mod externalptr_derive;
mod miniextendr_impl_trait;
mod miniextendr_trait;
mod typed_external_macro;

// Factor support
mod factor_derive;
mod match_arg_derive;
mod newtype_derive;

// Struct/enum dispatch for #[miniextendr] on structs and enums
mod struct_enum_dispatch;
// Its fallback's diagnostic for a Rust `...` parameter that is not last
mod misplaced_dots;

// r! proc-macro implementation
mod r_macro;

// reexport! proc-macro implementation
mod reexport;

// vctrs support
#[cfg(feature = "vctrs")]
mod vctrs_derive;
mod vctrs_generics;

mod crate_config;
mod naming;
pub(crate) use naming::r_wrapper_const_ident_for;

// Feature default mutual exclusivity guards
#[cfg(all(feature = "r6-default", feature = "s7-default"))]
compile_error!(
    "features \"r6-default\" and \"s7-default\" are mutually exclusive — \
     enable exactly one, or omit both to fall back to the unspecified default"
);
// Note: default-main-thread was removed — main thread is now the hardcoded default.
// worker-default still opts into worker thread execution.

pub(crate) use type_inspect::{
    SeveralOkContainer, classify_several_ok_container, first_type_argument,
    is_main_thread_bound_input, is_main_thread_bound_return, is_sexp_type, match_arg_choices_ty,
    second_type_argument,
};
pub(crate) use util::{
    extract_cfg_attrs, r_wrapper_raw_literal, source_location_doc, source_position,
};

/// Validate the signature of an `extern "C-unwind"` fn exported via `#[miniextendr]`.
///
/// R's `.Call` interface passes all arguments as `SEXP` and expects a `SEXP`
/// return value. For `extern "C-unwind"` functions the user writes the C symbol
/// directly, so the signature must satisfy those invariants statically —
/// otherwise the generated registration produces UB at runtime.
///
/// Called before any codegen so we fail fast on an invalid extern signature
/// rather than emitting a wrapper that would only matter after the error.
fn validate_extern_signature(
    abi: &syn::Abi,
    attrs: &[syn::Attribute],
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    output: &syn::ReturnType,
) -> syn::Result<()> {
    use syn::spanned::Spanned;

    // Reject `self` receivers up front — they are never valid for `.Call`
    // exports, and a missing-return-type error would only hide the real
    // problem (users write `fn foo(self)` to ask "can I export a method?").
    for input in inputs.iter() {
        if let syn::FnArg::Receiver(recv) = input {
            return Err(syn::Error::new_spanned(
                recv,
                "self parameter not allowed in standalone functions; \
                 use #[miniextendr(env|r6|s3|s4|s7)] on impl blocks instead",
            ));
        }
    }

    // Require one of #[no_mangle] / #[unsafe(no_mangle)] / #[export_name].
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
        return Err(syn::Error::new(
            attrs
                .first()
                .map(|attr| attr.span())
                .unwrap_or_else(|| abi.span()),
            "extern \"C-unwind\" functions need a visible C symbol for R's .Call interface. \
             Add one of:\n  \
             - `#[unsafe(no_mangle)]` (Rust 2024 edition)\n  \
             - `#[no_mangle]` (Rust 2021 edition)\n  \
             - `#[export_name = \"my_symbol\"]` (custom symbol name)",
        ));
    }

    // Return type must be SEXP.
    match output {
        non_return_type @ syn::ReturnType::Default => {
            return Err(syn::Error::new(
                non_return_type.span(),
                "extern \"C-unwind\" functions used with #[miniextendr] must return SEXP. \
                 Add `-> miniextendr_api::SEXP` as the return type. \
                 If you want automatic type conversion, remove `extern \"C-unwind\"` and let \
                 the macro generate the C wrapper.",
            ));
        }
        syn::ReturnType::Type(_rarrow, output_type) => match output_type.as_ref() {
            syn::Type::Path(type_path) => {
                if let Some(path_to_sexp) = type_path.path.segments.last().map(|x| &x.ident)
                    && path_to_sexp != "SEXP"
                {
                    return Err(syn::Error::new(
                        path_to_sexp.span(),
                        format!(
                            "extern \"C-unwind\" functions must return SEXP, found `{path_to_sexp}`. \
                             R's .Call interface expects SEXP return values. \
                             Change the return type to `miniextendr_api::SEXP`, or remove \
                             `extern \"C-unwind\"` to let the macro handle type conversion.",
                        ),
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new(
                    output_type.span(),
                    "extern \"C-unwind\" functions must return SEXP. \
                     R's .Call interface expects SEXP return values. \
                     Change the return type to `miniextendr_api::SEXP`, or remove \
                     `extern \"C-unwind\"` to let the macro handle type conversion.",
                ));
            }
        },
    }

    // Every input must be SEXP. Reject variadics and receivers.
    for input in inputs.iter() {
        match input {
            syn::FnArg::Receiver(recv) => {
                return Err(syn::Error::new_spanned(
                    recv,
                    "extern \"C-unwind\" functions cannot have a `self` parameter. \
                     R's .Call interface only accepts SEXP arguments. \
                     Use `#[miniextendr(env|r6|s3|s4|s7)]` on an impl block for methods.",
                ));
            }
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Rest(_) = pat_type.pat.as_ref() {
                    return Err(syn::Error::new_spanned(
                        pat_type,
                        "extern functions cannot use variadic (...) - .Call passes fixed arguments",
                    ));
                }
                let is_sexp = match pat_type.ty.as_ref() {
                    syn::Type::Path(type_path) => type_path
                        .path
                        .segments
                        .last()
                        .is_some_and(|seg| seg.ident == "SEXP"),
                    _ => false,
                };
                if !is_sexp && crate::type_inspect::is_lazy_dots_marker(&pat_type.ty) {
                    return Err(syn::Error::new_spanned(
                        &pat_type.ty,
                        "extern functions cannot take `LazyDots`: it is the generated R \
                         wrapper's frame (`environment()`), and `.Call()` reaches an extern \
                         function directly. Drop the `extern \"C-unwind\"` to let the macro \
                         generate the wrapper",
                    ));
                }
                if !is_sexp {
                    let is_dots_type = if let syn::Type::Reference(type_ref) = pat_type.ty.as_ref()
                    {
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
                    return Err(syn::Error::new_spanned(&pat_type.ty, msg));
                }
            }
        }
    }

    Ok(())
}

/// Builds the `let dots_typed = <dots>.typed(<spec>)...` statement injected
/// at the start of a function body when `#[miniextendr(dots =
/// typed_list!(...))]` is used.
///
/// Raises through `arg_error!` so validation keeps the argument-condition
/// kind, the R parameter name (`...`), and the crate's configured classes.
/// Display formatting preserves TypedListError's human-readable diagnostics.
pub(crate) fn build_dots_validation_stmt(
    dots_param: &syn::Ident,
    spec_tokens: &proc_macro2::TokenStream,
) -> syn::Stmt {
    syn::parse_quote! {
        let dots_typed = #dots_param.typed(#spec_tokens)
            .unwrap_or_else(|e| ::miniextendr_api::arg_error!(
                param = "...", "dots validation failed: {e}"
            ));
    }
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
/// This produces a C wrapper `C_<crate>_add` and an R wrapper `add()`.
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
/// Use `name: ...` as the last argument. It becomes a `name: &Dots`
/// parameter (`args: ...` → `args: &Dots`), through which the body reads
/// the dots. For dots the function ignores, write `_: ...`. rustc rejects a
/// bare `...` before the macro runs (the deny-by-default
/// `varargs_without_pattern` lint) and suggests `_: ...`. The R formal is
/// plain `...` either way.
///
/// An explicit `rest: &Dots` parameter is R's `...` too, at any position:
/// `fn f(x: i32, rest: &Dots, overwrite: bool)` becomes
/// `f <- function(x, ..., overwrite)`, and R matches a formal after `...` by
/// its exact name only. A function takes at most one `...`.
///
/// ### Typed Dots Validation
///
/// Use `#[miniextendr(dots = typed_list!(...))]` to automatically validate dots
/// and create a `dots_typed` variable with typed accessors:
///
/// ```ignore
/// #[miniextendr(dots = typed_list!(x => numeric(), y => integer(), z? => character()))]
/// pub fn my_func(_: ...) -> String {
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
/// - `#[miniextendr(worker)]` — opt into worker-thread execution
/// - `#[miniextendr(invisible)]` / `#[miniextendr(visible)]` — control return visibility
///   (identical to an `Invisible<T>` / `Visible<T>` return type; the two must agree).
///   Only unit-`NULL` returns are invisible by default; receiver-returning
///   method tails are visible unless marked (#1213).
/// - `#[miniextendr(check_interrupt)]` — check for a user interrupt once, before the
///   arguments are converted and the body runs
/// - `#[miniextendr(coerce)]` — coerce R type before conversion (also usable per-parameter)
/// - `#[miniextendr(rng)]` — load R's RNG state before the body and save it after;
///   a `&RngGuard` parameter (no R argument) receives the guard to draw from
/// - `#[miniextendr(rng(seed = <param>))]` — `rng`, plus the seed contract of
///   `stats::simulate()` in the R wrapper (#2198), for standalone functions and
///   `s3(...)` methods. `<param>` names the seed parameter by its Rust name.
///   After the argument checks the wrapper creates `.Random.seed` if it is
///   missing; with a non-`NULL` seed it calls `set.seed(seed)` (which validates
///   it, so the parameter has no R-side type check) and puts the session's
///   `.Random.seed` back on exit, also after an error or a panic. The result
///   gets a `seed` attribute: the seed with `kind = as.list(RNGkind())`, or the
///   pre-draw `.Random.seed` for `seed = NULL`; a value that can't carry
///   attributes is left as it is. A unit return, an impl or trait method, and a
///   name that is no evaluated R argument are compile errors.
/// - `#[miniextendr(serialize)]` — serialize the complete return through `AsSerialize<T>`
///   (requires the API `serde` feature); composes with return visibility markers
/// - `#[miniextendr(strict)]` — reject lossy conversions for i64/u64/isize/usize
/// - `#[miniextendr(unwrap_in_r)]` — return `Result<T, E>` to R without unwrapping
/// - `#[miniextendr(serde_error(tag = "..", prefix = "..", skip(..), rename(a = ".."))]` —
///   options for the serde-classed `Err` arm. The path itself is automatic under the
///   API crate's `serde` feature for every `Result<T, E>` with `E: Serialize + Display`.
///   `as_character("a", ..)`, `as_integer(..)`, `as_double(..)` and `as_logical(..)` give
///   payload fields that atomic R type, as the `serde::as_*` field helpers would from the
///   error type itself; like `skip` and `rename` they name fields as the error serializes
///   them, before any rename. A field takes one `as_*` option and is not also skipped
///   (compile errors, all reported at once). A
///   `miniextendr_api::condition::SerdeErrorShape` const holds any of these options;
///   `miniextendr_api::serde_error_shape!(Error = ENGINE)` declares it once for an error
///   type, and every function or method of the crate returning that type applies it
///   first, with or without this attribute. `shape = PATH` names another one per
///   function: `serde_error(shape = STRICT, as_double("n"))` applies the declared shape,
///   then `STRICT`, then the function's own options; the later wins for a field set more
///   than once. The path resolves from the function's module (`Self::X` on a method
///   names the impl's type); a misspelt one is a compile error.
/// - `#[miniextendr(dots = typed_list!(...))]` — validate dots, create `dots_typed`
/// - `#[miniextendr(dots = empty | empty(allow = [..]) | forward(allow = [..]) | data)]` —
///   what `...` takes; the R wrapper refuses the other arguments before
///   evaluating any, with a "did you mean" suggestion
/// - `#[miniextendr(internal)]` — adds `@keywords internal` to R wrapper
/// - `#[miniextendr(noexport)]` — suppresses `@export` from R wrapper
///
/// ## Declared R formals (`formal`)
///
/// A parameter's R formal is its name without leading underscores. For an R
/// name that is no Rust identifier (`row.names`, `na.rm`, `.data`), declare it
/// with `formal`, on the parameter of a function or with `formal(p = "..")` on
/// an impl method. It combines with the other per-parameter options:
///
/// ```ignore
/// #[miniextendr(s3(generic = "as.data.frame", class = "fit"))]
/// pub fn as_data_frame_fit(
///     x: List,
///     #[miniextendr(formal = "row.names", default = "NULL")] row_names: Option<String>,
///     #[miniextendr(default = "FALSE")] optional: bool,
///     _dots: ...
/// ) -> BuiltDataFrame { /* ... */ }
/// // as.data.frame.fit <- function(x, row.names = NULL, optional = FALSE, ...)
///
/// #[miniextendr(r6)]
/// impl Tally {
///     #[miniextendr(formal(na_rm = "na.rm"), defaults(na_rm = "FALSE"))]
///     pub fn add(&mut self, by: f64, na_rm: bool) -> f64 { /* ... */ }
/// }
/// ```
///
/// Every R-facing name uses the formal: the usage, the `.Call()` arguments,
/// the argument checks and their `e$param` (a failed conversion's too), the
/// `match_arg` prelude and the generated `@param` lines; the method-level
/// options keep the Rust name as their key. The formal must be a syntactic R
/// name, not a reserved word, `...` or `..1`, not a name the generated wrapper
/// uses itself (`.call`, `.val`, `.ptr`, the S3 dispatch variables, names
/// starting `.mx_`, `.miniextendr_`, `.rdata_` or `.__MX_`, a method's
/// receiver), and distinct from the other formals. The dots, an `NArgs`
/// parameter, S7 property accessors and trait methods take no `formal`.
///
/// # Impl blocks (class systems)
///
/// Apply `#[miniextendr(env|r6|s7|s3|s4)]` to an `impl Type` block.
/// A type's methods can be split over several impl blocks: they are merged into
/// one class. One block has the constructor and the class doc comment, and
/// every block repeats the class options.
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
/// - `#[miniextendr(s7(dispatch = "x, other"))]` - Multi-dispatch: the receiver
///   (`x`), then the method's leading parameters (`other`), in order
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
///     pub fn from_celsius(c: &Celsius) -> Self {
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
/// `#[miniextendr]` no longer generates ALTREP classes — `class`/`base`
/// attributes on a one-field struct are a compile error. Use the per-family
/// derives (`#[derive(AltrepInteger)]`, …) with `#[altrep(class = "...")]`
/// instead; registration is automatic there.
#[proc_macro_attribute]
pub fn miniextendr(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Spans a failed earlier expansion left behind belong to no output.
    let _ = deprecation::take();
    let expanded = proc_macro2::TokenStream::from(expand_miniextendr(attr, item));
    let warnings = deprecation::take();
    quote::quote! { #expanded #warnings }.into()
}

/// The `#[miniextendr]` expansion, before the deprecated-option warnings
/// ([`deprecation`]) are appended.
fn expand_miniextendr(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // rustc must select gated members before any expansion reads them (#2107).
    if let Ok(parsed) = syn::parse::<syn::Item>(item.clone()) {
        match cfg_gates::select(&attr.clone().into(), &parsed) {
            Ok(Some(selected)) => return selected.into(),
            Ok(None) => {}
            Err(error) => return error.into_compile_error().into(),
        }
    }

    // Try to parse as function first
    if syn::parse::<syn::ItemFn>(item.clone()).is_ok() {
        expand_fn(attr.into(), item.into())
    } else if syn::parse::<syn::ItemImpl>(item.clone()).is_ok() {
        // Delegate to impl block parser
        miniextendr_impl::expand_impl(attr, item)
    } else if syn::parse::<syn::ItemTrait>(item.clone()).is_ok() {
        // Delegate to trait ABI generator
        miniextendr_trait::expand_trait(attr, item)
    } else {
        // Delegate to struct/enum dispatch (handles ALTREP, ExternalPtr, list, dataframe, factor, match_arg)
        struct_enum_dispatch::expand_struct_or_enum(attr, item)
    }
}

/// `#[miniextendr]` on a function. It reads `proc_macro2` tokens and returns
/// the token type its caller asks for: `proc_macro`'s from the attribute, or
/// `proc_macro2`'s in the unit tests, which run outside a proc macro.
fn expand_fn<T: From<proc_macro2::TokenStream>>(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> T {
    let MiniextendrFnAttrs {
        force_worker,
        force_invisible,
        check_interrupt,
        coerce_all,
        rng,
        rng_seed,
        unwrap_in_r,
        serialize,
        wrap,
        serde_error,
        preconditions,
        call_attribution: call_attribution_attr,
        call_arg,
        return_pref,
        return_pref_span,
        s3_generic,
        s3_class,
        dots_spec,
        dots_policy,
        dots_span,
        lifecycle,
        strict,
        internal,
        noexport,
        export,
        doc,
        c_symbol,
        r_name: fn_r_name,
        postfix: fn_postfix,
        r_entry,
        r_post_checks,
        r_on_exit,
    } = match syn::parse2(attr) {
        Ok(attrs) => attrs,
        Err(err) => return err.into_compile_error().into(),
    };

    let mut parsed = match syn::parse2::<MiniextendrFunctionParsed>(item) {
        Ok(parsed) => parsed,
        Err(err) => return err.into_compile_error().into(),
    };

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

    // Validate: reject type/const generic functions.
    // Lifetime params ARE allowed — they are erased at codegen and produce a single monomorphic
    // symbol, so `#[no_mangle] extern "C-unwind" fn f<'a>(...)` is valid.
    // Type and const params require monomorphization → multiple symbols → cannot be #[no_mangle].
    {
        let params = &parsed.item().sig.generics.params;
        let has_type_or_const = params
            .iter()
            .any(|p| matches!(p, syn::GenericParam::Type(_) | syn::GenericParam::Const(_)));

        if has_type_or_const {
            let err = syn::Error::new_spanned(
                &parsed.item().sig.generics,
                "#[miniextendr] functions cannot have generic type or const parameters. \
                 Generic functions are incompatible with `extern \"C-unwind\"` and `#[no_mangle]` \
                 required for R FFI. Consider using trait objects or monomorphization instead. \
                 Explicit lifetime parameters are allowed (lifetimes are erased at codegen).",
            );
            return err.into_compile_error().into();
        }
    }

    // NB: no automatic `#[track_caller]` (dropped in #1121). It made a *direct*
    // panic's hook-location resolve to the generated wrapper glue instead of the
    // user's `panic!` line, which defeats the `(at file:line)` suffix now folded
    // into the R error message from the panic hook. See docs/TRACK_CALLER.md.
    parsed.add_inline_never_if_needed();

    // Extract commonly used values
    let uses_internal_c_wrapper = parsed.uses_internal_c_wrapper();
    let c_ident = if let Some(ref sym) = c_symbol {
        syn::Ident::new(sym, parsed.c_wrapper_ident().span())
    } else {
        parsed.c_wrapper_ident()
    };
    let r_wrapper_generator = parsed.r_wrapper_const_ident();

    // Extract references to parsed components
    let rust_ident = parsed.ident();
    let all_inputs = parsed.inputs();
    // Return-visibility markers (`Invisible<T>` / `Visible<T>`, #1213) are
    // peeled here so every later analysis sees the inner type; the marker's
    // decision is folded in below, next to the attribute form.
    let (visibility_marker, peeled_output) = {
        let output = parsed.output();
        if let syn::ReturnType::Type(_, ty) = output
            && let Some(err) = crate::type_inspect::visibility_marker_error(ty, "return")
        {
            return err.into_compile_error().into();
        }
        crate::type_inspect::peel_return_visibility(output)
    };
    let (return_wrap, peeled_output) = match crate::return_wrap::resolve(&peeled_output, wrap) {
        Ok(resolved) => resolved,
        Err(error) => return error.into_compile_error().into(),
    };
    if return_wrap
        .as_ref()
        .is_some_and(|wrap| wrap.conversion.is_some())
    {
        return syn::Error::new_spanned(
            parsed.output(),
            "ConvertTo/ConvertFrom require an inherent S7 impl method",
        )
        .into_compile_error()
        .into();
    }
    if return_wrap.is_some() && (serialize || unwrap_in_r) {
        return syn::Error::new_spanned(
            parsed.output(),
            "explicit class wrapping cannot be combined with serialize or unwrap_in_r",
        )
        .into_compile_error()
        .into();
    }
    if return_wrap.is_some() && parsed.abi().is_some() {
        return syn::Error::new_spanned(parsed.output(), "explicit class wrapping requires a generated Rust-to-R wrapper, not an extern function").into_compile_error().into();
    }
    let converted_output = crate::type_inspect::serialize_return_type(&peeled_output, serialize);
    let output = &converted_output;
    let abi = parsed.abi();
    if serialize && abi.is_some() {
        return syn::Error::new_spanned(
            parsed.output(),
            "`serialize` requires a generated Rust-to-R conversion and cannot be used on an extern function",
        )
        .into_compile_error()
        .into();
    }
    let attrs = parsed.attrs();
    let vis = parsed.vis();
    let generics = parsed.generics();
    let has_dots = parsed.has_dots();
    let dots_ident = parsed.dots_ident().cloned();
    // The R formal of each parameter (#1842): what every R-facing name below
    // is written as.
    let formals = parsed.formals();

    // Fail fast on invalid extern "C-unwind" signatures *before* any codegen,
    // so we never emit a wrapper that would be discarded by the surfaced error.
    if let Some(user_abi) = abi
        && let Err(e) = validate_extern_signature(user_abi, attrs, all_inputs, output)
    {
        return e.into_compile_error().into();
    }

    // Condition-call markers (#1566): a `Call` / `CallerCall` parameter is the
    // type-level spelling of `call = wrapper` / `call = caller`. It is not an
    // R formal: the C wrapper binds it from the hidden `__miniextendr_call`
    // slot, so every R-side consumer below sees `inputs` with the marker
    // removed while the Rust call site substitutes the slot.
    let mut call_marker: Option<(syn::Ident, r_wrapper_builder::CallAttribution)> = None;
    for arg in all_inputs.iter() {
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };
        let Some(kind) = crate::type_inspect::call_marker(&pt.ty) else {
            continue;
        };
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };
        let marker = kind
            .marker_name()
            .expect("a marker type selects an attribution that has a marker");
        if let Some((first, _)) = &call_marker {
            return syn::Error::new_spanned(
                pt,
                format!(
                    "a `#[miniextendr]` function takes at most one `Call` / `CallerCall` \
                     parameter (`{first}` already receives the condition call); the call \
                     slot is a single value"
                ),
            )
            .into_compile_error()
            .into();
        }
        if parsed.has_param_attrs(&crate::naming::ident_name(&pat_ident.ident)) {
            return syn::Error::new_spanned(
                pt,
                format!(
                    "per-parameter options (`coerce`, `match_arg`, `choices`, `default`, \
                     `several_ok`, `formal`) do not apply to a `{marker}` parameter: it is \
                     bound from the call slot, not from an R argument"
                ),
            )
            .into_compile_error()
            .into();
        }
        if kind == r_wrapper_builder::CallAttribution::Caller && !(noexport || internal) {
            return syn::Error::new_spanned(
                pt,
                "a `CallerCall` parameter attributes conditions to the wrapper's caller, which \
                 is only meaningful for a package-internal entry point; add `noexport` or \
                 `internal`.",
            )
            .into_compile_error()
            .into();
        }
        if kind == r_wrapper_builder::CallAttribution::Caller && call_arg {
            return syn::Error::new_spanned(
                pt,
                "a `CallerCall` parameter cannot be combined with `call_arg`: a `caller` wrapper \
                 already takes `.call`, where NULL means the caller's call. Keep the \
                 `CallerCall` parameter, or make it `Call` so that NULL means this function's \
                 own call.",
            )
            .into_compile_error()
            .into();
        }
        if call_attribution_attr == Some(r_wrapper_builder::CallAttribution::NoCall) {
            return syn::Error::new_spanned(
                pt,
                format!(
                    "`call = none` passes no call, so a `{marker}` parameter has nothing to \
                     receive; drop the parameter or the option"
                ),
            )
            .into_compile_error()
            .into();
        }
        if let Some(attr) = call_attribution_attr
            && attr != kind
        {
            return syn::Error::new_spanned(
                pt,
                format!(
                    "the `{marker}` parameter selects `{}` attribution but the attribute \
                     selects `{}` (`call = {}`); keep one of them (or make them agree)",
                    kind.name(),
                    attr.name(),
                    attr.name(),
                ),
            )
            .into_compile_error()
            .into();
        }
        call_marker = Some((pat_ident.ident.clone(), kind));
    }
    // Unevaluated-argument markers (#1835): a `Quoted` / `Quosure` parameter
    // stays an R formal, which the wrapper passes unevaluated
    // (`RArgumentBuilder::build_call_args_vec`). Every per-parameter option
    // would force it (a check, a coercion, a default evaluated in the wrapper's
    // frame), and a marker in any other position than the whole type or
    // `Missing<..>` would be converted as a forced value: all compile errors.
    // One exception: `default` alone on `Missing<..>` only writes the formal
    // (`x = NULL` in the usage). An omitted argument is still `missing()`, so
    // the wrapper passes the missing-argument sentinel and never evaluates the
    // default.
    let mut unevaluated_r_names: Vec<String> = Vec::new();
    for arg in all_inputs.iter() {
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };
        let Some(param) = crate::type_inspect::unevaluated_param(&pt.ty) else {
            if crate::type_inspect::mentions_unevaluated_marker(&pt.ty) {
                return syn::Error::new_spanned(
                    &pt.ty,
                    "`Quoted` / `Quosure` must be the parameter's whole type, or the type \
                     argument of `Missing<..>`: the wrapper passes the argument unevaluated \
                     only for those shapes",
                )
                .into_compile_error()
                .into();
            }
            continue;
        };
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };
        let marker = param.kind.name();
        let rust_name = crate::naming::ident_name(&pat_ident.ident);
        // `formal` only names the argument (#1842), so it applies.
        if parsed
            .param_attrs(&rust_name)
            .is_some_and(|attrs| !attrs.leaves_argument_unforced(param.optional))
        {
            let hint = if param.optional {
                format!(
                    "`Missing<{marker}>` takes `default` alone, which only writes the \
                     formal and is never evaluated"
                )
            } else {
                format!("Use `Missing<{marker}>` for an optional argument")
            };
            return syn::Error::new_spanned(
                pt,
                format!(
                    "per-parameter options (`coerce`, `match_arg`, `choices`, `several_ok`, \
                     `default`, `no_na`, `inherits`, `not_inherits`, `preconditions`) do not \
                     apply to a `{marker}` parameter: the wrapper passes the argument \
                     unevaluated, and each of them would force it (`formal`, which only \
                     names it, does apply). {hint}"
                ),
            )
            .into_compile_error()
            .into();
        }
        // `Checked<Quoted>` / `Unchecked<..>` is refused at parse time
        // (`MiniextendrFunctionParsed::parse`): no R-side type check to select.
        unevaluated_r_names.push(formals.of(&rust_name));
    }
    // The argument-count marker (#1860): an `NArgs` parameter is no R formal;
    // the wrapper passes `nargs()` at its position and the C wrapper converts
    // it like any argument, so it stays in `inputs`. `RArgumentBuilder`,
    // `r_formal_names`, the preconditions, the `@param` fillers and the
    // replacement-generic check each skip it.
    if let Err(err) =
        miniextendr_fn::check_nargs_params(all_inputs, |name| parsed.has_param_attrs(name))
    {
        return err.into_compile_error().into();
    }
    // The unforced dots (#1892): a `LazyDots` parameter is R's `...`, which
    // the wrapper passes as its frame; it takes no per-parameter option, and
    // any other shape (`&LazyDots`, `Option<..>`) is refused.
    if let Err(err) =
        miniextendr_fn::check_lazy_dots_params(all_inputs, |name| parsed.has_param_attrs(name))
    {
        return err.into_compile_error().into();
    }
    // The RNG guard (#2002): a `&RngGuard` parameter needs `rng`, whose
    // wrapper passes the guard it holds; it is no R formal either.
    if let Err(err) =
        miniextendr_fn::check_rng_guard_params(all_inputs, rng, |name| parsed.has_param_attrs(name))
    {
        return err.into_compile_error().into();
    }
    let has_rng_guard = all_inputs.iter().any(|arg| {
        matches!(arg, syn::FnArg::Typed(pt) if crate::type_inspect::is_rng_guard_param(&pt.ty))
    });
    let r_inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]> = all_inputs
        .iter()
        .filter(|arg| match arg {
            syn::FnArg::Typed(pt) => {
                crate::type_inspect::call_marker(&pt.ty).is_none()
                    && !crate::type_inspect::is_rng_guard_param(&pt.ty)
            }
            syn::FnArg::Receiver(_) => true,
        })
        .cloned()
        .collect();
    let inputs = &r_inputs;

    // A method for a replacement generic (`s3(generic = "$<-", ...)`) takes
    // the new value last, as `value` (#1853). Checked on the R formals, so a
    // trailing condition-call marker doesn't hide the parameter before it.
    if let Some(generic) = &s3_generic
        && let Err(err) = miniextendr_fn::check_replacement_value_param(
            crate::naming::split_generic(generic).1,
            inputs,
            &formals,
            rust_ident.span(),
        )
    {
        return err.into_compile_error().into();
    }

    // Check for @title/@description conflicts with implicit values (doc-lint feature)
    // Skip when `doc` attribute overrides the roxygen — implicit docs are irrelevant then.
    let doc_lint_warnings = if doc.is_some() {
        proc_macro2::TokenStream::new()
    } else {
        crate::roxygen::doc_conflict_warnings(attrs, rust_ident.span())
    };

    // Calling the Rust function: each R-backed parameter by name, a `Call` /
    // `CallerCall` marker from the C wrapper's `__miniextendr_call` slot (the
    // R wrapper's `.call = ...` argument, so a `CallerCall` sees `.mx_call`),
    // the RNG guard from the `rng` wrapper.
    let rust_inputs: Vec<proc_macro2::TokenStream> = all_inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(p) = pt.pat.as_ref()
            {
                let ident = &p.ident;
                if crate::type_inspect::is_rng_guard_param(&pt.ty) {
                    return Some(crate::type_inspect::rng_guard_arg());
                }
                return Some(match crate::type_inspect::call_marker(&pt.ty) {
                    Some(r_wrapper_builder::CallAttribution::Caller) => quote::quote! {
                        ::miniextendr_api::CallerCall::from_sexp(__miniextendr_call)
                    },
                    Some(_) => quote::quote! {
                        ::miniextendr_api::Call::from_sexp(__miniextendr_call)
                    },
                    None => quote::quote! { #ident },
                });
            }
            None
        })
        .collect();

    // Validate dots_spec usage (actual injection happens later in the function body)
    if dots_spec.is_some() && !has_dots {
        let err = syn::Error::new(
            dots_span.unwrap_or_else(proc_macro2::Span::call_site),
            "#[miniextendr(dots = typed_list!(...))] requires a `...` or `&Dots` parameter in the function signature",
        );
        return err.into_compile_error().into();
    }
    if dots_spec.is_some()
        && let Some(err) = miniextendr_fn::typed_list_on_lazy_dots(all_inputs, dots_span)
    {
        return err.into_compile_error().into();
    }
    // `dots = empty | forward(..) | data` (#2157): a standalone wrapper's
    // `...` is the function's own dots parameter.
    if let Some(policy) = &dots_policy
        && let Err(err) = policy.check_signature(has_dots, has_dots)
    {
        return err.into_compile_error().into();
    }

    // Analyze return type to determine:
    // - Whether it returns SEXP (affects thread strategy)
    // - Whether result should be invisible
    let rust_result_ident =
        syn::Ident::new("__miniextendr_rust_result", proc_macro2::Span::mixed_site());
    // `serde_error` only has an `Err` arm to act on when the fn returns `Result`.
    if serde_error.is_some() && !return_type_analysis::output_is_result(output) {
        return syn::Error::new_spanned(
            output,
            "`#[miniextendr(serde_error(..))]` requires a `Result<T, E>` return type: it classes \
             the condition raised from the `Err` arm",
        )
        .into_compile_error()
        .into();
    }
    let err_parts = c_wrapper_builder::ErrPartsMode::from_spec(serde_error.as_ref());

    // Crate-level defaults (#1454, #1566): `[package.metadata.miniextendr]` in
    // the crate's Cargo.toml. Read once per expansion; a malformed table is
    // reported here, on the first `#[miniextendr]` function, for every key it
    // may carry.
    let crate_config = match crate::crate_config::crate_config() {
        Ok(config) => config,
        Err(err) => {
            return syn::Error::new(proc_macro2::Span::call_site(), err.to_string())
                .into_compile_error()
                .into();
        }
    };

    // Resolve the R name once for both wrapper definitions and Rust-side errors.
    let is_s3_method = s3_generic.is_some() || s3_class.is_some();
    // Crate-level default for internal entry points (#1454):
    // `[package.metadata.miniextendr] noexport_postfix = "..."` applies to
    // every `noexport` / `internal` free function that names itself neither
    // via `r_name` nor via its own `postfix`. Exported functions and S3
    // methods keep their names. Precedence:
    // `r_name` > `postfix` > crate default > Rust name.
    let effective_postfix = match &fn_postfix {
        Some(postfix) => Some(postfix.clone()),
        None if (noexport || internal) && fn_r_name.is_none() && !is_s3_method => {
            crate_config.noexport_postfix.clone()
        }
        None => None,
    };
    let r_wrapper_ident_str: String;
    let s3_method_comment: String;
    // The registration tag of a method on a package-qualified generic (#2154).
    let mut s3_qualified_registration: Option<String> = None;

    if is_s3_method {
        // For S3 methods, function name is generic.class
        // generic defaults to Rust function name if not specified
        let generic = s3_generic
            .clone()
            .unwrap_or_else(|| crate::naming::ident_name(rust_ident));
        let (package, generic) = crate::naming::split_generic(&generic);
        // s3_class is guaranteed to be Some here because MiniextendrFnAttrs::parse
        // validates that s3(...) always has class specified
        let class = s3_class.as_ref().expect("s3_class validated at parse time");
        r_wrapper_ident_str = format!("{}.{}", generic, class);
        // Add @importFrom for vctrs generics so roxygen registers the dependency.
        // A qualified `vctrs::` generic registers lazily instead, so vctrs may
        // stay in Suggests.
        let import_comment =
            if package.is_none() && crate::vctrs_generics::is_vctrs_generic(generic) {
                format!("#' @importFrom vctrs {}\n", generic)
            } else {
                String::new()
            };
        s3_method_comment = format!("{}#' @method {} {}\n", import_comment, generic, class);
        s3_qualified_registration =
            package.map(|package| format!("#' @exportS3Method {package}::{generic} {class}\n"));
    } else if let Some(ref custom_name) = fn_r_name {
        r_wrapper_ident_str = custom_name.clone();
        s3_method_comment = String::new();
    } else if let Some(ref postfix) = effective_postfix {
        // `postfix = "_impl"` (or the crate default): the R wrapper is
        // `<rust_name><postfix>`; the C symbol keeps the Rust name.
        r_wrapper_ident_str = format!("{}{postfix}", crate::naming::ident_name(rust_ident));
        s3_method_comment = String::new();
    } else if abi.is_some() {
        r_wrapper_ident_str = format!("unsafe_{}", crate::naming::ident_name(rust_ident));
        s3_method_comment = String::new();
    } else {
        r_wrapper_ident_str = crate::naming::ident_name(rust_ident);
        s3_method_comment = String::new();
    };

    let return_analysis = return_type_analysis::analyze_return_type(
        output,
        &rust_result_ident,
        &r_wrapper_ident_str,
        unwrap_in_r,
        strict,
        &err_parts,
    );

    let returns_sexp = return_analysis.returns_sexp;

    // Marker (`Invisible<T>` / `Visible<T>`) > attribute (`invisible` /
    // `visible`) > shape default (unit-`NULL` returns are invisible).
    let is_invisible_return_type = match crate::type_inspect::resolve_visibility(
        visibility_marker,
        force_invisible,
        return_analysis.is_invisible,
        syn::spanned::Spanned::span(parsed.output()),
    ) {
        Ok(v) => v,
        Err(err) => return err.into_compile_error().into(),
    };

    // Check if any input parameter is main-thread-bound (SEXP or a !Send
    // framework wrapper like AltrepSexp — neither can move into the worker
    // closure, so the function must stay on the main thread). A `Call` /
    // `CallerCall` marker holds the call slot's SEXP and counts too.
    let has_sexp_inputs = all_inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            is_main_thread_bound_input(pat_type.ty.as_ref())
        } else {
            false
        }
    });

    // Check if the return type is main-thread-bound (BuiltDataFrame /
    // DataFrameShape / an R-backed view — all hold R memory and are !Send, so
    // they cannot cross back from the worker thread; `run_on_worker` requires
    // `T: Send`). Walks nested positions: Result<BuiltDataFrame, E>,
    // Option<DataFrameShape>, Vec<...>, tuples.
    let has_main_thread_bound_return = match output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => is_main_thread_bound_return(ty),
    };

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
    //    - Errors are always returned as tagged SEXP values; the R wrapper
    //      inspects the tag and raises a structured condition (`rust_*` class
    //      layering) past the Rust boundary.
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
    // Default: Main thread (simpler execution model, compatible with the
    // tagged-SEXP error transport)
    // Override: Use worker thread with #[miniextendr(worker)]
    //
    // Thread strategy:
    // - Main thread is always used unless force_worker is set
    // - force_worker cannot override hard requirements for main thread
    // - Hard requirements: returns_sexp, has_sexp_inputs (borrowed external
    //   arguments `&T` / `&mut T` / views included, #2106),
    //   has_main_thread_bound_return, has_dots, check_interrupt
    // A `Quoted` / `Quosure` parameter (also under `Missing<..>`, which
    // `is_main_thread_bound_input` does not look into) holds R objects and
    // evaluates R code (#1835). The RNG guard (#2002) is `!Send` and its
    // draws read R's RNG state.
    let requires_main_thread = returns_sexp
        || has_sexp_inputs
        || has_rng_guard
        || !unevaluated_r_names.is_empty()
        || has_main_thread_bound_return
        || has_dots
        || check_interrupt;
    let use_main_thread = !force_worker || requires_main_thread;

    // Extract cfg attributes to apply to generated items (needed by CWrapperContext)
    let cfg_attrs = extract_cfg_attrs(parsed.attrs());

    let source_loc_doc = source_location_doc(rust_ident.span());

    // Build individual per-parameter coerce, match_arg_several_ok, layered
    // choice (a choice type wrapped in `Missing` / `Option`, #1473 / #1551) and
    // `no_na` lists (the value is checked after the conversion too)
    let mut coerce_params_list: Vec<String> = Vec::new();
    let mut match_arg_several_ok_params_list: Vec<String> = Vec::new();
    let mut layered_choice_params_list: Vec<(String, rust_conversion_builder::ChoiceLeaf)> =
        Vec::new();
    let mut no_na_params_list: Vec<(String, Option<String>)> = Vec::new();
    for input in inputs.iter() {
        if let syn::FnArg::Typed(pt) = input
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
        {
            let param_name = crate::naming::ident_name(&pat_ident.ident);
            if parsed.has_coerce_attr(&param_name) {
                coerce_params_list.push(param_name.clone());
            }
            if let Some(attrs) = parsed.param_attrs(&param_name)
                && attrs.checks.no_na
            {
                no_na_params_list.push((param_name.clone(), attrs.checks.no_na_message.clone()));
            }
            if let Some(leaf) = parsed
                .param_attrs(&param_name)
                .and_then(miniextendr_fn::ParamAttrs::layered_leaf)
            {
                layered_choice_params_list.push((param_name, leaf));
            } else if parsed.has_match_arg_attr(&param_name) && parsed.has_several_ok(&param_name) {
                match_arg_several_ok_params_list.push(param_name);
            }
        }
    }

    // Build the call expression: rust_ident(rust_input_1, rust_input_2, ...).
    // A visibility marker is a transparent newtype: unwrap it so the return
    // conversion works on the inner value (the analysis above saw the inner type).
    let fn_call_expr = crate::type_inspect::prepare_return_value(
        quote::quote! { #rust_ident(#(#rust_inputs),*) },
        visibility_marker,
        serialize,
    );
    let fn_call_expr = match &return_wrap {
        Some(wrap) => wrap.prepare_value(fn_call_expr),
        None => fn_call_expr,
    };

    // Determine return handling: the rule methods share (`detect_return_handling`),
    // and unwrap_in_r (Result<T, E> → IntoR to pass result list to R).
    let return_pref_is_set = !matches!(return_pref, crate::miniextendr_fn::ReturnPref::Auto);
    let fn_return_handling = if unwrap_in_r && crate::return_type_analysis::output_is_result(output)
    {
        // In unwrap_in_r mode the IntoR here operates on the whole Result<T,E> (the
        // framework's IntoR for Result encodes it as a tagged list for R to decode), NOT on
        // the inner T. prefer= always targets that inner T, so there is nothing for it to
        // wrap here — reject rather than silently drop it (see apply_return_pref docstring
        // and the BUG4 audit finding).
        if return_pref_is_set {
            let span = return_pref_span.unwrap_or_else(proc_macro2::Span::call_site);
            return syn::Error::new(
                span,
                "`prefer = ...` cannot be combined with `unwrap_in_r` on a function returning \
                 `Result<T, E>`: `unwrap_in_r` converts the whole `Result<T, E>` via a single \
                 `IntoR` impl (a tagged list for R to decode), not the inner `T` alone, so \
                 there is no plain `T` for `prefer=` to wrap. Remove `prefer=`.",
            )
            .into_compile_error()
            .into();
        }
        c_wrapper_builder::ReturnHandling::IntoR
    } else {
        let auto = c_wrapper_builder::detect_return_handling(output, return_wrap.as_ref());
        // Apply return_pref override: wraps the result in AsList/AsExternalPtr/AsRNative.
        // Only applies to the plain IntoR variant — Option*/Result*/Unit/RawSexp/ExternalPtr
        // variants have no bare T to wrap and now hard-error instead of silently ignoring
        // prefer= (see apply_return_pref docstring).
        match apply_return_pref(auto, return_pref, return_pref_span) {
            Ok(handling) => handling,
            Err(err) => return err.into_compile_error().into(),
        }
    };

    // `rng(seed = <param>)` (#2198): the R formal the seed contract reads,
    // once the parameter and the return are known to suit it.
    let rng_seed_formal = match &rng_seed {
        Some(seed) => {
            use c_wrapper_builder::ReturnHandling;
            let unit_return = matches!(
                fn_return_handling,
                ReturnHandling::Unit | ReturnHandling::ResultUnit | ReturnHandling::OptionUnit
            );
            match miniextendr_fn::check_rng_seed(
                seed,
                all_inputs,
                inputs,
                &formals,
                &unevaluated_r_names,
                unit_return,
            ) {
                Ok(formal) => Some(formal),
                Err(err) => return err.into_compile_error().into(),
            }
        }
        None => None,
    };

    let thread_strategy = if use_main_thread {
        c_wrapper_builder::ThreadStrategy::MainThread
    } else {
        c_wrapper_builder::ThreadStrategy::WorkerThread
    };

    // Build CWrapperContext for the standalone fn
    let mut c_wrapper_builder =
        c_wrapper_builder::CWrapperContext::builder(rust_ident.clone(), c_ident.clone())
            .r_name(r_wrapper_ident_str.clone())
            .r_wrapper_const(r_wrapper_generator.clone())
            .inputs(inputs.iter().cloned().collect())
            .output(output.clone())
            .call_expr(fn_call_expr)
            .thread_strategy(thread_strategy)
            .return_handling(fn_return_handling)
            .err_parts(err_parts)
            .cfg_attrs(cfg_attrs.clone())
            .vis(vis.clone())
            .generics(generics.clone())
            .formals(formals.clone())
            .preserve_param_names();

    if uses_internal_c_wrapper {
        // Normal Rust fn: generate full C wrapper
    } else {
        // extern "C-unwind" fn: user wrote the C symbol; only emit R_CallMethodDef
        c_wrapper_builder = c_wrapper_builder.skip_wrapper();
    }

    if coerce_all {
        c_wrapper_builder = c_wrapper_builder.coerce_all();
    }
    for param in &coerce_params_list {
        c_wrapper_builder = c_wrapper_builder.with_coerce_param(param.clone());
    }
    for param in match_arg_several_ok_params_list {
        c_wrapper_builder = c_wrapper_builder.match_arg_several_ok(param);
    }
    for (param, leaf) in layered_choice_params_list {
        c_wrapper_builder = c_wrapper_builder.layered_choice(param, leaf);
    }
    for (param, message) in no_na_params_list {
        c_wrapper_builder = c_wrapper_builder.no_na(param, message);
    }
    for (param, markers) in parsed.param_markers() {
        c_wrapper_builder = c_wrapper_builder.param_markers(param.clone(), markers.clone());
    }
    if check_interrupt {
        c_wrapper_builder = c_wrapper_builder.check_interrupt();
    }
    if rng {
        c_wrapper_builder = c_wrapper_builder.rng();
    }
    if strict {
        c_wrapper_builder = c_wrapper_builder.strict();
    }

    let c_wrapper = c_wrapper_builder.build().generate();

    // region: R wrappers generation in `fn`
    // Build R formal parameters and call arguments using shared builder
    let mut arg_builder = RArgumentBuilder::new(inputs).with_formals(formals.clone());
    // Add user-specified parameter defaults, keyed by R formal like the
    // builder's. A `Missing<T>` default only writes the formal (#2161): the
    // call argument tests `missing()` first.
    let mut merged_defaults = formals.rekey(parsed.param_defaults());
    // For match_arg params, always use the choices placeholder as the formal
    // default — the write-time pass replaces it with `c("a", "b", ...)`.
    // If the user supplied `default = "\"X\""`, capture X as `preferred_default`
    // so the write-time pass rotates X to position 1; the user's literal does
    // NOT become the formal (otherwise R's `match.arg` would only see one
    // choice).
    //
    // Tuple: (placeholder, aliases_placeholder, rust_param,
    // preferred_default_unquoted_or_empty)
    let mut match_arg_placeholders: Vec<(String, String, String, String)> = Vec::new();
    for match_arg_param in parsed.match_arg_params() {
        let r_name = formals.of(match_arg_param);
        let preferred = match merged_defaults.get(&r_name) {
            Some(raw) => crate::match_arg_keys::extract_match_arg_default(raw),
            None => String::new(),
        };
        // The placeholders are keyed by the Rust name, never the formal
        // (`match_arg_keys::param_key`).
        let key = crate::match_arg_keys::param_key(match_arg_param);
        let placeholder = crate::match_arg_keys::choices_placeholder(&c_ident.to_string(), &key);
        // `Option<T>` (#1473): the formal is `NULL` (no choice); `no_default`
        // (#1828, which `default` cannot join): no formal default. The prelude
        // spells the choices out through the same placeholder either way, so
        // its entry is emitted below in every case.
        let formal = match parsed.param_attrs(match_arg_param) {
            Some(attrs) => attrs.choice_formal(&placeholder),
            None => Some(placeholder.clone()),
        };
        if let Some(formal) = formal {
            merged_defaults.insert(r_name.clone(), formal);
        }
        let aliases_placeholder =
            crate::match_arg_keys::aliases_placeholder(&c_ident.to_string(), &key);
        match_arg_placeholders.push((
            placeholder,
            aliases_placeholder,
            match_arg_param.clone(),
            preferred,
        ));
    }
    // Add c("a", "b", "c") default for choices params (idiomatic R match.arg
    // pattern); an `Option<T>` choices param defaults to `NULL` instead
    // (#1473), and a `no_default` one gets none (#1828).
    for (param_name, choices) in parsed.choices_params() {
        let r_name = formals.of(param_name);
        let quoted: Vec<String> = choices.iter().map(|c| format!("\"{}\"", c)).collect();
        let choices_expr = format!("c({})", quoted.join(", "));
        let formal = match parsed.param_attrs(param_name) {
            Some(attrs) => attrs.choice_formal(&choices_expr),
            None => Some(choices_expr),
        };
        if let Some(formal) = formal {
            merged_defaults.entry(r_name).or_insert(formal);
        }
    }
    arg_builder = arg_builder
        .with_defaults(merged_defaults)
        .with_absent_formals(parsed.absent_formals());

    let r_formals = arg_builder.build_formals();
    let mut r_call_args_strs = arg_builder.build_call_args_vec();

    // Prepend the `.call` parameter if using the internal C wrapper. Marker
    // (`Call` / `CallerCall`) > attribute (`call = wrapper | caller | none`)
    // > crate default (`call_attribution = "..."`, where `caller` applies to
    // `noexport` / `internal` entry points only) > `wrapper` (#1566). Every
    // form passes the call as written: `sys.call()` for `wrapper`, the
    // caller's `sys.call()` for `caller`; `none` passes `FALSE`, the "no
    // call" marker, so every condition has a NULL call (#1851). `caller`
    // also gives the wrapper a trailing `.call = NULL` formal (#1613, S3
    // methods excluded; see `call_formal` below), as does `call_arg`.
    // Attribution is independent of the preconditions.
    //
    // `call_arg` (#1834) turns `wrapper` into `Argument`: the same trailing
    // formal, whose `NULL` is the wrapper's own call. It is an explicit
    // choice, so the crate default doesn't apply with it (the parser and the
    // marker loop above refuse it with `caller` and `none`).
    //
    // An S3 method's `wrapper` attribution is `Generic` (#1851): the slot
    // takes the method's frame, `environment()`, and the raise helper turns
    // it into the generic's call (`summary(x)` for `summary.cls(x)`, the
    // assignment for a replacement method) only when a condition is raised.
    // With a `Call` parameter the frame is resolved up front instead
    // (`GenericEager`), since the body receives the slot.
    let call_attribution = r_wrapper_builder::CallAttribution::resolve(
        call_marker.as_ref().map(|(_, kind)| *kind),
        call_attribution_attr,
        crate_config.call_attribution.filter(|_| !call_arg),
        noexport || internal,
    )
    .with_call_arg(call_arg);
    let call_attribution = if is_s3_method {
        call_attribution.for_s3_method(call_marker.is_some())
    } else {
        call_attribution
    };
    if uses_internal_c_wrapper {
        r_call_args_strs.insert(0, call_attribution.dot_call_arg().to_string());
    } else if matches!(
        call_attribution_attr,
        Some(
            r_wrapper_builder::CallAttribution::Caller | r_wrapper_builder::CallAttribution::NoCall
        )
    ) || call_arg
    {
        // `extern "C-unwind"` fns have no generated call slot to redirect (a
        // `CallerCall` parameter already fails the extern signature check).
        let option = if call_arg {
            "`call_arg`"
        } else if call_attribution_attr == Some(r_wrapper_builder::CallAttribution::NoCall) {
            "`call = none`"
        } else {
            "`call = caller`"
        };
        return syn::Error::new_spanned(
            &parsed.item().sig.ident,
            format!(
                "{option} needs the generated call slot; an `extern \"C-unwind\"` function \
                 has no `.call` argument to attribute"
            ),
        )
        .into_compile_error()
        .into();
    }

    // Build the R body string consistently
    let c_ident_str = c_ident.to_string();
    let call_args_joined = r_call_args_strs.join(", ");
    let call_expr = if r_call_args_strs.is_empty() {
        format!(".Call({})", c_ident_str)
    } else {
        format!(".Call({}, {})", c_ident_str, call_args_joined)
    };
    let r_wrapper_return_str = {
        // Capture result, check for tagged condition value, raise R condition if present.
        let final_return = return_wrap
            .as_ref()
            .map_or_else(|| ".val".to_owned(), |wrap| wrap.r_expression(".val", None));
        let final_return = if is_invisible_return_type {
            format!("invisible({final_return})")
        } else {
            final_return
        };
        // `rng(seed = ..)` (#2198): the result records the seed, after its
        // condition check and before any wrap.
        let final_return = if rng_seed_formal.is_some() {
            format!(
                "{}\n  {final_return}",
                r_wrapper_builder::RngSeedContract::ATTRIBUTE
            )
        } else {
            final_return
        };
        // `call = caller` binds `.mx_call` at the head of the prelude (see
        // `combined_prelude` below), ahead of every R-side check.
        crate::method_return_builder::standalone_body_with_call_default(
            &call_expr,
            &final_return,
            "  ",
            call_attribution.raise_default(),
        )
    };
    // The trailing `.call = NULL` formal of a `caller` wrapper (#1613) or a
    // `call_arg` one (#1834): a hand-written helper between the public
    // function and the entry point, or an R function composing the function,
    // passes a frame there. S3 methods keep the zero-argument prelude: a
    // formal the generic lacks breaks generic/method consistency (`call_arg`
    // is refused on them).
    // `.call` joins the formals string only, never `inputs`, so the
    // precondition builder, the marker peels and the parameter fillers never
    // see it as a Rust parameter.
    let call_formal = call_attribution.formal().filter(|_| !is_s3_method);
    // Stable, consistent R formatting style: brace on same line, body indented, closing brace on its own line
    // r_formals is already a joined string from build_formals()
    let formals_joined = r_wrapper_builder::with_call_formal(&r_formals, call_formal);
    let mut roxygen_tags = if let Some(ref doc_text) = doc {
        // Custom doc override: each line becomes a separate roxygen tag entry
        doc_text.lines().map(|l| l.to_string()).collect()
    } else {
        // `noexport` renders no page (it appends `@noRd` below), so its docs
        // keep no links (#1818); the reader sees an explicit `@noRd` itself.
        crate::roxygen::roxygen_tags_from_attrs(
            attrs,
            crate::roxygen::RdPage::suppressed_if(noexport),
        )
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

    // Generated @param tags for the parameters the author left undocumented,
    // unless the block takes them from a `@describeIn` topic or
    // `@inheritParams` (#1590); the wrapper registry decides the rest per page
    // at write time. The match_arg placeholders feed the
    // MX_MATCH_ARG_PARAM_DOCS entries of the write-time resolver. The `.call`
    // formal gets its line after the parameters' (#1613, #1834), unless the
    // block renders no page (`noexport`, `@noRd`): it is no filler, so the
    // registry would not drop it there.
    let renders_no_page = noexport || crate::roxygen::has_roxygen_tag(&roxygen_tags, "noRd");
    let call_param_doc = call_formal
        .and(call_attribution.param_doc())
        .filter(|_| !renders_no_page);
    let match_arg_param_doc_placeholders = crate::roxygen::push_fn_param_tags(
        &mut roxygen_tags,
        inputs,
        &parsed,
        &c_ident_str,
        call_param_doc,
    );

    // A standalone function's reference page is titled by its R wrapper name — never
    // the doc-comment prose. rustdoc summaries are markdown (intra-doc links, code
    // spans) that roxygen2 can't resolve as a `\title`; the prose is promoted to
    // `@description` by `roxygen_tags_from_attrs` instead. Without a `@title`, roxygen2
    // skips the `.Rd` entirely (#1054), so inject the wrapper name when none exists.
    // A block that renders no page (`@noRd`, or `noexport` which appends it below)
    // needs no title (#1552).
    let has_no_rd_tag = crate::roxygen::has_roxygen_tag(&roxygen_tags, "noRd");
    if !roxygen_tags.is_empty()
        && !has_no_rd_tag
        && !noexport
        && !crate::roxygen::has_roxygen_tag(&roxygen_tags, "title")
    {
        roxygen_tags.insert(0, format!("@title {}", r_wrapper_ident_str));
    }

    let roxygen_tags_str = crate::roxygen::format_roxygen_tags(&roxygen_tags);
    let has_export_tag = crate::roxygen::has_export_tag(&roxygen_tags);
    let has_internal_tag = crate::roxygen::has_roxygen_tag(&roxygen_tags, "keywords internal");
    // `@source` provenance is opt-in per crate (#1552); the `# Generated from
    // Rust fn … (file:line:col)` comment below stays the navigation pointer.
    let source_comment = crate::roxygen::source_tag_with(
        crate_config.source_tags,
        &format!("Generated by miniextendr from Rust fn `{rust_ident}`"),
    )
    .map(|line| format!("{line}\n"))
    .unwrap_or_default();
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
        // A method on `pkg::generic` registers through `@exportS3Method
        // pkg::generic class`: roxygen2 writes `S3method(pkg::generic, class)`,
        // which R applies once `pkg`'s namespace loads, so `pkg` may stay in
        // Suggests (#2154).
        s3_qualified_registration.unwrap_or_else(|| "#' @export\n".to_string())
    } else {
        String::new()
    };
    // `noexport` means no man page at all (docs/CLASS_SYSTEMS.md export-control
    // table): inject @noRd so roxygen2 skips the Rd and the write-time page
    // pass (registry.rs) gives the fn no default title. Without this, an
    // unexported fn keeps a \usage entry in man/, which R CMD check flags as a
    // code/documentation mismatch.
    let no_rd_comment = if noexport && !has_no_rd_tag {
        "#' @noRd\n"
    } else {
        ""
    };
    // Generate match.arg prelude for parameters with #[miniextendr(match_arg)]
    // Collect (r_param_name, rust_name, rust_type) for each match_arg param
    let match_arg_param_info: Vec<(String, String, &syn::Type)> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let rust_name = crate::naming::ident_name(&pat_ident.ident);
                if parsed.has_match_arg_attr(&rust_name) {
                    let r_name = formals.of(&rust_name);
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
            // Every form names the choice list through the same placeholder the
            // formal default carries (populated by the write-time pass as
            // `c("a", "b", ...)`; it substitutes every occurrence). Strict
            // several_ok (#1472): every element must match; NULL selects every
            // choice. `Option<T>` (#1473): NULL means no choice and skips the
            // check; `Missing<T>` (#1551) skips it for an omitted argument and,
            // for a single choice, for the formal passed whole (#2152), as the
            // `.mx_absent_<p>` flag bound in front of it says (#2182). The
            // helpers read a factor as its labels (#1552). Under `call = caller`
            // every form raises with `.mx_call` (#1548).
            let key = crate::match_arg_keys::param_key(rust_name);
            let placeholder =
                crate::match_arg_keys::choices_placeholder(&c_ident.to_string(), &key);
            let aliases = crate::match_arg_keys::aliases_placeholder(&c_ident.to_string(), &key);
            if let Some(attrs) = parsed.param_attrs(rust_name) {
                lines.extend(call_attribution.match_arg_lines(
                    r_param,
                    &placeholder,
                    Some(&aliases),
                    attrs,
                ));
            }
        }
        lines.join("\n  ")
    };

    // Generate the match.arg prelude for choices params: the same three forms
    // as `match_arg`, with the literal choice list known here.
    let choices_prelude = {
        let mut lines = Vec::new();
        for arg in inputs.iter() {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let rust_name = crate::naming::ident_name(&pat_ident.ident);
                if let Some(attrs) = parsed.param_attrs(&rust_name)
                    && let Some(choices) = attrs.choices.as_ref()
                {
                    let r_name = formals.of(&rust_name);
                    let quoted: Vec<String> =
                        choices.iter().map(|c| format!("\"{}\"", c)).collect();
                    let choices_expr = format!("c({})", quoted.join(", "));
                    lines.extend(call_attribution.match_arg_lines(
                        &r_name,
                        &choices_expr,
                        None,
                        attrs,
                    ));
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

    // Generate R-side precondition checks (one `isTRUE()` guard per check)
    // Skip both match_arg and choices params (already validated by match.arg)
    // and `Quoted` / `Quosure` params, which no check may force (#1835).
    let mut skip_params = parsed.precondition_skip_params();
    skip_params.extend(unevaluated_r_names.iter().cloned());
    // A parameter drops its type-derived checks when its own spelling
    // (`Unchecked<T>`, `#[miniextendr(no_preconditions)]` on it), else the
    // function's `no_preconditions`, else the crate's `preconditions = false`,
    // else the `no-preconditions-default` feature says so (#1566,
    // `r_preconditions::resolve_type_checks`). TryFromSexp still raises a
    // typed Rust error on mismatched input. The savings were measured against
    // the former `stopifnot()` block (~1230 ns / 1-arg or ~3900 ns / 5-arg);
    // the guards cost about half. The per-parameter `inherits` /
    // `not_inherits` / `no_na` checks stay: the Rust conversion does not
    // repeat them. (The C wrapper's
    // post-conversion `no_na` check covers only what a type reads as `NA`
    // beyond `anyNA()`, such as `"NA"` for `AsNumeric`, except on an
    // `Either`, which has no R guard and is checked only in Rust.)
    let precondition_prelude = {
        // `set.seed()` validates the seed of `rng(seed = ..)` (#2198), so
        // its parameter has no type-derived check.
        let mut unchecked = r_preconditions::unchecked_params(
            inputs,
            &formals,
            |name| parsed.param_attrs(name).and_then(|a| a.preconditions),
            preconditions,
            None,
            crate_config.preconditions,
        );
        unchecked.extend(rng_seed_formal.iter().cloned());
        // A coerced integer-element vector reads via `&[i32]` (INTSXP-only), so its
        // precondition tightens to `is.integer` (issue #616). `coerce_params_list`
        // holds Rust names; the options are keyed by R formal.
        let precondition_opts = r_preconditions::PreconditionOptions {
            coerce_all,
            coerce_params: coerce_params_list.iter().map(|p| formals.of(p)).collect(),
            explicit: parsed.explicit_checks(),
            unchecked,
            formals: formals.clone(),
        };
        let precondition_output =
            r_preconditions::build_precondition_checks(inputs, &skip_params, &precondition_opts);
        // One guard per check, raising the shared argument error (#1591):
        // attributed to the wrapper's own call, or under `call = caller` to
        // `.mx_call` (#1548).
        precondition_output
            .guards(call_attribution.r_check_call())
            .join("\n  ")
    };

    // `dots = empty | forward(..)` (#2157): refuse what `...` doesn't take,
    // ahead of the argument checks, so a misspelt argument is reported before
    // the missing one it was meant to be.
    let dots_check = dots_policy
        .as_ref()
        .and_then(|policy| policy.r_check(&formals_joined, call_attribution.r_check_call()));

    // Combine all preludes: call attribution (`.mx_call` under `call = caller`,
    // first so the R-side checks below can raise with it, #1548), r_entry,
    // on.exit, lifecycle, the `...` check, static preconditions, match.arg,
    // choices, r_post_checks, and last the seed contract of `rng(seed = ..)`
    // (#2198), so that a refused argument changes no RNG state. (Missing<T>
    // forwarding lives inline in the `.Call()` args — see
    // `build_call_args_vec` — because a prelude binding of the missing
    // sentinel errors on lookup. What the match.arg and choices parts bind
    // for it is a flag: `.mx_absent_<p>` of a single-choice parameter, #2182.)
    let on_exit_str = r_on_exit.as_ref().map(|oe| oe.to_r_code());
    let rng_seed_prelude = rng_seed_formal
        .as_deref()
        .map(|formal| r_wrapper_builder::RngSeedContract { formal }.prelude("  "));
    let attribution_prelude = call_attribution.prelude(call_formal.is_some());
    let combined_prelude = {
        let mut parts = Vec::new();
        if !attribution_prelude.is_empty() {
            parts.push(attribution_prelude.as_str());
        }
        if let Some(ref entry) = r_entry {
            parts.push(entry.as_str());
        }
        if let Some(ref s) = on_exit_str {
            parts.push(s.as_str());
        }
        if let Some(ref lc) = lifecycle_prelude {
            parts.push(lc.as_str());
        }
        if let Some(ref check) = dots_check {
            parts.push(check.as_str());
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
        if let Some(ref seed) = rng_seed_prelude {
            parts.push(seed.as_str());
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n  "))
        }
    };

    let r_wrapper_string = if let Some(prelude) = combined_prelude {
        format!(
            "{}{}{}{}{}{}{} <- function({}) {{\n  {}\n  {}\n}}",
            roxygen_tags_str,
            source_comment,
            s3_method_comment,
            internal_comment,
            no_rd_comment,
            export_comment,
            crate::naming::r_def_name(&r_wrapper_ident_str),
            formals_joined,
            prelude,
            r_wrapper_return_str
        )
    } else {
        format!(
            "{}{}{}{}{}{}{} <- function({}) {{\n  {}\n}}",
            roxygen_tags_str,
            source_comment,
            s3_method_comment,
            internal_comment,
            no_rd_comment,
            export_comment,
            crate::naming::r_def_name(&r_wrapper_ident_str),
            formals_joined,
            r_wrapper_return_str
        )
    };
    // Use a raw string literal for better readability in macro expansion
    let r_wrapper_str = r_wrapper_raw_literal(
        &r_wrapper_string,
        &r_shadowing::formal_names([(inputs, formals.clone())]),
    );

    // endregion

    // Generate doc strings with links
    let r_wrapper_doc = format!(
        "R wrapper code for [`{}`], calls [`{}`].",
        rust_ident, c_ident
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
    // (the gate above guarantees the dots parameter exists).
    if let (Some(spec_tokens), Some(dots_param)) = (&dots_spec, &dots_ident) {
        let validation_stmt = build_dots_validation_stmt(dots_param, spec_tokens);
        original_item.block.stmts.insert(0, validation_stmt);
    }

    let original_item = original_item;

    // Generate MX_MATCH_ARG_CHOICES entries for placeholder → choices replacement
    // Resolve the `MatchArg`-bound type used in the choices_str closure: for
    // `several_ok` params that's the inner element of the container, otherwise
    // it's the param type directly.
    let choices_ty_for = |rust_param: &str| -> Option<&syn::Type> {
        let (_, _, param_ty) = match_arg_param_info
            .iter()
            .find(|(_, rn, _)| rn == rust_param)?;
        Some(match_arg_choices_ty(
            param_ty,
            parsed.has_several_ok(rust_param),
        ))
    };

    let match_arg_choices_entries: Vec<proc_macro2::TokenStream> = match_arg_placeholders
        .iter()
        .filter_map(
            |(placeholder, aliases_placeholder, rust_param, preferred_default)| {
                let choices_ty = choices_ty_for(rust_param)?;
                let entry_ident = syn::Ident::new(
                    &format!(
                        "match_arg_choices_entry_{}",
                        crate::match_arg_keys::placeholder_ident_suffix(placeholder)
                    ),
                    proc_macro2::Span::call_site(),
                );
                Some(crate::match_arg_keys::choices_entry_tokens(
                    &cfg_attrs,
                    &entry_ident,
                    placeholder,
                    aliases_placeholder,
                    choices_ty,
                    preferred_default,
                ))
            },
        )
        .collect();

    // Generate MX_MATCH_ARG_PARAM_DOCS entries for @param doc placeholder → choice description
    let match_arg_param_doc_entries: Vec<proc_macro2::TokenStream> =
        match_arg_param_doc_placeholders
            .iter()
            .filter_map(|(doc_placeholder, rust_param)| {
                let choices_ty = choices_ty_for(rust_param)?;
                let attrs = parsed.param_attrs(rust_param)?;
                let entry_ident = syn::Ident::new(
                    &format!(
                        "match_arg_param_doc_entry_{}",
                        crate::match_arg_keys::placeholder_ident_suffix(doc_placeholder)
                    ),
                    proc_macro2::Span::call_site(),
                );
                Some(crate::match_arg_keys::param_doc_entry_tokens(
                    &cfg_attrs,
                    &entry_ident,
                    doc_placeholder,
                    attrs.several_ok,
                    &attrs.choice_doc_suffix(),
                    choices_ty,
                ))
            })
            .collect();

    // Generate doc comment linking to C wrapper and R wrapper constant
    let fn_r_wrapper_doc = format!(
        "See [`{}`] for C wrapper, [`{}`] for R wrapper.",
        c_ident, r_wrapper_generator
    );

    let expanded: T = quote::quote! {
        // rust function with doc link to R wrapper
        #[doc = #fn_r_wrapper_doc]
        #original_item

        // C wrapper
        #(#cfg_attrs)*
        #c_wrapper

        // R wrapper (self-registers via distributed_slice)
        #(#cfg_attrs)*
        #[doc = #r_wrapper_doc]
        #[doc = ::core::concat!("Wraps Rust function `", ::core::stringify!(#rust_ident), "`.")]
        #[doc = #source_loc_doc]
        #[doc = ::core::concat!("Generated from source file `", ::core::file!(), "`.")]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        static #r_wrapper_generator: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::Function,
                source_file: ::core::file!(),
                source_line: #source_line_lit,
                content: ::core::concat!(
                    "# Generated from Rust fn `",
                    ::core::stringify!(#rust_ident),
                    "` (",
                    ::core::file!(),
                    ":",
                    #source_line_lit,
                    ":",
                    #source_col_lit,
                    ")",
                    #r_wrapper_str
                ),
            };

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

/// Maps a `ReturnPref` attribute value onto an auto-detected `ReturnHandling`.
///
/// Only the plain `IntoR` variant has a bare `T` for `prefer=` to wrap, so it is the
/// only variant substituted with its pref-specific counterpart
/// (`AsListOf`/`AsExternalPtrOf`/`AsNativeOf`). Every other variant (`Unit`, `RawSexp`,
/// `ExternalPtr`, `Option*`, `Result*`) has its own fixed SEXP-shape rule that
/// `prefer=` cannot compose with — returning a compile error for those is better than
/// silently dropping the attribute (see the BUG4 audit finding: `prefer = "list"` on an
/// `Option<T>` return used to be accepted and silently ignored).
fn apply_return_pref(
    auto: c_wrapper_builder::ReturnHandling,
    pref: crate::miniextendr_fn::ReturnPref,
    pref_span: Option<proc_macro2::Span>,
) -> syn::Result<c_wrapper_builder::ReturnHandling> {
    use crate::miniextendr_fn::ReturnPref;
    use c_wrapper_builder::ReturnHandling;

    let wrapped = match pref {
        ReturnPref::Auto => return Ok(auto),
        ReturnPref::List => match auto {
            ReturnHandling::IntoR => Some(ReturnHandling::AsListOf),
            _ => None,
        },
        ReturnPref::ExternalPtr => match auto {
            ReturnHandling::IntoR => Some(ReturnHandling::AsExternalPtrOf),
            _ => None,
        },
        ReturnPref::Native => match auto {
            ReturnHandling::IntoR => Some(ReturnHandling::AsNativeOf),
            _ => None,
        },
    };

    wrapped.ok_or_else(|| {
        let (pref_name, wrapper_name) = return_pref_names(pref);
        let span = pref_span.unwrap_or_else(proc_macro2::Span::call_site);
        syn::Error::new(
            span,
            format!(
                "`prefer = \"{pref_name}\"` cannot be honored on this return type. \
                 `prefer=` only applies to a function returning a plain `T: IntoR` value, \
                 which it wraps in `{wrapper_name}` before conversion. This function's return \
                 type falls into a different codegen category ({}) with its own fixed \
                 SEXP-shape rule, so there is no plain `T` for `prefer=` to wrap. Remove \
                 `prefer=`, or change the return type to a plain `T`.",
                return_handling_category_description(&auto),
            ),
        )
    })
}

/// Human-readable `(attribute value, wrapper type)` pair for a [`ReturnPref`](crate::miniextendr_fn::ReturnPref),
/// used to phrase the `apply_return_pref` compile error.
fn return_pref_names(pref: crate::miniextendr_fn::ReturnPref) -> (&'static str, &'static str) {
    use crate::miniextendr_fn::ReturnPref;
    match pref {
        ReturnPref::Auto => ("auto", ""),
        ReturnPref::List => ("list", "AsList"),
        ReturnPref::ExternalPtr => ("externalptr", "AsExternalPtr"),
        ReturnPref::Native => ("native", "AsRNative"),
    }
}

/// Human-readable description of a [`ReturnHandling`](c_wrapper_builder::ReturnHandling)
/// category, for the `apply_return_pref` compile error. Only describes the categories
/// [`c_wrapper_builder::detect_return_handling`] can actually produce;
/// the wildcard arm covers variants that never reach `apply_return_pref` as `auto`
/// (`IntoR` itself, method-only `SelfHandle`, and the `As*Of` variants `apply_return_pref`
/// produces as *output*, never takes as input).
fn return_handling_category_description(rh: &c_wrapper_builder::ReturnHandling) -> &'static str {
    use c_wrapper_builder::ReturnHandling;
    match rh {
        ReturnHandling::Unit => "the unit return type `()`",
        ReturnHandling::RawSexp => "a raw `SEXP` return type",
        ReturnHandling::ExternalPtr => {
            "a `Self`-returning constructor, already converted via `ExternalPtr::new`"
        }
        ReturnHandling::OptionUnit => "`Option<()>`",
        ReturnHandling::OptionSexp => "`Option<SEXP>`",
        ReturnHandling::OptionIntoR | ReturnHandling::OptionClassPayload => "`Option<T>`",
        ReturnHandling::ResultUnit => "`Result<(), E>`",
        ReturnHandling::ResultSexp => "`Result<SEXP, E>`",
        ReturnHandling::ResultIntoR => "`Result<T, E>`",
        ReturnHandling::ResultNullOnErr => "`Result<T, ()>`",
        _ => "this return type",
    }
}

/// Generate thread-safe wrappers for R FFI functions.
///
/// Apply this to an `extern "C-unwind"` block to generate, **for each
/// non-variadic function**, a pair of entry points:
///
/// - The original name (e.g. `Rf_allocVector`) — a safe Rust wrapper that
///   runs directly on R's main thread, routes through
///   `miniextendr_api::worker::with_r_thread` from an active miniextendr
///   worker context, and panics for arbitrary off-main callers.
/// - A `*_unchecked` sibling (`Rf_allocVector_unchecked`) — the raw
///   `extern "C-unwind"` declaration with no main-thread assertion and no
///   worker round-trip.
///
/// User code should reach for the checked variant by default; the unchecked
/// sibling exists for three known-safe contexts:
///
/// 1. **Inside ALTREP callbacks** — R is already calling us on the main
///    thread, so the assertion would always pass and the route would
///    deadlock the call back to R.
/// 2. **Inside a `with_r_unwind_protect` body** — the guard has established
///    main-thread context, and re-entering `with_r_thread` would nest two
///    `R_UnwindProtect` frames (paying the longjmp-leak cost twice).
/// 3. **Inside a `with_r_thread` body** — the assertion is redundant; you
///    are already where you needed to be.
///
/// The build-time lint **MXL301** enforces this: calling `*_unchecked`
/// outside one of those three contexts is a compile-time error. Without the
/// `worker-thread` feature, the checked variant still enforces the recorded
/// main-thread contract; it simply has no worker route available.
///
/// # Tradeoffs at a glance
///
/// | Variant | Asserts main thread | Routes to main | When to use |
/// |---|---|---|---|
/// | `Rf_foo` (checked) | yes (debug) | yes (from worker) | default |
/// | `Rf_foo_unchecked` | no | no | ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread` |
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
                        #[doc = ::core::concat!("Unchecked FFI binding for `", ::core::stringify!(#fn_name), "`.")]
                        #[doc = #source_loc_doc_lit]
                        #[doc = ::core::concat!("Generated from source file `", ::core::file!(), "`.")]
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
                            #[doc = ::core::concat!("Generated from source file `", ::core::file!(), "`.")]
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
                            #[doc = ::core::concat!("Generated from source file `", ::core::file!(), "`.")]
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
/// This gives newtypes native buffer access, borrowed `&[T]` conversions, and
/// participation in the `Coerce<R>` traits. Add `#[derive(IntoR)]` to return
/// owned values: `T`, `Vec<T>`, `Box<[T]>`, and their supported nested containers.
/// `RNativeType` describes storage; `IntoR` selects the output representation.
/// Keeping them separate also permits an `ExternalPtr` representation.
/// The inner type must implement `RNativeType`. Add `#[derive(IsNA)]`
/// to forward missing-value detection when the inner type supports NA.
///
/// # Supported Struct Forms
///
/// Both tuple structs and single-field named structs are supported:
///
/// ```ignore
/// use miniextendr_api::RNativeType;
///
/// // Tuple struct (most common)
/// #[repr(transparent)]
/// #[derive(Clone, Copy, RNativeType)]
/// struct UserId(i32);
///
/// // Named single-field struct
/// #[repr(transparent)]
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
///     const R_NA: Self = UserId(<i32 as RNativeType>::R_NA);
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
/// - Must use `#[repr(transparent)]` so native R buffers have the same layout
/// - The inner type must implement `RNativeType` (`i32`, `f64`, `RLogical`, `u8`, `Rcomplex`)
/// - Must also be `Copy` (`RNativeType: Copy`): derive `Clone, Copy` with it
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

    // RNativeType exposes slices of Self over the inner type's R allocation.
    // A one-field Rust struct alone does not guarantee the required layout.
    let transparent = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("repr"))
        .any(|attr| {
            attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|reprs| reprs.iter().any(|repr| repr.path().is_ident("transparent")))
        });
    if !transparent {
        return syn::Error::new_spanned(
            name,
            "#[derive(RNativeType)] requires #[repr(transparent)] for native R buffer access",
        )
        .into_compile_error()
        .into();
    }

    let expanded = quote::quote! {
        impl #impl_generics ::miniextendr_api::RNativeType for #name #ty_generics #where_clause {
            const SEXP_TYPE: ::miniextendr_api::SEXPTYPE =
                <#inner_ty as ::miniextendr_api::RNativeType>::SEXP_TYPE;

            const R_NA: Self = {
                let val = <#inner_ty as ::miniextendr_api::RNativeType>::R_NA;
                #elt_ctor
            };

            #[inline]
            unsafe fn dataptr_mut(sexp: ::miniextendr_api::SEXP) -> *mut Self {
                // Newtype is repr(transparent), so we can cast the pointer
                unsafe {
                    <#inner_ty as ::miniextendr_api::RNativeType>::dataptr_mut(sexp).cast()
                }
            }

            #[inline]
            fn elt(sexp: ::miniextendr_api::SEXP, i: isize) -> Self {
                let val = <#inner_ty as ::miniextendr_api::RNativeType>::elt(sexp, i);
                #elt_ctor
            }
        }

    };

    expanded.into()
}

/// Derive `IsNA` for a native newtype by forwarding to its single field.
///
/// Supports tuple, named and generic newtypes that also implement
/// `RNativeType`. The inner field must implement `IsNA`; existing where
/// clauses are preserved. This derive is separate from `RNativeType`
/// because raw `u8` storage has no NA representation.
///
/// ```
/// use miniextendr_api::{IsNA, RNativeType};
///
/// #[repr(transparent)]
/// #[derive(Clone, Copy, RNativeType, IsNA)]
/// struct Temperature { celsius: f64 }
///
/// assert!(Temperature::R_NA.is_na());
/// assert!(!Temperature { celsius: f64::NAN }.is_na());
/// ```
///
/// A generic native wrapper supports raw bytes too, but its `IsNA` impl is
/// available only when the inner type has NA:
///
/// ```compile_fail
/// use miniextendr_api::{IsNA, RNativeType};
///
/// #[repr(transparent)]
/// #[derive(Clone, Copy, RNativeType, IsNA)]
/// struct Native<T: RNativeType>(T);
///
/// fn requires_na<T: IsNA>() {}
/// requires_na::<Native<u8>>();
/// ```
#[proc_macro_derive(IsNA)]
pub fn derive_is_na(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    newtype_derive::derive_is_na(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
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
/// - Shared runtime `ErasedObject<MyCounter>` storage and destructor
/// - `__MX_BASE_VTABLE_MyCounter` - Base vtable with drop/query
/// - `__mx_wrap_MyCounter()` - Constructor returning `*mut mx_erased`
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

    // Standalone `#[derive(ExternalPtr)]` always emits the `IntoExternalPtr`
    // marker (enabling the blanket `IntoR`); only the struct-level
    // `prefer = "native"` dispatch path suppresses it (#1283).
    externalptr_derive::derive_external_ptr(input, true)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive an `all.equal()` method for the R class of an `ExternalPtr` type,
/// comparing the struct field by field (#2149).
///
/// Without it, `all.equal()` on two objects of a pointer-backed class
/// returns `TRUE` whatever they hold: base R compares an external pointer
/// only by its `data.class()`. The derived method compares each field with
/// base `all.equal()` on the field's R value, so each field keeps its own
/// method: numbers keep R's tolerance, lists and data frames use base's
/// methods, and a field holding another pointer class `T` uses `T`'s own
/// `all.equal()` method (another `AllEqual` type).
///
/// ```ignore
/// #[derive(ExternalPtr, AllEqual)]
/// #[all_equal(tolerance = 1e-6)]
/// pub struct Fit {
///     #[all_equal(label = "Estimates")]
///     estimates: Vec<f64>,
///     #[all_equal(tolerance = 0)]
///     iterations: i32,
///     model: ExternalPtr<Model>,
///     #[all_equal(skip)]
///     cache: RefCell<Option<Vec<f64>>>,
/// }
///
/// #[miniextendr(s3)]
/// impl Fit { /* ... */ }
/// ```
///
/// ```r
/// all.equal(fit_a, fit_b)                    # TRUE, or c("Estimates: Mean relative difference: 2e-05", ...)
/// all.equal(fit_a, fit_b, tolerance = 1e-3)  # the call's tolerance replaces the struct's default
/// ```
///
/// The type also needs `#[derive(ExternalPtr)]` and an impl block with a
/// class system (`env`, `r6`, `s3`, `s4` or `s7`): the method is
/// `all.equal.<Class>`, registered with `S3method()`, and for S7 also with
/// `S7::method(all.equal, Class)`. Installing a package with the derive on a
/// type without a class, or with a vctrs class (whose objects are R vectors
/// that `all.equal()` already compares), fails with a message naming it.
///
/// # The comparison
///
/// The method is `all.equal.<Class>(target, current, tolerance, scale, ...,
/// check.attributes, check.names)`, in `all.equal.numeric()`'s argument
/// order. A `current` of another class gives base's
/// `"target is <class>, current is <class>"`. Otherwise each compared field
/// is compared with `all.equal(target_field, current_field, ...)`, and the
/// result is `TRUE` or the messages of every differing field, in field
/// order, each prefixed as `all.equal.list()` does (`Component “estimates”:
/// ...`, `Component 1: ...` for a tuple field) or with the field's `label`.
/// An object restored from a saved session (`readRDS()`) is refused with
/// the usual `miniextendr_restored_*` error.
///
/// A field's R value is a clone converted with `IntoR`, or:
///
/// - `#[all_equal(with = "Self::f")]`: the value of a getter
///   `fn(&Self) -> impl IntoR`;
/// - a `Computed` field: the value of its `#[r_data(get = "...")]` getter;
/// - a `Sidecar<T>` field: its value in the pointer's protection list;
/// - an `ExternalPtr<T>` or `Option<ExternalPtr<T>>` field: the pointer
///   itself, not a clone.
///
/// The `RSidecar` selector is not compared.
///
/// A field whose type names a pointer class `T` of the package (`T`,
/// `ExternalPtr<T>`, `Option<ExternalPtr<T>>`, or a `Vec` of `T` or
/// `ExternalPtr<T>`) has its value given `T`'s R class, as a method returning that type does, so `T`'s own
/// method compares it: give `T` the derive too, since without a method of
/// its own `T` is compared by `all.equal.default()`, which finds any two
/// pointers equal. The derive can't see a getter's type, so a pointer a
/// `with` getter or a `Computed` field returns stays bare, as does a class
/// of another package (#2191).
///
/// # Settings
///
/// `tolerance`, `scale`, `check_attributes` and `check_names` (the R
/// arguments `check.attributes` / `check.names`) are formals of the method.
/// Each is passed to a field's `all.equal()` only when something sets it, so
/// a field's own method otherwise keeps its default (a nested `AllEqual`
/// type its own struct-level settings). Precedence: the field's
/// `#[all_equal(...)]`, then the call, then the struct's `#[all_equal(...)]`
/// (the formal's default), then the field's own method's default. Other
/// arguments of the call are passed on to every field through `...`.
///
/// On the struct, `#[all_equal(...)]` takes the settings. On a field it
/// takes the settings, which the call can't override, and:
///
/// - `skip`: the field is not compared (alone: it takes no other option);
/// - `label = "..."`: the field's messages start with `"<label>: "`;
/// - `with = "Self::f"`: the getter whose value is compared.
#[proc_macro_derive(AllEqual, attributes(all_equal))]
pub fn derive_all_equal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    all_equal_derive::derive_all_equal(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP integer vector data types.
///
/// Auto-implements `AltrepLen`, `AltIntegerData`, and the low-level ALTREP
/// trait impls (`Altrep`, `AltVec`, `AltInteger`, `InferBase`).
///
/// # Attributes
///
/// - `#[altrep(len = "field_name")]` - Specify length field (auto-detects "len" or "length")
/// - `#[altrep(elt = "field_name")]` - For constant vectors, specify which field provides elements
/// - `#[altrep(dataptr)]` - Enable direct data-pointer access
/// - `#[altrep(serialize)]` - Enable ALTREP serialization support
/// - `#[altrep(subset)]` - Enable `Extract_subset` optimization
/// - `#[altrep(no_lowlevel)]` - Skip the automatic low-level trait impls
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
#[proc_macro_derive(AltrepInteger, attributes(altrep, r_data))]
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
#[proc_macro_derive(AltrepReal, attributes(altrep, r_data))]
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
#[proc_macro_derive(AltrepLogical, attributes(altrep, r_data))]
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
#[proc_macro_derive(AltrepRaw, attributes(altrep, r_data))]
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
#[proc_macro_derive(AltrepString, attributes(altrep, r_data))]
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
#[proc_macro_derive(AltrepComplex, attributes(altrep, r_data))]
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
///
/// The generated class accepts R's writes (`x[[i]] <- v`): the first one copies
/// the elements into a plain list in data2, which element reads use from then on.
#[proc_macro_derive(AltrepList, attributes(altrep, r_data))]
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
#[proc_macro_derive(Altrep, attributes(altrep, r_data))]
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
/// - Named structs: extract by field name. A field whose name appears more
///   than once in the list is an error (`SexpError::DuplicateName`), since the
///   struct could keep only one of the values; other names, repeated or not,
///   are ignored.
/// - Tuple structs: extract by position (0, 1, 2, ...)
/// - Fields annotated `#[into_list(ignore)]` are not read and are initialized with
///   `Default::default()`, so their types must be `Default`
#[proc_macro_derive(TryFromList, attributes(into_list))]
pub fn derive_try_from_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_try_from_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferList`: emits an `IntoR` impl selecting list as the type's default
/// Rust→R conversion (via `IntoList::into_list`).
///
/// A type carries exactly one representation default: stacking two `Prefer*`
/// derives is a compile error. Each `Prefer*` derive emits a fixed-name marker
/// const, so a second one triggers a guided `duplicate definitions with name
/// __miniextendr_conflicting_Prefer_derives__keep_ONE_or_use_call_site_As_wrappers`
/// error (alongside the raw conflicting-`IntoR`-impl error) — keep one `Prefer*`,
/// or drop them all and choose a representation per return value at the call site
/// with an `As*` wrapper (`AsList`, `AsExternalPtr`, `AsDataFrame`, ...).
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

/// Derive `PreferExternalPtr`: return a type with a hand-written `TypedExternal`
/// implementation as an R external pointer. `#[derive(ExternalPtr)]` already
/// supplies `IntoR`; adding this derive to it creates conflicting implementations.
///
/// # Example
///
/// ```ignore
/// #[derive(PreferExternalPtr)]
/// struct Model { weights: Vec<f64> }
///
/// impl miniextendr_api::TypedExternal for Model {
///     const TYPE_NAME: &'static str = "Model";
///     const TYPE_NAME_CSTR: &'static [u8] = b"Model\0";
///     const TYPE_ID_CSTR: &'static [u8] = concat!(module_path!(), "::Model\0").as_bytes();
/// }
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
/// #[repr(transparent)]
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

/// Derive `PreferVctrs`: emits an `IntoR` impl converting the type to its R vctrs object via
/// `IntoVctrs::into_vctrs`.
///
/// Pair with `#[derive(Vctrs)]` (which supplies `IntoVctrs`) so the type can be returned
/// directly from `#[miniextendr]` functions instead of `value.into_vctrs().map_err(...)`.
///
/// # Example
///
/// ```ignore
/// #[derive(Vctrs, PreferVctrs)]
/// #[vctrs(class = "percent", base = "double")]
/// struct Percent { #[vctrs(data)] values: Vec<f64> }
/// // IntoR builds the `percent` vctrs vector; a build failure becomes an R error.
/// ```
#[cfg(feature = "vctrs")]
#[proc_macro_derive(PreferVctrs)]
pub fn derive_prefer_vctrs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_vctrs(input)
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
///
/// # Requirements
///
/// Of each field type the derive asks only what turning that field into
/// columns needs, and a missing impl is reported at the field:
///
/// - a scalar or opaque column: its column converts to R (`Vec<T>: IntoR`;
///   in an enum, `Vec<Option<T>>`);
/// - `[T; N]`, or `width` / `expand` over an owned `Vec<T>` / `Box<[T]>`:
///   `T`'s columns convert to R; the elements are moved, so `T` needs no
///   `Clone` or `Copy`. Over a borrowed `&[T; N]` / `&[T]`, `T` must also be
///   `Clone`;
/// - a flattened struct or nested enum: `#[derive(DataFrameRow)]`;
/// - `as_factor`: `UnitEnumFactor`, which `#[derive(DataFrameRow)]` on a
///   unit-only enum and `#[derive(RFactor)]` both provide;
/// - `as_list`: `IntoList`;
/// - `skip`: nothing.
///
/// Everything else is generated only where the field types allow it. The
/// companion is `Debug` / `Clone` when every column type is. A struct
/// companion iterates its rows when every `skip` field's type is `Default`
/// (they are rebuilt with `Default::default()`). With `rayon`,
/// `from_rows_par` fills the columns in parallel when the rows and column
/// values are `Send`, placeholder-filled struct columns are `Default`, and
/// flattened / `as_list` values are `Clone`; otherwise it fills them
/// sequentially.
///
/// # Public surface (which verbs to call)
///
/// Every capability the derive provides has a documented, trait-based (or `std`)
/// verb — reach for these, not any incidental inherent plumbing:
///
/// - **Rows → R `data.frame`**: `rows.into_dataframe()?` (owned, GC-rooted
///   `BuiltDataFrame`) or `rows.wrap_data_frame()` (deferred `IntoR` wrapper);
///   parallel variant `rows.into_dataframe_par()?`. From the `IntoDataFrame` /
///   `AsDataFrameExt` traits (both re-exported from `miniextendr_api::prelude`).
/// - **R `data.frame` → rows**: `Vec::<Row>::from_dataframe(&df)?` (parallel:
///   `Vec::<Row>::from_dataframe_par(&df)?`), from the `FromDataFrame` trait — or
///   the one-call `Row::try_from_dataframe(sexp)` reader on the row type.
/// - **Rows ↔ the pure-Rust columnar companion** (`<Row>DataFrame`, `Vec`-columns,
///   no R involved): the `ColumnarFrame` trait (in the prelude) —
///   `<Row>DataFrame::from_rows(rows)` / `from_rows_par(rows)` (parallel build of
///   the *companion*, which `into_dataframe_par` does not give you) and, for
///   row-iterable companions, `companion.into_rows()`. `Vec<Row>: Into<companion>`
///   and the companion's `IntoIterator` are the equivalent `std` verbs.
/// - **Enum split representation**: `rows.into_dataframe_split()` returns one
///   `data.frame` per variant (only that variant's columns — no NA fill) as a
///   GC-rooted `DataFrameShape` — `Bare` for a single-variant enum,
///   `PerVariantList` (an R named list) otherwise — from the
///   `IntoDataFrameSplit` trait (in the prelude). Enum rows only; struct
///   derives don't partition.
///
/// The generated `<Row>DataFrame` / `<Row>DataFrameIter` types are intermediate
/// column-oriented companions; you rarely name them directly.
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
///
/// # Requirements
///
/// The enum must be `Copy` (`RFactor` and its supertrait `MatchArg` require it):
/// derive `Copy, Clone` with it.
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
/// - `#[match_arg(alias = "name")]` - Another spelling of a variant's choice, matched when typed
///   in full; the errors that list the choices name it after them (`(or "name" for "choice")`)
/// - `#[match_arg(alias = "name", listed = false)]` - The same, named by no error
///
/// # Requirements
///
/// The enum must be `Copy` (`MatchArg: Copy`): derive `Copy, Clone` with it.
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

/// Derive an R condition type for a struct or enum: `RConditionData` (the R
/// class vector, the message and the `data` fields handlers read as
/// `e$<name>`) and `From<Self>` for `miniextendr_api::RCondition`.
///
/// - `#[condition(class = "…")]` on the type sets the family class (default:
///   the type name in snake_case); on an enum variant it sets the member class
///   (default: `<family>_<variant in snake_case>`). Variants report
///   `c(<member>, <family>)`, structs `c(<family>)`.
/// - `#[condition(message = "…")]` on a struct or a variant is a `format!`
///   string over the fields (tuple fields are `_0`, `_1`, …). Without it the
///   message is the type's `Display` rendering.
/// - Every field becomes a `data` entry under its own name, converted with
///   `RValue::from(field.clone())`, so its type must be `Clone`.
///   `#[condition(rename = "…")]`,
///   `#[condition(skip)]` and `#[condition(debug)]` (attach the `Debug`
///   rendering instead) adjust that; tuple fields need `rename` or `skip`. The
///   reserved slots `message`, `call` and `kind` are rejected at compile time.
/// - `#[condition(kind = error | warning | message | condition)]` on the type
///   or a variant (the variant wins) is the kind `RCondition::from(value)`
///   gives it; the default is `error`. Then `.defer()` signals it after the
///   function returns its value, and `.raise()` signals it now.
///
/// The kind is read only through `RCondition::from`. Returned as `Err(value)`
/// from a `Result<T, E>`, the value is always an R error, and
/// [`defer_warning`](https://docs.rs/miniextendr-api) / `defer_message` signal
/// the kind they are named after. Generic types are not supported.
#[proc_macro_derive(RCondition, attributes(condition))]
pub fn derive_r_condition(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    condition_derive::derive_r_condition(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `TryFromSexp` for a single-field newtype: forward the R → Rust
/// conversion to the inner type.
///
/// Generates a scalar `TryFromSexp` impl that delegates to the inner type (so the
/// newtype inherits its exact SEXPTYPE checks, NA policy, and error text), plus a
/// `TryFromSexpElement` marker impl. The marker lets `miniextendr-api`'s container
/// blankets light up `Vec<T>` / `Option<T>` / `Vec<Option<T>>` automatically,
/// each one where the inner type's container converts.
///
/// Don't use it on a type that also gets `try_from_sexp_via_str_parse!`: both
/// implement `TryFromSexpElement`.
///
/// # Usage
///
/// ```ignore
/// use uuid::Uuid;
///
/// #[derive(TryFromSexp)]            // R -> Rust only
/// struct Pattern(regex::Regex);
///
/// #[derive(TryFromSexp, IntoR)]     // round-trip; Vec/Option containers work too
/// struct UserId(Uuid);
/// ```
///
/// Direction is chosen by which derive you list — derive only `TryFromSexp` for
/// inner types that read from R but cannot be written back (e.g. `regex::Regex`).
///
/// # Refusing values: `#[try_from_sexp(validate = path)]`
///
/// The containers read the inner type and wrap it; they never run the
/// newtype's own `try_from_sexp`. To refuse a value the inner type accepts,
/// name a check, `fn(SEXP) -> Result<(), E>` with `E: Into<SexpError>`. It
/// becomes `TryFromSexpElement::check_sexp`, which runs on the R value before
/// it is read, in every shape: the scalar, on any input but `NULL` for
/// `Option<T>`, and for `Vec<T>` and `Vec<Option<T>>` once on the whole vector
/// of an atomic inner type, or on each element (but `NULL`) of a list when the
/// inner type is `List` (a list of R objects, each with its own class).
///
/// ```ignore
/// use miniextendr_api::condition::RError;
/// use miniextendr_api::{SEXP, SexpExt, TryFromSexp};
///
/// fn plain_number(x: SEXP) -> Result<(), RError> {
///     if x.inherits_class(c"difftime") {
///         return Err(RError::new("give the duration as a plain number").class("pkg_unit_error"));
///     }
///     Ok(())
/// }
///
/// #[derive(TryFromSexp)]
/// #[try_from_sexp(validate = plain_number)]
/// struct Duration(f64);
/// ```
///
/// An `RError` refusal (or one of any `RConditionData` type) keeps its classes
/// and fields on the argument error, before the crate's
/// `conversion_error_class`, with `e$param` and the message
/// `'<p>' must be <expected>: <message>`. With `validate`, the newtype's
/// scalar error is `SexpError`, so the inner type's error must convert into it,
/// as every built-in conversion error and every `RConditionData` type does.
/// `validate` is the only key, and the attribute goes on the struct.
#[proc_macro_derive(TryFromSexp, attributes(try_from_sexp))]
pub fn derive_try_from_sexp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    newtype_derive::derive_try_from_sexp(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `IntoR` for a single-field newtype: forward the Rust → R conversion to
/// the inner type.
///
/// Generates a scalar `IntoR` impl that delegates to the inner type, plus an
/// `IntoRNewtype` marker (powering the `Option<T>` / `Vec<Option<T>>` container
/// blankets) and a concrete `IntoRVecElement` impl (powering `Vec<T>`). See
/// `#[derive(TryFromSexp)]` for usage.
///
/// Do not derive both `IntoR` and `MatchArg` on the same type: both feed the
/// single `IntoR for Vec<T>` blanket slot and would collide (E0119).
#[proc_macro_derive(IntoR)]
pub fn derive_into_r(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    newtype_derive::derive_into_r(input)
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
/// pub fn my_func(_: ...) -> String {
///     // `dots_typed` is automatically created and validated
///     let x: f64 = dots_typed.get("x").expect("x");
///     let y: f64 = dots_typed.get("y").expect("y");
///     format!("x={}, y={}", x, y)
/// }
/// ```
///
/// This injects validation at the start of the function body, reading the
/// dots binding (`__miniextendr_dots` for `_: ...`, otherwise the name of the
/// `name: ...` / `name: &Dots` parameter):
/// ```ignore
/// let dots_typed = __miniextendr_dots.typed(typed_list!(...))
///     .unwrap_or_else(|e| miniextendr_api::arg_error!(
///         param = "...", "dots validation failed: {e}"
///     ));
/// ```
///
/// See the [`#[miniextendr]`](macro@miniextendr) attribute documentation for more details.
///
#[proc_macro]
pub fn typed_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as typed_list::TypedListInput);
    typed_list::expand_typed_list(parsed).into()
}

/// Define a compile-time-validated wrapper for an R `data.frame` input.
///
/// `typed_dataframe!` mirrors [`typed_list!`] for the data.frame shape:
/// declare the columns once, get a struct that implements `TryFromSexp`
/// (validating both the `data.frame` class and per-column SEXPTYPE) plus
/// per-column borrowed accessors that return `&[T]`.
///
/// # Syntax
///
/// ```ignore
/// typed_dataframe! {
///     /// The shape we accept for the Theoph PK dataset.
///     pub TheophDf {
///         subject: i32,
///         weight: f64,
///         dose: f64,
///         flag: Option<i32>,   // optional column
///     }
/// }
/// ```
///
/// For strict mode (reject any column not declared):
/// ```ignore
/// typed_dataframe! {
///     @exact;
///     pub Strict { x: i32 }
/// }
/// ```
///
/// # Supported element types
///
/// v1 supports column element types that implement
/// `miniextendr_api::RNativeType`:
///
/// - `i32` — `INTSXP`
/// - `f64` — `REALSXP`
/// - `u8` — `RAWSXP`
/// - `miniextendr_api::RLogical` — `LGLSXP`
/// - `miniextendr_api::Rcomplex` — `CPLXSXP`
///
/// `String`/`&str` column types are not yet supported (character vectors
/// don't expose a contiguous slice). `bool` is also not yet supported as
/// a direct field type — use `RLogical` and convert per-element, or
/// follow the open follow-up issues from PR #698.
///
/// # Generated API
///
/// For each `name: T` column the macro emits:
/// - `pub fn name(&self) -> &[T]` (required)
/// - `pub fn name(&self) -> Option<&[T]>` (optional, `Option<T>`)
///
/// Plus housekeeping:
/// - `pub fn nrow(&self) -> usize`
/// - `pub fn ncol(&self) -> usize` (count of *declared* columns)
/// - `pub fn as_sexp(&self) -> SEXP`
///
/// All borrowed accessors are bound to `&self`; the SEXP is protected
/// by the surrounding `#[miniextendr]` call wrapper while the struct is
/// alive.
///
/// # Error reporting
///
/// `TryFromSexp::try_from_sexp` batches every per-column error into a
/// single `SexpError::InvalidValue`, so the R user sees one diagnostic
/// covering all missing or wrong-typed columns rather than a sequence of
/// stop-on-first-failure messages.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, typed_dataframe};
///
/// typed_dataframe! {
///     pub TheophDf {
///         subject: i32,
///         weight: f64,
///         dose: f64,
///     }
/// }
///
/// #[miniextendr]
/// pub fn theoph_nrow(df: TheophDf) -> i32 {
///     // df.subject() -> &[i32], df.weight() -> &[f64]
///     // Lengths are guaranteed equal across columns (data.frame invariant).
///     df.nrow() as i32
/// }
/// ```
///
/// [`typed_list!`]: macro@typed_list
#[proc_macro]
pub fn typed_dataframe(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as typed_dataframe::TypedDataframeInput);
    typed_dataframe::expand_typed_dataframe(parsed).into()
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

/// Evaluate R code written as **Rust tokens**, validated at compile time.
///
/// `r!` takes a single R expression as a token stream, `stringify!`s it into a
/// static R source string at build time, and evaluates it via
/// `miniextendr_api::expression::r_eval_str` (the same protect-safe parse + eval
/// path as `r_str!`).
///
/// # What you get today
///
/// Because the argument is a Rust token tree, the Rust front-end already
/// rejects **unbalanced delimiters** (`r!(f(1, 2)` won't compile) and
/// lexically invalid tokens before R ever sees the string — a cheap
/// compile-time guard over the pure-runtime `r_str!`. The source is lowered to
/// a `&'static str` (`stringify!`), so there is no `format!` allocation at the
/// call site.
///
/// This proc-macro additionally validates a conservative subset of known-bad
/// R syntax constructs (trailing binary operators, consecutive non-unary
/// binary operators, bare `if`/`while`/`for` without a body, etc.) and emits
/// a precise compile error pointing at the offending token. Empty (missing)
/// call arguments — `f(, x)`, `matrix(, 2, 2)` — are valid R and pass.
///
/// # What is deferred
///
/// Direct `Rf_lang*` call-tree lowering (skipping the runtime parser entirely)
/// is tracked as a follow-up in issue #938 (item 2). Until then `r!` parses
/// its static string at first evaluation, exactly like `r_str!`.
///
/// # Non-goals
///
/// A complete R grammar validator is not achievable over Rust tokens:
/// - Single-quoted strings (`'hello'`) and backtick-quoted names (`` `foo` ``)
///   already die at the Rust lexer — nothing to validate.
/// - `%op%` tokenises as `%`, ident, `%` and is accepted without analysis.
/// - Anything the validator cannot confidently classify as wrong passes through
///   unvalidated (conservative reject-only-known-bad design).
///
/// # Forms
///
/// - `r!(R tokens…)` — evaluate in `R_GlobalEnv`.
/// - `r!(env: e; R tokens…)` — evaluate in the environment SEXP `e`. The
///   leading `env: <expr> ;` is consumed as Rust, the rest is R source.
///
/// Both evaluate to `Result<SEXP, REvalError>`; the `SEXP` is **unprotected**.
///
/// # Safety
///
/// Expands to an `unsafe` block; the underlying FFI is `#[r_ffi_checked]`, so
/// calls from a worker thread are serialized onto the R thread.
///
/// # Example
///
/// ```ignore
/// let three = r!(1L + 2L)?;
/// let rows = r!(getFromNamespace(".theoph_rows", "dataframeflows")())?;
/// let in_env = r!(env: my_env; x + 1)?;
/// ```
#[proc_macro]
pub fn r(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    r_macro::expand(input)
}

/// Re-export other packages' R functions from this package.
///
/// Each item names a package and one of its exports. The generated R
/// wrappers get roxygen2's re-export block for it, so `devtools::document()`
/// writes `importFrom(pkg, name)` and `export(name)` to `NAMESPACE` and lists
/// `name` on `man/reexports.Rd`:
///
/// ```r
/// #' @importFrom generics tidy
/// #' @export
/// generics::tidy
/// ```
///
/// The usual reason is a generic the package registers S3 methods for, such
/// as `generics::tidy()` for `#[miniextendr(s3(generic = "tidy", class =
/// "my_fit"))]`: with only the package attached, a bare `tidy(fit)` then finds
/// the generic and dispatches to the method.
///
/// ```ignore
/// use miniextendr_api::reexport;
///
/// reexport!(generics::tidy, generics::glance, generics::augment);
/// reexport!(data.table::fread, stats::na.omit, magrittr::"%>%");
/// ```
///
/// - The package and the name are identifiers joined by `.`. A name that is
///   not a syntactic R name is a string literal (`magrittr::"%>%"`); the R
///   code backticks it.
/// - The package has to be in the `Imports` field of `DESCRIPTION`, which the
///   `importFrom()` needs; `R CMD check` reports a missing one.
/// - Any number of invocations may name the same `pkg::name`, so each module
///   can re-export the generics its methods are for; the wrappers get one
///   block per `pkg::name`.
/// - The wrapper writer refuses, at install time, a name re-exported from two
///   packages, and a name the package also defines or exports, in its
///   generated wrappers (an S3 generic of an impl block, a function, an
///   `@rawNamespace export(...)`) or a hand-written file in `R/` (a top-level
///   `name <- function(...)`, an `@export name`).
/// - A package name that is not an R package name, a name that is not an R
///   name, and two packages for one name in one invocation are compile
///   errors, all reported at once.
#[proc_macro]
pub fn reexport(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    reexport::expand(input.into()).into()
}

/// Internal proc macro used by TPIE (Trait-Provided Impl Expansion).
///
/// Called by the metadata macro re-exported alongside each `#[miniextendr]` trait.
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
///
/// Call it at the crate root: it also defines the crate's marker type, under
/// which `serde_error_shape!` declarations are keyed and which, with the API's
/// `serde` feature, every `#[miniextendr]` function or method returning a
/// `Result` names.
#[proc_macro]
pub fn miniextendr_init(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pkg_name: syn::Ident = if input.is_empty() {
        // Auto-detect from CARGO_CRATE_NAME (set by cargo during compilation).
        // Cargo normalizes hyphens → underscores, so this is almost always a
        // valid Rust/C identifier. Still parse through syn so malformed values
        // surface as a compile error rather than an ICE.
        let name = match std::env::var("CARGO_CRATE_NAME") {
            Ok(n) => n,
            Err(_) => {
                return syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "CARGO_CRATE_NAME not set. Either pass the package name explicitly: \
                     miniextendr_init!(mypkg), or ensure you're building with cargo.",
                )
                .into_compile_error()
                .into();
            }
        };
        match syn::parse_str::<syn::Ident>(&name) {
            Ok(id) => id,
            Err(_) => {
                return syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!(
                        "CARGO_CRATE_NAME `{name}` is not a valid C identifier; \
                         R_init_<pkg> must match `[A-Za-z_][A-Za-z0-9_]*`. \
                         Pass the name explicitly: miniextendr_init!(my_pkg)."
                    ),
                )
                .into_compile_error()
                .into();
            }
        }
    } else {
        syn::parse_macro_input!(input as syn::Ident)
    };
    let fn_name = syn::Ident::new(&format!("R_init_{}", pkg_name), pkg_name.span());
    let unload_name = syn::Ident::new(&format!("R_unload_{}", pkg_name), pkg_name.span());

    // Build a byte string literal with NUL terminator for the package name.
    let mut name_bytes = pkg_name.to_string().into_bytes();
    name_bytes.push(0);
    let byte_lit = syn::LitByteStr::new(&name_bytes, pkg_name.span());

    // The crate's `conversion_error_class`, for the wrappers preamble's
    // `.miniextendr_conversion_error_class`: the R-side argument checks raise
    // with the classes the conversion arms of this crate carry (#1591).
    let conversion_error_class = crate::crate_config::conversion_error_class();
    let init_marker = crate::miniextendr_fn::init_marker_ident();

    let expanded = quote::quote! {
        /// The crate's marker type. A `serde_error_shape!` declaration is an
        /// impl keyed by it, and the generated `Err` arms look the declaration
        /// up under it, so the macro must be called at the crate root.
        #[doc(hidden)]
        pub enum #init_marker {}

        /// The package crate's `conversion_error_class`, read by the
        /// wrapper-generation pass and the raising guards (host-only; the
        /// `wasm_registry.rs` snapshot carries it to wasm32).
        #[cfg(not(target_arch = "wasm32"))]
        #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CONVERSION_ERROR_CLASS)]
        #[linkme(crate = ::miniextendr_api::linkme)]
        #[doc(hidden)]
        static __MX_CONVERSION_ERROR_CLASS: ::miniextendr_api::registry::ConversionErrorClassEntry =
            ::miniextendr_api::registry::ConversionErrorClassEntry {
                classes: &[#(#conversion_error_class),*],
            };

        // wasm32: pull in the host-generated `wasm_registry.rs` snapshot
        // (committed by the user crate, regenerated by `just wasm-prepare`).
        // The path is relative to the file invoking `miniextendr_init!()` —
        // by convention `<crate>/src/rust/lib.rs`, so the snapshot sits at
        // `<crate>/src/rust/wasm_registry.rs`. Module is `#[doc(hidden)]`
        // because it's purely an internal bridge between the user crate's
        // wrapper / vtable / register-fn `#[no_mangle]` exports and
        // `miniextendr_api::registry::install_wasm_runtime_slices`.
        #[cfg(target_arch = "wasm32")]
        #[path = "wasm_registry.rs"]
        #[doc(hidden)]
        mod __miniextendr_wasm_registry;

        #[unsafe(no_mangle)]
        pub unsafe extern "C-unwind" fn #fn_name(
            dll: *mut ::miniextendr_api::sys::DllInfo,
        ) {
            let __miniextendr_init_result = ::std::panic::catch_unwind(
                ::std::panic::AssertUnwindSafe(|| {
                    // wasm32: install the pre-generated runtime tables before
                    // package_init runs. linkme didn't gather anything (the slices
                    // are OnceLock-backed on wasm32), so register_routines /
                    // universal_query would otherwise see empty slices, and the
                    // raising guards no crate conversion classes (#1768).
                    #[cfg(target_arch = "wasm32")]
                    ::miniextendr_api::registry::install_wasm_runtime_slices(
                        __miniextendr_wasm_registry::MX_CALL_DEFS_WASM,
                        __miniextendr_wasm_registry::MX_ALTREP_REGISTRATIONS_WASM,
                        __miniextendr_wasm_registry::MX_TRAIT_DISPATCH_WASM,
                        __miniextendr_wasm_registry::MX_CONVERSION_ERROR_CLASS_WASM,
                    );

                    unsafe {
                        // SAFETY: byte literal is a valid NUL-terminated string produced by the macro.
                        let pkg_name = ::std::ffi::CStr::from_bytes_with_nul_unchecked(#byte_lit);
                        ::miniextendr_api::init::package_init(dll, pkg_name);
                    }
                }),
            );
            if let ::core::result::Result::Err(payload) = __miniextendr_init_result {
                // package_init normally raises through its R guard. This is
                // the last-resort boundary for runtime/guard initialization:
                // no Rust-owned message or payload may survive Rf_error.
                ::core::mem::drop(payload);
                unsafe {
                    ::miniextendr_api::sys::Rf_error_unchecked(
                        c"miniextendr package initialization panicked before completing".as_ptr(),
                    );
                }
            }
        }

        /// R_unload_<pkg> entry point — R calls this on `detach(unload=TRUE)` /
        /// `dyn.unload()`. Signals the miniextendr worker thread (if enabled)
        /// to exit cleanly. See `#103`.
        #[unsafe(no_mangle)]
        pub unsafe extern "C-unwind" fn #unload_name(
            _dll: *mut ::miniextendr_api::sys::DllInfo,
        ) {
            ::miniextendr_api::worker::miniextendr_runtime_shutdown();
        }

        /// Linker anchor: stub.c takes the address of this symbol to force the
        /// linker to pull in the user crate's archive member from the staticlib.
        /// With codegen-units = 1, this single member contains all linkme
        /// distributed_slice entries. The name is package-independent so stub.c
        /// doesn't need configure substitution.
        ///
        /// Defined as a function rather than a static so it stays exported under
        /// the webR wasm RUSTFLAG -Zdefault-visibility=hidden, which keeps
        /// no_mangle functions exported (like the R_init entry point) but hides
        /// no_mangle statics. A hidden anchor breaks wasm side-module dlopen
        /// (bad export type, undefined). See miniextendr webR notes (#494).
        #[unsafe(no_mangle)]
        pub extern "C" fn miniextendr_force_link() {}
    };

    expanded.into()
}

#[cfg(test)]
mod tests;
