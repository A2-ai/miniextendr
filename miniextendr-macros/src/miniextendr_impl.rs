//! # Impl-block Parsing and Wrapper Generation
//!
//! This module handles `#[miniextendr]` applied to inherent impl blocks,
//! generating R wrappers for different class systems.
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         #[miniextendr(r6)]                              │
//! │                         impl MyType { ... }                             │
//! └─────────────────────────────────────────────────────────────────────────┘
//!                                    │
//!                                    ▼
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                           PARSING PHASE                                 │
//! │  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────────┐ │
//! │  │   ImplAttrs     │    │  ParsedMethod   │    │    ParsedImpl       │ │
//! │  │ - class_system  │    │ - ident         │───▶│ - type_ident        │ │
//! │  │ - class_name    │    │ - receiver      │    │ - class_system      │ │
//! │  └─────────────────┘    │ - sig           │    │ - methods[]         │ │
//! │                         │ - doc_tags      │    │ - doc_tags          │ │
//! │                         │ - method_attrs  │    └─────────────────────┘ │
//! │                         └─────────────────┘                             │
//! └─────────────────────────────────────────────────────────────────────────┘
//!                                    │
//!                                    ▼
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        CODE GENERATION PHASE                            │
//! │                                                                         │
//! │  For each method:                                                       │
//! │  ┌─────────────────────────────────────────────────────────────────┐   │
//! │  │ generate_c_wrapper_for_method()                                 │   │
//! │  │   └─▶ CWrapperContext (shared builder from c_wrapper_builder)   │   │
//! │  │         - thread strategy (main vs worker)                      │   │
//! │  │         - SEXP→Rust conversion                                  │   │
//! │  │         - return handling                                       │   │
//! │  └─────────────────────────────────────────────────────────────────┘   │
//! │                                                                         │
//! │  For the whole impl:                                                    │
//! │  ┌─────────────────────────────────────────────────────────────────┐   │
//! │  │ generate_{class_system}_r_wrapper()                             │   │
//! │  │   - generate_env_r_wrapper()   → Type$method(self, ...)         │   │
//! │  │   - generate_r6_r_wrapper()    → R6Class with methods           │   │
//! │  │   - generate_s3_r_wrapper()    → generic + method.Type          │   │
//! │  │   - generate_s4_r_wrapper()    → setClass + setMethod           │   │
//! │  │   - generate_s7_r_wrapper()    → new_class + method<-           │   │
//! │  └─────────────────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────────────────┘
//!                                    │
//!                                    ▼
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                              OUTPUTS                                    │
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐ │
//! │  │ C wrapper fns   │  │ R wrapper code  │  │  R_CallMethodDef       │ │
//! │  │ C_Type__method  │  │ (as const str)  │  │  registration entries  │ │
//! │  └─────────────────┘  └─────────────────┘  └─────────────────────────┘ │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Supported Class Systems
//!
//! | System | Syntax | R Pattern | Use Case |
//! |--------|--------|-----------|----------|
//! | **Env** | `#[miniextendr]` | `obj$method()` | Simple, environment-based dispatch |
//! | **R6** | `#[miniextendr(r6)]` | `R6Class` with `$new()` | OOP with encapsulation |
//! | **S3** | `#[miniextendr(s3)]` | `generic(obj)` dispatch | Idiomatic R generics |
//! | **S4** | `#[miniextendr(s4)]` | `setClass`/`setMethod` | Formal OOP, multiple dispatch |
//! | **S7** | `#[miniextendr(s7)]` | `new_class`/`new_generic` | Modern R OOP |
//! | **vctrs** | `#[miniextendr(vctrs)]` | `new_vctr`/`new_rcrd`/`new_list_of` | vctrs-compatible vectors |
//!
//! ## Method Categorization
//!
//! Methods are categorized by their receiver type:
//!
//! | Receiver | [`ReceiverKind`] | Generated as |
//! |----------|------------------|--------------|
//! | `&self` | `Ref` | Instance method (immutable) |
//! | `&mut self` | `RefMut` | Instance method (mutable, chainable) |
//! | `self: &ExternalPtr<Self>` | `ExternalPtrRef` | Instance method (immutable, full ExternalPtr access) |
//! | `self: &mut ExternalPtr<Self>` | `ExternalPtrRefMut` | Instance method (mutable, full ExternalPtr access) |
//! | `self: ExternalPtr<Self>` | `ExternalPtrValue` | Instance method (owned ExternalPtr, full access) |
//! | `self` | `Value` | Consuming method (not supported in v1) |
//! | (none) | `None` | Static method or constructor |
//!
//! Special methods:
//! - **Constructor**: Returns `Self`, marked with `#[miniextendr(constructor)]` or named `new`
//! - **Finalizer**: R6 only, marked with `#[miniextendr(r6(finalize))]`
//! - **Private**: R6 only, marked with `#[miniextendr(r6(private))]`
//!
//! ## Shared Builders
//!
//! This module uses shared infrastructure from:
//! - [`crate::c_wrapper_builder`]: C wrapper generation with thread strategy
//! - [`crate::r_wrapper_builder`]: R function signatures and `.Call()` args
//! - [`crate::method_return_builder`]: Return value handling per class system
//! - [`crate::roxygen`]: Documentation extraction from Rust doc comments
//!
//! ## Example
//!
//! ```ignore
//! #[miniextendr(r6)]
//! impl Counter {
//!     fn new(value: i32) -> Self { Counter { value } }
//!     fn get(&self) -> i32 { self.value }
//!     fn increment(&mut self) { self.value += 1; }
//! }
//! ```
//!
//! Generates:
//! - C wrappers: `C_Counter__new`, `C_Counter__get`, `C_Counter__increment`
//! - R6Class with `initialize`, `get`, `increment` methods
//! - Registration entries for R's `.Call()` interface

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

/// Check if a type is `ExternalPtr<...>` (possibly fully qualified).
fn is_external_ptr_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "ExternalPtr")
            .unwrap_or(false)
    } else {
        false
    }
}

/// Replace every occurrence of the `self` keyword/ident in a TokenStream
/// with a replacement identifier. Does NOT touch `Self` (capital S).
fn replace_self_in_tokens(
    tokens: proc_macro2::TokenStream,
    replacement: &str,
) -> proc_macro2::TokenStream {
    let replacement_ident = proc_macro2::Ident::new(replacement, proc_macro2::Span::call_site());
    tokens
        .into_iter()
        .map(|tt| match tt {
            proc_macro2::TokenTree::Ident(ref ident) if ident == "self" => {
                proc_macro2::TokenTree::Ident(replacement_ident.clone())
            }
            proc_macro2::TokenTree::Group(group) => {
                let new_stream = replace_self_in_tokens(group.stream(), replacement);
                let mut new_group = proc_macro2::Group::new(group.delimiter(), new_stream);
                new_group.set_span(group.span());
                proc_macro2::TokenTree::Group(new_group)
            }
            other => other,
        })
        .collect()
}

/// Rewrite methods with ExternalPtr-based receivers so they compile on stable Rust
/// (which lacks `arbitrary_self_types`).
///
/// Handles:
/// - `self: &ExternalPtr<Self>` → `__miniextendr_self: &ExternalPtr<Self>`
/// - `self: &mut ExternalPtr<Self>` → `__miniextendr_self: &mut ExternalPtr<Self>`
/// - `self: ExternalPtr<Self>` → `__miniextendr_self: ExternalPtr<Self>`
///
/// Also replaces all `self` references in the method body with `__miniextendr_self`.
fn rewrite_external_ptr_receivers(mut item_impl: syn::ItemImpl) -> syn::ItemImpl {
    for item in &mut item_impl.items {
        let syn::ImplItem::Fn(method) = item else {
            continue;
        };
        let Some(syn::FnArg::Receiver(receiver)) = method.sig.inputs.first() else {
            continue;
        };
        if receiver.colon_token.is_none() {
            continue;
        }

        // Determine if this is an ExternalPtr receiver and build the replacement param.
        let new_param: Option<syn::FnArg> =
            if let syn::Type::Reference(type_ref) = receiver.ty.as_ref() {
                // self: &ExternalPtr<Self> or self: &mut ExternalPtr<Self>
                if is_external_ptr_type(&type_ref.elem) {
                    let mutability = type_ref.mutability;
                    let inner_ty = &type_ref.elem;
                    Some(syn::parse_quote! {
                        __miniextendr_self: &#mutability #inner_ty
                    })
                } else {
                    None
                }
            } else if is_external_ptr_type(receiver.ty.as_ref()) {
                // self: ExternalPtr<Self> (by value)
                let inner_ty = &receiver.ty;
                Some(syn::parse_quote! {
                    __miniextendr_self: #inner_ty
                })
            } else {
                None
            };

        let Some(new_param) = new_param else {
            continue;
        };

        // Replace first parameter
        let inputs: Vec<syn::FnArg> = method.sig.inputs.iter().cloned().collect();
        let mut new_inputs: Vec<syn::FnArg> = Vec::with_capacity(inputs.len());
        new_inputs.push(new_param);
        new_inputs.extend(inputs.into_iter().skip(1));
        method.sig.inputs = new_inputs.into_iter().collect();

        // Replace `self` in method body
        let old_body = method.block.clone();
        let new_tokens = replace_self_in_tokens(old_body.into_token_stream(), "__miniextendr_self");
        method.block =
            syn::parse2(new_tokens).expect("failed to reparse method body after self replacement");
    }
    item_impl
}

/// Strip `#[miniextendr(...)]` attributes and roxygen doc tags from an impl block and
/// all of its items (functions, constants, types, macros).
///
/// Called before re-emitting the original impl block so that proc-macro attributes
/// do not appear in the compiler output. Returns the cleaned impl block.
fn strip_miniextendr_attrs_from_impl(mut item_impl: syn::ItemImpl) -> syn::ItemImpl {
    item_impl.attrs = crate::roxygen::strip_roxygen_from_attrs(&item_impl.attrs);
    item_impl
        .attrs
        .retain(|attr| !attr.path().is_ident("miniextendr"));
    for item in &mut item_impl.items {
        match item {
            syn::ImplItem::Fn(fn_item) => {
                fn_item.attrs = crate::roxygen::strip_roxygen_from_attrs(&fn_item.attrs);
                fn_item
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
            }
            syn::ImplItem::Const(const_item) => {
                const_item.attrs = crate::roxygen::strip_roxygen_from_attrs(&const_item.attrs);
                const_item
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
            }
            syn::ImplItem::Type(type_item) => {
                type_item.attrs = crate::roxygen::strip_roxygen_from_attrs(&type_item.attrs);
                type_item
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
            }
            syn::ImplItem::Macro(macro_item) => {
                macro_item.attrs = crate::roxygen::strip_roxygen_from_attrs(&macro_item.attrs);
                macro_item
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
            }
            _ => {}
        }
    }
    item_impl
}

/// Class system flavor for wrapper generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassSystem {
    /// Environment-style with `$`/`[[` dispatch
    Env,
    /// R6::R6Class
    R6,
    /// S7::new_class
    S7,
    /// S3 structure() with class attribute
    S3,
    /// S4 setClass
    S4,
    /// vctrs-compatible S3 class (vctr, rcrd, or list_of)
    Vctrs,
}

impl ClassSystem {
    /// Convert to an identifier for token transport (e.g., in macro_rules! expansion).
    pub fn to_ident(self) -> syn::Ident {
        let name = match self {
            ClassSystem::Env => "env",
            ClassSystem::R6 => "r6",
            ClassSystem::S7 => "s7",
            ClassSystem::S3 => "s3",
            ClassSystem::S4 => "s4",
            ClassSystem::Vctrs => "vctrs",
        };
        syn::Ident::new(name, proc_macro2::Span::call_site())
    }

    /// Parse from an identifier (inverse of `to_ident`).
    pub fn from_ident(ident: &syn::Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "env" => Some(ClassSystem::Env),
            "r6" => Some(ClassSystem::R6),
            "s7" => Some(ClassSystem::S7),
            "s3" => Some(ClassSystem::S3),
            "s4" => Some(ClassSystem::S4),
            "vctrs" => Some(ClassSystem::Vctrs),
            _ => None,
        }
    }
}

/// Case-insensitive parsing of class system names from strings.
///
/// Accepts: `"env"`, `"r6"`, `"s3"`, `"s4"`, `"s7"`, `"vctrs"` (any casing).
impl std::str::FromStr for ClassSystem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "env" => Ok(ClassSystem::Env),
            "r6" => Ok(ClassSystem::R6),
            "s7" => Ok(ClassSystem::S7),
            "s3" => Ok(ClassSystem::S3),
            "s4" => Ok(ClassSystem::S4),
            "vctrs" => Ok(ClassSystem::Vctrs),
            _ => Err(format!("unknown class system: {}", s)),
        }
    }
}

/// Kind of vctrs class being created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VctrsKind {
    /// Simple vctr backed by a base vector (new_vctr)
    #[default]
    Vctr,
    /// Record type with named fields (new_rcrd)
    Rcrd,
    /// Homogeneous list with ptype (new_list_of)
    ListOf,
}

/// Case-insensitive parsing of vctrs kind names from strings.
///
/// Accepts: `"vctr"`, `"rcrd"` (or `"record"`), `"list_of"` (or `"listof"`).
impl std::str::FromStr for VctrsKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vctr" => Ok(VctrsKind::Vctr),
            "rcrd" | "record" => Ok(VctrsKind::Rcrd),
            "list_of" | "listof" => Ok(VctrsKind::ListOf),
            _ => Err(format!(
                "unknown vctrs kind: {} (expected vctr, rcrd, or list_of)",
                s
            )),
        }
    }
}

/// Attributes for vctrs class generation.
#[derive(Debug, Clone, Default)]
pub struct VctrsAttrs {
    /// The vctrs kind (vctr, rcrd, list_of)
    pub kind: VctrsKind,
    /// Base type for vctr (e.g., "double", "integer", "character")
    pub base: Option<String>,
    /// Whether to inherit base type in class vector
    pub inherit_base_type: Option<bool>,
    /// Prototype type for list_of (R expression)
    pub ptype: Option<String>,
    /// Abbreviation for vec_ptype_abbr (for printing)
    pub abbr: Option<String>,
}

/// Receiver kind for methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverKind {
    /// No env - static/associated function
    None,
    /// `&self` - immutable borrow
    Ref,
    /// `&mut self` - mutable borrow
    RefMut,
    /// `self` - consuming (not supported in v1)
    Value,
    /// `self: &ExternalPtr<Self>` — immutable borrow of the wrapping ExternalPtr
    ExternalPtrRef,
    /// `self: &mut ExternalPtr<Self>` — mutable borrow of the wrapping ExternalPtr
    ExternalPtrRefMut,
    /// `self: ExternalPtr<Self>` — owned ExternalPtr (not consuming the inner T)
    ExternalPtrValue,
}

impl ReceiverKind {
    /// Returns true if this is an instance method (has self).
    pub fn is_instance(&self) -> bool {
        matches!(
            self,
            ReceiverKind::Ref
                | ReceiverKind::RefMut
                | ReceiverKind::ExternalPtrRef
                | ReceiverKind::ExternalPtrRefMut
                | ReceiverKind::ExternalPtrValue
        )
    }

    /// Returns true if this is a mutable instance receiver.
    pub fn is_mut(&self) -> bool {
        matches!(self, ReceiverKind::RefMut | ReceiverKind::ExternalPtrRefMut)
    }
}

/// Parsed method from an impl block.
///
/// # Default Parameters
///
/// Default parameters are specified using method-level syntax:
/// - `#[miniextendr(defaults(param = "value", ...))]` on the method
///
/// Note: Parameter-level `#[miniextendr(default = "...")]` syntax is only supported
/// for standalone functions, not impl methods (Rust language limitation).
///
/// Defaults cannot be specified for `self` parameters (compile error).
#[derive(Debug)]
pub struct ParsedMethod {
    /// The method's name (e.g., `new`, `get`, `set_value`).
    pub ident: syn::Ident,
    /// How this method receives `self`: `&self`, `&mut self`, by value, or not at all (static).
    pub env: ReceiverKind,
    /// Method signature with the `self` receiver stripped. Used for C wrapper generation
    /// where `self` is handled separately as a SEXP parameter.
    pub sig: syn::Signature,
    /// Rust visibility of the method. Non-`pub` methods become private in R6;
    /// only `pub` methods get `@export` in R wrappers.
    pub vis: syn::Visibility,
    /// Roxygen tag lines extracted from Rust doc comments
    pub doc_tags: Vec<String>,
    /// Per-method attributes for class system overrides. Also carries the
    /// `match_arg(...)` / `choices(...)` / `several_ok` parameter annotations
    /// via its `per_param_match_arg` / `per_param_choices` / `per_param_several_ok`
    /// fields — see [`MethodAttrs`] for the parsing surface.
    pub method_attrs: MethodAttrs,
    /// Parameter default values from `#[miniextendr(default = "...")]`
    pub param_defaults: std::collections::HashMap<String, String>,
}

/// R6-specific per-method markers, separated from [`MethodAttrs`] so the
/// `r6` parser branch and R6 class generator own a self-contained bag.
///
/// All R6 boolean flags live here.  Using any of these markers under a
/// non-R6 class system (`#[miniextendr(s3)]`, `s4`, `s7`, `env`) is a
/// compile-time error caught by [`ParsedMethod::validate_method_attrs`].
#[derive(Debug, Default)]
pub struct R6MethodAttrs {
    /// Mark as active binding getter (`#[miniextendr(r6(active))]`).
    pub active: bool,
    /// Span of the `r6(active)` marker — used for error reporting when the
    /// marker is misused in a non-R6 class generator.
    pub active_span: Option<proc_macro2::Span>,
    /// R6 active-binding *setter* (paired with an `active` getter by `prop`).
    pub setter: bool,
    /// R6 active-binding property name (defaults to the method name).
    pub prop: Option<String>,
    /// Mark as private method (`#[miniextendr(r6(private))]`).
    /// Also inferred from non-`pub` Rust visibility.
    pub private: bool,
    /// Span of the `r6(private)` marker — points the validator's diagnostic
    /// at the offending marker rather than the method ident.
    pub private_span: Option<proc_macro2::Span>,
    /// Mark as finalizer (`#[miniextendr(r6(finalize))]`).
    /// Also inferred when the method consumes `self` and does not return `Self`.
    pub finalize: bool,
    /// Span of the `r6(finalize)` marker — see `private_span`.
    pub finalize_span: Option<proc_macro2::Span>,
    /// Mark as R6 deep-clone handler (`#[miniextendr(r6(deep_clone))]`).
    /// This method is wired into `private$deep_clone` in the R6Class definition.
    pub deep_clone: bool,
    /// Span of the `r6(deep_clone)` marker — see `private_span`.
    pub deep_clone_span: Option<proc_macro2::Span>,
}

/// S7-specific per-method markers, separated from [`MethodAttrs`] so the S7
/// class generator has a self-contained bag of its own state (property
/// getters/setters, generic-dispatch controls, convert() wiring) and the other
/// class generators don't have to look past them.
///
/// # Mapping from `s7(...)` attribute keys
///
/// | Attribute | Field |
/// |-----------|-------|
/// | `s7(getter)` | `getter: true` |
/// | `s7(setter)` | `setter: true` |
/// | `s7(prop = "name")` | `prop: Some("name")` |
/// | `s7(default = "expr")` | `default: Some("expr")` |
/// | `s7(validate)` | `validate: true` |
/// | `s7(required)` | `required: true` |
/// | `s7(frozen)` | `frozen: true` |
/// | `s7(deprecated = "msg")` | `deprecated: Some("msg")` |
/// | `s7(no_dots)` | `no_dots: true` |
/// | `s7(dispatch = "x,y")` | `dispatch: Some("x,y")` |
/// | `s7(fallback)` | `fallback: true` |
/// | `s7(convert_from = "T")` | `convert_from: Some("T")` |
/// | `s7(convert_to = "T")` | `convert_to: Some("T")` |
#[derive(Debug, Default)]
pub struct S7MethodAttrs {
    pub getter: bool,
    pub setter: bool,
    pub prop: Option<String>,
    pub default: Option<String>,
    pub validate: bool,
    pub required: bool,
    pub frozen: bool,
    pub deprecated: Option<String>,
    pub no_dots: bool,
    pub dispatch: Option<String>,
    pub fallback: bool,
    pub convert_from: Option<String>,
    pub convert_to: Option<String>,
}

/// Per-method attributes for class system customization.
#[derive(Debug, Default)]
pub struct MethodAttrs {
    /// Skip this method
    pub ignore: bool,
    /// Mark as constructor
    pub constructor: bool,
    /// R6-specific method markers. All R6 boolean flags live here.
    /// Only consumed by the R6 class generator and R6-aware accessor methods
    /// (`ParsedMethod::is_active`, `is_private`, `is_finalizer`).
    pub r6: R6MethodAttrs,
    /// Generate as `as.<class>()` S3 method (e.g., "data.frame", "list", "character").
    ///
    /// When set, generates an S3 method for R's `as.<class>()` generic:
    /// ```r
    /// as.data.frame.MyType <- function(x, ...) {
    ///     .Call(C_MyType__as_data_frame, .call = match.call(), x)
    /// }
    /// ```
    ///
    /// Valid values: data.frame, list, character, numeric, double, integer,
    /// logical, matrix, vector, factor, Date, POSIXct, complex, raw,
    /// environment, function
    pub as_coercion: Option<String>,
    /// Span of `as = "..."` for error reporting.
    pub as_coercion_span: Option<proc_macro2::Span>,
    /// Override generic name for S3/S4/S7 methods.
    ///
    /// Use this to implement methods for existing generics (like `print`, `format`, `length`)
    /// without creating a new generic. When set, the generated code:
    /// - Uses the specified generic name instead of the method name
    /// - Skips creating a new generic (assumes it already exists)
    /// - Creates only the method implementation (e.g., `print.MyClass`)
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s3)]
    /// impl MyType {
    ///     #[miniextendr(generic = "print")]
    ///     fn show(&self) -> String {
    ///         format!("MyType: {}", self.value)
    ///     }
    /// }
    /// ```
    /// This generates `print.MyType` that calls the `show` method.
    pub generic: Option<String>,
    /// Override class suffix for S3 methods.
    ///
    /// Use this to implement double-dispatch methods (like vctrs coercion)
    /// where the class suffix differs from the type name or contains multiple classes.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s3(generic = "vec_ptype2", class = "my_vctr.my_vctr"))]
    /// fn ptype2_self(x: Robj, y: Robj, dots: ...) -> Robj {
    ///     // Return prototype
    /// }
    /// ```
    /// This generates `vec_ptype2.my_vctr.my_vctr` for vctrs double-dispatch.
    pub class: Option<String>,
    /// Worker thread execution (default: auto-detect based on types)
    pub worker: bool,
    /// Force main thread execution (unsafe)
    pub unsafe_main_thread: bool,
    /// Enable R interrupt checking
    pub check_interrupt: bool,
    /// Enable coercion for this method's parameters
    pub coerce: bool,
    /// Enable RNG state management (GetRNGstate/PutRNGstate)
    pub rng: bool,
    /// Return `Result<T, E>` to R without unwrapping.
    pub unwrap_in_r: bool,
    /// Transport Rust-origin errors as tagged values; R wrapper raises condition.
    pub error_in_r: bool,
    /// Parameter defaults from `#[miniextendr(defaults(param = "value", ...))]`
    pub defaults: std::collections::HashMap<String, String>,
    /// Span of `defaults(...)` for error reporting.
    pub defaults_span: Option<proc_macro2::Span>,
    /// Per-parameter `match_arg` / `several_ok` / `choices` state for this
    /// method, keyed by the Rust parameter name.
    ///
    /// Method-level (not parameter-level) because Rust's parser rejects
    /// attribute macros on fn parameters inside impl items. Standalone
    /// functions take the per-param syntax directly; impl methods spell the
    /// same data through `#[miniextendr(match_arg(p1, p2))]`,
    /// `#[miniextendr(match_arg_several_ok(p))]`, and
    /// `#[miniextendr(choices(p = "a, b"))]` on the method attribute.
    ///
    /// Uses the shared [`ParamAttrs`](crate::miniextendr_fn::ParamAttrs)
    /// struct — the `coerce` / `default` fields are unused on the impl path.
    pub per_param: std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
    /// Span of `match_arg(...)` / `choices(...)` for error reporting.
    pub match_arg_span: Option<proc_macro2::Span>,
    /// S7-specific method markers. Only consumed by the S7 class generator;
    /// all other generators ignore this field.
    pub s7: S7MethodAttrs,
    // region: Lifecycle support
    /// Lifecycle specification for deprecation/experimental status on methods.
    ///
    /// Use `#[miniextendr(lifecycle = "deprecated")]` or
    /// `#[miniextendr(lifecycle(stage = "deprecated", when = "0.4.0", with = "new_method()"))]`
    /// on methods in impl blocks.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(r6)]
    /// impl MyType {
    ///     #[miniextendr(lifecycle = "deprecated")]
    ///     pub fn old_method(&self) -> i32 { 0 }
    /// }
    /// ```
    pub lifecycle: Option<crate::lifecycle::LifecycleSpec>,
    /// vctrs protocol method override.
    ///
    /// Use `#[miniextendr(vctrs(format))]` to mark a method as implementing a vctrs
    /// protocol S3 generic. The method will be generated as `format.<class>` instead
    /// of the default Rust method name.
    ///
    /// Supported protocols: format, vec_proxy, vec_proxy_equal, vec_proxy_compare,
    /// vec_proxy_order, vec_restore, obj_print_data, obj_print_header, obj_print_footer.
    pub vctrs_protocol: Option<String>,
    /// Override R method name.
    ///
    /// Use `#[miniextendr(r_name = "add_one")]` to give the R method a different name
    /// than the Rust method. The C symbol is still derived from the Rust name.
    /// Cannot be combined with `generic = "..."` on the same method.
    pub r_name: Option<String>,
    /// R code to inject at the very top of the method body (before all built-in checks).
    pub r_entry: Option<String>,
    /// R code to inject after all built-in checks, immediately before `.Call()`.
    pub r_post_checks: Option<String>,
    /// Register `on.exit()` cleanup code in the R method wrapper.
    ///
    /// Short form: `#[miniextendr(r_on_exit = "close(con)")]`
    /// Long form: `#[miniextendr(r_on_exit(expr = "close(con)", add = false))]`
    pub r_on_exit: Option<crate::miniextendr_fn::ROnExit>,
}

/// Fully parsed `#[miniextendr]` impl block, ready for code generation.
///
/// Contains the type identity, chosen class system, all parsed methods, the original
/// impl block (with miniextendr attrs stripped for re-emission), and all class-system-specific
/// configuration options. Created by [`ParsedImpl::parse`] and consumed by the per-class-system
/// R wrapper generators and [`generate_method_c_wrapper`].
#[derive(Debug)]
pub struct ParsedImpl {
    /// The Rust type name being implemented (e.g., `Counter`).
    pub type_ident: syn::Ident,
    /// Which R class system to generate wrappers for.
    pub class_system: ClassSystem,
    /// Optional override for the R class name. When `None`, uses `type_ident` as the class name.
    pub class_name: Option<String>,
    /// Optional label for distinguishing multiple impl blocks of the same type.
    pub label: Option<String>,
    /// Roxygen tag lines extracted from `///` doc comments on the impl block.
    /// Used for class-level documentation (e.g., the R6 class docstring or S3 type description).
    pub doc_tags: Vec<String>,
    /// All parsed methods in this impl block, in source order.
    pub methods: Vec<ParsedMethod>,
    /// The original impl block with `#[miniextendr]` and roxygen attrs stripped.
    /// Re-emitted as-is so the Rust compiler sees the actual method implementations.
    pub original_impl: syn::ItemImpl,
    /// `#[cfg(...)]` attributes from the impl block, propagated to all generated items
    /// (C wrappers, R wrapper constants, call def arrays) for conditional compilation.
    pub cfg_attrs: Vec<syn::Attribute>,
    /// vctrs-specific attributes (only used when class_system is Vctrs)
    pub vctrs_attrs: VctrsAttrs,
    /// R6 parent class name for inheritance (e.g., `"ParentClass"`).
    /// Propagated from [`ImplAttrs::r6_inherit`].
    pub r6_inherit: Option<String>,
    /// R6 portable flag. When `Some(true)`, generates a portable R6 class.
    /// Propagated from [`ImplAttrs::r6_portable`].
    pub r6_portable: Option<bool>,
    /// R6 cloneable flag. Controls whether `$clone()` is available on instances.
    /// Propagated from [`ImplAttrs::r6_cloneable`].
    pub r6_cloneable: Option<bool>,
    /// R6 lock_objects flag. When `Some(true)`, prevents adding new fields after creation.
    /// Propagated from [`ImplAttrs::r6_lock_objects`].
    pub r6_lock_objects: Option<bool>,
    /// R6 lock_class flag. When `Some(true)`, prevents modifying the class definition.
    /// Propagated from [`ImplAttrs::r6_lock_class`].
    pub r6_lock_class: Option<bool>,
    /// S7 parent class name for inheritance (e.g., `"ParentClass"`).
    /// Propagated from [`ImplAttrs::s7_parent`].
    pub s7_parent: Option<String>,
    /// When true, marks this as an abstract S7 class that cannot be instantiated.
    /// Propagated from [`ImplAttrs::s7_abstract`].
    pub s7_abstract: bool,
    /// When true, auto-include sidecar `#[r_data]` field accessors in the class definition.
    /// For R6: active bindings are added via `$set("active", ...)` after class creation.
    /// For S7: properties are spliced from `.rdata_properties_{Type}` into `new_class()`.
    pub r_data_accessors: bool,
    /// Strict conversion mode: methods returning lossy types use checked conversions.
    pub strict: bool,
    /// Mark class as internal: adds `@keywords internal`, suppresses `@export`.
    pub internal: bool,
    /// Suppress `@export` without adding `@keywords internal`.
    pub noexport: bool,
    /// Deprecation warnings for `@param` tags found on the impl block.
    /// Appended to the final TokenStream output.
    pub param_warnings: proc_macro2::TokenStream,
}

/// Attributes parsed from `#[miniextendr(...)]` on an impl block.
///
/// These control which R class system to use, class naming, multi-impl labeling,
/// and class-system-specific options (R6 inheritance, S7 parent, vctrs kind, etc.).
///
/// Parsed by the [`syn::parse::Parse`] implementation which handles all supported
/// attribute formats like `#[miniextendr(r6, class = "Custom", label = "ops")]`.
#[derive(Debug)]
pub struct ImplAttrs {
    /// Which R class system to generate wrappers for.
    /// Defaults to `Env` unless overridden by feature flags (`default-r6`, `default-s7`).
    pub class_system: ClassSystem,
    /// Optional override for the R class name. When `None`, the Rust type name is used.
    pub class_name: Option<String>,
    /// Optional label for distinguishing multiple impl blocks of the same type.
    ///
    /// When a type has multiple `#[miniextendr]` impl blocks, each must have a
    /// distinct label. The label is used in:
    /// - Generated wrapper names (e.g., `C_Type_label__method`)
    /// - Module registration (e.g., `impl Type as "label"`)
    ///
    /// Single impl blocks don't require labels.
    pub label: Option<String>,
    /// vctrs-specific attributes (only used when class_system is Vctrs)
    pub vctrs_attrs: VctrsAttrs,
    // endregion
    // region: R6-specific configuration
    /// R6 parent class for inheritance.
    /// Use `#[miniextendr(r6(inherit = "ParentClass"))]` to specify the parent.
    pub r6_inherit: Option<String>,
    /// R6 portable flag. Default TRUE. Set to false for non-portable R6 classes.
    pub r6_portable: Option<bool>,
    /// R6 cloneable flag. Controls whether `$clone()` is available.
    pub r6_cloneable: Option<bool>,
    /// R6 lock_objects flag. Controls whether fields can be added after creation.
    pub r6_lock_objects: Option<bool>,
    /// R6 lock_class flag. Controls whether the class definition can be modified.
    pub r6_lock_class: Option<bool>,
    // endregion
    // region: S7-specific configuration
    /// S7 parent class for inheritance.
    /// Use `#[miniextendr(s7(parent = "ParentClass"))]` to specify the parent.
    pub s7_parent: Option<String>,
    /// S7 abstract class flag. Abstract classes cannot be instantiated.
    pub s7_abstract: bool,
    // endregion
    // region: Sidecar integration
    /// When true, auto-include `#[r_data]` field accessors in the class definition.
    /// For R6: active bindings via `$set("active", ...)` post-creation.
    /// For S7: properties spliced from `.rdata_properties_{Type}`.
    pub r_data_accessors: bool,
    // endregion
    // region: Strict conversion mode
    /// When true, methods returning lossy types (i64/u64/isize/usize + Vec variants)
    /// use `strict::checked_*()` instead of `IntoR::into_sexp()`, panicking on overflow.
    pub strict: bool,
    /// Mark class as internal: adds `@keywords internal`, suppresses `@export`.
    pub internal: bool,
    /// Suppress `@export` without adding `@keywords internal`.
    pub noexport: bool,
    /// When true on a trait impl (`impl Trait for Type`), the impl block is NOT
    /// emitted (a blanket impl already provides it), but C wrappers and R wrappers
    /// ARE generated from the method signatures in the body.
    pub blanket: bool,
    // endregion
}

impl syn::parse::Parse for ImplAttrs {
    /// Parses `#[miniextendr(...)]` impl-level options.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut class_system = if cfg!(feature = "default-r6") {
            ClassSystem::R6
        } else if cfg!(feature = "default-s7") {
            ClassSystem::S7
        } else {
            ClassSystem::Env
        };
        let mut class_system_span: Option<(&str, proc_macro2::Span)> = None;
        let mut class_name = None;
        let mut label = None;
        let mut vctrs_attrs = VctrsAttrs::default();
        let mut r6_inherit = None;
        let mut r6_portable = None;
        let mut r6_cloneable = None;
        let mut r6_lock_objects = None;
        let mut r6_lock_class = None;
        let mut s7_parent = None;
        let mut s7_abstract = false;
        let mut r_data_accessors = false;
        let mut strict: Option<bool> = None;
        let mut internal = false;
        let mut noexport = false;
        let mut blanket = false;

        // Parse attributes. The first identifier can be either:
        // - A class system (env, r6, s3, s4, s7, vctrs)
        // - A key in a key=value pair (class, label)
        //
        // Valid formats:
        // - #[miniextendr]
        // - #[miniextendr(r6)]
        // - #[miniextendr(label = "foo")]
        // - #[miniextendr(r6, label = "foo")]
        // - #[miniextendr(r6, class = "CustomName", label = "foo")]
        // - #[miniextendr(vctrs)]
        // - #[miniextendr(vctrs(kind = "rcrd", base = "double", abbr = "my_abbr"))]
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let ident_str = ident.to_string();

            // Check if this is a key=value pair
            if input.peek(syn::Token![=]) {
                let _: syn::Token![=] = input.parse()?;
                match ident_str.as_str() {
                    "class" => {
                        let value: syn::LitStr = input.parse()?;
                        class_name = Some(value.value());
                    }
                    "label" => {
                        let value: syn::LitStr = input.parse()?;
                        label = Some(value.value());
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!(
                                "unknown impl block option `{}`; expected one of: \
                                 env, r6, s3, s4, s7, vctrs (class system), \
                                 class = \"...\" (R class name), \
                                 label = \"...\" (multi-impl label), \
                                 strict (strict type conversion)",
                                ident_str,
                            ),
                        ));
                    }
                }
            } else if ident_str == "vctrs" {
                // vctrs class system with optional nested attributes
                if let Some((prev_name, _prev_span)) = class_system_span {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "multiple class systems specified (`{}` and `{}`); use only one of: env, r6, s3, s4, s7, vctrs",
                            prev_name, ident_str
                        ),
                    ));
                }
                class_system_span = Some(("vctrs", ident.span()));
                class_system = ClassSystem::Vctrs;

                // Check for nested vctrs options: vctrs(kind = "rcrd", base = "double", ...)
                if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);

                    while !content.is_empty() {
                        let key: syn::Ident = content.parse()?;
                        let _: syn::Token![=] = content.parse()?;
                        let key_str = key.to_string();

                        match key_str.as_str() {
                            "kind" => {
                                let value: syn::LitStr = content.parse()?;
                                vctrs_attrs.kind = value
                                    .value()
                                    .parse()
                                    .map_err(|e| syn::Error::new(value.span(), e))?;
                            }
                            "base" => {
                                let value: syn::LitStr = content.parse()?;
                                vctrs_attrs.base = Some(value.value());
                            }
                            "inherit_base_type" => {
                                let value: syn::LitBool = content.parse()?;
                                vctrs_attrs.inherit_base_type = Some(value.value());
                            }
                            "ptype" => {
                                let value: syn::LitStr = content.parse()?;
                                vctrs_attrs.ptype = Some(value.value());
                            }
                            "abbr" => {
                                let value: syn::LitStr = content.parse()?;
                                vctrs_attrs.abbr = Some(value.value());
                            }
                            _ => {
                                return Err(syn::Error::new(
                                    key.span(),
                                    format!(
                                        "unknown vctrs option: {} (expected kind, base, inherit_base_type, ptype, abbr)",
                                        key_str
                                    ),
                                ));
                            }
                        }

                        // Consume trailing comma if present
                        if content.peek(syn::Token![,]) {
                            let _: syn::Token![,] = content.parse()?;
                        }
                    }
                }
            } else if ident_str == "r6" {
                // R6 class system with optional nested attributes
                // r6 or r6(inherit = "Parent", portable = false, cloneable, lock_class)
                if let Some((prev_name, _prev_span)) = class_system_span {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "multiple class systems specified (`{}` and `{}`); use only one of: env, r6, s3, s4, s7, vctrs",
                            prev_name, ident_str
                        ),
                    ));
                }
                class_system_span = Some(("r6", ident.span()));
                class_system = ClassSystem::R6;

                if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);

                    while !content.is_empty() {
                        let key: syn::Ident = content.parse()?;
                        let key_str = key.to_string();

                        match key_str.as_str() {
                            "inherit" => {
                                let _: syn::Token![=] = content.parse()?;
                                let value: syn::LitStr = content.parse()?;
                                r6_inherit = Some(value.value());
                            }
                            "portable" => {
                                if content.peek(syn::Token![=]) {
                                    let _: syn::Token![=] = content.parse()?;
                                    let value: syn::LitBool = content.parse()?;
                                    r6_portable = Some(value.value());
                                } else {
                                    r6_portable = Some(true);
                                }
                            }
                            "cloneable" => {
                                if content.peek(syn::Token![=]) {
                                    let _: syn::Token![=] = content.parse()?;
                                    let value: syn::LitBool = content.parse()?;
                                    r6_cloneable = Some(value.value());
                                } else {
                                    r6_cloneable = Some(true);
                                }
                            }
                            "lock_objects" => {
                                if content.peek(syn::Token![=]) {
                                    let _: syn::Token![=] = content.parse()?;
                                    let value: syn::LitBool = content.parse()?;
                                    r6_lock_objects = Some(value.value());
                                } else {
                                    r6_lock_objects = Some(true);
                                }
                            }
                            "lock_class" => {
                                if content.peek(syn::Token![=]) {
                                    let _: syn::Token![=] = content.parse()?;
                                    let value: syn::LitBool = content.parse()?;
                                    r6_lock_class = Some(value.value());
                                } else {
                                    r6_lock_class = Some(true);
                                }
                            }
                            "r_data_accessors" => {
                                r_data_accessors = true;
                            }
                            _ => {
                                return Err(syn::Error::new(
                                    key.span(),
                                    format!(
                                        "unknown r6 option: {} (expected inherit, portable, cloneable, lock_objects, lock_class, r_data_accessors)",
                                        key_str
                                    ),
                                ));
                            }
                        }

                        // Consume trailing comma if present
                        if content.peek(syn::Token![,]) {
                            let _: syn::Token![,] = content.parse()?;
                        }
                    }
                }
            } else if ident_str == "s7" {
                // S7 class system with optional nested attributes
                // s7 or s7(parent = "Parent", abstract)
                if let Some((prev_name, _prev_span)) = class_system_span {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "multiple class systems specified (`{}` and `{}`); use only one of: env, r6, s3, s4, s7, vctrs",
                            prev_name, ident_str
                        ),
                    ));
                }
                class_system_span = Some(("s7", ident.span()));
                class_system = ClassSystem::S7;

                if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);

                    while !content.is_empty() {
                        // Use parse_any to accept `abstract` (a reserved keyword)
                        use syn::ext::IdentExt;
                        let key = syn::Ident::parse_any(&content)?;
                        let key_str = key.to_string();

                        match key_str.as_str() {
                            "parent" => {
                                let _: syn::Token![=] = content.parse()?;
                                let value: syn::LitStr = content.parse()?;
                                s7_parent = Some(value.value());
                            }
                            "abstract" => {
                                if content.peek(syn::Token![=]) {
                                    let _: syn::Token![=] = content.parse()?;
                                    let value: syn::LitBool = content.parse()?;
                                    s7_abstract = value.value();
                                } else {
                                    s7_abstract = true;
                                }
                            }
                            "r_data_accessors" => {
                                r_data_accessors = true;
                            }
                            _ => {
                                return Err(syn::Error::new(
                                    key.span(),
                                    format!(
                                        "unknown s7 option: {} (expected parent, abstract, r_data_accessors)",
                                        key_str
                                    ),
                                ));
                            }
                        }

                        // Consume trailing comma if present
                        if content.peek(syn::Token![,]) {
                            let _: syn::Token![,] = content.parse()?;
                        }
                    }
                }
            } else if ident_str == "blanket" {
                blanket = true;
            } else if ident_str == "strict" {
                strict = Some(true);
            } else if ident_str == "no_strict" {
                strict = Some(false);
            } else if ident_str == "internal" {
                internal = true;
            } else if ident_str == "noexport" {
                noexport = true;
            } else {
                // This is a class system identifier
                let parsed_system: ClassSystem = ident_str
                    .parse()
                    .map_err(|e| syn::Error::new(ident.span(), e))?;
                if let Some((prev_name, _prev_span)) = class_system_span {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "multiple class systems specified (`{}` and `{}`); use only one of: env, r6, s3, s4, s7, vctrs",
                            prev_name, ident_str
                        ),
                    ));
                }
                class_system_span = Some((
                    match parsed_system {
                        ClassSystem::Env => "env",
                        ClassSystem::R6 => "r6",
                        ClassSystem::S3 => "s3",
                        ClassSystem::S4 => "s4",
                        ClassSystem::S7 => "s7",
                        ClassSystem::Vctrs => "vctrs",
                    },
                    ident.span(),
                ));
                class_system = parsed_system;
            }

            // Consume trailing comma if present
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(ImplAttrs {
            class_system,
            class_name,
            label,
            vctrs_attrs,
            r6_inherit,
            r6_portable,
            r6_cloneable,
            r6_lock_objects,
            r6_lock_class,
            s7_parent,
            s7_abstract,
            r_data_accessors,
            strict: strict.unwrap_or(cfg!(feature = "default-strict")),
            internal,
            noexport,
            blanket,
        })
    }
}

impl ParsedMethod {
    /// Validate method attributes for the given class system.
    /// Returns an error if unsupported attributes are used.
    fn validate_method_attrs(
        attrs: &MethodAttrs,
        class_system: ClassSystem,
        span: proc_macro2::Span,
    ) -> syn::Result<()> {
        // R6-only boolean markers must not appear under any other class system.
        if class_system != ClassSystem::R6 {
            if attrs.r6.active {
                return Err(syn::Error::new(
                    attrs.r6.active_span.unwrap_or(span),
                    "`active` is only valid for R6 class systems",
                ));
            }
            if attrs.r6.private {
                return Err(syn::Error::new(
                    attrs.r6.private_span.unwrap_or(span),
                    "`private` is only valid for R6 class systems",
                ));
            }
            if attrs.r6.finalize {
                return Err(syn::Error::new(
                    attrs.r6.finalize_span.unwrap_or(span),
                    "`finalize` is only valid for R6 class systems",
                ));
            }
            if attrs.r6.deep_clone {
                return Err(syn::Error::new(
                    attrs.r6.deep_clone_span.unwrap_or(span),
                    "`deep_clone` is only valid for R6 class systems",
                ));
            }
        }

        // convert_from and convert_to are mutually exclusive on the same method
        // - convert_from expects a static method (no &self, takes source type)
        // - convert_to expects an instance method (&self, returns target type)
        if attrs.s7.convert_from.is_some() && attrs.s7.convert_to.is_some() {
            return Err(syn::Error::new(
                span,
                "cannot specify both `convert_from` and `convert_to` on the same method; \
                 convert_from is for static methods, convert_to is for instance methods",
            ));
        }

        // r_name and generic are mutually exclusive
        if attrs.r_name.is_some() && attrs.generic.is_some() {
            return Err(syn::Error::new(
                span,
                "`r_name` and `generic` cannot be used on the same method. \
                 Use `r_name` for a simple rename, or `generic`/`class` for S3/S4/S7 generic dispatch.",
            ));
        }

        // Worker attribute is now supported on methods
        // (validation happens during wrapper generation based on return type)

        Ok(())
    }

    /// Split a comma-separated choices list (as given to `choices(param = "a, b, c")`)
    /// into individual trimmed entries. Surrounding double-quotes are tolerated so
    /// users can spell the list either way: `"a, b"` or `"\"a\", \"b\""`.
    fn split_choice_list(raw: &str) -> Vec<String> {
        raw.split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Parse method attributes in #[miniextendr(class_system(...))] format.
    ///
    /// Supported formats:
    /// - `#[miniextendr(r6(ignore, constructor, finalize, private, generic = "...")]`
    /// - `#[miniextendr(s3(ignore, constructor, generic = "..."))]`
    /// - `#[miniextendr(s7(ignore, constructor, generic = "..."))]`
    /// - etc.
    fn parse_method_attrs(attrs: &[syn::Attribute]) -> syn::Result<MethodAttrs> {
        use syn::spanned::Spanned;
        let mut method_attrs = MethodAttrs::default();
        // Use Option<bool> for fields that support feature defaults.
        let mut worker: Option<bool> = None;
        let mut unsafe_main_thread: Option<bool> = None;
        let mut coerce: Option<bool> = None;
        let mut error_in_r: Option<bool> = None;

        for attr in attrs {
            // Parse new-style #[miniextendr(class_system(...))] attributes
            if !attr.path().is_ident("miniextendr") {
                continue;
            }

            // Parse the nested content: miniextendr(class_system(options...)) or miniextendr(defaults(...))
            attr.parse_nested_meta(|meta| {
                // Note: "vctrs" is handled separately below for protocol method overrides
                let is_class_meta = meta.path.is_ident("env")
                    || meta.path.is_ident("r6")
                    || meta.path.is_ident("s7")
                    || meta.path.is_ident("s3")
                    || meta.path.is_ident("s4");

                if is_class_meta {
                    // Parse the inner options: r6(ignore, constructor, ...)
                    meta.parse_nested_meta(|inner| {
                        if inner.path.is_ident("ignore") {
                            method_attrs.ignore = true;
                        } else if inner.path.is_ident("constructor") {
                            method_attrs.constructor = true;
                        } else if inner.path.is_ident("finalize") {
                            method_attrs.r6.finalize = true;
                            method_attrs.r6.finalize_span = Some(inner.path.span());
                        } else if inner.path.is_ident("private") {
                            method_attrs.r6.private = true;
                            method_attrs.r6.private_span = Some(inner.path.span());
                        } else if inner.path.is_ident("active") {
                            method_attrs.r6.active = true;
                            method_attrs.r6.active_span = Some(inner.path.span());
                        } else if inner.path.is_ident("setter") {
                            // Active binding setter: works for both R6 and S7
                            method_attrs.r6.setter = true;
                            method_attrs.s7.setter = true;
                        } else if inner.path.is_ident("worker") {
                            worker = Some(true);
                        } else if inner.path.is_ident("no_worker") {
                            worker = Some(false);
                        } else if inner.path.is_ident("main_thread") {
                            unsafe_main_thread = Some(true);
                        } else if inner.path.is_ident("no_main_thread") {
                            unsafe_main_thread = Some(false);
                        } else if inner.path.is_ident("check_interrupt") {
                            method_attrs.check_interrupt = true;
                        } else if inner.path.is_ident("coerce") {
                            coerce = Some(true);
                        } else if inner.path.is_ident("no_coerce") {
                            coerce = Some(false);
                        } else if inner.path.is_ident("rng") {
                            method_attrs.rng = true;
                        } else if inner.path.is_ident("unwrap_in_r") {
                            if error_in_r == Some(true) {
                                return Err(syn::Error::new_spanned(inner.path, "`error_in_r` and `unwrap_in_r` are mutually exclusive"));
                            }
                            method_attrs.unwrap_in_r = true;
                        } else if inner.path.is_ident("error_in_r") {
                            if method_attrs.unwrap_in_r {
                                return Err(syn::Error::new_spanned(inner.path, "`error_in_r` and `unwrap_in_r` are mutually exclusive"));
                            }
                            error_in_r = Some(true);
                        } else if inner.path.is_ident("no_error_in_r") {
                            error_in_r = Some(false);
                        } else if inner.path.is_ident("generic") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.generic = Some(value.value());
                        } else if inner.path.is_ident("class") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.class = Some(value.value());
                        } else if inner.path.is_ident("getter") {
                            method_attrs.s7.getter = true;
                        } else if inner.path.is_ident("validate") {
                            method_attrs.s7.validate = true;
                        } else if inner.path.is_ident("prop") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            let prop_value = value.value();
                            // Set both S7 and R6 prop - the class system will use the appropriate one
                            method_attrs.s7.prop = Some(prop_value.clone());
                            method_attrs.r6.prop = Some(prop_value);
                        } else if inner.path.is_ident("default") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7.default = Some(value.value());
                        } else if inner.path.is_ident("required") {
                            method_attrs.s7.required = true;
                        } else if inner.path.is_ident("frozen") {
                            method_attrs.s7.frozen = true;
                        } else if inner.path.is_ident("deprecated") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7.deprecated = Some(value.value());
                        } else if inner.path.is_ident("no_dots") {
                            method_attrs.s7.no_dots = true;
                        } else if inner.path.is_ident("dispatch") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7.dispatch = Some(value.value());
                        } else if inner.path.is_ident("fallback") {
                            method_attrs.s7.fallback = true;
                        } else if inner.path.is_ident("convert_from") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7.convert_from = Some(value.value());
                        } else if inner.path.is_ident("convert_to") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7.convert_to = Some(value.value());
                        } else if inner.path.is_ident("deep_clone") {
                            method_attrs.r6.deep_clone = true;
                            method_attrs.r6.deep_clone_span = Some(inner.path.span());
                        } else if inner.path.is_ident("r_name") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            let val = value.value();
                            if val.is_empty() {
                                return Err(syn::Error::new_spanned(value, "r_name must not be empty"));
                            }
                            method_attrs.r_name = Some(val);
                        } else if inner.path.is_ident("r_entry") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.r_entry = Some(value.value());
                        } else if inner.path.is_ident("r_post_checks") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.r_post_checks = Some(value.value());
                        } else if inner.path.is_ident("r_on_exit") {
                            if inner.input.peek(syn::Token![=]) {
                                // Short form: r_on_exit = "expr"
                                let _: syn::Token![=] = inner.input.parse()?;
                                let value: syn::LitStr = inner.input.parse()?;
                                method_attrs.r_on_exit = Some(crate::miniextendr_fn::ROnExit {
                                    expr: value.value(),
                                    add: true,
                                    after: true,
                                });
                            } else {
                                // Long form: r_on_exit(expr = "...", add = false, after = false)
                                let mut expr = None;
                                let mut add = true;
                                let mut after = true;
                                inner.parse_nested_meta(|meta| {
                                    if meta.path.is_ident("expr") {
                                        let _: syn::Token![=] = meta.input.parse()?;
                                        let value: syn::LitStr = meta.input.parse()?;
                                        expr = Some(value.value());
                                    } else if meta.path.is_ident("add") {
                                        let _: syn::Token![=] = meta.input.parse()?;
                                        let value: syn::LitBool = meta.input.parse()?;
                                        add = value.value;
                                    } else if meta.path.is_ident("after") {
                                        let _: syn::Token![=] = meta.input.parse()?;
                                        let value: syn::LitBool = meta.input.parse()?;
                                        after = value.value;
                                    } else {
                                        return Err(meta.error(
                                            "unknown r_on_exit option; expected `expr`, `add`, or `after`",
                                        ));
                                    }
                                    Ok(())
                                })?;
                                let expr = expr.ok_or_else(|| {
                                    inner.error("r_on_exit(...) requires `expr = \"...\"` specifying the R expression")
                                })?;
                                method_attrs.r_on_exit = Some(crate::miniextendr_fn::ROnExit { expr, add, after });
                            }
                        } else {
                            return Err(inner.error(
                                "unknown method option; expected one of: ignore, constructor, finalize, private, active, worker, no_worker, main_thread, no_main_thread, check_interrupt, coerce, no_coerce, rng, unwrap_in_r, error_in_r, no_error_in_r, generic, class, getter, setter, validate, prop, default, required, frozen, deprecated, no_dots, dispatch, fallback, convert_from, convert_to, deep_clone, r_on_exit"
                            ));
                        }
                        Ok(())
                    })?;
                } else if meta.path.is_ident("defaults") {
                    // Capture span for error reporting
                    method_attrs.defaults_span = Some(meta.path.span());
                    // Parse defaults(param = "value", param2 = "value2", ...)
                    meta.parse_nested_meta(|inner| {
                        // Get parameter name
                        let param_name = inner
                            .path
                            .get_ident()
                            .ok_or_else(|| inner.error("expected parameter name"))?
                            .to_string();
                        // Parse = "value"
                        let _: syn::Token![=] = inner.input.parse()?;
                        let value: syn::LitStr = inner.input.parse()?;
                        method_attrs.defaults.insert(param_name, value.value());
                        Ok(())
                    })?;
                } else if meta.path.is_ident("match_arg") {
                    // `match_arg(param1, param2, ...)` — scalar match_arg params.
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    meta.parse_nested_meta(|inner| {
                        let name = inner
                            .path
                            .get_ident()
                            .ok_or_else(|| inner.error("expected parameter name"))?
                            .to_string();
                        method_attrs
                            .per_param
                            .entry(name)
                            .or_default()
                            .match_arg = true;
                        Ok(())
                    })?;
                } else if meta.path.is_ident("match_arg_several_ok") {
                    // `match_arg_several_ok(param1, param2, ...)` — match_arg + several_ok,
                    // for Vec/slice/array/Box<[_]>-typed parameters.
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    meta.parse_nested_meta(|inner| {
                        let name = inner
                            .path
                            .get_ident()
                            .ok_or_else(|| inner.error("expected parameter name"))?
                            .to_string();
                        let entry = method_attrs.per_param.entry(name).or_default();
                        entry.match_arg = true;
                        entry.several_ok = true;
                        Ok(())
                    })?;
                } else if meta.path.is_ident("choices") {
                    // `choices(param = "a, b, c", param2 = "x, y")` — explicit string choice lists.
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    meta.parse_nested_meta(|inner| {
                        let name = inner
                            .path
                            .get_ident()
                            .ok_or_else(|| inner.error("expected parameter name"))?
                            .to_string();
                        let _: syn::Token![=] = inner.input.parse()?;
                        let value: syn::LitStr = inner.input.parse()?;
                        let choices = Self::split_choice_list(&value.value());
                        method_attrs.per_param.entry(name).or_default().choices = Some(choices);
                        Ok(())
                    })?;
                } else if meta.path.is_ident("choices_several_ok") {
                    // `choices_several_ok(param = "a, b, c")` — choices + several_ok.
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    meta.parse_nested_meta(|inner| {
                        let name = inner
                            .path
                            .get_ident()
                            .ok_or_else(|| inner.error("expected parameter name"))?
                            .to_string();
                        let _: syn::Token![=] = inner.input.parse()?;
                        let value: syn::LitStr = inner.input.parse()?;
                        let choices = Self::split_choice_list(&value.value());
                        let entry = method_attrs.per_param.entry(name).or_default();
                        entry.choices = Some(choices);
                        entry.several_ok = true;
                        Ok(())
                    })?;
                } else if meta.path.is_ident("unsafe") {
                    // Parse unsafe(main_thread) - same syntax as standalone functions
                    meta.parse_nested_meta(|inner| {
                        if inner.path.is_ident("main_thread") {
                            unsafe_main_thread = Some(true);
                        } else {
                            return Err(inner.error(
                                "unknown `unsafe(...)` option; only `main_thread` is supported",
                            ));
                        }
                        Ok(())
                    })?;
                } else if meta.path.is_ident("check_interrupt") {
                    method_attrs.check_interrupt = true;
                } else if meta.path.is_ident("coerce") {
                    coerce = Some(true);
                } else if meta.path.is_ident("no_coerce") {
                    coerce = Some(false);
                } else if meta.path.is_ident("rng") {
                    method_attrs.rng = true;
                } else if meta.path.is_ident("unwrap_in_r") {
                    if error_in_r == Some(true) {
                        return Err(syn::Error::new_spanned(meta.path, "`error_in_r` and `unwrap_in_r` are mutually exclusive"));
                    }
                    method_attrs.unwrap_in_r = true;
                } else if meta.path.is_ident("error_in_r") {
                    if method_attrs.unwrap_in_r {
                        return Err(syn::Error::new_spanned(meta.path, "`error_in_r` and `unwrap_in_r` are mutually exclusive"));
                    }
                    error_in_r = Some(true);
                } else if meta.path.is_ident("no_error_in_r") {
                    error_in_r = Some(false);
                } else if meta.path.is_ident("as") {
                    // Parse as = "data.frame", as = "list", etc.
                    method_attrs.as_coercion_span = Some(meta.path.span());
                    let _: syn::Token![=] = meta.input.parse()?;
                    let value: syn::LitStr = meta.input.parse()?;
                    let coercion_type = value.value();

                    // Validate the coercion type
                    const SUPPORTED_AS_TYPES: &[&str] = &[
                        "data.frame",
                        "list",
                        "character",
                        "numeric",
                        "double",
                        "integer",
                        "logical",
                        "matrix",
                        "vector",
                        "factor",
                        "Date",
                        "POSIXct",
                        "complex",
                        "raw",
                        "environment",
                        "function",
                        "tibble",
                        "data.table",
                        "array",
                        "ts",
                    ];

                    if !SUPPORTED_AS_TYPES.contains(&coercion_type.as_str()) {
                        return Err(syn::Error::new(
                            value.span(),
                            format!(
                                "unsupported `as` type: \"{}\". Supported types: {}",
                                coercion_type,
                                SUPPORTED_AS_TYPES.join(", ")
                            ),
                        ));
                    }

                    method_attrs.as_coercion = Some(coercion_type);
                } else if meta.path.is_ident("lifecycle") {
                    // lifecycle = "stage" or lifecycle(stage = "deprecated", when = "0.4.0", ...)
                    if meta.input.peek(syn::Token![=]) {
                        // lifecycle = "stage"
                        let _: syn::Token![=] = meta.input.parse()?;
                        let value: syn::LitStr = meta.input.parse()?;
                        let stage = crate::lifecycle::LifecycleStage::from_str(&value.value())
                            .ok_or_else(|| {
                                syn::Error::new(
                                    value.span(),
                                    "invalid lifecycle stage; expected one of: experimental, stable, superseded, soft-deprecated, deprecated, defunct",
                                )
                            })?;
                        method_attrs.lifecycle = Some(crate::lifecycle::LifecycleSpec::new(stage));
                    } else {
                        // lifecycle(stage = "deprecated", when = "0.4.0", ...)
                        let mut spec = crate::lifecycle::LifecycleSpec::default();
                        meta.parse_nested_meta(|inner| {
                            let key = inner.path.get_ident()
                                .ok_or_else(|| inner.error("expected identifier"))?
                                .to_string();
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            match key.as_str() {
                                "stage" => {
                                    spec.stage = crate::lifecycle::LifecycleStage::from_str(&value.value())
                                        .ok_or_else(|| syn::Error::new(value.span(), "invalid lifecycle stage"))?;
                                }
                                "when" => spec.when = Some(value.value()),
                                "what" => spec.what = Some(value.value()),
                                "with" => spec.with = Some(value.value()),
                                "details" => spec.details = Some(value.value()),
                                "id" => spec.id = Some(value.value()),
                                _ => return Err(inner.error(
                                    "unknown lifecycle option; expected: stage, when, what, with, details, id"
                                )),
                            }
                            Ok(())
                        })?;
                        method_attrs.lifecycle = Some(spec);
                    }
                } else if meta.path.is_ident("vctrs") {
                    // vctrs protocol method: vctrs(format), vctrs(vec_proxy), etc.
                    meta.parse_nested_meta(|inner| {
                        let raw_name = inner.path.get_ident()
                            .ok_or_else(|| inner.error("expected protocol name"))?
                            .to_string();

                        // Normalize short aliases to full protocol names
                        const PROTOCOL_ALIASES: &[(&str, &str)] = &[
                            ("print_data", "obj_print_data"),
                            ("print_header", "obj_print_header"),
                            ("print_footer", "obj_print_footer"),
                            ("proxy", "vec_proxy"),
                            ("proxy_equal", "vec_proxy_equal"),
                            ("proxy_compare", "vec_proxy_compare"),
                            ("proxy_order", "vec_proxy_order"),
                            ("restore", "vec_restore"),
                        ];
                        let protocol = PROTOCOL_ALIASES
                            .iter()
                            .find(|(alias, _)| *alias == raw_name)
                            .map(|(_, full)| full.to_string())
                            .unwrap_or_else(|| raw_name.to_string());

                        const VALID_PROTOCOLS: &[&str] = &[
                            "format", "vec_proxy", "vec_proxy_equal", "vec_proxy_compare",
                            "vec_proxy_order", "vec_restore", "obj_print_data",
                            "obj_print_header", "obj_print_footer",
                        ];
                        if !VALID_PROTOCOLS.contains(&protocol.as_str()) {
                            return Err(inner.error(format!(
                                "unknown vctrs protocol: {}; expected one of: {}",
                                raw_name,
                                VALID_PROTOCOLS.join(", ")
                            )));
                        }
                        method_attrs.vctrs_protocol = Some(protocol);
                        Ok(())
                    })?;
                } else if meta.path.is_ident("r_name") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let value: syn::LitStr = meta.input.parse()?;
                    let val = value.value();
                    if val.is_empty() {
                        return Err(syn::Error::new_spanned(value, "r_name must not be empty"));
                    }
                    method_attrs.r_name = Some(val);
                } else if meta.path.is_ident("r_entry") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let value: syn::LitStr = meta.input.parse()?;
                    method_attrs.r_entry = Some(value.value());
                } else if meta.path.is_ident("r_post_checks") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let value: syn::LitStr = meta.input.parse()?;
                    method_attrs.r_post_checks = Some(value.value());
                } else if meta.path.is_ident("r_on_exit") {
                    if meta.input.peek(syn::Token![=]) {
                        // Short form: r_on_exit = "expr"
                        let _: syn::Token![=] = meta.input.parse()?;
                        let value: syn::LitStr = meta.input.parse()?;
                        method_attrs.r_on_exit = Some(crate::miniextendr_fn::ROnExit {
                            expr: value.value(),
                            add: true,
                            after: true,
                        });
                    } else {
                        // Long form: r_on_exit(expr = "...", add = false, after = false)
                        let mut expr = None;
                        let mut add = true;
                        let mut after = true;
                        meta.parse_nested_meta(|inner| {
                            if inner.path.is_ident("expr") {
                                let _: syn::Token![=] = inner.input.parse()?;
                                let value: syn::LitStr = inner.input.parse()?;
                                expr = Some(value.value());
                            } else if inner.path.is_ident("add") {
                                let _: syn::Token![=] = inner.input.parse()?;
                                let value: syn::LitBool = inner.input.parse()?;
                                add = value.value;
                            } else if inner.path.is_ident("after") {
                                let _: syn::Token![=] = inner.input.parse()?;
                                let value: syn::LitBool = inner.input.parse()?;
                                after = value.value;
                            } else {
                                return Err(inner.error(
                                    "unknown r_on_exit option; expected `expr`, `add`, or `after`",
                                ));
                            }
                            Ok(())
                        })?;
                        let expr = expr.ok_or_else(|| {
                            meta.error("r_on_exit(...) requires `expr = \"...\"` specifying the R expression")
                        })?;
                        method_attrs.r_on_exit = Some(crate::miniextendr_fn::ROnExit { expr, add, after });
                    }
                } else {
                    return Err(meta.error(
                        "unknown attribute; expected one of: env, r6, s3, s4, s7, vctrs, defaults, unsafe, check_interrupt, coerce, no_coerce, rng, unwrap_in_r, error_in_r, no_error_in_r, as, lifecycle, r_name, r_entry, r_post_checks, r_on_exit"
                    ));
                }
                Ok(())
            })?;
        }

        // Resolve feature defaults for fields not explicitly set
        method_attrs.worker = worker.unwrap_or(cfg!(feature = "default-worker"));
        method_attrs.unsafe_main_thread = unsafe_main_thread.unwrap_or(true);
        method_attrs.coerce = coerce.unwrap_or(cfg!(feature = "default-coerce"));
        let resolved_error_in_r = error_in_r.unwrap_or(true);

        // Validate: rng requires error_in_r
        if method_attrs.rng && !resolved_error_in_r {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`rng` requires `error_in_r` (PutRNGstate must run after .Call returns; \
                 non-error_in_r diverges via longjmp, skipping PutRNGstate)",
            ));
        }

        if resolved_error_in_r && method_attrs.unwrap_in_r {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`error_in_r` (default) and `unwrap_in_r` are mutually exclusive; use `no_error_in_r` to opt out",
            ));
        }
        method_attrs.error_in_r = resolved_error_in_r;

        Ok(method_attrs)
    }

    /// Detect the [`ReceiverKind`] from a method's function signature.
    ///
    /// Inspects the first parameter to determine whether this is a static function
    /// (`None`), immutable borrow (`Ref`), mutable borrow (`RefMut`), consuming
    /// method (`Value`), or ExternalPtr receiver (`ExternalPtrRef`, `ExternalPtrRefMut`,
    /// `ExternalPtrValue`). Handles both standard receivers (`&self`, `&mut self`) and
    /// typed receivers (`self: &Self`, `self: ExternalPtr<Self>`, etc.).
    fn detect_env(sig: &syn::Signature) -> ReceiverKind {
        match sig.inputs.first() {
            Some(syn::FnArg::Receiver(r)) => {
                // Check for standard &self / &mut self
                if r.reference.is_some() {
                    if r.mutability.is_some() {
                        ReceiverKind::RefMut
                    } else {
                        ReceiverKind::Ref
                    }
                } else if r.colon_token.is_some() {
                    // Check for typed receiver (self: &Self, self: &mut Self,
                    // self: &ExternalPtr<Self>, self: &mut ExternalPtr<Self>)
                    if let syn::Type::Reference(type_ref) = r.ty.as_ref() {
                        if is_external_ptr_type(&type_ref.elem) {
                            if type_ref.mutability.is_some() {
                                ReceiverKind::ExternalPtrRefMut
                            } else {
                                ReceiverKind::ExternalPtrRef
                            }
                        } else if type_ref.mutability.is_some() {
                            ReceiverKind::RefMut
                        } else {
                            ReceiverKind::Ref
                        }
                    } else if is_external_ptr_type(r.ty.as_ref()) {
                        // self: ExternalPtr<Self> — owned ExternalPtr
                        ReceiverKind::ExternalPtrValue
                    } else {
                        // self: Box<Self>, self: Rc<Self>, etc. - treat as by value
                        ReceiverKind::Value
                    }
                } else {
                    ReceiverKind::Value
                }
            }
            _ => ReceiverKind::None,
        }
    }

    /// Create a copy of the method signature with the `self` receiver removed.
    ///
    /// The C wrapper receives `self` as a separate SEXP argument and extracts it
    /// from an `ErasedExternalPtr`, so the receiver must not appear in the
    /// parameter list used for SEXP-to-Rust conversion codegen.
    fn sig_without_env(sig: &syn::Signature) -> syn::Signature {
        let mut sig = sig.clone();
        if let Some(syn::FnArg::Receiver(_)) = sig.inputs.first() {
            sig.inputs = sig.inputs.into_iter().skip(1).collect();
        }
        sig
    }

    /// Parse a method from an impl item.
    ///
    /// Regular doc comments are auto-converted to `@description` for all class systems.
    pub fn from_impl_item(item: syn::ImplItemFn, _class_system: ClassSystem) -> syn::Result<Self> {
        use syn::spanned::Spanned;
        let env = Self::detect_env(&item.sig);
        let mut method_attrs = Self::parse_method_attrs(&item.attrs)?;

        // match_arg / choices on impl methods: unlike standalone functions, Rust
        // doesn't accept `#[miniextendr(...)]` on method parameters inside an impl
        // (attribute macros aren't allowed there — "expected non-macro attribute").
        // The surface is instead method-level: `#[miniextendr(match_arg(p), choices(q = "a, b"))]`.
        // `parse_method_attrs` already filled the sets; validate that every named
        // param exists on the signature so typos fail at compile time.
        let sig_param_names: std::collections::HashSet<String> = item
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pt) => match pt.pat.as_ref() {
                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.to_string()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        for annotated in method_attrs.per_param.iter().filter_map(|(name, a)| {
            if a.match_arg || a.choices.is_some() {
                Some(name)
            } else {
                None
            }
        }) {
            if !sig_param_names.contains(annotated) {
                return Err(syn::Error::new(
                    method_attrs
                        .match_arg_span
                        .unwrap_or_else(|| item.sig.ident.span()),
                    format!("match_arg/choices references non-existent parameter `{annotated}`"),
                ));
            }
        }
        // Validate: no defaults on self parameter (any kind: &self, &mut self, self)
        if env != ReceiverKind::None && method_attrs.defaults.contains_key("self") {
            return Err(syn::Error::new(
                method_attrs
                    .defaults_span
                    .unwrap_or_else(|| item.sig.ident.span()),
                "cannot specify default for self parameter in defaults(...)",
            ));
        }

        // Validate: all defaults reference existing parameters
        let param_names: std::collections::HashSet<String> = item
            .sig
            .inputs
            .iter()
            .filter_map(|input| {
                if let syn::FnArg::Typed(pat_type) = input
                    && let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                {
                    Some(pat_ident.ident.to_string())
                } else {
                    None
                }
            })
            .collect();

        let mut invalid_params: Vec<String> = method_attrs
            .defaults
            .keys()
            .filter(|key| *key != "self" && !param_names.contains(*key))
            .cloned()
            .collect();
        invalid_params.sort();

        if !invalid_params.is_empty() {
            return Err(syn::Error::new(
                method_attrs
                    .defaults_span
                    .unwrap_or_else(|| item.sig.ident.span()),
                format!(
                    "defaults(...) references non-existent parameter(s): {}",
                    invalid_params.join(", ")
                ),
            ));
        }

        // Validate type-based constraints on each parameter
        for input in &item.sig.inputs {
            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };
            let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
                continue;
            };
            let param_name = pat_ident.ident.to_string();

            // Validate Missing nesting and Missing<Dots>
            crate::miniextendr_fn::validate_param_type(pat_type.ty.as_ref(), pat_type.ty.span())?;

            // Validate: no defaults on Dots-type parameters
            if crate::miniextendr_fn::is_dots_type(pat_type.ty.as_ref())
                && method_attrs.defaults.contains_key(&param_name)
            {
                return Err(syn::Error::new(
                    method_attrs
                        .defaults_span
                        .unwrap_or_else(|| pat_ident.ident.span()),
                    format!(
                        "variadic (...) parameter `{}` cannot have a default value",
                        param_name
                    ),
                ));
            }
        }

        // Extract lifecycle from #[deprecated] attribute if not already set via #[miniextendr(lifecycle = ...)]
        if method_attrs.lifecycle.is_none() {
            method_attrs.lifecycle = item
                .attrs
                .iter()
                .find_map(crate::lifecycle::parse_rust_deprecated);
        }

        // Auto-convert regular doc comments to @description for all class systems
        let mut doc_tags = crate::roxygen::roxygen_tags_from_attrs_for_r6_method(&item.attrs);

        // Inject lifecycle badge into method roxygen tags if present
        if let Some(ref spec) = method_attrs.lifecycle {
            crate::lifecycle::inject_lifecycle_badge(&mut doc_tags, spec);
        }

        // Get parameter defaults from method-level #[miniextendr(defaults(...))] attribute
        let param_defaults = method_attrs.defaults.clone();

        // Validate: Missing<T> parameters must not have defaults
        for arg in item.sig.inputs.iter() {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let name = pat_ident.ident.to_string();
                if crate::r_wrapper_builder::is_missing_type(pt.ty.as_ref())
                    && param_defaults.contains_key(&name)
                {
                    let span = method_attrs.defaults_span.unwrap_or(item.sig.ident.span());
                    return Err(syn::Error::new(
                        span,
                        format!(
                            "`Missing<T>` parameter `{}` cannot have a default value. \
                             `Missing<T>` detects omitted arguments via `missing()` in R, \
                             which is incompatible with default values in the R function signature. \
                             Use `Option<T>` with a default instead.",
                            name
                        ),
                    ));
                }
            }
        }

        // Validate: `self` by value (consuming) methods are not fully supported
        // They're either: constructor (returns Self), finalizer (marked or inferred), or error
        if env == ReceiverKind::Value {
            let returns_self = matches!(&item.sig.output, syn::ReturnType::Type(_, ty)
                if matches!(ty.as_ref(), syn::Type::Path(p)
                    if p.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false)));

            // Allow if: constructor (returns Self) or explicitly marked as finalize
            let is_allowed = returns_self || method_attrs.constructor || method_attrs.r6.finalize;

            if !is_allowed {
                return Err(syn::Error::new(
                    item.sig.fn_token.span,
                    format!(
                        "method `{}` takes `self` by value (consuming), which is not fully supported.\n\
                         \n\
                         Methods that consume `self` cannot be called from R because R uses reference \
                         semantics via ExternalPtr - the R object would remain alive after the Rust \
                         value is consumed.\n\
                         \n\
                         Options:\n\
                         1. Use `&self` or `&mut self` instead of `self`\n\
                         2. If this is a finalizer (cleanup method), add `#[miniextendr(finalize)]`\n\
                         3. If this returns a new Self (builder pattern), add `#[miniextendr(constructor)]`",
                        item.sig.ident
                    ),
                ));
            }
        }

        Ok(ParsedMethod {
            ident: item.sig.ident.clone(),
            env,
            sig: Self::sig_without_env(&item.sig),
            vis: item.vis,
            doc_tags,
            method_attrs,
            param_defaults,
        })
    }

    /// Returns true if this method should be included in the class.
    pub fn should_include(&self) -> bool {
        // Skip ignored methods
        !self.method_attrs.ignore
    }

    /// Returns true if this method should be private in R6.
    /// Inferred from Rust visibility: anything not `pub` is private.
    pub fn is_private(&self) -> bool {
        // Explicit attribute takes precedence
        if self.method_attrs.r6.private {
            return true;
        }
        // Infer from visibility: anything not `pub` is private
        !matches!(self.vis, syn::Visibility::Public(_))
    }

    /// Returns true if this is likely a constructor.
    /// Inferred from: no env + named "new" + returns Self.
    pub fn is_constructor(&self) -> bool {
        self.method_attrs.constructor
            || (self.env == ReceiverKind::None && self.ident == "new" && self.returns_self())
    }

    /// Returns true if this is likely a finalizer.
    /// Inferred from: consumes self (by value) + doesn't return Self.
    pub fn is_finalizer(&self) -> bool {
        self.method_attrs.r6.finalize || (self.env == ReceiverKind::Value && !self.returns_self())
    }

    /// Returns true if this method should be an R6 active binding.
    /// Active bindings provide property-like access (obj$name instead of obj$name()).
    pub fn is_active(&self) -> bool {
        self.method_attrs.r6.active
    }

    /// R-facing method name.
    ///
    /// Returns `r_name` if set, otherwise the Rust ident as a string.
    pub fn r_method_name(&self) -> String {
        self.method_attrs
            .r_name
            .clone()
            .unwrap_or_else(|| self.ident.to_string())
    }

    /// C wrapper identifier for this method.
    ///
    /// Format: `C_{Type}__{method}` or `C_{Type}_{label}__{method}` if labeled.
    pub fn c_wrapper_ident(&self, type_ident: &syn::Ident, label: Option<&str>) -> syn::Ident {
        if let Some(label) = label {
            format_ident!("C_{}_{}_{}", type_ident, label, self.ident)
        } else {
            format_ident!("C_{}__{}", type_ident, self.ident)
        }
    }

    /// Generate lifecycle prelude R code for this method, if lifecycle is specified.
    ///
    /// The `what` parameter describes the method in the format appropriate for the class system:
    /// - Env/R6: `"Type$method()"`
    /// - S3: `"method.Type()"`
    /// - S7: `"method()`" (dispatched generics)
    pub fn lifecycle_prelude(&self, what: &str) -> Option<String> {
        self.method_attrs
            .lifecycle
            .as_ref()
            .and_then(|spec| spec.r_prelude(what))
    }

    /// Returns true if this method returns Self.
    pub fn returns_self(&self) -> bool {
        matches!(&self.sig.output, syn::ReturnType::Type(_, ty)
            if matches!(ty.as_ref(), syn::Type::Path(p)
                if p.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false)))
    }

    /// Returns true if this method has no return type (returns unit `()`).
    pub fn returns_unit(&self) -> bool {
        match &self.sig.output {
            syn::ReturnType::Default => true,
            syn::ReturnType::Type(_, ty) => {
                matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty())
            }
        }
    }
}

impl ParsedImpl {
    /// Parse an impl block with class system attribute.
    ///
    /// Note: Trait impls (`impl Trait for Type`) are handled by `expand_impl`
    /// before this function is called, so we only handle inherent impls here.
    pub fn parse(attrs: ImplAttrs, item_impl: syn::ItemImpl) -> syn::Result<Self> {
        // Extract type identifier
        let type_ident =
            match item_impl.self_ty.as_ref() {
                syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.clone()).ok_or_else(
                    || {
                        syn::Error::new_spanned(
                            &item_impl.self_ty,
                            "#[miniextendr] impl blocks require a named type (e.g., `impl MyType`)",
                        )
                    },
                )?,
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item_impl.self_ty,
                        "#[miniextendr] impl blocks require a named struct type. \
                     Found a non-path type. Use `impl MyStruct { ... }` with a concrete struct.",
                    ));
                }
            };

        // Reject all generics until codegen fully supports them.
        // The wrapper generation uses `type_ident` without generic args, which would
        // fail to compile or mis-resolve types for generic impls.
        if !item_impl.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &item_impl.generics,
                "generic impl blocks are not supported by #[miniextendr]. \
                 R's .Call interface requires monomorphic C symbols, so generic type parameters \
                 cannot be used. Remove the generic parameters and use a concrete type instead.",
            ));
        }

        // Reject unsupported attributes on the impl block
        for attr in &item_impl.attrs {
            if attr.path().is_ident("export_name") {
                return Err(syn::Error::new_spanned(
                    attr,
                    "#[export_name] is not supported with #[miniextendr]; \
                     the macro generates its own C symbol names",
                ));
            }
        }

        // Parse methods and validate attributes
        let mut methods = Vec::new();
        for item in &item_impl.items {
            if let syn::ImplItem::Fn(fn_item) = item {
                let method = ParsedMethod::from_impl_item(fn_item.clone(), attrs.class_system)?;
                // Validate method attributes for this class system
                ParsedMethod::validate_method_attrs(
                    &method.method_attrs,
                    attrs.class_system,
                    fn_item.sig.ident.span(),
                )?;
                methods.push(method);
            }
        }

        // Extract cfg attributes
        let cfg_attrs: Vec<_> = item_impl
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("cfg"))
            .cloned()
            .collect();
        let raw_doc_tags = crate::roxygen::roxygen_tags_from_attrs(&item_impl.attrs);
        let (doc_tags, param_warnings) = crate::roxygen::strip_method_tags(
            &raw_doc_tags,
            &type_ident.to_string(),
            item_impl.impl_token.span,
        );

        Ok(ParsedImpl {
            type_ident,
            class_system: attrs.class_system,
            class_name: attrs.class_name,
            label: attrs.label,
            doc_tags,
            methods,
            // Strip miniextendr attributes (and roxygen tags) before re-emitting,
            // then rewrite ExternalPtr receivers for stable Rust compatibility.
            original_impl: rewrite_external_ptr_receivers(strip_miniextendr_attrs_from_impl(
                item_impl,
            )),
            cfg_attrs,
            vctrs_attrs: attrs.vctrs_attrs,
            r6_inherit: attrs.r6_inherit,
            r6_portable: attrs.r6_portable,
            r6_cloneable: attrs.r6_cloneable,
            r6_lock_objects: attrs.r6_lock_objects,
            r6_lock_class: attrs.r6_lock_class,
            s7_parent: attrs.s7_parent,
            s7_abstract: attrs.s7_abstract,
            r_data_accessors: attrs.r_data_accessors,
            strict: attrs.strict,
            internal: attrs.internal,
            noexport: attrs.noexport,
            param_warnings,
        })
    }

    /// Get the class name (override or type name).
    pub fn class_name(&self) -> String {
        self.class_name
            .clone()
            .unwrap_or_else(|| self.type_ident.to_string())
    }

    /// Get methods that should be included.
    pub fn included_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| m.should_include())
    }

    /// Get the constructor method (fn new() -> Self), if included.
    /// Respects `#[...(ignore)]` and visibility filters.
    pub fn constructor(&self) -> Option<&ParsedMethod> {
        self.methods
            .iter()
            .find(|m| m.should_include() && self.is_method_constructor(m))
    }

    /// Class-system-aware constructor detection.
    ///
    /// The default `ParsedMethod::is_constructor` requires the method to return
    /// `Self`. For vctrs impls that's too strict: the canonical vctrs
    /// constructor pattern returns the underlying vector payload (e.g.
    /// `Vec<f64>`) which `vctrs::new_vctr()` then wraps — returning `Self`
    /// would produce an `ExternalPtr` that `new_vctr` can't accept as `.data`.
    fn is_method_constructor(&self, m: &ParsedMethod) -> bool {
        if m.method_attrs.constructor {
            return true;
        }
        if m.env != ReceiverKind::None || m.ident != "new" {
            return false;
        }
        match self.class_system {
            ClassSystem::Vctrs => true,
            _ => m.returns_self(),
        }
    }

    /// Get public instance methods (have env, not private, not active).
    pub fn public_instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
                && !m.is_private()
                && !m.is_active()
        })
    }

    /// Get private instance methods (have env, private visibility, not active).
    pub fn private_instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
                && m.is_private()
                && !m.is_active()
        })
    }

    /// Get active binding getter methods for R6 (have env, marked active, not setter).
    /// Active bindings provide property-like access (obj$name instead of obj$name()).
    pub fn active_instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
                && m.is_active()
                && !m.method_attrs.r6.setter // Exclude setters
        })
    }

    /// Get active binding setter methods for R6 (have env, marked as r6_setter).
    pub fn active_setter_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods
            .iter()
            .filter(|m| m.should_include() && m.env.is_instance() && m.method_attrs.r6.setter)
    }

    /// Find the setter method for a given property name.
    pub fn find_setter_for_prop(&self, prop_name: &str) -> Option<&ParsedMethod> {
        self.active_setter_methods().find(|m| {
            // Match by explicit prop name or by method name with "set_" prefix removed
            if let Some(ref explicit_prop) = m.method_attrs.r6.prop {
                explicit_prop == prop_name
            } else {
                // Try to match by stripping "set_" prefix from method name
                let method_name = m.ident.to_string();
                method_name.strip_prefix("set_").unwrap_or(&method_name) == prop_name
            }
        })
    }

    /// Get instance methods (have env) - includes both public and private.
    pub fn instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include() && m.env.is_instance() && !m.is_constructor() && !m.is_finalizer()
        })
    }

    /// Get static methods (no env, not constructor, not finalizer).
    pub fn static_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env == ReceiverKind::None
                && !self.is_method_constructor(m)
                && !m.is_finalizer()
        })
    }

    /// Get methods with `#[miniextendr(as = "...")]` attribute.
    ///
    /// These generate S3 methods for R's `as.<class>()` generics like
    /// `as.data.frame.MyType`, `as.list.MyType`, etc.
    pub fn as_coercion_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods
            .iter()
            .filter(|m| m.should_include() && m.method_attrs.as_coercion.is_some())
    }

    /// Get the finalizer method, if any.
    pub fn finalizer(&self) -> Option<&ParsedMethod> {
        self.methods
            .iter()
            .find(|m| m.should_include() && m.is_finalizer())
    }

    /// Module constant identifier for R wrapper parts.
    ///
    /// Format: `R_WRAPPERS_IMPL_{TYPE}` or `R_WRAPPERS_IMPL_{TYPE}_{LABEL}` if labeled.
    pub fn r_wrappers_const_ident(&self) -> syn::Ident {
        let type_upper = self.type_ident.to_string().to_uppercase();
        if let Some(ref label) = self.label {
            let label_upper = label.to_uppercase();
            format_ident!("R_WRAPPERS_IMPL_{}_{}", type_upper, label_upper)
        } else {
            format_ident!("R_WRAPPERS_IMPL_{}", type_upper)
        }
    }

    /// Returns the label if present.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

/// Generate a C-callable wrapper function for a single method in an impl block.
///
/// Produces a `#[no_mangle] extern "C"` function named `C_{Type}__{method}` that:
/// 1. Accepts SEXP arguments (including `self_sexp` for instance methods)
/// 2. Extracts `&self` / `&mut self` from an `ErasedExternalPtr` for instance methods
/// 3. Converts SEXP arguments to Rust types
/// 4. Calls the actual Rust method
/// 5. Converts the return value back to SEXP
///
/// Thread strategy is determined automatically: instance methods always run on the main
/// thread (because `self_ref` is a non-Send borrow), while static methods use the worker
/// thread unless `unsafe(main_thread)` is specified.
///
/// Also emits an `R_CallMethodDef` constant for R routine registration, and appends
/// generated R wrapper code fragments to the `r_wrappers_const` string constant.
///
/// # Arguments
///
/// * `parsed_impl` - The parsed impl block providing type identity, cfg attrs, and options
/// * `method` - The parsed method to generate a wrapper for
/// * `r_wrappers_const` - Identifier of the const that accumulates R wrapper code fragments
pub fn generate_method_c_wrapper(
    parsed_impl: &ParsedImpl,
    method: &ParsedMethod,
    r_wrappers_const: &syn::Ident,
) -> TokenStream {
    use crate::c_wrapper_builder::{CWrapperContext, ReturnHandling, ThreadStrategy};

    let type_ident = &parsed_impl.type_ident;
    let method_ident = &method.ident;
    let c_ident = method.c_wrapper_ident(type_ident, parsed_impl.label());

    // Determine thread strategy
    // Instance methods must use main thread because self_ref is a borrow that can't cross threads
    // Static methods use worker thread only when worker=true (set by explicit #[miniextendr(worker)]
    // or by the default-worker feature flag)
    let thread_strategy = if method.method_attrs.unsafe_main_thread || method.env.is_instance() {
        ThreadStrategy::MainThread
    } else if method.method_attrs.worker {
        ThreadStrategy::WorkerThread
    } else {
        ThreadStrategy::MainThread
    };

    // Build rust argument names from the signature
    let rust_args: Vec<syn::Ident> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                Some(pat_ident.ident.clone())
            } else {
                None
            }
        })
        .collect();

    // Generate self extraction for instance methods
    // SEXP is now Send+Sync, so this works for both main and worker threads
    let pre_call = if method.env.is_instance() {
        let self_extraction = match method.env {
            ReceiverKind::RefMut => {
                quote! {
                    let mut self_ptr = unsafe {
                        ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(self_sexp)
                    };
                    let self_ref = self_ptr.downcast_mut::<#type_ident>()
                        .expect(concat!("expected ExternalPtr<", stringify!(#type_ident), ">"));
                }
            }
            ReceiverKind::Ref => {
                quote! {
                    let self_ptr = unsafe {
                        ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(self_sexp)
                    };
                    let self_ref = self_ptr.downcast_ref::<#type_ident>()
                        .expect(concat!("expected ExternalPtr<", stringify!(#type_ident), ">"));
                }
            }
            ReceiverKind::ExternalPtrRef => {
                quote! {
                    let __self_ptr = unsafe {
                        ::miniextendr_api::externalptr::ExternalPtr::<#type_ident>::wrap_sexp(self_sexp)
                            .expect(concat!("expected ExternalPtr<", stringify!(#type_ident), ">"))
                    };
                }
            }
            ReceiverKind::ExternalPtrRefMut => {
                quote! {
                    let mut __self_ptr = unsafe {
                        ::miniextendr_api::externalptr::ExternalPtr::<#type_ident>::wrap_sexp(self_sexp)
                            .expect(concat!("expected ExternalPtr<", stringify!(#type_ident), ">"))
                    };
                }
            }
            ReceiverKind::ExternalPtrValue => {
                quote! {
                    let __self_ptr = unsafe {
                        ::miniextendr_api::externalptr::ExternalPtr::<#type_ident>::wrap_sexp(self_sexp)
                            .expect(concat!("expected ExternalPtr<", stringify!(#type_ident), ">"))
                    };
                }
            }
            _ => unreachable!(),
        };
        vec![self_extraction]
    } else {
        vec![]
    };

    // Generate call expression
    let call_expr = match method.env {
        ReceiverKind::Ref | ReceiverKind::RefMut => {
            quote! { self_ref.#method_ident(#(#rust_args),*) }
        }
        ReceiverKind::ExternalPtrRef => {
            quote! { #type_ident::#method_ident(&__self_ptr, #(#rust_args),*) }
        }
        ReceiverKind::ExternalPtrRefMut => {
            quote! { #type_ident::#method_ident(&mut __self_ptr, #(#rust_args),*) }
        }
        ReceiverKind::ExternalPtrValue => {
            quote! { #type_ident::#method_ident(__self_ptr, #(#rust_args),*) }
        }
        ReceiverKind::None | ReceiverKind::Value => {
            quote! { #type_ident::#method_ident(#(#rust_args),*) }
        }
    };

    // Determine return handling strategy
    let return_handling = if method.returns_self() {
        ReturnHandling::ExternalPtr
    } else if method.method_attrs.unwrap_in_r && output_is_result(&method.sig.output) {
        ReturnHandling::IntoR
    } else {
        crate::c_wrapper_builder::detect_return_handling(&method.sig.output)
    };

    // Build the context using the builder
    let mut builder = CWrapperContext::builder(method_ident.clone(), c_ident)
        .r_wrapper_const(r_wrappers_const.clone())
        .inputs(method.sig.inputs.clone())
        .output(method.sig.output.clone())
        .pre_call(pre_call)
        .call_expr(call_expr)
        .thread_strategy(thread_strategy)
        .return_handling(return_handling)
        .cfg_attrs(parsed_impl.cfg_attrs.clone())
        .type_context(type_ident.clone());

    if method.env.is_instance() {
        builder = builder.has_self();
    }

    if method.method_attrs.coerce {
        builder = builder.coerce_all();
    }

    if method.method_attrs.check_interrupt {
        builder = builder.check_interrupt();
    }

    if method.method_attrs.rng {
        builder = builder.rng();
    }

    if parsed_impl.strict {
        builder = builder.strict();
    }

    if method.method_attrs.error_in_r {
        builder = builder.error_in_r();
    }

    // Forward match_arg + several_ok parameter names so `RustConversionBuilder` swaps
    // in `match_arg_vec_from_sexp` for the Vec/slice/array/Box<[_]> conversion path.
    // Scalar match_arg doesn't need this — R's match.arg() validated the choice and
    // `TryFromSexp for EnumType` (auto-generated by `#[derive(MatchArg)]`) decodes it.
    for (rust_name, attrs) in &method.method_attrs.per_param {
        if attrs.match_arg && attrs.several_ok {
            builder = builder.match_arg_several_ok(rust_name.clone());
        }
    }

    let c_wrapper_and_def = builder.build().generate();

    // Emit one `__match_arg_choices__<param>` helper fn + linkme registrations for each
    // match_arg-annotated parameter so the R wrapper can look up the enum's
    // `MatchArg::CHOICES` at call time (C extern) and the package-load write step can
    // substitute the placeholder default with the literal choices (distributed_slice).
    let match_arg_helpers = generate_method_match_arg_helpers(parsed_impl, method);

    quote! {
        #c_wrapper_and_def
        #match_arg_helpers
    }
}

/// Generate `__match_arg_choices__<param>` helper C wrappers plus the two linkme
/// registrations (`MX_CALL_DEFS` and `MX_MATCH_ARG_CHOICES`) per match_arg parameter.
///
/// Mirrors the standalone-fn emission in `lib.rs` so both surfaces resolve through the
/// same runtime paths — `C_*__match_arg_choices__*` is called from R's prelude, and
/// `MX_MATCH_ARG_CHOICES` drives write-time placeholder substitution when the cdylib
/// emits the final R wrapper file.
fn generate_method_match_arg_helpers(
    parsed_impl: &ParsedImpl,
    method: &ParsedMethod,
) -> TokenStream {
    if !method
        .method_attrs
        .per_param
        .values()
        .any(|a| a.match_arg || a.choices.is_some())
    {
        return TokenStream::new();
    }

    let type_ident = &parsed_impl.type_ident;
    let c_ident = method.c_wrapper_ident(type_ident, parsed_impl.label());
    let c_ident_str = c_ident.to_string();
    let cfg_attrs = &parsed_impl.cfg_attrs;

    let mut out = TokenStream::new();

    for (rust_name, attrs) in method.method_attrs.per_param.iter() {
        if !attrs.match_arg {
            continue;
        }
        // Find the parameter type from the (already-normalized) signature.
        let Some(param_ty) = find_param_type(&method.sig.inputs, rust_name) else {
            continue;
        };
        // For several_ok, unwrap the container (Vec<Mode>, Box<[Mode]>, [Mode; N], &[Mode])
        // so the helper returns the inner enum's CHOICES, not the container type.
        let several_ok = attrs.several_ok;
        let choices_ty = if several_ok {
            crate::classify_several_ok_container(param_ty)
                .map(|(_, inner)| inner.clone())
                .unwrap_or_else(|| param_ty.clone())
        } else {
            param_ty.clone()
        };

        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);

        // C helper fn that R calls via .__mx_choices_<param> <- .Call(C_...)
        let helper_c_name_str = crate::match_arg_keys::choices_helper_c_name(&c_ident_str, &r_name);
        let helper_fn_ident = syn::Ident::new(&helper_c_name_str, proc_macro2::Span::call_site());
        let helper_def_ident =
            crate::match_arg_keys::choices_helper_def_ident(&c_ident_str, &r_name);
        let helper_c_name = syn::LitCStr::new(
            std::ffi::CString::new(helper_c_name_str.clone())
                .expect("valid C string")
                .as_c_str(),
            proc_macro2::Span::call_site(),
        );

        // Placeholder that the write-time substitution pass replaces with the
        // literal `c("a", "b", ...)` default. Shape matches the standalone-fn convention.
        let placeholder = crate::r_class_formatter::match_arg_placeholder(&c_ident_str, &r_name);
        let entry_ident = syn::Ident::new(
            &format!(
                "match_arg_choices_entry_{}",
                crate::match_arg_keys::placeholder_ident_suffix(&placeholder)
            ),
            proc_macro2::Span::call_site(),
        );
        let doc_placeholder =
            crate::r_class_formatter::match_arg_param_doc_placeholder(&c_ident_str, &r_name);
        let doc_entry_ident = syn::Ident::new(
            &format!(
                "match_arg_param_doc_entry_{}",
                crate::match_arg_keys::placeholder_ident_suffix(&doc_placeholder)
            ),
            proc_macro2::Span::call_site(),
        );

        let choices_entry_tokens = crate::match_arg_keys::choices_entry_tokens(
            cfg_attrs,
            &entry_ident,
            &placeholder,
            &choices_ty,
        );
        let param_doc_entry_tokens = crate::match_arg_keys::param_doc_entry_tokens(
            cfg_attrs,
            &doc_entry_ident,
            &doc_placeholder,
            several_ok,
            &choices_ty,
        );

        out.extend(quote! {
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
                    fun: Some(::std::mem::transmute::<
                        unsafe extern "C-unwind" fn(
                            ::miniextendr_api::ffi::SEXP,
                        ) -> ::miniextendr_api::ffi::SEXP,
                        unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void,
                    >(#helper_fn_ident)),
                    numArgs: 1i32,
                }
            };

            #choices_entry_tokens
            #param_doc_entry_tokens
        });
    }

    out
}

/// Find a parameter's Rust type from a stripped signature by identifier name.
fn find_param_type<'a>(
    inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    name: &str,
) -> Option<&'a syn::Type> {
    for arg in inputs {
        if let syn::FnArg::Typed(pt) = arg
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            && pat_ident.ident == name
        {
            return Some(pt.ty.as_ref());
        }
    }
    None
}

/// Check whether a function's return type is syntactically `Result<_, _>`.
///
/// This performs a shallow name check on the last path segment -- it does not resolve
/// type aliases. Used to decide whether `unwrap_in_r` should strip the `Result` wrapper
/// before converting to SEXP.
fn output_is_result(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Type(_, ty) => matches!(
            ty.as_ref(),
            syn::Type::Path(p)
                if p.path
                    .segments
                    .last()
                    .map(|s| s.ident == "Result")
                    .unwrap_or(false)
        ),
        syn::ReturnType::Default => false,
    }
}

// region: Class-system R wrapper generators (sub-modules)

/// Environment-based class wrapper generator (`obj$method()` dispatch).
mod env_class;
/// R6 class wrapper generator (`R6Class` with `$new()`, active bindings, private methods).
mod r6_class;
/// S3 class wrapper generator (`structure()` + `generic.Class` dispatch).
mod s3_class;
/// S4 class wrapper generator (`setClass` / `setMethod` formal OOP).
mod s4_class;
/// S7 class wrapper generator (`new_class` / `new_generic` modern R OOP).
mod s7_class;
/// vctrs-compatible class wrapper generator (`new_vctr` / `new_rcrd` / `new_list_of`).
mod vctrs_class;

pub(crate) use env_class::generate_env_r_wrapper;
pub(crate) use r6_class::generate_r6_r_wrapper;
pub(crate) use s3_class::generate_s3_r_wrapper;
pub(crate) use s4_class::generate_s4_r_wrapper;
pub(crate) use s7_class::generate_s7_r_wrapper;
#[cfg(test)]
use s7_class::rust_type_to_s7_class;
pub(crate) use vctrs_class::generate_vctrs_r_wrapper;

/// Generate R S3 method wrappers for `as.<class>()` coercion methods.
///
/// For each method with `#[miniextendr(as = "...")]`, generates an S3 method like:
///
/// ```r
/// #' @export
/// #' @method as.data.frame MyType
/// as.data.frame.MyType <- function(x, ...) {
///     .Call(C_MyType__as_data_frame, .call = match.call(), x)
/// }
/// ```
///
/// This function is called by each class system generator to append the
/// `as.*` methods to the R wrapper output.
pub fn generate_as_coercion_methods(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::MethodContext;

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;

    // Check if class has @noRd - if so, skip documentation
    let class_doc_tags = &parsed_impl.doc_tags;
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let class_has_internal = crate::roxygen::has_roxygen_tag(class_doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !class_has_internal && !parsed_impl.noexport;

    let mut lines = Vec::new();

    for method in parsed_impl.as_coercion_methods() {
        // Get the coercion target (e.g., "data.frame", "list", "character")
        let coercion_target = match &method.method_attrs.as_coercion {
            Some(target) => target.clone(),
            None => continue,
        };

        // Build method context for .Call generation
        let ctx = MethodContext::new(method, type_ident, parsed_impl.label());

        // Normalize coercion target for R generic name
        // R has both as.numeric and as.double - they're equivalent, but we use the specified one
        // Some targets use non-standard S3 generic names (e.g., tibble uses as_tibble, not as.tibble)
        let r_generic = match coercion_target.as_str() {
            "numeric" => "as.numeric".to_string(),
            "double" => "as.double".to_string(),
            "tibble" => "as_tibble".to_string(),
            "ts" => "as.ts".to_string(),
            other => format!("as.{}", other),
        };

        // S3 method name: as.data.frame.MyType
        let s3_method_name = format!("{}.{}", r_generic, class_name);

        // Documentation
        if !class_has_no_rd {
            // Add documentation from the method
            if !method.doc_tags.is_empty() {
                crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
            }
            lines.push(format!("#' @name {}", s3_method_name));
            lines.push(format!("#' @rdname {}", class_name));
            lines.push(format!(
                "#' @source Generated by miniextendr from `{}::{}`",
                type_ident, method.ident
            ));
        }

        // Export and method registration
        if should_export {
            lines.push("#' @export".to_string());
        }
        lines.push(format!("#' @method {} {}", r_generic, class_name));

        // Function signature: always takes x and ... for S3 method compatibility
        // Additional parameters from the method are included
        let method_params =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);
        let formals = if method_params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", method_params)
        };

        lines.push(format!("{} <- function({}) {{", s3_method_name, formals));

        // Build the .Call() invocation
        let call = ctx.instance_call("x");
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(method.method_attrs.error_in_r);
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate `impl As*` trait impls for methods with `#[miniextendr(as = "...")]`.
///
/// For each `as` coercion method, generates a forwarding trait impl:
/// ```ignore
/// impl ::miniextendr_api::as_coerce::AsDataFrame for MyType {
///     fn as_data_frame(&self) -> Result<::miniextendr_api::List, ::miniextendr_api::as_coerce::AsCoerceError> {
///         self.as_data_frame()  // inherent method preferred over trait method
///     }
/// }
/// ```
///
/// Skips methods with extra parameters beyond `&self` (trait methods have fixed signatures)
/// and skips non-standard targets (like "tibble", "data.table") that don't have corresponding traits.
pub fn generate_as_coercion_trait_impls(parsed_impl: &ParsedImpl) -> TokenStream {
    let type_ident = &parsed_impl.type_ident;
    let cfg_attrs = &parsed_impl.cfg_attrs;

    let mut impls = Vec::new();

    for method in parsed_impl.as_coercion_methods() {
        let coercion_target = match &method.method_attrs.as_coercion {
            Some(target) => target.as_str(),
            None => continue,
        };

        // Skip methods with extra params beyond &self — trait methods have fixed &self-only signatures.
        // `sig.inputs` already has self stripped, so non-empty means extra params.
        if !method.sig.inputs.is_empty() {
            continue;
        }

        // Skip non-instance methods (trait requires &self)
        if method.env != ReceiverKind::Ref {
            continue;
        }

        // Map coercion target to (trait name, trait method name, return type tokens).
        // Only the 15 standard targets that have corresponding traits in as_coerce.
        let (trait_name, trait_method): (&str, &str) = match coercion_target {
            "data.frame" => ("AsDataFrame", "as_data_frame"),
            "list" => ("AsList", "as_list"),
            "character" => ("AsCharacter", "as_character"),
            "numeric" | "double" => ("AsNumeric", "as_numeric"),
            "integer" => ("AsInteger", "as_integer"),
            "logical" => ("AsLogical", "as_logical"),
            "matrix" => ("AsMatrix", "as_matrix"),
            "vector" => ("AsVector", "as_vector"),
            "factor" => ("AsFactor", "as_factor"),
            "Date" => ("AsDate", "as_date"),
            "POSIXct" => ("AsPOSIXct", "as_posixct"),
            "complex" => ("AsComplex", "as_complex"),
            "raw" => ("AsRaw", "as_raw"),
            "environment" => ("AsEnvironment", "as_environment"),
            "function" => ("AsFunction", "as_function"),
            _ => continue, // Non-standard targets (tibble, data.table, etc.)
        };

        let trait_ident = syn::Ident::new(trait_name, proc_macro2::Span::call_site());
        let trait_method_ident = syn::Ident::new(trait_method, proc_macro2::Span::call_site());
        let user_method_ident = &method.ident;

        // Return type: data.frame and list return Result<List, AsCoerceError>,
        // all others return Result<SEXP, AsCoerceError>
        let return_type = match coercion_target {
            "data.frame" | "list" => quote! {
                ::core::result::Result<::miniextendr_api::List, ::miniextendr_api::as_coerce::AsCoerceError>
            },
            _ => quote! {
                ::core::result::Result<::miniextendr_api::ffi::SEXP, ::miniextendr_api::as_coerce::AsCoerceError>
            },
        };

        impls.push(quote! {
            #(#cfg_attrs)*
            impl ::miniextendr_api::as_coerce::#trait_ident for #type_ident {
                fn #trait_method_ident(&self) -> #return_type {
                    self.#user_method_ident()
                }
            }
        });
    }

    quote! { #(#impls)* }
}

/// Top-level entry point for expanding `#[miniextendr]` on impl blocks.
///
/// Dispatches between two cases:
/// 1. **Inherent impls** (`impl Type { ... }`): Parses [`ImplAttrs`] and [`ParsedImpl`],
///    then generates C wrappers, R wrapper code, `R_CallMethodDef` arrays, and
///    `as.<class>()` trait impls for the chosen class system.
/// 2. **Trait impls** (`impl Trait for Type { ... }`): Generates trait ABI vtables,
///    cross-package shims, and R wrappers via
///    [`expand_miniextendr_impl_trait`](crate::miniextendr_impl_trait::expand_miniextendr_impl_trait).
///
/// # Arguments
///
/// * `attr` - The token stream inside `#[miniextendr(...)]` (class system, options)
/// * `item` - The full `impl` block token stream
///
/// # Returns
///
/// A token stream containing the original impl block (with miniextendr attrs stripped),
/// C wrapper functions, an R wrapper string constant, a `R_CallMethodDef` array constant,
/// and any forwarding trait impls for `as.<class>()` coercion.
pub fn expand_impl(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_impl = match syn::parse::<syn::ItemImpl>(item.clone()) {
        Ok(i) => i,
        Err(e) => return e.into_compile_error().into(),
    };

    // Check if this is a trait impl (impl Trait for Type)
    if item_impl.trait_.is_some() {
        // Delegate to trait ABI vtable generator
        return crate::miniextendr_impl_trait::expand_miniextendr_impl_trait(attr, item);
    }

    // Otherwise, this is an inherent impl - parse class system attrs
    let attrs = match syn::parse::<ImplAttrs>(attr) {
        Ok(a) => a,
        Err(e) => return e.into_compile_error().into(),
    };

    let parsed = match ParsedImpl::parse(attrs, item_impl) {
        Ok(p) => p,
        Err(e) => return e.into_compile_error().into(),
    };

    // Generate constants for module registration (needed for doc links)
    let type_ident = &parsed.type_ident;
    let cfg_attrs = &parsed.cfg_attrs;
    let r_wrappers_const = parsed.r_wrappers_const_ident();

    // Generate C wrappers for all included methods
    let c_wrappers: Vec<TokenStream> = parsed
        .included_methods()
        .map(|m| generate_method_c_wrapper(&parsed, m, &r_wrappers_const))
        .collect();

    // Generate R wrapper string based on class system
    let mut r_wrapper_string = match parsed.class_system {
        ClassSystem::Env => generate_env_r_wrapper(&parsed),
        ClassSystem::R6 => generate_r6_r_wrapper(&parsed),
        ClassSystem::S3 => generate_s3_r_wrapper(&parsed),
        ClassSystem::S7 => generate_s7_r_wrapper(&parsed),
        ClassSystem::S4 => generate_s4_r_wrapper(&parsed),
        ClassSystem::Vctrs => generate_vctrs_r_wrapper(&parsed),
    };

    // Append as.<class>() coercion methods (works with all class systems)
    let as_coercion_wrappers = generate_as_coercion_methods(&parsed);
    if !as_coercion_wrappers.is_empty() {
        r_wrapper_string.push_str("\n\n");
        r_wrapper_string.push_str(&as_coercion_wrappers);
    }

    let original_impl = &parsed.original_impl;

    // Generate forwarding trait impls for as.<class>() coercion methods
    let trait_impls = generate_as_coercion_trait_impls(&parsed);

    let r_wrapper_str = crate::r_wrapper_raw_literal(&r_wrapper_string);

    // Generate doc comment linking to R wrapper constant
    let r_wrapper_doc = format!(
        "See [`{}`] for the generated R wrapper code.",
        r_wrappers_const
    );
    let source_loc_doc = crate::source_location_doc(type_ident.span());
    let source_start = type_ident.span().start();
    let source_line_lit = syn::LitInt::new(&source_start.line.to_string(), type_ident.span());
    let source_col_lit =
        syn::LitInt::new(&(source_start.column + 1).to_string(), type_ident.span());

    let param_warnings = &parsed.param_warnings;

    // Build MX_CLASS_NAMES entry for cross-reference resolution.
    // r_class_name is the R-visible name (may differ from type_ident when
    // `class = "Override"` was set on the impl block).
    let r_class_name_str = parsed.class_name();
    let class_system_str = parsed.class_system.to_ident().to_string();
    let class_names_const = syn::Ident::new(
        &format!(
            "__mx_class_name_entry_{}",
            type_ident.to_string().to_lowercase()
        ),
        type_ident.span(),
    );

    let expanded = quote! {
        // Original impl block with doc link to R wrapper
        #[doc = #r_wrapper_doc]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #original_impl

        // Warnings for @param tags on impl blocks
        #param_warnings

        // C wrappers and call method defs
        #(#c_wrappers)*

        // Forwarding trait impls for as.<class>() coercion methods
        #trait_impls

        // R wrapper registration via distributed slice
        #(#cfg_attrs)*
        #[doc = concat!(
            "R wrapper code for impl block on `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS)]
                #[linkme(crate = ::miniextendr_api::linkme)]
        static #r_wrappers_const: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::Class,
                source_file: file!(),
                content: concat!(
                    "# Generated from Rust impl `",
                    stringify!(#type_ident),
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

        // Class name registration for cross-reference placeholder resolution.
        // Maps the Rust type name to the R-visible class name at link time.
        #(#cfg_attrs)*
        #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CLASS_NAMES)]
        #[linkme(crate = ::miniextendr_api::linkme)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        static #class_names_const: ::miniextendr_api::registry::ClassNameEntry =
            ::miniextendr_api::registry::ClassNameEntry {
                rust_type: stringify!(#type_ident),
                r_class_name: #r_class_name_str,
                class_system: #class_system_str,
            };
    };

    expanded.into()
}

#[cfg(test)]
mod tests;
// endregion
