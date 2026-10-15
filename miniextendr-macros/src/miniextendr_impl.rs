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
//! │  │ C_*_Type__method│  │ (as const str)  │  │  registration entries  │ │
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
//! | `self: ExternalPtrRef<'_, Self>` | `ExternalPtrRef` | Instance method (immutable, with the R object: sidecar fields) |
//! | `self: ExternalPtrMut<'_, Self>` | `ExternalPtrMut` | Instance method (mutable, with the R object: sidecar fields) |
//! | `self: ExternalPtr<Self>` | `ExternalPtrValue` | Instance method (the handle itself, unborrowed) |
//! | `self` | `Value` | Consuming method: `-> Self` writes the result back into the same handle; `-> Result<Self, E>` / `Option<Self>` run on a clone (`T: Clone`) and overwrite on success; any other return consumes the handle (later use errors) |
//! | (none) | `None` | Static method or constructor |
//!
//! Special methods:
//! - **Constructor**: Returns `Self` with no receiver, marked with `#[miniextendr(constructor)]` or named `new`
//! - **Finalizer**: R6 only, marked with `#[miniextendr(r6(finalize))]` (never inferred from the receiver)
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
//! - C wrappers: `C_<crate>_Counter__new`, `C_<crate>_Counter__get`, `C_<crate>_Counter__increment`
//! - R6Class with `initialize`, `get`, `increment` methods
//! - Registration entries for R's `.Call()` interface

use proc_macro2::TokenStream;
use quote::quote;

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

/// Refuse the receivers `self: &ExternalPtr<Self>` / `self: &mut
/// ExternalPtr<Self>`, which #2106 replaced by the views: a handle behind a
/// reference reads its value unchecked, while a view holds the borrow.
fn old_handle_receiver(sig: &syn::Signature) -> syn::Result<()> {
    let Some(syn::FnArg::Receiver(receiver)) = sig.inputs.first() else {
        return Ok(());
    };
    let syn::Type::Reference(reference) = receiver.ty.as_ref() else {
        return Ok(());
    };
    if receiver.colon_token.is_none() || !is_external_ptr_type(&reference.elem) {
        return Ok(());
    }
    let (view, binding) = if reference.mutability.is_some() {
        ("ExternalPtrMut", "mut self")
    } else {
        ("ExternalPtrRef", "self")
    };
    Err(syn::Error::new_spanned(
        &receiver.ty,
        format!(
            "a method takes its object as `{binding}: {view}<'_, Self>`: the wrapper borrows the \
             value for the call, and the view carries the R object for the sidecar accessors \
             (`Self::field(&self)`); use `&self` / `&mut self` when the body needs only the \
             value, or `self: ExternalPtr<Self>` for the unborrowed handle"
        ),
    ))
}

/// Rewrite methods with handle receivers so they compile on stable Rust
/// (which lacks `arbitrary_self_types`).
///
/// Handles:
/// - `self: ExternalPtrRef<'_, Self>` → `__miniextendr_self: ExternalPtrRef<'_, Self>`
/// - `self: ExternalPtrMut<'_, Self>` → `__miniextendr_self: ExternalPtrMut<'_, Self>`
/// - `self: ExternalPtr<Self>` → `__miniextendr_self: ExternalPtr<Self>`
///
/// Also replaces all `self` references in the method body with `__miniextendr_self`.
fn rewrite_external_ptr_receivers(mut item_impl: syn::ItemImpl) -> syn::ItemImpl {
    for item in &mut item_impl.items {
        let syn::ImplItem::Fn(method) = item else {
            continue;
        };
        if crate::trait_receiver::owner_receiver(&method.sig).is_some() {
            crate::trait_receiver::rewrite(&mut method.sig, Some(&mut method.block));
            continue;
        }
        let Some(syn::FnArg::Receiver(receiver)) = method.sig.inputs.first() else {
            continue;
        };
        if receiver.colon_token.is_none() || !is_external_ptr_type(receiver.ty.as_ref()) {
            continue;
        }

        // self: ExternalPtr<Self> (by value)
        let receiver_ident =
            proc_macro2::Ident::new("__miniextendr_self", proc_macro2::Span::mixed_site());
        let binding_mutability = receiver.mutability;
        let inner_ty = &receiver.ty;
        let new_param: syn::FnArg = syn::parse_quote! {
            #binding_mutability #receiver_ident: #inner_ty
        };

        // Replace first parameter
        let inputs: Vec<syn::FnArg> = method.sig.inputs.iter().cloned().collect();
        let mut new_inputs: Vec<syn::FnArg> = Vec::with_capacity(inputs.len());
        new_inputs.push(new_param);
        new_inputs.extend(inputs.into_iter().skip(1));
        method.sig.inputs = new_inputs.into_iter().collect();

        crate::receiver_body::rewrite(&mut method.block, &receiver_ident);
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

/// What the impl option `r_data_accessors` asks for: the class-level form of
/// the `pub` `#[r_data]` fields of `#[derive(ExternalPtr)]`, next to the
/// standalone `Type_get_f()` / `Type_set_f()` every class gets.
///
/// R6 gets active bindings, S7 properties, and S3, S4 and env classes `$` /
/// `[[` field methods (#1848, `field_syntax.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RDataAccessors {
    /// No option: only the standalone accessors.
    #[default]
    Off,
    /// `r_data_accessors`: a getter and a setter per field.
    GetSet,
    /// `r_data_accessors = "get"` (S3, S4, env): `$` / `[[` read the fields
    /// and no `$<-` / `[[<-` is generated, for a class whose own replacement
    /// methods write them (for example copy-on-modify setters that return a
    /// new object).
    GetOnly,
}

impl RDataAccessors {
    /// Whether the class wires its fields in at all.
    pub fn enabled(self) -> bool {
        self != RDataAccessors::Off
    }

    /// Whether the class gets field setters (`$<-` / `[[<-`).
    pub fn setters(self) -> bool {
        self == RDataAccessors::GetSet
    }
}

/// How the S3 readers of `s3(r_data_accessors)` look a field up on a bare
/// pointer (`field_syntax.rs`): `s3(r_data_accessors, lookup = "list")`.
/// Neither lookup matches a prefix of a name (#1922).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldLookup {
    /// The default (#1921): exact names and whole-number positions only; an
    /// unknown name raises `miniextendr_no_field` and any other index the
    /// bad-index error.
    #[default]
    Strict,
    /// `lookup = "list"` (#1891): a list's lenience without its prefix
    /// match. An unknown name or `NA` reads `NULL`, and `[[` takes a logical
    /// or a fractional position.
    List,
}

/// Parses an `r_data_accessors` key inside `s3(...)` / `s4(...)` / `env(...)`
/// / `r6(...)` / `s7(...)`, after the key: nothing, or `= "get"` (S3, S4 and
/// env only).
fn parse_r_data_accessors_value(
    content: syn::parse::ParseStream,
    key: &syn::Ident,
    system: ClassSystem,
) -> syn::Result<RDataAccessors> {
    if !content.peek(syn::Token![=]) {
        return Ok(RDataAccessors::GetSet);
    }
    let _: syn::Token![=] = content.parse()?;
    let value: syn::LitStr = content.parse()?;
    if !matches!(system, ClassSystem::S3 | ClassSystem::S4 | ClassSystem::Env) {
        return Err(syn::Error::new(
            key.span(),
            "the getters-only form `r_data_accessors = \"get\"` applies to `s3`, `s4` and \
             `env` classes; R6 active bindings and S7 properties take the bare \
             `r_data_accessors`",
        ));
    }
    if value.value() != "get" {
        return Err(syn::Error::new(
            value.span(),
            format!(
                "unknown `r_data_accessors` value \"{}\"; expected `r_data_accessors` (getters \
                 and setters) or `r_data_accessors = \"get\"` (getters only)",
                value.value()
            ),
        ));
    }
    Ok(RDataAccessors::GetOnly)
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
    /// `self` (or `self: Self`) - consuming. The C wrapper moves the value out
    /// of the R handle: `-> Self` writes the result back (same handle, in-place
    /// semantics), `-> Result<Self, E>` / `-> Option<Self>` run on a clone and
    /// overwrite on success, anything else leaves the handle consumed.
    Value,
    /// `self: ExternalPtrRef<'_, Self>` — shared borrow, with the R object
    ExternalPtrRef,
    /// `self: ExternalPtrMut<'_, Self>` — exclusive borrow, with the R object
    ExternalPtrMut,
    /// `self: ExternalPtr<Self>` — the handle, unborrowed (not consuming the inner T)
    ExternalPtrValue,
}

impl ReceiverKind {
    /// Returns true if this is an instance method (has self), including the
    /// consuming `self` receiver.
    pub fn is_instance(&self) -> bool {
        matches!(
            self,
            ReceiverKind::Ref
                | ReceiverKind::RefMut
                | ReceiverKind::Value
                | ReceiverKind::ExternalPtrRef
                | ReceiverKind::ExternalPtrMut
                | ReceiverKind::ExternalPtrValue
        )
    }

    /// Returns true if this is a mutable instance receiver.
    pub fn is_mut(&self) -> bool {
        matches!(self, ReceiverKind::RefMut | ReceiverKind::ExternalPtrMut)
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
    /// Return-type visibility marker (#1213): `Some(true)` for
    /// `-> Invisible<T>`, `Some(false)` for `-> Visible<T>`. The marker is
    /// peeled from [`Self::sig`] at parse time, so every return-shape
    /// predicate sees `T`; the C wrapper unwraps the newtype before conversion.
    pub visibility_marker: Option<bool>,
    /// Explicit target class wrapping, shared by marker and attribute spellings.
    pub return_wrap: Option<crate::return_wrap::ReturnWrap>,
    /// The method's name (e.g., `new`, `get`, `set_value`).
    pub ident: syn::Ident,
    /// How this method receives `self`: `&self`, `&mut self`, by value, or not at all (static).
    pub env: ReceiverKind,
    /// Method signature with the `self` receiver stripped. Used for C wrapper generation
    /// where `self` is handled separately as a SEXP parameter. The parameter
    /// markers (`Checked<T>` / `Unchecked<T>`, #1566) are peeled from it, as
    /// the visibility marker is from the output: every R-side consumer and
    /// the conversion see the inner type.
    pub sig: syn::Signature,
    /// The parameter markers peeled from [`Self::sig`] (Rust name →
    /// markers, outermost first): the C wrapper wraps the converted value in
    /// them before the call.
    pub param_markers: Vec<(String, Vec<crate::type_inspect::ParamMarker>)>,
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
    /// Whether this method accepts dots, either from raw `...` rewritten to
    /// `&Dots` or from an explicit `&Dots` parameter, anywhere.
    pub has_dots: bool,
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
    /// Opt out of the per-class `<ClassName>_<method>` fast-dispatch shortcut
    /// (set via `#[miniextendr(s7(no_shortcut))]`). The S7 generic + method are
    /// still emitted; only the standalone shortcut function is suppressed.
    pub no_shortcut: bool,
}

/// Per-method attributes for class system customization.
#[derive(Debug, Default)]
pub struct MethodAttrs {
    /// Skip this method
    pub ignore: bool,
    /// Mark as constructor
    pub constructor: bool,
    /// `invisible` / `visible` method option (#1213): `Some(true)` makes the R
    /// wrapper return `invisible(...)`, `Some(false)` forces a visible return.
    /// Same decision as an `Invisible<T>` / `Visible<T>` return-type marker;
    /// the two must not disagree. Nothing is invisible by default.
    pub force_invisible: Option<bool>,
    /// R6-specific method markers. All R6 boolean flags live here.
    /// Only consumed by the R6 class generator and R6-aware accessor methods
    /// (`ParsedMethod::is_active`, `is_private`, `is_finalizer`).
    pub r6: R6MethodAttrs,
    /// Generate as `as.<class>()` S3 method (e.g., "data.frame", "list", "character").
    ///
    /// When set, generates an S3 method for R's `as.<class>()` generic:
    /// ```r
    /// as.data.frame.MyType <- function(x, ...) {
    ///     .Call(C_MyType__as_data_frame, .call = sys.call(), x)
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
    /// First formal of an existing S3/S4 generic (for example `object` for show).
    pub receiver: Option<String>,
    /// Omit the automatic dispatch dots for an existing S3/S4 generic.
    pub no_dispatch_dots: bool,
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
    /// The method's own `preconditions` / `no_preconditions` (bare; #1017,
    /// #1566): `Some(true)` keeps the type-derived R-side checks of the
    /// parameters that do not decide for themselves, `Some(false)` drops
    /// them; the last one written wins. `None` defers to the impl block,
    /// the crate default and the feature. The list forms
    /// `preconditions(p)` / `no_preconditions(p)` land in
    /// [`Self::per_param`].
    pub preconditions: Option<bool>,
    /// Enable RNG state management (GetRNGstate/PutRNGstate)
    pub rng: bool,
    /// `typed_list!(...)` spec from `#[miniextendr(dots = typed_list!(...))]` on
    /// this method, mirroring `MiniextendrFnAttrs.dots_spec`. When set, a
    /// `dots_typed` binding is injected at the top of the method body. Per-method
    /// (dots apply to an individual signature), not an impl-block-level option.
    pub dots_spec: Option<proc_macro2::TokenStream>,
    /// Return `Result<T, E>` to R without unwrapping.
    pub unwrap_in_r: bool,
    /// Build the `Err` arm's condition from the error's serde output
    /// (`#[miniextendr(serde_error)]`, optionally `serde_error(tag = .., prefix = ..)`).
    pub serde_error: Option<crate::miniextendr_fn::SerdeErrorSpec>,
    /// Serialize the complete return value through `AsSerialize<T>`.
    pub serialize: bool,
    /// Explicit return class system (`wrap = "r6"` and siblings).
    pub wrap: Option<ClassSystem>,
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
    /// `#[miniextendr(match_arg_several_ok(p))]`,
    /// `#[miniextendr(choices(p = "a, b"))]` and
    /// `#[miniextendr(no_default(p))]` on the method attribute.
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
    /// Append a fixed suffix to the Rust method name for the R-facing name
    /// (`#[miniextendr(postfix = "_impl")]` on `fn bump` yields `bump_impl`).
    /// Exclusive with `r_name` and `generic`; the C symbol is unchanged.
    pub postfix: Option<String>,
    /// R code to inject at the very top of the method body (before all built-in checks).
    pub r_entry: Option<String>,
    /// R code to inject after all built-in checks, immediately before `.Call()`.
    pub r_post_checks: Option<String>,
    /// Register `on.exit()` cleanup code in the R method wrapper.
    ///
    /// Short form: `#[miniextendr(r_on_exit = "close(con)")]`
    /// Long form: `#[miniextendr(r_on_exit(expr = "close(con)", add = false))]`
    pub r_on_exit: Option<crate::miniextendr_fn::ROnExit>,
    /// Mark this method as internal: adds `@keywords internal`, suppresses export.
    ///
    /// For R6 active bindings this emits a class-level
    /// `#' @field name (internal)` so the binding stays satisfied for roxygen2
    /// (which warns on undocumented R6 bindings even when `@field name NULL` is
    /// present) but is clearly marked internal in the docs.
    pub internal: bool,
    /// Suppress export for this method without adding `@keywords internal`.
    ///
    /// For R6 active bindings this emits a class-level
    /// `#' @field name (internal)` (see `internal` above for why we don't use
    /// roxygen2's `@field name NULL` opt-out).
    pub noexport: bool,
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
    /// The class-level form of the sidecar `#[r_data]` fields
    /// (`r_data_accessors`, see [`RDataAccessors`]).
    /// For R6: active bindings are added via `$set("active", ...)` after class creation.
    /// For S7: properties are spliced from `.rdata_properties_{Type}` into `new_class()`.
    /// For S3, S4 and env: `$` / `[[` field methods (`field_syntax.rs`, #1848).
    pub r_data_accessors: RDataAccessors,
    /// `s3(r_data_accessors, lookup = "list")`: how the S3 readers look a
    /// field up (see [`FieldLookup`]).
    pub field_lookup: FieldLookup,
    /// Strict conversion mode: methods returning lossy types use checked conversions.
    pub strict: bool,
    /// The impl block's `preconditions` / `no_preconditions`, inherited by
    /// every method wrapper. See [`ImplAttrs::preconditions`].
    pub preconditions: Option<bool>,
    /// Mark class as internal: adds `@keywords internal`, suppresses `@export`.
    pub internal: bool,
    /// Suppress `@export` without adding `@keywords internal`.
    pub noexport: bool,
    /// Generate the block as it reads when another impl block of the type
    /// has the constructor: the constructor this block would infer stays a
    /// static method. `expand_impl` sets it for the second rendering of a
    /// block that infers one ([`ParsedImpl::inferred_constructor`]).
    pub constructor_demoted: bool,
    /// Deprecation warnings for `@param` tags found on the impl block.
    /// Appended to the final TokenStream output.
    pub param_warnings: proc_macro2::TokenStream,
    /// Parameter names declared via `@param` in the impl-block doc comments.
    ///
    /// For R6 classes, roxygen2 8.0.0 inherits class-level `@param` tags into
    /// all methods. These are the names that appeared in impl-block-level `@param`
    /// tags (extracted before stripping). The R6 generator uses this set to suppress
    /// `(no documentation available)` placeholders for covered params and to emit
    /// class-level `@param` lines in the class header.
    pub class_param_names: std::collections::HashSet<String>,
}

/// Attributes parsed from `#[miniextendr(...)]` on an impl block.
///
/// These control which R class system to use, class naming, and
/// class-system-specific options (R6 inheritance, S7 parent, vctrs kind, etc.).
///
/// Parsed by the [`syn::parse::Parse`] implementation which handles all supported
/// attribute formats like `#[miniextendr(r6, class = "Custom")]`.
#[derive(Debug)]
pub struct ImplAttrs {
    /// Which R class system to generate wrappers for.
    /// Defaults to `Env` unless overridden by feature flags (`r6-default`, `s7-default`).
    pub class_system: ClassSystem,
    /// Optional override for the R class name. When `None`, the Rust type name is used.
    pub class_name: Option<String>,
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
    /// The class-level form of the `#[r_data]` field accessors
    /// (`r6(r_data_accessors)`, `s3(r_data_accessors = "get")`, ...).
    /// For R6: active bindings via `$set("active", ...)` post-creation.
    /// For S7: properties spliced from `.rdata_properties_{Type}`.
    /// For S3, S4 and env: `$` / `[[` field methods (#1848).
    pub r_data_accessors: RDataAccessors,
    /// How the S3 readers look a field up (`lookup = "list"`, #1921).
    pub field_lookup: FieldLookup,
    // endregion
    // region: Strict conversion mode
    /// When true, methods returning lossy types (i64/u64/isize/usize + Vec variants)
    /// use `strict::checked_*()` instead of `IntoR::into_sexp()`, panicking on overflow.
    pub strict: bool,
    // endregion
    // region: Preconditions
    /// `Some(false)` drops the R-side type-check guards from the generated
    /// method wrappers, `Some(true)` keeps them: those of an inherent impl,
    /// including the setter branch of an R6 active binding, and those of a
    /// trait impl, including the empty-body (TPIE) form. TryFromSexp still
    /// raises on bad input, with the same argument-error condition; the
    /// message comes from the conversion. Set by the bare
    /// `#[miniextendr(no_preconditions)]` / `preconditions` (the last one
    /// written wins). A method's own `preconditions` / `no_preconditions` and
    /// each parameter's spelling come first; `None` defers to the crate
    /// default, then the `no-preconditions-default` feature
    /// (`crate::r_preconditions::resolve_type_checks`).
    pub preconditions: Option<bool>,
    // endregion
    /// Mark class as internal: adds `@keywords internal`, suppresses `@export`.
    pub internal: bool,
    /// Suppress `@export` without adding `@keywords internal`.
    pub noexport: bool,
    /// When true on a trait impl (`impl Trait for Type`), the impl block is NOT
    /// emitted (a blanket impl already provides it), but C wrappers and R wrappers
    /// ARE generated from the method signatures in the body.
    pub blanket: bool,
}

/// The impl doc tags that are class settings rather than class documentation:
/// they decide whether every block's members are documented and exported, so
/// each impl block of a type repeats them ([`ParsedImpl::class_settings`]).
const CLASS_SETTING_TAGS: [&str; 2] = ["noRd", "keywords internal"];

/// Every impl-block option, for the "unknown impl block option" errors of the
/// bare and the `key = value` forms.
const IMPL_OPTIONS_HELP: &str = "env, env(...), r6, r6(...), s3, s3(...), s4, s4(...), s7, \
     s7(...), vctrs, vctrs(...) (class system), class = \"...\" (R class name), \
     blanket, strict, no_strict, preconditions, no_preconditions, internal, noexport";

impl syn::parse::Parse for ImplAttrs {
    /// Parses `#[miniextendr(...)]` impl-level options.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut class_system = if cfg!(feature = "r6-default") {
            ClassSystem::R6
        } else if cfg!(feature = "s7-default") {
            ClassSystem::S7
        } else {
            ClassSystem::Env
        };
        let mut class_system_span: Option<(&str, proc_macro2::Span)> = None;
        let mut class_name = None;
        let mut vctrs_attrs = VctrsAttrs::default();
        let mut r6_inherit = None;
        let mut r6_portable = None;
        let mut r6_cloneable = None;
        let mut r6_lock_objects = None;
        let mut r6_lock_class = None;
        let mut s7_parent = None;
        let mut s7_abstract = false;
        let mut r_data_accessors = RDataAccessors::Off;
        let mut field_lookup: Option<(FieldLookup, proc_macro2::Span)> = None;
        let mut strict: Option<bool> = None;
        let mut preconditions: Option<bool> = None;
        let mut internal = false;
        let mut noexport = false;
        let mut blanket = false;

        // Parse attributes. The first identifier can be either:
        // - A class system (env, r6, s3, s4, s7, vctrs)
        // - A key in a key=value pair (class)
        //
        // Valid formats:
        // - #[miniextendr]
        // - #[miniextendr(r6)]
        // - #[miniextendr(r6, class = "CustomName")]
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
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!(
                                "unknown impl block option `{ident_str}`; expected one of: \
                                 {IMPL_OPTIONS_HELP}"
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
                        if key == "r_data_accessors" {
                            return Err(syn::Error::new(
                                key.span(),
                                "vctrs classes have no `r_data_accessors`: a vctr is an R \
                                 vector with no external pointer, and vctrs keeps `$` on a \
                                 record for its own fields. Use the standalone \
                                 `Type_get_<field>()` / `Type_set_<field>()`",
                            ));
                        }
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
                                r_data_accessors =
                                    parse_r_data_accessors_value(&content, &key, ClassSystem::R6)?;
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
                                r_data_accessors =
                                    parse_r_data_accessors_value(&content, &key, ClassSystem::S7)?;
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
            } else if ident_str == "preconditions" {
                preconditions = Some(true);
            } else if ident_str == "no_preconditions" {
                preconditions = Some(false);
            } else if ident_str == "internal" {
                internal = true;
            } else if ident_str == "noexport" {
                noexport = true;
            } else {
                // A class system identifier, or no option at all.
                let parsed_system: ClassSystem = ident_str.parse().map_err(|_| {
                    syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown impl block option `{ident_str}`; expected one of: \
                             {IMPL_OPTIONS_HELP}"
                        ),
                    )
                })?;
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

                // `s3(...)`, `s4(...)`, `env(...)`: the field syntax option.
                if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);
                    while !content.is_empty() {
                        let key: syn::Ident = content.parse()?;
                        if key == "lookup" && parsed_system == ClassSystem::S3 {
                            if field_lookup.is_some() {
                                return Err(syn::Error::new(key.span(), "`lookup` is given twice"));
                            }
                            let _: syn::Token![=] = content.parse()?;
                            let value: syn::LitStr = content.parse()?;
                            let lookup = match value.value().as_str() {
                                "strict" => FieldLookup::Strict,
                                "list" => FieldLookup::List,
                                other => {
                                    return Err(syn::Error::new(
                                        value.span(),
                                        format!(
                                            "unknown `lookup` value \"{other}\"; expected \
                                             \"strict\" (exact names and whole-number \
                                             positions, the default) or \"list\" (an unknown \
                                             name or `NA` reads `NULL`)"
                                        ),
                                    ));
                                }
                            };
                            field_lookup = Some((lookup, key.span()));
                        } else if key == "r_data_accessors" {
                            r_data_accessors =
                                parse_r_data_accessors_value(&content, &key, parsed_system)?;
                        } else {
                            let lookup = if parsed_system == ClassSystem::S3 {
                                ", lookup = \"list\""
                            } else {
                                ""
                            };
                            return Err(syn::Error::new(
                                key.span(),
                                format!(
                                    "unknown {ident_str} option: {key} (expected \
                                     r_data_accessors, or r_data_accessors = \"get\" for \
                                     getters only{lookup})"
                                ),
                            ));
                        }
                        if content.peek(syn::Token![,]) {
                            let _: syn::Token![,] = content.parse()?;
                        }
                    }
                    if let Some((_, span)) = field_lookup
                        && !r_data_accessors.enabled()
                    {
                        return Err(syn::Error::new(
                            span,
                            "`lookup` sets how the readers of `s3(r_data_accessors)` look a \
                             field up; add `r_data_accessors` (or `r_data_accessors = \"get\"`)",
                        ));
                    }
                }
            }

            // Consume trailing comma if present
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(ImplAttrs {
            class_system,
            class_name,
            vctrs_attrs,
            r6_inherit,
            r6_portable,
            r6_cloneable,
            r6_lock_objects,
            r6_lock_class,
            s7_parent,
            s7_abstract,
            r_data_accessors,
            field_lookup: field_lookup.map(|(lookup, _)| lookup).unwrap_or_default(),
            strict: strict.unwrap_or(cfg!(feature = "strict-default")),
            preconditions,
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
        if (attrs.receiver.is_some() || attrs.no_dispatch_dots)
            && (!matches!(class_system, ClassSystem::S3 | ClassSystem::S4)
                || attrs.generic.is_none())
        {
            return Err(syn::Error::new(
                span,
                "`receiver` and `no_dots` require an explicit S3/S4 `generic = \"...\"`",
            ));
        }

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

        // postfix derives the R name from the Rust name; it cannot combine
        // with another naming source.
        if attrs.postfix.is_some() && attrs.r_name.is_some() {
            return Err(syn::Error::new(
                span,
                "`postfix` and `r_name` both set the R method name; use one of them.",
            ));
        }
        if attrs.postfix.is_some() && attrs.generic.is_some() {
            return Err(syn::Error::new(
                span,
                "`postfix` and `generic` cannot be used on the same method. \
                 Generic dispatch names the R method after the generic.",
            ));
        }

        if attrs.serialize && (attrs.unwrap_in_r || attrs.serde_error.is_some()) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`serialize` cannot be combined with `unwrap_in_r` or `serde_error`: it serializes the complete return value, including any Result variant",
            ));
        }

        // serde_error classes the raised condition; unwrap_in_r never raises.
        if attrs.serde_error.is_some() && attrs.unwrap_in_r {
            return Err(syn::Error::new(
                span,
                "`serde_error` cannot be used with `unwrap_in_r`: `unwrap_in_r` returns the \
                 `Result` to R as a value, so there is no raised condition to class.",
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

        // Static methods can select worker dispatch. Receiver and dots borrows
        // keep their wrappers on the main thread (see generate_method_c_wrapper).

        Ok(())
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
                        } else if inner.path.is_ident("invisible") {
                            method_attrs.force_invisible = Some(true);
                        } else if inner.path.is_ident("visible") {
                            method_attrs.force_invisible = Some(false);
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
                            method_attrs.unwrap_in_r = true;
                        } else if inner.path.is_ident("wrap") {
                            let value: syn::LitStr = inner.value()?.parse()?;
                            method_attrs.wrap = Some(crate::return_wrap::parse_system(&value)?);
                        } else if inner.path.is_ident("serialize") {
                            method_attrs.serialize = true;
                        } else if inner.path.is_ident("serde_error") {
                            method_attrs.serde_error =
                                Some(crate::miniextendr_fn::parse_serde_error_nested(&inner)?);
                        } else if inner.path.is_ident("generic") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.generic = Some(value.value());
                        } else if inner.path.is_ident("receiver") {
                            let value: syn::LitStr = inner.value()?.parse()?;
                            let name = crate::naming::parse_r_receiver(&value)?;
                            if method_attrs.receiver.replace(name).is_some() {
                                return Err(inner.error("receiver may be specified only once"));
                            }
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
                            if meta.path.is_ident("s3") || meta.path.is_ident("s4") {
                                method_attrs.no_dispatch_dots = true;
                            } else {
                                method_attrs.s7.no_dots = true;
                            }
                        } else if inner.path.is_ident("dispatch") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7.dispatch = Some(value.value());
                        } else if inner.path.is_ident("fallback") {
                            method_attrs.s7.fallback = true;
                        } else if inner.path.is_ident("no_shortcut") {
                            method_attrs.s7.no_shortcut = true;
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
                        } else if inner.path.is_ident("postfix") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            let val = value.value();
                            crate::miniextendr_fn::validate_postfix(&val, &value)?;
                            method_attrs.postfix = Some(val);
                        } else if inner.path.is_ident("call") {
                            return Err(inner.error(
                                "`call = caller` is only supported on standalone `#[miniextendr]` \
                                 functions (the internal-entry-point pattern); class methods keep \
                                 their own call attribution",
                            ));
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
                                "unknown method option; expected one of: ignore, constructor, finalize, private, active, worker, no_worker, main_thread, no_main_thread, check_interrupt, coerce, no_coerce, rng, unwrap_in_r, serialize, serde_error, generic, class, getter, setter, validate, prop, default, required, frozen, deprecated, no_dots, dispatch, fallback, no_shortcut, convert_from, convert_to, deep_clone, r_on_exit, r_name, postfix, invisible, visible"
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
                } else if meta.path.is_ident("preconditions")
                    || meta.path.is_ident("no_preconditions")
                {
                    // Bare: the whole method keeps / drops its type-derived
                    // R-side checks. `preconditions(p, q)` /
                    // `no_preconditions(p)`: those parameters. Last one wins.
                    let keep = meta.path.is_ident("preconditions");
                    if meta.input.peek(syn::token::Paren) {
                        method_attrs.match_arg_span.get_or_insert(meta.path.span());
                        crate::miniextendr_fn::parse_method_preconditions(
                            &meta,
                            &mut method_attrs.per_param,
                            keep,
                        )?;
                    } else {
                        method_attrs.preconditions = Some(keep);
                    }
                } else if meta.path.is_ident("no_na") {
                    // `no_na(p, q(message = "..."))` — R-side `!anyNA(p)` checks.
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    crate::miniextendr_fn::parse_method_no_na(&meta, &mut method_attrs.per_param)?;
                } else if let Some(check) = crate::miniextendr_fn::ClassCheck::of(&meta.path) {
                    // `inherits(p = "cls_a, cls_b", q(class = "cls", message = "..."))` —
                    // R-side `inherits(p, c(...))` checks; `not_inherits(...)` the
                    // same shape, `!inherits(p, c(...))`.
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    crate::miniextendr_fn::parse_method_class_check(
                        &meta,
                        &mut method_attrs.per_param,
                        check,
                    )?;
                } else if meta.path.is_ident("match_arg") {
                    // `match_arg(param1, param2, ...)` — scalar match_arg params.
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    meta.parse_nested_meta(|inner| {
                        let name = crate::miniextendr_fn::method_check_param(&inner)?;
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
                        let name = crate::miniextendr_fn::method_check_param(&inner)?;
                        let entry = method_attrs.per_param.entry(name).or_default();
                        entry.match_arg = true;
                        entry.several_ok = true;
                        Ok(())
                    })?;
                } else if meta.path.is_ident("no_default") {
                    // `no_default(param1, param2, ...)` — choice params whose R
                    // formal has no default (#1828).
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    crate::miniextendr_fn::parse_method_no_default(
                        &meta,
                        &mut method_attrs.per_param,
                    )?;
                } else if meta.path.is_ident("choices") {
                    // `choices(param = "a, b, c", param2 = "x, y")` — explicit string choice lists.
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    meta.parse_nested_meta(|inner| {
                        let name = crate::miniextendr_fn::method_check_param(&inner)?;
                        let _: syn::Token![=] = inner.input.parse()?;
                        let value: syn::LitStr = inner.input.parse()?;
                        let choices = crate::r_wrapper_builder::split_choice_list(&value.value());
                        method_attrs.per_param.entry(name).or_default().choices = Some(choices);
                        Ok(())
                    })?;
                } else if meta.path.is_ident("choices_several_ok") {
                    // `choices_several_ok(param = "a, b, c")` — choices + several_ok.
                    method_attrs.match_arg_span.get_or_insert(meta.path.span());
                    meta.parse_nested_meta(|inner| {
                        let name = crate::miniextendr_fn::method_check_param(&inner)?;
                        let _: syn::Token![=] = inner.input.parse()?;
                        let value: syn::LitStr = inner.input.parse()?;
                        let choices = crate::r_wrapper_builder::split_choice_list(&value.value());
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
                    method_attrs.unwrap_in_r = true;
                } else if meta.path.is_ident("wrap") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    method_attrs.wrap = Some(crate::return_wrap::parse_system(&value)?);
                } else if meta.path.is_ident("serialize") {
                    method_attrs.serialize = true;
                } else if meta.path.is_ident("serde_error") {
                    method_attrs.serde_error =
                        Some(crate::miniextendr_fn::parse_serde_error_nested(&meta)?);
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
                } else if meta.path.is_ident("postfix") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let value: syn::LitStr = meta.input.parse()?;
                    let val = value.value();
                    crate::miniextendr_fn::validate_postfix(&val, &value)?;
                    method_attrs.postfix = Some(val);
                } else if meta.path.is_ident("call") {
                    return Err(meta.error(
                        "`call = caller` is only supported on standalone `#[miniextendr]` \
                         functions (the internal-entry-point pattern); class methods keep \
                         their own call attribution",
                    ));
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
                } else if meta.path.is_ident("noexport") {
                    method_attrs.noexport = true;
                } else if meta.path.is_ident("internal") {
                    method_attrs.internal = true;
                } else if meta.path.is_ident("invisible") {
                    method_attrs.force_invisible = Some(true);
                } else if meta.path.is_ident("visible") {
                    method_attrs.force_invisible = Some(false);
                } else if meta.path.is_ident("dots") {
                    // `dots = typed_list!(...)` — attribute sugar mirroring the
                    // standalone-fn path (`miniextendr_fn.rs`): capture the
                    // `typed_list!(...)` spec so a `dots_typed` binding can be
                    // injected at the top of the method body in `from_impl_item`.
                    let _: syn::Token![=] = meta.input.parse()?;
                    let mac: syn::Macro = meta.input.parse()?;
                    if !mac.path.is_ident("typed_list") {
                        return Err(syn::Error::new_spanned(
                            &mac.path,
                            "dots expects `typed_list!(...)` macro",
                        ));
                    }
                    method_attrs.dots_spec = Some(quote::quote!(#mac));
                } else {
                    return Err(meta.error(
                        "unknown attribute; expected one of: env, r6, s3, s4, s7, vctrs, defaults, unsafe, check_interrupt, coerce, no_coerce, preconditions, no_preconditions, rng, unwrap_in_r, serialize, serde_error, as, lifecycle, r_name, postfix, r_entry, r_post_checks, r_on_exit, noexport, internal, invisible, visible, match_arg, match_arg_several_ok, choices, choices_several_ok, no_default, inherits, not_inherits, no_na, dots = typed_list!(...)"
                    ));
                }
                Ok(())
            })?;
        }

        // Resolve feature defaults for fields not explicitly set
        method_attrs.worker = worker.unwrap_or(cfg!(feature = "worker-default"));
        // Main-thread execution is already the fallback when worker is false.
        // Only an explicit override may take precedence over worker selection.
        method_attrs.unsafe_main_thread = unsafe_main_thread.unwrap_or(false);
        method_attrs.coerce = coerce.unwrap_or(cfg!(feature = "coerce-default"));

        // Method-level `internal` / `noexport` are currently only honoured by the R6
        // active-binding doc path (emits a class-level
        // `#' @field <name> (internal)` — the documented roxygen2 8.0.0
        // `@field name NULL` opt-out doesn't actually silence
        // `r6_resolve_fields`'s "Undocumented R6 active binding" warning, so we
        // use a minimal real description instead). Accepting them silently
        // elsewhere would be a no-op surface: regular instance methods, static
        // methods, and trait methods don't currently consume these flags. Reject
        // them at parse time so the failure is loud rather than silent.
        // Class-level `#[miniextendr(internal)]` / `noexport` continue to work for whole
        // impl blocks via `parsed_impl.internal` / `parsed_impl.noexport`.
        if (method_attrs.internal || method_attrs.noexport) && !method_attrs.r6.active {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "method-level `internal` / `noexport` are currently only supported on R6 \
                 active bindings (use `#[miniextendr(r6(active), noexport)]`). For other \
                 method positions, set `internal` / `noexport` at the impl-block level \
                 (`#[miniextendr(s7, internal)] impl Foo { ... }`) instead.",
            ));
        }

        Ok(method_attrs)
    }

    /// Detect the [`ReceiverKind`] from a method's function signature.
    ///
    /// Inspects the first parameter to determine whether this is a static function
    /// (`None`), immutable borrow (`Ref`), mutable borrow (`RefMut`), consuming
    /// method (`Value`), or handle receiver (`ExternalPtrRef`, `ExternalPtrMut`,
    /// `ExternalPtrValue`). Handles both standard receivers (`&self`, `&mut self`) and
    /// typed receivers (`self: &Self`, `self: ExternalPtr<Self>`, etc.).
    fn detect_env(sig: &syn::Signature) -> ReceiverKind {
        match crate::trait_receiver::owner_receiver(sig) {
            Some(crate::trait_receiver::OwnerReceiver::Shared) => {
                return ReceiverKind::ExternalPtrRef;
            }
            Some(crate::trait_receiver::OwnerReceiver::Mutable) => {
                return ReceiverKind::ExternalPtrMut;
            }
            None => {}
        }
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
                    // Check for typed receiver (self: &Self, self: &mut Self;
                    // `old_handle_receiver` refuses `self: &ExternalPtr<Self>`)
                    if let syn::Type::Reference(type_ref) = r.ty.as_ref() {
                        if type_ref.mutability.is_some() {
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
    /// Regular doc comments are auto-converted to `@description` for all class
    /// systems. `class_page` is whether the class block renders a help page:
    /// the method's docs go nowhere else when it does not.
    pub fn from_impl_item(
        item: &mut syn::ImplItemFn,
        class_page: crate::roxygen::RdPage,
    ) -> syn::Result<Self> {
        use syn::spanned::Spanned;
        old_handle_receiver(&item.sig)?;
        let env = Self::detect_env(&item.sig);
        let mut method_attrs = Self::parse_method_attrs(&item.attrs)?;

        // The `&Dots` parameter is R's `...` wherever it sits (Rust `...`
        // becomes a trailing one); at most one per method.
        crate::miniextendr_fn::rewrite_variadic_dots(&mut item.sig)?;
        let dots_ident = crate::miniextendr_fn::find_dots_param(&item.sig.inputs)?;
        let has_dots = dots_ident.is_some();

        // Parameter markers (`Checked<T>` / `Unchecked<T>`, #1566): peeled
        // from the signature the codegen sees (the re-emitted impl keeps
        // them), and merged with the method's `preconditions(p)` /
        // `no_preconditions(p)`, which a marker must not contradict.
        let param_markers = crate::miniextendr_fn::collect_param_markers(&item.sig.inputs)?;
        for arg in &item.sig.inputs {
            let syn::FnArg::Typed(pt) = arg else {
                continue;
            };
            let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
                continue;
            };
            let name = crate::naming::ident_name(&pat_ident.ident);
            let Some((_, markers)) = param_markers.iter().find(|(n, _)| *n == name) else {
                continue;
            };
            let entry = method_attrs.per_param.entry(name.clone()).or_default();
            entry.preconditions = crate::miniextendr_fn::merge_param_policy(
                markers,
                entry.preconditions,
                &name,
                true,
                &pt.ty,
            )?;
        }
        let mut peeled_sig = item.sig.clone();
        peeled_sig.inputs = crate::miniextendr_fn::peeled_inputs(&item.sig.inputs, &param_markers)?;

        // `dots = typed_list!(...)` sugar: inject the `dots_typed` binding at the
        // top of the method body, reusing the shared helper that the standalone-fn
        // path uses. This mutates `item.block`, which is the same node re-emitted
        // as `original_impl` (see `ParsedImpl::parse`), so it covers all six class
        // systems by construction (they emit only R/C wrappers, not the Rust body).
        if let Some(spec) = &method_attrs.dots_spec {
            let dots_ident = dots_ident.as_ref().ok_or_else(|| {
                syn::Error::new(
                    item.sig.ident.span(),
                    "dots = typed_list!(...) requires the method to take a dots parameter \
                     (`...` or a `&Dots` parameter)",
                )
            })?;
            if let Some(err) =
                crate::miniextendr_fn::typed_list_on_lazy_dots(&item.sig.inputs, None)
            {
                return Err(err);
            }
            let stmt = crate::build_dots_validation_stmt(dots_ident, spec);
            item.block.stmts.insert(0, stmt);
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
                    Some(crate::naming::ident_name(&pat_ident.ident))
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
        for input in &peeled_sig.inputs {
            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };
            let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
                continue;
            };
            let param_name = crate::naming::ident_name(&pat_ident.ident);

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
        let mut doc_tags =
            crate::roxygen::roxygen_tags_from_attrs_for_r6_method(&item.attrs, class_page);

        // Inject lifecycle badge into method roxygen tags if present
        if let Some(ref spec) = method_attrs.lifecycle {
            crate::lifecycle::inject_lifecycle_badge(&mut doc_tags, spec);
        }

        // Get parameter defaults from method-level #[miniextendr(defaults(...))] attribute
        let param_defaults = method_attrs.defaults.clone();

        // match_arg / choices / inherits / not_inherits / no_na on impl methods: unlike standalone
        // functions, Rust doesn't accept `#[miniextendr(...)]` on method parameters
        // inside an impl (attribute macros aren't allowed there — "expected
        // non-macro attribute"). The surface is instead method-level:
        // `#[miniextendr(match_arg(p), choices(q = "a, b"))]`. Every named param
        // must exist on the signature, so typos fail at compile time; `Option<T>`
        // scalar match_arg/choices params are the optional form (#1473);
        // `Missing<T>` ones and defaults on `Option<T>` are rejected.
        crate::miniextendr_fn::finalize_method_param_attrs(
            &mut method_attrs.per_param,
            &peeled_sig.inputs,
            &param_markers,
            &param_defaults,
            method_attrs
                .match_arg_span
                .unwrap_or_else(|| item.sig.ident.span()),
        )?;

        // Validate: Missing<T> parameters must not have defaults
        for arg in peeled_sig.inputs.iter() {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let name = crate::naming::ident_name(&pat_ident.ident);
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

        // Consuming `self` receivers (#1432): supported for every return shape
        // (see `ReceiverKind::Value`). Two things stay rejected: a receiver type
        // the wrapper cannot hand over (`self: Box<Self>` / `Rc<Self>` ...), and
        // the `constructor` marker, which describes receiverless `fn new()`.
        if env == ReceiverKind::Value {
            if let Some(syn::FnArg::Receiver(r)) = item.sig.inputs.first()
                && r.colon_token.is_some()
                && !matches!(r.ty.as_ref(), syn::Type::Path(tp)
                    if tp.path.segments.last().is_some_and(|seg| seg.ident == "Self")
                        && tp.path.segments.len() == 1)
            {
                return Err(syn::Error::new_spanned(
                    &r.ty,
                    "unsupported receiver type: the R handle stores the value itself, so a \
                     method can take `self`, `self: Self`, `&self`, `&mut self`, \
                     `self: ExternalPtrRef<'_, Self>`, `self: ExternalPtrMut<'_, Self>` or \
                     `self: ExternalPtr<Self>` — not a smart pointer around `Self`",
                ));
            }
            if method_attrs.constructor {
                return Err(syn::Error::new(
                    item.sig.fn_token.span,
                    format!(
                        "method `{}` takes `self` and is marked `constructor`; a constructor has no \
                         receiver (`fn new(...) -> Self`). A consuming builder step needs no marker: \
                         `fn {}(self, ...) -> Self` writes its result back into the same R object.",
                        item.sig.ident, item.sig.ident
                    ),
                ));
            }
        }

        // Return-visibility marker (#1213): validate, record, and strip it from
        // the signature the codegen sees (whose parameters are already peeled).
        let mut sig = Self::sig_without_env(&peeled_sig);
        let visibility_marker = match &item.sig.output {
            syn::ReturnType::Type(_, ty) => {
                if let Some(err) = crate::type_inspect::visibility_marker_error(ty, "return") {
                    return Err(err);
                }
                let (marker, peeled) =
                    crate::type_inspect::peel_return_visibility(&item.sig.output);
                sig.output = peeled;
                marker
            }
            syn::ReturnType::Default => None,
        };
        if let (Some(marker), Some(attr)) = (visibility_marker, method_attrs.force_invisible)
            && marker != attr
        {
            return Err(syn::Error::new_spanned(
                &item.sig.output,
                "the return type's visibility marker and the `invisible` / `visible` method option disagree; keep one of them (or make them agree)",
            ));
        }

        let (return_wrap, output) = crate::return_wrap::resolve(&sig.output, method_attrs.wrap)?;
        if return_wrap.is_some() && (method_attrs.serialize || method_attrs.unwrap_in_r) {
            return Err(syn::Error::new_spanned(
                &item.sig.output,
                "explicit class wrapping cannot be combined with serialize or unwrap_in_r",
            ));
        }
        sig.output = crate::type_inspect::serialize_return_type(&output, method_attrs.serialize);
        Ok(ParsedMethod {
            visibility_marker,
            return_wrap,
            ident: item.sig.ident.clone(),
            env,
            sig,
            param_markers,
            vis: item.vis.clone(),
            doc_tags,
            method_attrs,
            param_defaults,
            has_dots,
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

    /// Returns true if this method is the R6 finalizer.
    ///
    /// Only the explicit `#[miniextendr(r6(finalize))]` marker qualifies. It
    /// used to be inferred from "takes `self` by value and does not return
    /// `Self`", which silently hid consuming methods on every class system
    /// (#1432).
    pub fn is_finalizer(&self) -> bool {
        self.method_attrs.r6.finalize
    }

    /// Returns true if this method should be an R6 active binding.
    /// Active bindings provide property-like access (obj$name instead of obj$name()).
    pub fn is_active(&self) -> bool {
        self.method_attrs.r6.active
    }

    /// R-facing method name.
    ///
    /// Returns `r_name` if set, otherwise the Rust ident with `postfix`
    /// appended when given, otherwise the Rust ident as a string.
    pub fn r_method_name(&self) -> String {
        if let Some(r_name) = &self.method_attrs.r_name {
            return r_name.clone();
        }
        match &self.method_attrs.postfix {
            Some(postfix) => format!("{}{postfix}", crate::naming::ident_name(&self.ident)),
            None => crate::naming::ident_name(&self.ident),
        }
    }

    /// C wrapper identifier for this method.
    ///
    /// Format: `C_{crate}_{Type}__{method}` — crate-prefixed for webR
    /// cross-package symbol uniqueness (#1273). Unique per type: two impl
    /// blocks of one type can't both define a method (E0592).
    pub fn c_wrapper_ident(&self, type_ident: &syn::Ident) -> syn::Ident {
        crate::naming::impl_method_c_wrapper_ident(type_ident, &self.ident)
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

    /// Returns true if this method returns `Result<Self, E>` — a fallible
    /// constructor-shaped method (e.g. `from_r`, `try_new`). On the R side this
    /// is treated exactly like a bare `Self` return (wrapped class object via
    /// [`crate::ReturnStrategy::for_method`]); the C wrapper still raises on
    /// `Err` via the normal `Result` error path (see
    /// [`crate::c_wrapper_builder::ReturnHandling::ResultExternalPtr`]).
    pub fn returns_result_self(&self) -> bool {
        let syn::ReturnType::Type(_, ty) = &self.sig.output else {
            return false;
        };
        let syn::Type::Path(p) = ty.as_ref() else {
            return false;
        };
        let Some(seg) = p.path.segments.last() else {
            return false;
        };
        if seg.ident != "Result" {
            return false;
        }
        let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
            return false;
        };
        let Some(syn::GenericArgument::Type(ok_ty)) = ab.args.first() else {
            return false;
        };
        matches!(ok_ty, syn::Type::Path(ip)
            if ip.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false))
    }

    /// Returns true if this method returns `Option<Self>` — a lookup-shaped
    /// fallible constructor (e.g. `try_find`). On the R side this is treated
    /// exactly like a bare `Self` return (wrapped class object via
    /// [`crate::ReturnStrategy::for_method`]); the C wrapper still raises on
    /// `None` via the normal `Option` error path (see
    /// [`crate::c_wrapper_builder::ReturnHandling::OptionExternalPtr`]).
    /// Symmetric with [`Self::returns_result_self`].
    pub fn returns_option_self(&self) -> bool {
        let syn::ReturnType::Type(_, ty) = &self.sig.output else {
            return false;
        };
        let syn::Type::Path(p) = ty.as_ref() else {
            return false;
        };
        let Some(seg) = p.path.segments.last() else {
            return false;
        };
        if seg.ident != "Option" {
            return false;
        }
        let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
            return false;
        };
        let Some(syn::GenericArgument::Type(some_ty)) = ab.args.first() else {
            return false;
        };
        matches!(some_ty, syn::Type::Path(ip)
            if ip.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false))
    }

    /// Returns the bare type name when this method returns a type that may be a
    /// different registered ExternalPtr-backed class.
    ///
    /// This is deliberately syntactic and conservative about primitives and
    /// common containers. The write-time wrapper resolver checks the complete
    /// class registry, so an unregistered capitalized type falls back to the
    /// direct `.val` return.
    ///
    /// Four shapes are recognized:
    /// - A bare capitalized path, e.g. `-> Board`.
    /// - `Option<T>` where `T` is a bare capitalized path. The C wrapper
    ///   already unwraps this and raises on `None`
    ///   (`ReturnHandling::OptionIntoRUnwrap`), so the successful `.val` is a
    ///   bare pointer — identical to the bare-class case on the R side.
    /// - `Result<T, E>` where `T` is a bare capitalized path and `E` is not
    ///   `()`. `Result<T, ()>` is excluded: the unit-error sentinel maps to
    ///   `ReturnHandling::ResultNullOnErr`, where `.val` can be `NULL` on
    ///   `Ok`, and wrapping `NULL` in a class constructor would break.
    /// - `ExternalPtr<T>` (the explicit-handle spelling, #1375), including
    ///   through the `Option`/`Result` wrappers above (e.g.
    ///   `Option<ExternalPtr<Board>>`) — `IntoR for ExternalPtr<T>` already
    ///   produces a bare `EXTPTRSXP` `.val`, identical in shape to the bare
    ///   `T` case, so [`inner_class_ident`](Self::inner_class_ident) peels
    ///   through it to the same wrapped name.
    ///
    /// `Option<Self>` / `Result<Self, _>` are excluded here too (the inner
    /// ident is `Self`) — `ReturnStrategy::for_method` checks
    /// `returns_result_self()` / `returns_option_self()` before this method,
    /// so those already take the `ReturnSelf` path regardless.
    /// List-shaped returns (`Vec<Class>` and its `Option`/`Result` wrappers)
    /// are handled separately by
    /// [`returns_other_class_list`](Self::returns_other_class_list); any other
    /// nested container is not recognized: the inner type must be a bare path
    /// (or an `ExternalPtr<...>` wrapping one) with no other path arguments.
    pub fn returns_other_class(&self) -> Option<syn::Ident> {
        let syn::ReturnType::Type(_, ty) = &self.sig.output else {
            return None;
        };
        let syn::Type::Path(p) = ty.as_ref() else {
            return None;
        };
        let seg = p.path.segments.last()?;

        match &seg.arguments {
            syn::PathArguments::None => Self::class_ident_from_ident(&seg.ident),
            syn::PathArguments::AngleBracketed(_) if seg.ident == "Option" => {
                Self::inner_class_ident(crate::first_type_argument(seg)?)
            }
            syn::PathArguments::AngleBracketed(_) if seg.ident == "Result" => {
                // `Result<T, ()>` is a deliberate NULL-on-Err sentinel
                // (`ResultNullOnErr`) rather than a fallible constructor —
                // `.val` may be `NULL`, which a class constructor can't wrap.
                let err_is_unit = crate::second_type_argument(seg)
                    .is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
                if err_is_unit {
                    return None;
                }
                Self::inner_class_ident(crate::first_type_argument(seg)?)
            }
            // Bare explicit-handle return (#1375): `-> ExternalPtr<Class>`
            // wraps identically to `-> Class` — delegate to
            // `inner_class_ident`, which peels the `ExternalPtr` segment.
            syn::PathArguments::AngleBracketed(_) if seg.ident == "ExternalPtr" => {
                Self::inner_class_ident(ty.as_ref())
            }
            _ => None,
        }
    }

    /// Returns the bare element type name when this method returns a *list* of
    /// values that may be a different registered ExternalPtr-backed class.
    ///
    /// The list-shaped sibling of
    /// [`returns_other_class`](Self::returns_other_class) (#1284). Like the
    /// scalar detector it is deliberately syntactic: the write-time resolver
    /// checks the complete class registry, and an unregistered element type
    /// falls back to the bare list of external pointers.
    ///
    /// Four shapes are recognized (the element type must pass the same bare
    /// capitalized-path filter as the scalar case, optionally through a
    /// trailing `ExternalPtr<...>` segment):
    /// - `Vec<Class>` — the C wrapper converts via `IntoR` into a `VECSXP` of
    ///   external pointers (the `IntoRVecElement` impl emitted by
    ///   `#[derive(ExternalPtr)]`), so `.val` is a bare list.
    /// - `Option<Vec<Class>>` — the C wrapper already unwraps and raises on
    ///   `None` (`ReturnHandling::OptionIntoRUnwrap`), so the successful
    ///   `.val` is the same bare list.
    /// - `Result<Vec<Class>, E>` where `E` is not `()` — the C wrapper raises
    ///   on `Err` (`ReturnHandling::ResultIntoR`); same bare list on `Ok`.
    ///   `Result<Vec<Class>, ()>` is excluded: the unit-error sentinel maps to
    ///   `ReturnHandling::ResultNullOnErr`, where `.val` can be `NULL`, and
    ///   `lapply(NULL, ...)` would silently turn that `NULL` into `list()`.
    /// - `Vec<ExternalPtr<Class>>` (the explicit-handle spelling, #1375),
    ///   including through the `Option`/`Result` wrappers above — the element
    ///   filter is [`inner_class_ident`](Self::inner_class_ident), which peels
    ///   the `ExternalPtr` segment, so this resolves identically to
    ///   `Vec<Class>`.
    ///
    /// Not recognized (deliberately):
    /// - `Vec<Self>` — `ParsedMethod` doesn't carry the enclosing impl's type
    ///   ident, so this detector can't name the target class. Handled instead
    ///   by [`returns_self_list`](Self::returns_self_list), a boolean sibling
    ///   that `ReturnStrategy::for_method` treats the same as this method
    ///   returning `Some`; `MethodReturnBuilder::with_return_class_from_method`
    ///   substitutes the actual name using the `type_ident` every call site
    ///   already has in scope (#1375).
    /// - `Vec<Option<Class>>` — has no `IntoR` impl (the `Vec<Option<T>>`
    ///   blanket slot is occupied by the newtype funnel) and would need a
    ///   NULL-tolerant per-element wrap; tracked as a follow-up.
    pub fn returns_other_class_list(&self) -> Option<syn::Ident> {
        let syn::ReturnType::Type(_, ty) = &self.sig.output else {
            return None;
        };
        let syn::Type::Path(p) = ty.as_ref() else {
            return None;
        };
        let seg = p.path.segments.last()?;

        match &seg.arguments {
            syn::PathArguments::AngleBracketed(_) if seg.ident == "Vec" => {
                Self::vec_inner_class_ident(ty)
            }
            syn::PathArguments::AngleBracketed(_) if seg.ident == "Option" => {
                Self::vec_inner_class_ident(crate::first_type_argument(seg)?)
            }
            syn::PathArguments::AngleBracketed(_) if seg.ident == "Result" => {
                // `Result<T, ()>` maps to `ResultNullOnErr` — `.val` may be
                // `NULL`, which `lapply` would silently coerce to `list()`.
                let err_is_unit = crate::second_type_argument(seg)
                    .is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
                if err_is_unit {
                    return None;
                }
                Self::vec_inner_class_ident(crate::first_type_argument(seg)?)
            }
            _ => None,
        }
    }

    /// Applies [`inner_class_ident`](Self::inner_class_ident) to the element
    /// type of a `Vec<T>`: `ty` must be a `Vec` path whose single type
    /// argument is either a bare `syn::Type::Path` or an `ExternalPtr<...>`
    /// wrapping one (peeled by `inner_class_ident`, #1375).
    fn vec_inner_class_ident(ty: &syn::Type) -> Option<syn::Ident> {
        let syn::Type::Path(p) = ty else {
            return None;
        };
        let seg = p.path.segments.last()?;
        if seg.ident != "Vec" {
            return None;
        }
        Self::inner_class_ident(crate::first_type_argument(seg)?)
    }

    /// Shared name filter for [`returns_other_class`](Self::returns_other_class):
    /// rejects `Self`, builtin scalar/string names, and known container names;
    /// accepts only idents starting with an ASCII-uppercase letter.
    fn class_ident_from_ident(ident: &syn::Ident) -> Option<syn::Ident> {
        let name = ident.to_string();
        if name == "Self"
            || is_builtin_return_type_name(&name)
            || is_known_return_container_name(&name)
        {
            return None;
        }

        name.chars()
            .next()
            .is_some_and(|c| c.is_ascii_uppercase())
            .then(|| ident.clone())
    }

    /// Applies [`class_ident_from_ident`](Self::class_ident_from_ident) to a
    /// container's inner type argument (e.g. the `T` in `Option<T>`).
    ///
    /// Two shapes qualify:
    /// - A bare `syn::Type::Path` with no path arguments of its own — this is
    ///   what excludes nested containers like `Option<Vec<T>>`, whose inner
    ///   `Vec<T>` carries `PathArguments::AngleBracketed`.
    /// - `ExternalPtr<T>` (#1375): the explicit-handle spelling wraps
    ///   identically to a bare `T` at the value level (`IntoR for
    ///   ExternalPtr<T>` already produces a bare `EXTPTRSXP`), so the handle
    ///   is peeled with the same helper the explicit `wrap` path uses.
    fn inner_class_ident(ty: &syn::Type) -> Option<syn::Ident> {
        let syn::Type::Path(p) = crate::type_inspect::peel_external_ptr(ty) else {
            return None;
        };
        let seg = p.path.segments.last()?;
        match &seg.arguments {
            syn::PathArguments::None => Self::class_ident_from_ident(&seg.ident),
            _ => None,
        }
    }

    /// Returns `true` when this method returns `Vec<Self>` — the
    /// receiver-class sibling of
    /// [`returns_other_class_list`](Self::returns_other_class_list) that this
    /// syntactic detector can't resolve to a *name* on its own, because
    /// `ParsedMethod` doesn't carry the enclosing impl's type ident (#1375).
    ///
    /// `ReturnStrategy::for_method` treats a `true` result exactly like
    /// `returns_other_class_list()` returning `Some` (both select
    /// `ReturnOtherClassList`); `MethodReturnBuilder::with_return_class_from_method`
    /// does the actual name substitution, using the `type_ident` every
    /// class-system generator already has in scope — preferred over widening
    /// `ParsedMethod` to carry the impl ident (see that method's rustdoc for
    /// why: the substituted name must be the Rust type name, which is not
    /// always the same string as the R-facing class name).
    ///
    /// Same three container shapes as `returns_other_class_list`, with the
    /// element check inverted (the element must *be* `Self`, not be excluded
    /// for being `Self`): `Vec<Self>`, `Option<Vec<Self>>`,
    /// `Result<Vec<Self>, E>` (`E` not `()`).
    pub fn returns_self_list(&self) -> bool {
        let syn::ReturnType::Type(_, ty) = &self.sig.output else {
            return false;
        };
        let syn::Type::Path(p) = ty.as_ref() else {
            return false;
        };
        let Some(seg) = p.path.segments.last() else {
            return false;
        };

        match &seg.arguments {
            syn::PathArguments::AngleBracketed(_) if seg.ident == "Vec" => Self::is_vec_of_self(ty),
            syn::PathArguments::AngleBracketed(_) if seg.ident == "Option" => {
                crate::first_type_argument(seg).is_some_and(Self::is_vec_of_self)
            }
            syn::PathArguments::AngleBracketed(_) if seg.ident == "Result" => {
                // Mirrors the `Result<T, ()>` exclusion in
                // `returns_other_class_list`: `ResultNullOnErr`'s `.val` may
                // be `NULL`, which `lapply` would silently coerce to `list()`.
                let err_is_unit = crate::second_type_argument(seg)
                    .is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
                !err_is_unit && crate::first_type_argument(seg).is_some_and(Self::is_vec_of_self)
            }
            _ => false,
        }
    }

    /// True when `ty` is `Vec<Self>` — a bare `Self` ident, matching the
    /// looser ident-only check [`returns_self`](Self::returns_self) uses (no
    /// generic-argument check on `Self` itself, which cannot carry any in
    /// this position).
    fn is_vec_of_self(ty: &syn::Type) -> bool {
        let syn::Type::Path(p) = ty else {
            return false;
        };
        let Some(seg) = p.path.segments.last() else {
            return false;
        };
        if seg.ident != "Vec" {
            return false;
        }
        matches!(crate::first_type_argument(seg), Some(syn::Type::Path(ip))
            if ip.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false))
    }

    /// Returns true if this method returns a reference to `Self` (`&Self` or
    /// `&mut Self`) — the idiomatic Rust in-place builder signature.
    ///
    /// These methods mutate (`&mut self`) or read (`&self`) the receiver and
    /// return a borrow of the same value so calls can be chained in Rust
    /// (`b.set_a(1).set_b(2)`). On the R side this maps to a pipe-friendly free
    /// function (S3) / chainable instance method that returns the *same*
    /// ExternalPtr handle, so the R idiom `obj |> set_a(1) |> set_b(2)` works
    /// with in-place value semantics (no clone). See
    /// [`crate::c_wrapper_builder::ReturnHandling::SelfHandle`].
    pub fn returns_self_ref(&self) -> bool {
        matches!(&self.sig.output, syn::ReturnType::Type(_, ty) if Self::is_self_ref_type(ty))
    }

    /// `&Self` / `&mut Self` test shared by the bare and wrapped forms.
    fn is_self_ref_type(ty: &syn::Type) -> bool {
        matches!(ty, syn::Type::Reference(r)
            if matches!(r.elem.as_ref(), syn::Type::Path(p)
                if p.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false)))
    }

    /// First type argument of `Wrapper<..>` when the output's last path segment
    /// is `wrapper` (`Result` / `Option`).
    fn wrapped_output_arg(&self, wrapper: &str) -> Option<&syn::Type> {
        let syn::ReturnType::Type(_, ty) = &self.sig.output else {
            return None;
        };
        let syn::Type::Path(p) = ty.as_ref() else {
            return None;
        };
        let seg = p.path.segments.last()?;
        if seg.ident != wrapper {
            return None;
        }
        let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
            return None;
        };
        match ab.args.first()? {
            syn::GenericArgument::Type(t) => Some(t),
            _ => None,
        }
    }

    /// `-> Result<&Self, E>` / `-> Result<&mut Self, E>`: a fallible in-place
    /// builder step (#1433). `Ok` hands back the same handle like
    /// [`Self::returns_self_ref`]; `Err` raises through the normal `Result`
    /// error path.
    pub fn returns_result_self_ref(&self) -> bool {
        self.wrapped_output_arg("Result")
            .is_some_and(Self::is_self_ref_type)
    }

    /// `-> Option<&Self>` / `-> Option<&mut Self>`: the `Option` sibling of
    /// [`Self::returns_result_self_ref`]; `None` raises the usual absence error.
    pub fn returns_option_self_ref(&self) -> bool {
        self.wrapped_output_arg("Option")
            .is_some_and(Self::is_self_ref_type)
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

    /// Whether the R wrapper returns this method's value with `invisible()`
    /// (#1213): an `Invisible<T>` return-type marker or the `invisible` method
    /// option. Methods are visible by default, including the receiver-returning
    /// chainable tails.
    pub fn is_invisible(&self) -> bool {
        self.visibility_marker
            .or(self.method_attrs.force_invisible)
            .unwrap_or(false)
    }
}

fn is_builtin_return_type_name(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "f32"
            | "f64"
            | "bool"
            | "char"
            | "str"
            | "String"
            | "usize"
            | "isize"
    )
}

fn is_known_return_container_name(name: &str) -> bool {
    matches!(
        name,
        "Vec" | "Option" | "Result" | "Box" | "HashMap" | "BTreeMap"
    )
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

        // Reject type/const generics — the generated #[no_mangle] C wrappers are
        // incompatible with type/const params (they require monomorphization → multiple
        // symbols). Lifetime params ARE allowed: lifetimes are erased at codegen and the
        // wrapper generation uses `type_ident` (the bare struct ident) without generic
        // args, which is correct because lifetime arguments are never needed in generated
        // code (they are inferred or elided).
        {
            let params = &item_impl.generics.params;
            let has_type_or_const = params
                .iter()
                .any(|p| matches!(p, syn::GenericParam::Type(_) | syn::GenericParam::Const(_)));

            if has_type_or_const {
                return Err(syn::Error::new_spanned(
                    &item_impl.generics,
                    "generic impl blocks are not supported by #[miniextendr]. \
                     R's .Call interface requires monomorphic C symbols, so generic type \
                     parameters cannot be used. Remove the generic parameters and use a \
                     concrete type instead. \
                     Explicit lifetime parameters are allowed (lifetimes are erased at codegen).",
                ));
            }
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

        // The class block renders no page under `@noRd` or a plain `noexport`
        // (`ClassDocBuilder::build`'s gate: `internal`, as the attribute or the
        // `@keywords internal` tag, keeps the page). Its docs and every
        // method's (R6 folds its method blocks into the class block) then
        // keep no links (#1818).
        let class_page = crate::roxygen::RdPage::suppressed_if(
            crate::roxygen::attrs_have_roxygen_tag(&item_impl.attrs, "noRd")
                || (attrs.noexport
                    && !attrs.internal
                    && !crate::roxygen::attrs_have_roxygen_tag(
                        &item_impl.attrs,
                        "keywords internal",
                    )),
        );

        // Parse methods and validate attributes. Method parsing also normalizes
        // raw variadic `...` syntax in the impl clone that gets re-emitted.
        let mut original_impl = item_impl.clone();
        let mut methods = Vec::new();
        for item in &mut original_impl.items {
            if let syn::ImplItem::Fn(fn_item) = item {
                let mut method = ParsedMethod::from_impl_item(fn_item, class_page)?;
                crate::s7_conversion::configure(
                    &mut method,
                    fn_item,
                    attrs.class_system,
                    &type_ident,
                )?;
                // Validate method attributes for this class system
                ParsedMethod::validate_method_attrs(
                    &method.method_attrs,
                    attrs.class_system,
                    fn_item.sig.ident.span(),
                )?;
                if method.method_attrs.serde_error.is_some()
                    && !crate::return_type_analysis::output_is_result(&fn_item.sig.output)
                {
                    return Err(syn::Error::new(
                        fn_item.sig.ident.span(),
                        "`#[miniextendr(serde_error(..))]` requires a `Result<T, E>` return type: it \
                         classes the condition raised from the `Err` arm",
                    ));
                }
                methods.push(method);
            }
        }

        // MXL120: For vctrs impls, reject constructors that return Self or the named type,
        // and reject all instance-method receivers (&self, &mut self, self, self: ExternalPtr<Self>).
        //
        // The generated R wrapper passes the constructor result to vctrs::new_vctr() (or
        // new_rcrd/new_list_of), which requires a plain vector — not an ExternalPtr.
        // Returning Self produces an EXTPTRSXP which new_vctr rejects with
        // ".data must be a vector type".
        //
        // Instance-method receivers (&self, &mut self, etc.) are equally broken: the vctrs
        // S3 dispatch passes the R object (an S3-classed base vector — REALSXP, INTSXP, etc.)
        // as `self_sexp`. The C wrapper's receiver prelude (`resolve_receiver`, then
        // `ErasedExternalPtr::from_sexp`) panics because the base vector is not an ExternalPtr
        // and carries no `.ptr`.  There is no Rust `Self`
        // stored anywhere — the vector payload IS the R object.  Instance methods must be
        // expressed as static methods receiving the vector data by parameter.
        if attrs.class_system == ClassSystem::Vctrs {
            for method in &methods {
                // Check 1: constructor return type
                let is_ctor = (method.method_attrs.constructor
                    || (method.env == ReceiverKind::None && method.ident == "new"))
                    && method.env != ReceiverKind::Ref
                    && method.env != ReceiverKind::RefMut;
                if is_ctor && vctrs_ctor_returns_self_or_type(&method.sig.output, &type_ident) {
                    return Err(syn::Error::new_spanned(
                        &method.sig.output,
                        format!(
                            "[MXL120] vctrs constructor `{}` must not return `Self` or `{}`.\n\
                             \n\
                             The generated R wrapper passes the constructor result to \
                             `vctrs::new_vctr()` (or `new_rcrd`/`new_list_of`), which requires a \
                             plain vector payload — not an ExternalPtr (`EXTPTRSXP`).\n\
                             \n\
                             Fix: return the vector payload directly instead of `Self`.\n\
                             For example, return `Vec<f64>` (for vctr), a `std::collections::HashMap` \
                             / named-list struct (for rcrd), or a `Vec<Vec<T>>` (for list_of).",
                            method.ident, type_ident
                        ),
                    ));
                }

                // Check 2: instance-method receivers are not supported on vctrs impls
                if method.env.is_instance() {
                    let receiver_spelling = match method.env {
                        ReceiverKind::Ref => "&self",
                        ReceiverKind::RefMut => "&mut self",
                        ReceiverKind::Value => "self",
                        ReceiverKind::ExternalPtrRef => "self: ExternalPtrRef<'_, Self>",
                        ReceiverKind::ExternalPtrMut => "self: ExternalPtrMut<'_, Self>",
                        ReceiverKind::ExternalPtrValue => "self: ExternalPtr<Self>",
                        ReceiverKind::None => unreachable!(),
                    };
                    return Err(syn::Error::new_spanned(
                        &method.ident,
                        format!(
                            "[MXL120] vctrs impl method `{}` uses a `{}` receiver, which is not \
                             supported on `#[miniextendr(vctrs(...))]` impls.\n\
                             \n\
                             A vctrs object is an S3-classed base vector (REALSXP, INTSXP, etc.). \
                             There is no Rust `Self` stored inside the R SEXP — the vector payload \
                             IS the R object.  The C wrapper cannot reconstruct `Self` from a base \
                             vector, so calling an instance method would panic at runtime.\n\
                             \n\
                             Fix: convert this method to a static method whose parameters receive \
                             the vector data directly.  For example:\n\
                             \n\
                             // Before (broken):\n\
                             // pub fn value(&self) -> f64 {{ ... }}\n\
                             \n\
                             // After (correct):\n\
                             // pub fn value(amounts: Vec<f64>) -> Vec<f64> {{ ... }}",
                            method.ident, receiver_spelling
                        ),
                    ));
                }
            }
        }

        // Extract cfg attributes
        let cfg_attrs: Vec<_> = item_impl
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("cfg"))
            .cloned()
            .collect();
        let raw_doc_tags = crate::roxygen::roxygen_tags_from_attrs(&item_impl.attrs, class_page);
        // For R6: keep class-level @param tags (roxygen2 8.0.0 inherits them into
        // all methods); for other class systems use the stricter filter. Each
        // stripped tag's warning lives in its own anonymous `const _: () = { .. };`
        // scope (#1118), so no per-block disambiguator is needed here.
        let (doc_tags, param_warnings) = if attrs.class_system == ClassSystem::R6 {
            crate::roxygen::strip_method_tags_r6(
                &raw_doc_tags,
                &type_ident.to_string(),
                item_impl.impl_token.span,
            )
        } else {
            crate::roxygen::strip_method_tags(
                &raw_doc_tags,
                &type_ident.to_string(),
                item_impl.impl_token.span,
            )
        };
        // Build the set of parameter names declared at class level so the R6
        // generator can suppress placeholder lines for covered params.
        let class_param_names = crate::roxygen::extract_param_names(&doc_tags);

        let parsed = ParsedImpl {
            type_ident,
            class_system: attrs.class_system,
            class_name: attrs.class_name,
            doc_tags,
            methods,
            // Strip miniextendr attributes (and roxygen tags) before re-emitting,
            // then rewrite ExternalPtr receivers for stable Rust compatibility.
            original_impl: rewrite_external_ptr_receivers(strip_miniextendr_attrs_from_impl(
                original_impl,
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
            field_lookup: attrs.field_lookup,
            strict: attrs.strict,
            preconditions: attrs.preconditions,
            internal: attrs.internal,
            noexport: attrs.noexport,
            constructor_demoted: false,
            param_warnings,
            class_param_names,
        };
        parsed.check_constructor_markers()?;
        parsed.reject_unsupported_describe_in()?;
        field_syntax::check_collisions(&parsed)?;
        Ok(parsed)
    }

    /// Reject a method-level `@describeIn` that roxygen2 cannot honour (#1590).
    ///
    /// roxygen2 lists a `@describeIn` block in its destination's "Functions"
    /// section through the R object the block documents, and rejects the tag
    /// next to `@name` / `@rdname`, so `MethodDocBuilder` drops both for it.
    /// That only works where the method's block sits on a plain R function or
    /// S3 method: S3 instance methods (`generic.Class`), the static methods of
    /// S3 / S4 / S7 / vctrs (`class_method()`), and the S4 constructor
    /// (`Class()`). Everywhere else the block has no object to list:
    ///
    /// - S4 / S7 instance methods are registered by `setMethod()` /
    ///   `S7::method<-` after the `if (!exists(...))` generic guard the block
    ///   is attached to;
    /// - Env and R6 methods are `Class$method <-` assignments or live inside
    ///   `R6Class()`;
    /// - the S3 / vctrs / S7 constructor is documented by the class block.
    ///
    /// There the tag would only earn a roxygen2 warning and a dropped block,
    /// so it is a compile error pointing at `@rdname` instead.
    fn reject_unsupported_describe_in(&self) -> syn::Result<()> {
        for method in self.included_methods() {
            if crate::roxygen::describe_in_topic(&method.doc_tags).is_none() {
                continue;
            }
            let is_ctor = self.is_method_constructor(method);
            let is_static = method.env == ReceiverKind::None && !is_ctor;
            let supported = match self.class_system {
                ClassSystem::S3 | ClassSystem::Vctrs => !is_ctor,
                ClassSystem::S4 => is_ctor || is_static,
                ClassSystem::S7 => is_static,
                ClassSystem::Env | ClassSystem::R6 => false,
            };
            if supported {
                continue;
            }
            let kind = if is_ctor {
                "constructor"
            } else if is_static {
                "static method"
            } else {
                "instance method"
            };
            let system = match self.class_system {
                ClassSystem::S3 => "S3",
                ClassSystem::Vctrs => "vctrs",
                ClassSystem::S4 => "S4",
                ClassSystem::S7 => "S7",
                ClassSystem::Env => "Env",
                ClassSystem::R6 => "R6",
            };
            return Err(syn::Error::new_spanned(
                &method.ident,
                format!(
                    "`@describeIn` is not supported on the {system} {kind} `{}`: its R wrapper \
                     is not an R function roxygen2 can list in the destination's \"Functions\" \
                     section, so roxygen2 would drop the block. Use `@rdname <topic>` to \
                     document it on that page instead. `@describeIn` works on S3 instance \
                     methods, on the static methods of S3, S4, S7 and vctrs classes, and on \
                     S4 constructors.",
                    method.ident,
                ),
            ));
        }
        Ok(())
    }

    /// Get the class name (override or type name).
    pub fn class_name(&self) -> String {
        self.class_name
            .clone()
            .unwrap_or_else(|| self.type_ident.to_string())
    }

    /// The class-level settings, written like the attribute, which every impl
    /// block of a type must spell the same (#1776): the class system with its
    /// options, `class = "..."`, `internal` / `noexport`, and the `@noRd` /
    /// `@keywords internal` tags. They shape the members each block
    /// generates, so the wrapper writer refuses blocks that differ. The
    /// per-method defaults (`strict`, `preconditions`) may differ.
    pub fn class_settings(&self) -> String {
        let mut options: Vec<String> = Vec::new();
        match self.class_system {
            ClassSystem::Vctrs => {
                let vctrs = &self.vctrs_attrs;
                let kind = match vctrs.kind {
                    VctrsKind::Vctr => "vctr",
                    VctrsKind::Rcrd => "rcrd",
                    VctrsKind::ListOf => "list_of",
                };
                options.push(format!("kind = {kind:?}"));
                if let Some(base) = &vctrs.base {
                    options.push(format!("base = {base:?}"));
                }
                if let Some(inherit) = vctrs.inherit_base_type {
                    options.push(format!("inherit_base_type = {inherit}"));
                }
                if let Some(ptype) = &vctrs.ptype {
                    options.push(format!("ptype = {ptype:?}"));
                }
                if let Some(abbr) = &vctrs.abbr {
                    options.push(format!("abbr = {abbr:?}"));
                }
            }
            ClassSystem::R6 => {
                if let Some(inherit) = &self.r6_inherit {
                    options.push(format!("inherit = {inherit:?}"));
                }
                for (name, value) in [
                    ("portable", self.r6_portable),
                    ("cloneable", self.r6_cloneable),
                    ("lock_objects", self.r6_lock_objects),
                    ("lock_class", self.r6_lock_class),
                ] {
                    if let Some(value) = value {
                        options.push(format!("{name} = {value}"));
                    }
                }
            }
            ClassSystem::S7 => {
                if let Some(parent) = &self.s7_parent {
                    options.push(format!("parent = {parent:?}"));
                }
                if self.s7_abstract {
                    options.push("abstract".to_string());
                }
            }
            ClassSystem::Env | ClassSystem::S3 | ClassSystem::S4 => {}
        }
        match self.r_data_accessors {
            RDataAccessors::Off => {}
            RDataAccessors::GetSet => options.push("r_data_accessors".to_string()),
            RDataAccessors::GetOnly => options.push("r_data_accessors = \"get\"".to_string()),
        }
        if self.field_lookup == FieldLookup::List {
            options.push("lookup = \"list\"".to_string());
        }

        let system = self.class_system.to_ident().to_string();
        let mut settings = vec![if options.is_empty() {
            system
        } else {
            format!("{system}({})", options.join(", "))
        }];
        if let Some(class_name) = &self.class_name {
            settings.push(format!("class = {class_name:?}"));
        }
        if self.internal {
            settings.push("internal".to_string());
        }
        if self.noexport {
            settings.push("noexport".to_string());
        }
        for tag in CLASS_SETTING_TAGS {
            if crate::roxygen::has_roxygen_tag(&self.doc_tags, tag) {
                settings.push(format!("@{tag}"));
            }
        }
        settings.join(", ")
    }

    /// What this impl block has that only one block of a type may: the
    /// constructor and S7 properties. The wrapper writer defines the class
    /// from the block that has them (#1776); the impl docs of every block
    /// merge into the class's roxygen block ([`ClassWrapper::class_docs`]).
    pub fn class_definitions(&self) -> Vec<String> {
        let mut definitions = Vec::new();
        if self.declared_constructor().is_some() {
            definitions.push("a constructor".to_string());
        } else if let Some(inferred) = self.inferred_constructor() {
            definitions.push(format!("the inferred constructor `{}`", inferred.ident));
        }
        if self.class_system == ClassSystem::S7
            && self.included_methods().any(|m| {
                let s7 = &m.method_attrs.s7;
                s7.getter || s7.setter || s7.validate
            })
        {
            definitions.push("S7 properties".to_string());
        }
        definitions
    }

    /// Get methods that should be included.
    pub fn included_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| m.should_include())
    }

    /// The class constructor of this impl block, if it has one: the
    /// [declared](Self::declared_constructor) constructor, else the
    /// [inferred](Self::inferred_constructor) one. Every other receiver-less
    /// method is a static method.
    pub fn constructor(&self) -> Option<&ParsedMethod> {
        self.declared_constructor()
            .or_else(|| self.inferred_constructor())
    }

    /// The constructor the block declares: the method marked
    /// `#[miniextendr(<system>(constructor))]` (at most one,
    /// [`check_constructor_markers`](Self::check_constructor_markers)), else
    /// a receiver-less `new` returning `Self`. For vctrs, `new` may return
    /// anything: the canonical vctrs constructor returns the vector payload
    /// (e.g. `Vec<f64>`), which `vctrs::new_vctr()` then wraps; returning
    /// `Self` would produce an `ExternalPtr` that `new_vctr` can't accept as
    /// `.data`.
    pub fn declared_constructor(&self) -> Option<&ParsedMethod> {
        self.included_methods()
            .find(|m| m.method_attrs.constructor)
            .or_else(|| {
                self.included_methods().find(|m| {
                    m.env == ReceiverKind::None
                        && m.ident == "new"
                        && (self.class_system == ClassSystem::Vctrs || m.returns_self())
                })
            })
    }

    /// The methods that could be the class constructor: the receiver-less
    /// methods returning `Self` (bare `Self`, as [`ParsedMethod::returns_self`]
    /// reads it, so not `Result<Self, E>` or `Option<Self>`), other than the
    /// declared constructor and an S7 `convert_from` method, in source order.
    /// Without a declared constructor, the one candidate of a type is its
    /// constructor; with several, the type needs a declared one. None for
    /// vctrs, whose constructor returns the vector payload rather than `Self`.
    pub fn constructor_candidates(&self) -> impl Iterator<Item = &ParsedMethod> {
        let declared = self.declared_constructor();
        self.included_methods().filter(move |m| {
            self.class_system != ClassSystem::Vctrs
                && m.env == ReceiverKind::None
                && m.returns_self()
                && m.method_attrs.s7.convert_from.is_none()
                && !declared.is_some_and(|d| std::ptr::eq(d, *m))
        })
    }

    /// The constructor this block infers: its one
    /// [candidate](Self::constructor_candidates), whatever its name, when it
    /// declares no constructor. None in the rendering for a type whose
    /// constructor another impl block declares
    /// ([`constructor_demoted`](Self::constructor_demoted)). The wrapper
    /// writer decides per type which rendering it writes.
    pub fn inferred_constructor(&self) -> Option<&ParsedMethod> {
        if self.constructor_demoted || self.declared_constructor().is_some() {
            return None;
        }
        let mut candidates = self.constructor_candidates();
        let first = candidates.next()?;
        candidates.next().is_none().then_some(first)
    }

    /// Whether `m` is the class [constructor](Self::constructor).
    fn is_method_constructor(&self, m: &ParsedMethod) -> bool {
        self.constructor().is_some_and(|c| std::ptr::eq(c, m))
    }

    /// The roxygen tags of the static method `m`, called as `call` (e.g.
    /// `Counter$from_value()`). The constructor under its own name
    /// ([`constructor_keeps_own_name`](Self::constructor_keeps_own_name))
    /// points at the class constructor `constructor` (e.g. `Counter$new()`)
    /// instead of repeating its docs, keeping the method's `@noRd` /
    /// `@keywords`, and its `@param` tags when `params` (roxygen2 merges them
    /// with the class constructor's).
    fn static_doc_tags<'a>(
        &self,
        m: &'a ParsedMethod,
        call: &str,
        constructor: &str,
        params: bool,
    ) -> std::borrow::Cow<'a, [String]> {
        if !self.is_method_constructor(m) {
            return std::borrow::Cow::Borrowed(&m.doc_tags);
        }
        let kept = |tag: &&String| {
            let name = tag.split_whitespace().next().unwrap_or("");
            name == "@noRd" || name == "@keywords" || (params && name == "@param")
        };
        let mut tags = vec![format!(
            "@description `{call}` is another name for the constructor `{constructor}`."
        )];
        tags.extend(m.doc_tags.iter().filter(kept).cloned());
        std::borrow::Cow::Owned(tags)
    }

    /// Refuse two methods marked `#[miniextendr(<system>(constructor))]`: a
    /// class has one constructor.
    fn check_constructor_markers(&self) -> syn::Result<()> {
        let marker = format!(
            "#[miniextendr({}(constructor))]",
            self.class_system.to_ident()
        );
        let mut marked = self
            .included_methods()
            .filter(|m| m.method_attrs.constructor);
        if let (Some(first), Some(second)) = (marked.next(), marked.next()) {
            return Err(syn::Error::new_spanned(
                &second.ident,
                format!(
                    "`{}` and `{}` are both marked `{marker}`; a class has one constructor. \
                     Drop the marker from one: it becomes a static method.",
                    first.ident, second.ident,
                ),
            ));
        }
        Ok(())
    }

    /// Get public instance methods (have env, not private, not active, not an
    /// R6 finalizer or `deep_clone` handler).
    pub fn public_instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
                && !m.method_attrs.r6.deep_clone
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
                let method_name = crate::naming::ident_name(&m.ident);
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

    /// Get static methods (no env, not finalizer): the receiver-less methods
    /// other than the constructor, and the constructor too when its R name
    /// isn't `new` ([`constructor_keeps_own_name`](Self::constructor_keeps_own_name)).
    pub fn static_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env == ReceiverKind::None
                && (!self.is_method_constructor(m) || self.constructor_keeps_own_name(m))
                && !m.is_finalizer()
        })
    }

    /// Whether the constructor `m` is also a static method under its own
    /// name: `Type$from_value()` next to `Type$new()`, `Type_from_value()`
    /// next to `Type()`, `type_from_value()` next to `new_type()`. A
    /// constructor named `new` has no second name, and a vctrs constructor
    /// returns the vector payload, which only `new_<class>()` classes.
    fn constructor_keeps_own_name(&self, m: &ParsedMethod) -> bool {
        self.class_system != ClassSystem::Vctrs && m.r_method_name() != "new"
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
}

/// Generate a C-callable wrapper function for a single method in an impl block.
///
/// Produces a `#[no_mangle] extern "C-unwind"` function named `C_{crate}_{Type}__{method}` that:
/// 1. Accepts SEXP arguments (including `self_sexp` for instance methods)
/// 2. Extracts `&self` / `&mut self` from an `ErasedExternalPtr` for instance methods
/// 3. Converts SEXP arguments to Rust types
/// 4. Calls the actual Rust method
/// 5. Converts the return value back to SEXP
///
/// Thread strategy is determined automatically: instance methods always run on the main
/// thread (because `self_ref` is a non-Send borrow), as do methods taking `...`. Other
/// static methods use the main thread unless `worker` or `worker-default` selects
/// worker dispatch; `unsafe(main_thread)` keeps one on main regardless, and so do the
/// requirements that keep a standalone function there (R values in or out,
/// `check_interrupt`).
///
/// Also emits an `R_CallMethodDef` constant for R routine registration.
///
/// # Arguments
///
/// * `parsed_impl` - The parsed impl block providing type identity, cfg attrs, and options
/// * `method` - The parsed method to generate a wrapper for
pub fn generate_method_c_wrapper(parsed_impl: &ParsedImpl, method: &ParsedMethod) -> TokenStream {
    use crate::c_wrapper_builder::{CWrapperContext, ReturnHandling, ThreadStrategy};

    let type_ident = &parsed_impl.type_ident;
    let method_ident = &method.ident;
    let c_ident = method.c_wrapper_ident(type_ident);
    let self_sexp = crate::naming::self_sexp_ident();
    let self_ptr = syn::Ident::new("self_ptr", proc_macro2::Span::mixed_site());
    let self_ref = syn::Ident::new("self_ref", proc_macro2::Span::mixed_site());
    let self_slot = syn::Ident::new("self_slot", proc_macro2::Span::mixed_site());
    let external_ptr = syn::Ident::new("__self_ptr", proc_macro2::Span::mixed_site());
    let taken = syn::Ident::new("__mx_taken", proc_macro2::Span::mixed_site());

    // Determine thread strategy
    // Instance methods must use main thread because self_ref is a borrow that can't cross threads
    // Static methods use worker thread only when worker=true (set by explicit #[miniextendr(env(worker))]
    // or by the worker-default feature flag), and, as for standalone functions, only when
    // no R value crosses the call (a main-thread-bound input or return) and no
    // `check_interrupt` asks for R's interrupt check.
    let thread_strategy = if method.method_attrs.unsafe_main_thread
        || method.env.is_instance()
        || method.has_dots
        || method.method_attrs.check_interrupt
        || crate::type_inspect::signature_holds_r_values(&method.sig)
    {
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

    /// How a consuming (`self` by value) receiver is handled (#1432).
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum ValueMode {
        /// `-> Self`: move out, call, write the result back into the same slot.
        WriteBack,
        /// `-> Result<Self, E>` / `-> Option<Self>`: call on a clone, overwrite on success.
        Fallible,
        /// Any other return: move out, call, leave the slot consumed.
        Terminal,
    }
    let value_mode = if method.returns_self() {
        ValueMode::WriteBack
    } else if method.returns_result_self() || method.returns_option_self() {
        ValueMode::Fallible
    } else {
        ValueMode::Terminal
    };

    // Generate self extraction for instance methods. Each receiver borrows
    // the value's RefCell for the whole call (shared for `&self`, exclusive
    // otherwise), so a callback into R that reaches the same object gets the
    // `miniextendr_borrow_conflict` error instead of a second reference. The
    // receiver is resolved before the arguments are converted and borrowed
    // (or moved out) after: an R error that longjmps out of an argument
    // conversion would skip the borrow's release and leave the object borrowed.
    let (pre_call, post_conversion) = if method.env.is_instance() {
        let owner = syn::Ident::new("__mx_owner", proc_macro2::Span::mixed_site());
        let cell = syn::Ident::new("__mx_cell", proc_macro2::Span::mixed_site());
        let guard = syn::Ident::new("__mx_guard", proc_macro2::Span::mixed_site());
        let resolve = quote! {
            let #owner =
                ::miniextendr_api::externalptr::ReceiverOwner::resolve::<#type_ident>(#self_sexp);
        };
        // The receiver's RefCell; `handle_downcast_failed` refuses a consumed
        // value, a pointer restored from a saved session, or another type.
        let cell_lookup = quote! {
            #resolve
            let #self_ptr = unsafe {
                ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(#owner.get())
            };
            let #cell = match #self_ptr.downcast_cell::<#type_ident>() {
                ::core::option::Option::Some(cell) => cell,
                ::core::option::Option::None => ::miniextendr_api::externalptr::handle_downcast_failed::<#type_ident>(&#self_ptr),
            };
        };
        let (lookup, borrow) = match method.env {
            ReceiverKind::RefMut => (
                cell_lookup,
                quote! {
                    let mut #guard = ::miniextendr_api::externalptr::borrow_value_mut(#cell);
                    let #self_ref = &mut *#guard;
                },
            ),
            ReceiverKind::Ref => (
                cell_lookup,
                quote! {
                    let #guard = ::miniextendr_api::externalptr::borrow_value(#cell);
                    let #self_ref = &*#guard;
                },
            ),
            ReceiverKind::Value => match value_mode {
                // `self -> Result<Self, E>` / `Option<Self>`: keep the stored
                // value; the call runs on a clone and the slot is overwritten
                // only on success (`T: Clone`, see `ConsumingFallible`).
                ValueMode::Fallible => (
                    cell_lookup,
                    quote! {
                        let mut #self_slot = ::miniextendr_api::externalptr::borrow_value_mut(#cell);
                    },
                ),
                // `self -> Self` and terminal `self -> T`: move the value out,
                // leaving the `ConsumedSlot` marker until (and unless) the
                // result is written back. A call that reaches the object
                // meanwhile finds it consumed.
                ValueMode::WriteBack | ValueMode::Terminal => (
                    quote! {
                        #resolve
                        let mut #self_ptr = unsafe {
                            ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(#owner.get())
                        };
                    },
                    quote! {
                        let #taken = match #self_ptr.take_for_consuming::<#type_ident>() {
                            ::core::option::Option::Some(v) => v,
                            ::core::option::Option::None => ::miniextendr_api::externalptr::handle_downcast_failed::<#type_ident>(&#self_ptr),
                        };
                    },
                ),
            },
            // A view receiver holds the borrow itself, with the R object
            // the receiver resolved to (rooted by `#owner` for the call).
            ReceiverKind::ExternalPtrRef => (
                cell_lookup,
                quote! {
                    let #self_ref = unsafe {
                        ::miniextendr_api::ExternalPtrRef::<#type_ident>::from_raw_parts(
                            #owner.owner(),
                            #cell,
                        )
                    };
                },
            ),
            ReceiverKind::ExternalPtrMut => (
                cell_lookup,
                quote! {
                    let #self_ref = unsafe {
                        ::miniextendr_api::ExternalPtrMut::<#type_ident>::from_raw_parts(
                            #owner.owner(),
                            #cell,
                        )
                    };
                },
            ),
            // The handle receiver takes no borrow: the body reads through
            // `borrow()` / `borrow_mut()`, or consumes the handle.
            ReceiverKind::ExternalPtrValue => (
                quote! {
                    #resolve
                    let #external_ptr =
                        ::miniextendr_api::externalptr::wrap_receiver::<#type_ident>(#owner.get());
                },
                TokenStream::new(),
            ),
            _ => unreachable!(),
        };
        (vec![lookup], vec![borrow])
    } else {
        (vec![], vec![])
    };

    // Fallible in-place builders (#1433): `&mut self -> Result<&mut Self, E>` /
    // `Option<&mut Self>` (and the `&self` forms). The returned borrow is
    // dropped and the wrapper returns the same handle on success.
    let fallible_self_ref = method.env.is_instance()
        && !matches!(method.env, ReceiverKind::Value)
        && (method.returns_result_self_ref() || method.returns_option_self_ref());

    // Generate call expression. A visibility marker (`Invisible<T>` /
    // `Visible<T>`, #1213) is a transparent newtype around the real return:
    // unwrap it (`.0`) before the conversion, which was analysed on the inner
    // type (`method.sig.output` is already peeled).
    let unwrap_marker = |call: TokenStream| -> TokenStream {
        let call = crate::type_inspect::prepare_return_value(
            call,
            method.visibility_marker,
            method.method_attrs.serialize,
        );
        match &method.return_wrap {
            Some(wrap) => wrap.prepare_value(call),
            None => call,
        }
    };
    let call_expr = match method.env {
        ReceiverKind::Ref | ReceiverKind::RefMut if fallible_self_ref => {
            let call = unwrap_marker(quote! { #self_ref.#method_ident(#(#rust_args),*) });
            quote! { #call.map(|_| ()) }
        }
        ReceiverKind::Ref | ReceiverKind::RefMut => {
            unwrap_marker(quote! { #self_ref.#method_ident(#(#rust_args),*) })
        }
        ReceiverKind::Value => match value_mode {
            ValueMode::WriteBack => {
                let call =
                    unwrap_marker(quote! { #type_ident::#method_ident(#taken, #(#rust_args),*) });
                quote! {{
                    let __mx_out = #call;
                    #self_ptr.restore_after_consuming::<#type_ident>(__mx_out);
                }}
            }
            ValueMode::Fallible => {
                let call = unwrap_marker(quote! {
                    #type_ident::#method_ident(
                        ::miniextendr_api::externalptr::clone_for_consuming::<#type_ident>(&*#self_slot),
                        #(#rust_args),*
                    )
                });
                quote! { #call.map(|__mx_v| { *#self_slot = __mx_v; }) }
            }
            ValueMode::Terminal => {
                unwrap_marker(quote! { #type_ident::#method_ident(#taken, #(#rust_args),*) })
            }
        },
        ReceiverKind::ExternalPtrRef | ReceiverKind::ExternalPtrMut => {
            unwrap_marker(quote! { #type_ident::#method_ident(#self_ref, #(#rust_args),*) })
        }
        ReceiverKind::ExternalPtrValue => {
            unwrap_marker(quote! { #type_ident::#method_ident(#external_ptr, #(#rust_args),*) })
        }
        ReceiverKind::None => unwrap_marker(quote! { #type_ident::#method_ident(#(#rust_args),*) }),
    };

    // Determine return handling strategy
    let return_handling = if method.env == ReceiverKind::Value && value_mode == ValueMode::WriteBack
    {
        // `self -> Self`: the call expression already wrote the result back;
        // hand back the same handle (in-place semantics, like `&mut Self`).
        ReturnHandling::SelfHandle
    } else if (method.env == ReceiverKind::Value && value_mode == ValueMode::Fallible)
        || fallible_self_ref
    {
        // Fallible in-place step: the call expression yields `Result<(), E>` /
        // `Option<()>`; raise on failure, same handle on success.
        if method.returns_result_self() || method.returns_result_self_ref() {
            ReturnHandling::SelfHandleResult
        } else {
            ReturnHandling::SelfHandleOption
        }
    } else if method.returns_self() {
        ReturnHandling::ExternalPtr
    } else if method.returns_self_ref() && method.env.is_instance() {
        // In-place builder (`&mut self -> &mut Self` / `&self -> Self`): the
        // method mutates/borrows the receiver and returns a borrow of it. Hand
        // back the same ExternalPtr handle (`self_sexp`) so chaining preserves
        // identity with no clone. See `ReturnHandling::SelfHandle`.
        ReturnHandling::SelfHandle
    } else if method.method_attrs.unwrap_in_r
        && crate::return_type_analysis::output_is_result(&method.sig.output)
    {
        ReturnHandling::IntoR
    } else {
        crate::c_wrapper_builder::detect_return_handling(&method.sig.output)
    };

    // Build the context using the builder
    let mut builder = CWrapperContext::builder(method_ident.clone(), c_ident)
        .r_name(method.r_method_name())
        .inputs(method.sig.inputs.clone())
        .output(method.sig.output.clone())
        .pre_call(pre_call)
        .post_conversion(post_conversion)
        .call_expr(call_expr)
        .thread_strategy(thread_strategy)
        .return_handling(return_handling)
        .err_parts(crate::c_wrapper_builder::ErrPartsMode::from_spec(
            method.method_attrs.serde_error.as_ref(),
        ))
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

    // Forward match_arg + several_ok parameter names so `RustConversionBuilder` swaps
    // in `match_arg_vec_from_sexp` for the Vec/slice/array/Box<[_]> conversion path.
    // Scalar match_arg doesn't need this — R's match.arg() validated the choice and
    // `TryFromSexp for EnumType` (auto-generated by `#[derive(MatchArg)]`) decodes it.
    // A choice type under `Missing` / `Option` layers (#1473, #1551) likewise
    // bypasses `TryFromSexp`: no `Option<UserEnum>` impl can exist downstream
    // (orphan rule).
    // `no_na(..)` parameters also have their converted value checked, for what
    // the type reads as `NA` beyond the R guard's `anyNA()`.
    for (rust_name, attrs) in &method.method_attrs.per_param {
        if let Some(leaf) = attrs.layered_leaf() {
            builder = builder.layered_choice(rust_name.clone(), leaf);
        } else if attrs.match_arg && attrs.several_ok {
            builder = builder.match_arg_several_ok(rust_name.clone());
        }
        if attrs.checks.no_na {
            builder = builder.no_na(rust_name.clone(), attrs.checks.no_na_message.clone());
        }
    }
    // `Checked<T>` / `Unchecked<T>` parameters (#1566) convert as `T` and are
    // wrapped back before the call.
    for (rust_name, markers) in &method.param_markers {
        builder = builder.param_markers(rust_name.clone(), markers.clone());
    }

    let c_wrapper_and_def = builder.build().generate();

    let match_arg_entries = generate_method_match_arg_entries(parsed_impl, method);

    quote! {
        #c_wrapper_and_def
        #match_arg_entries
    }
}

/// Register enum choices, aliases, defaults and parameter documentation for
/// write-time placeholder substitution. R calls no choice-lookup C helper.
fn generate_method_match_arg_entries(
    parsed_impl: &ParsedImpl,
    method: &ParsedMethod,
) -> TokenStream {
    if !method.method_attrs.per_param.values().any(|a| a.match_arg) {
        return TokenStream::new();
    }

    let type_ident = &parsed_impl.type_ident;
    let c_ident = method.c_wrapper_ident(type_ident);
    let c_ident_str = c_ident.to_string();
    let cfg_attrs = &parsed_impl.cfg_attrs;

    let mut out = TokenStream::new();

    // Signature order, so the emitted items don't follow the map's order.
    for (rust_name, attrs) in crate::miniextendr_fn::per_param_in_signature_order(
        &method.sig.inputs,
        &method.method_attrs.per_param,
    ) {
        if !attrs.match_arg {
            continue;
        }
        // Find the parameter type from the (already-normalized) signature.
        let Some(param_ty) = find_param_type(&method.sig.inputs, rust_name) else {
            continue;
        };
        // several_ok containers (Vec<Mode>, Box<[Mode]>, [Mode; N], &[Mode]) and the
        // `Option<Mode>` scalar form both wrap the enum; register the enum's
        // CHOICES, not the wrapper's.
        let several_ok = attrs.several_ok;
        let choices_ty = crate::match_arg_choices_ty(param_ty, several_ok).clone();

        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);

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

        let preferred_default = attrs
            .default
            .as_deref()
            .map(crate::match_arg_keys::extract_match_arg_default)
            .unwrap_or_default();
        let aliases_placeholder = crate::match_arg_keys::aliases_placeholder(&c_ident_str, &r_name);
        let choices_entry_tokens = crate::match_arg_keys::choices_entry_tokens(
            cfg_attrs,
            &entry_ident,
            &placeholder,
            &aliases_placeholder,
            &choices_ty,
            &preferred_default,
        );
        let param_doc_entry_tokens = crate::match_arg_keys::param_doc_entry_tokens(
            cfg_attrs,
            &doc_entry_ident,
            &doc_placeholder,
            several_ok,
            &attrs.choice_doc_suffix(),
            &choices_ty,
        );

        out.extend(quote! {
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
            && crate::naming::unraw(&pat_ident.ident) == name
        {
            return Some(pt.ty.as_ref());
        }
    }
    None
}

/// Returns true if a vctrs constructor's return type is `Self`, `&Self`, `&mut Self`,
/// the named impl type, `Box<Self>`, `Result<Self, _>`, or `Result<NamedType, _>`.
///
/// These are all invalid for a vctrs constructor because the generated R wrapper
/// passes the return value to `vctrs::new_vctr()` / `new_rcrd()` / `new_list_of()`,
/// which require a plain vector payload — not an `ExternalPtr` (`EXTPTRSXP`).
fn vctrs_ctor_returns_self_or_type(output: &syn::ReturnType, type_ident: &syn::Ident) -> bool {
    let syn::ReturnType::Type(_, ty) = output else {
        return false;
    };
    ty_is_self_or_named(ty.as_ref(), type_ident)
}

/// Recursively checks whether `ty` is `Self`, `&Self`, `&mut Self`, `Box<Self>`,
/// the named type, or `Result<(Self | NamedType), _>`.
fn ty_is_self_or_named(ty: &syn::Type, type_ident: &syn::Ident) -> bool {
    match ty {
        syn::Type::Path(p) => {
            let last = match p.path.segments.last() {
                Some(s) => s,
                None => return false,
            };
            // Plain `Self` or `TypeName`
            if last.ident == "Self" || last.ident == *type_ident {
                return true;
            }
            // `Result<Self, _>` or `Result<TypeName, _>`
            if last.ident == "Result"
                && let syn::PathArguments::AngleBracketed(ref args) = last.arguments
                && let Some(syn::GenericArgument::Type(first_ty)) = args.args.first()
            {
                return ty_is_self_or_named(first_ty, type_ident);
            }
            // `Box<Self>` or `Box<TypeName>`
            if last.ident == "Box"
                && let syn::PathArguments::AngleBracketed(ref args) = last.arguments
                && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
            {
                return ty_is_self_or_named(inner, type_ident);
            }
            false
        }
        // `&Self` or `&mut Self`
        syn::Type::Reference(r) => ty_is_self_or_named(r.elem.as_ref(), type_ident),
        _ => false,
    }
}

// region: Class-system R wrapper generators (sub-modules)

/// Environment-based class wrapper generator (`obj$method()` dispatch).
mod env_class;
/// `$` / `[[` field methods of `r_data_accessors` for S3, S4 and env classes (#1848).
mod field_syntax;
/// R6 class wrapper generator (`R6Class` with `$new()`, active bindings, private methods).
mod r6_class;
/// S3 class wrapper generator (`structure()` + `generic.Class` dispatch).
mod s3_class;
/// S4 class wrapper generator (`setClass` / `setMethod` formal OOP).
pub(crate) mod s4_class;
/// S7 class wrapper generator (`new_class` / `new_generic` modern R OOP).
pub(crate) mod s7_class;
/// vctrs-compatible class wrapper generator (`new_vctr` / `new_rcrd` / `new_list_of`).
mod vctrs_class;

pub(crate) use env_class::generate_env_class;
#[cfg(test)]
pub(crate) use env_class::generate_env_r_wrapper;
pub(crate) use r6_class::generate_r6_class;
#[cfg(test)]
pub(crate) use r6_class::generate_r6_r_wrapper;
pub(crate) use s3_class::generate_s3_class;
#[cfg(test)]
pub(crate) use s3_class::generate_s3_r_wrapper;
pub(crate) use s4_class::generate_s4_class;
#[cfg(test)]
pub(crate) use s4_class::generate_s4_r_wrapper;
pub(crate) use s7_class::generate_s7_class;
#[cfg(test)]
pub(crate) use s7_class::generate_s7_r_wrapper;
#[cfg(test)]
use s7_class::rust_type_to_s7_class;
pub(crate) use vctrs_class::generate_vctrs_class;
#[cfg(test)]
pub(crate) use vctrs_class::generate_vctrs_r_wrapper;

/// The first line of R6's `initialize` and S7's `constructor` for a class
/// without a constructor: called without a `.ptr`, they raise an error
/// rather than build an object holding no pointer. The factories and other
/// classes' methods always pass the `.ptr`.
pub(crate) fn no_constructor_guard(class_name: &str) -> String {
    format!(
        "if (is.null(.ptr)) stop(\"`{class_name}` has no constructor: its objects come from \
         the functions that return one.\", call. = FALSE)"
    )
}

/// The line of [`ClassWrapper::head`] where the class's roxygen block takes
/// the [`ClassWrapper::class_doc_lines`] of every impl block of the type.
/// Spelled the same as `miniextendr_api::registry::CLASS_DOC_LINES_MARKER`,
/// which the wrapper writer replaces.
pub(crate) const CLASS_DOC_LINES_MARKER: &str = ".__MX_CLASS_DOC_LINES__";

/// The line of [`ClassWrapper::head`] where the class's roxygen block takes
/// the [`ClassWrapper::class_docs`] of every impl block of the type, the
/// defining block's first. Spelled the same as
/// `miniextendr_api::registry::CLASS_DOCS_MARKER`.
pub(crate) const CLASS_DOCS_MARKER: &str = ".__MX_CLASS_DOCS__";

/// Opens a roxygen line of a class's roxygen block that a doc tag written on
/// another impl block of the type replaces: a generated `@title`, `@name`,
/// `@rdname`, `@export`, constructor `@param` or R6 `@field`. The wrapper
/// writer drops the line when another block's [`ClassWrapper::class_docs`]
/// have the tag (for `@param` / `@field`, for the same name), and otherwise
/// keeps it without the prefix. Spelled the same as
/// `miniextendr_api::registry::CLASS_DEFAULT_PREFIX`.
pub(crate) const CLASS_DEFAULT_PREFIX: &str = ".__MX_CLASS_DEFAULT__";

/// `line` as a generated default of the class's roxygen block
/// ([`CLASS_DEFAULT_PREFIX`]).
pub(crate) fn class_default(line: impl std::fmt::Display) -> String {
    format!("{CLASS_DEFAULT_PREFIX}{line}")
}

/// Opens a class's roxygen block that only carries the impl docs
/// ([`docs_only_class_block`]). The wrapper writer writes the block, up to its
/// closing `NULL`, only when the type's impl docs or member-derived class docs
/// say something, so a readers-only impl adds no page (#1918). Spelled the
/// same as `miniextendr_api::registry::CLASS_DOCS_ONLY_MARKER`.
pub(crate) const CLASS_DOCS_ONLY_MARKER: &str = ".__MX_CLASS_DOCS_ONLY__";

/// The roxygen block of an S3 or vctrs class without a constructor that the
/// package names and builds itself (`class = "..."`, #1918), documenting no
/// object: it carries the impl docs.
///
/// - On the impl's `@rdname` topic, a page the package documents itself, it
///   adds no alias and no title, and sorts after the topic's own blocks
///   ([`crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS`]), so their name wins.
/// - Otherwise its page is named for the class, aliased by it, and titled by
///   default (`kind`, the class system in the title) unless the impl docs
///   give a `@title`.
///
/// Ends with `NULL`, before which [`ClassWrapper::mark_class_doc_lines`]
/// goes; the writer drops the block when there is nothing to say
/// ([`CLASS_DOCS_ONLY_MARKER`]).
pub(crate) fn docs_only_class_block(parsed_impl: &ParsedImpl, kind: &str) -> Vec<String> {
    let class_name = parsed_impl.class_name();
    let tags = &parsed_impl.doc_tags;
    let mut lines = vec![CLASS_DOCS_ONLY_MARKER.to_string()];
    let no_rd = crate::roxygen::has_roxygen_tag(tags, "noRd")
        || (parsed_impl.noexport && !parsed_impl.internal);
    if no_rd {
        if !crate::roxygen::has_roxygen_tag(tags, "noRd") {
            lines.push("#' @noRd".to_string());
        }
    } else {
        if let Some(topic) = crate::roxygen::rdname_value(tags) {
            lines.push(format!("#' @name {topic}"));
            lines.push("#' @aliases NULL".to_string());
            if !crate::roxygen::has_roxygen_tag(tags, "order") {
                lines.push(format!("#' {}", crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS));
            }
        } else {
            // The block's own `@title` / `@name` replace the defaults.
            if !crate::roxygen::has_roxygen_tag(tags, "title") {
                lines.push(class_default(format!(
                    "#' @title {class_name} {kind} Class"
                )));
            }
            if !crate::roxygen::has_roxygen_tag(tags, "name") {
                lines.push(class_default(format!("#' @name {class_name}")));
            }
        }
        if parsed_impl.internal && !crate::roxygen::has_roxygen_tag(tags, "keywords internal") {
            lines.push("#' @keywords internal".to_string());
        }
    }
    lines.push(CLASS_DOCS_MARKER.to_string());
    lines.push("NULL".to_string());
    lines
}

/// Whether a type's class docs say something, beyond the page and export
/// tags: the wrapper writer keeps a [`CLASS_DOCS_ONLY_MARKER`] block then.
#[cfg(test)]
fn class_docs_say_something(class_docs: &[String], class_doc_lines: &[String]) -> bool {
    const PAGE_TAGS: &[&str] = &["rdname", "noRd", "keywords", "order", "export"];
    !class_doc_lines.is_empty()
        || class_docs.iter().any(|line| {
            line.strip_prefix("#' @")
                .and_then(|rest| rest.split_whitespace().next())
                .is_none_or(|tag| !PAGE_TAGS.contains(&tag))
        })
}

/// One impl block's R wrapper, in the parts the wrapper writer merges per
/// Rust type (#1776): the class is defined once, from the defining block's
/// `head` and `tail`, and every block's `members` attach to it. See
/// `miniextendr_api::registry::ClassBlockEntry`.
#[derive(Debug, Default)]
pub struct ClassWrapper {
    /// The class definition: the class's roxygen block, with the
    /// [`CLASS_DOCS_MARKER`] line where the impl docs go and the
    /// [`CLASS_DOC_LINES_MARKER`] line where member-derived class docs go,
    /// the class object, the constructor and the class-level helpers.
    pub head: Vec<String>,
    /// The block's own doc comment as roxygen lines (with the constructor's,
    /// for S3 and vctrs, whose class block is the constructor's block).
    pub class_docs: Vec<String>,
    /// Roxygen lines the block's members add to the class's roxygen block.
    pub class_doc_lines: Vec<String>,
    /// The block's methods and static methods.
    pub members: Vec<String>,
    /// What follows the members of every block, such as R6's `$lock()`.
    pub tail: Vec<String>,
    /// The names the members bind in the class's own namespaces, worded for
    /// the writer's message when two blocks bind the same one.
    pub member_names: Vec<String>,
}

impl ClassWrapper {
    /// Put the [`CLASS_DOC_LINES_MARKER`] before the last line of `head`, the
    /// tag that closes the class's roxygen block, and add the lifecycle
    /// imports of the block's methods to the class docs.
    fn mark_class_doc_lines(&mut self, head: &mut Vec<String>, parsed_impl: &ParsedImpl) {
        let pos = head.len().saturating_sub(1);
        head.insert(pos, CLASS_DOC_LINES_MARKER.to_string());
        if let Some(import) = crate::lifecycle::collect_lifecycle_imports(
            parsed_impl
                .methods
                .iter()
                .filter_map(|m| m.method_attrs.lifecycle.as_ref()),
        ) {
            self.class_doc_lines.push(format!("#' {import}"));
        }
    }

    /// The wrapper as the writer writes a type with this one impl block.
    #[cfg(test)]
    pub fn render(&self) -> String {
        let mut lines = Vec::new();
        let keep_docs_only = class_docs_say_something(&self.class_docs, &self.class_doc_lines);
        let mut skipping = false;
        for line in &self.head {
            if skipping {
                skipping = line != "NULL";
                continue;
            }
            if line == CLASS_DOCS_ONLY_MARKER {
                skipping = !keep_docs_only;
            } else if line == CLASS_DOCS_MARKER {
                lines.extend(self.class_docs.iter().cloned());
            } else if line == CLASS_DOC_LINES_MARKER {
                lines.extend(self.class_doc_lines.iter().map(|l| strip_class_default(l)));
            } else {
                lines.push(strip_class_default(line));
            }
        }
        lines.extend(self.members.iter().cloned());
        lines.extend(self.tail.iter().cloned());
        lines.join("\n")
    }
}

/// `line` without its [`CLASS_DEFAULT_PREFIX`], as the writer keeps a
/// default that no other block replaces.
#[cfg(test)]
fn strip_class_default(line: &str) -> String {
    line.strip_prefix(CLASS_DEFAULT_PREFIX)
        .unwrap_or(line)
        .to_string()
}

/// Generate R S3 method wrappers for `as.<class>()` coercion methods.
///
/// For each method with `#[miniextendr(as = "...")]`, generates an S3 method like:
///
/// ```r
/// #' @export
/// #' @method as.data.frame MyType
/// as.data.frame.MyType <- function(x, ...) {
///     .Call(C_MyType__as_data_frame, .call = sys.call(), x)
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
        let ctx = MethodContext::new(method, type_ident)
            .with_impl_preconditions(parsed_impl.preconditions);

        // Normalize coercion target for R generic name.
        // `as.numeric()` is a thin base-R wrapper that dispatches via the internal
        // generic `as.double` (not `as.numeric` itself) — an `as.numeric.<Class>`
        // S3 method is never consulted. Register under `as.double` for both
        // "numeric" and "double" targets so `as.double(x)` AND `as.numeric(x)`
        // both dispatch to it.
        // Some targets use non-standard S3 generic names (e.g., tibble uses as_tibble, not as.tibble)
        let r_generic = match coercion_target.as_str() {
            "numeric" | "double" => "as.double".to_string(),
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
            // The method's own doc tags were pushed verbatim above, so only
            // inject the defaults a user did not supply (same rule as
            // `MethodDocBuilder`): a method-level `@rdname` splits the
            // coercion onto its own page instead of being duplicated, and a
            // `@describeIn` (which roxygen2 rejects next to `@name` /
            // `@rdname`) takes its page from the S3 method object.
            let describe_in = crate::roxygen::describe_in_topic(&method.doc_tags).is_some();
            if !describe_in && !crate::roxygen::has_roxygen_tag(&method.doc_tags, "name") {
                lines.push(format!("#' @name {}", s3_method_name));
            }
            if !describe_in {
                if crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
                    // Own page: needs a title (see `MethodDocBuilder::build`).
                    if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "title") {
                        lines.push(format!("#' @title {}", s3_method_name));
                    }
                } else {
                    lines.push(format!("#' @rdname {}", class_name));
                }
            }
            crate::roxygen::push_order_after_topic_blocks(
                &mut lines,
                &method.doc_tags,
                &class_name,
            );
            lines.extend(crate::roxygen::method_source_tag(type_ident, &method.ident));
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
            .with_invisible(method.is_invisible())
            .with_class_name(class_name.clone())
            .with_return_class_from_method(method, &type_ident.to_string());
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate `impl RCoerce*` trait impls for methods with `#[miniextendr(as = "...")]`.
///
/// For each `as` coercion method, generates a forwarding trait impl:
/// ```ignore
/// impl ::miniextendr_api::r_coerce::RCoerceDataFrame for MyType {
///     fn as_data_frame(&self) -> Result<::miniextendr_api::List, ::miniextendr_api::r_coerce::RCoerceError> {
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
        // Only the 15 standard targets that have corresponding traits in r_coerce.
        let (trait_name, trait_method): (&str, &str) = match coercion_target {
            "data.frame" => ("RCoerceDataFrame", "as_data_frame"),
            "list" => ("RCoerceList", "as_list"),
            "character" => ("RCoerceCharacter", "as_character"),
            "numeric" | "double" => ("RCoerceNumeric", "as_numeric"),
            "integer" => ("RCoerceInteger", "as_integer"),
            "logical" => ("RCoerceLogical", "as_logical"),
            "matrix" => ("RCoerceMatrix", "as_matrix"),
            "vector" => ("RCoerceVector", "as_vector"),
            "factor" => ("RCoerceFactor", "as_factor"),
            "Date" => ("RCoerceDate", "as_date"),
            "POSIXct" => ("RCoercePOSIXct", "as_posixct"),
            "complex" => ("RCoerceComplex", "as_complex"),
            "raw" => ("RCoerceRaw", "as_raw"),
            "environment" => ("RCoerceEnvironment", "as_environment"),
            "function" => ("RCoerceFunction", "as_function"),
            _ => continue, // Non-standard targets (tibble, data.table, etc.)
        };

        let trait_ident = syn::Ident::new(trait_name, proc_macro2::Span::call_site());
        let trait_method_ident = syn::Ident::new(trait_method, proc_macro2::Span::call_site());
        let user_method_ident = &method.ident;

        // Return type: data.frame and list return Result<List, RCoerceError>,
        // all others return Result<SEXP, RCoerceError>
        let return_type = match coercion_target {
            "data.frame" | "list" => quote! {
                ::core::result::Result<::miniextendr_api::List, ::miniextendr_api::r_coerce::RCoerceError>
            },
            _ => quote! {
                ::core::result::Result<::miniextendr_api::SEXP, ::miniextendr_api::r_coerce::RCoerceError>
            },
        };

        impls.push(quote! {
            #(#cfg_attrs)*
            impl ::miniextendr_api::r_coerce::#trait_ident for #type_ident {
                fn #trait_method_ident(&self) -> #return_type {
                    self.#user_method_ident()
                }
            }
        });
    }

    quote! { #(#impls)* }
}

/// Detect S7 fast-path shortcut name collisions within an inherent impl block.
///
/// The S7 generator (`s7_class`) emits a standalone `<ClassName>_<method>`
/// shortcut function for each non-fallback, non-`no_shortcut` instance method,
/// and a `<ClassName>_<r_method_name>` function for each static method. These
/// share one R namespace, so two definitions with the same name silently
/// clobber each other (last write wins). This typically arises from an
/// `r_name` override that aliases an instance shortcut onto a static method
/// (or vice versa).
///
/// Returns an error pointing at the offending method, advising the user to
/// rename (via `r_name`) or opt the instance method out with
/// `#[miniextendr(s7(no_shortcut))]`. Non-S7 class systems are a no-op.
///
/// Note: collisions with `#[derive(ExternalPtr)]` sidecar accessors
/// (`<ClassName>_get_<field>` / `_set_<field>`) are *not* detected here — the
/// sidecar field names live in the derive's distributed-slice registry and are
/// not visible to this macro invocation. See #991 for the write-time check.
fn check_s7_shortcut_collisions(parsed: &ParsedImpl) -> syn::Result<()> {
    if parsed.class_system != ClassSystem::S7 {
        return Ok(());
    }
    let class_name = parsed.class_name();

    // Map emitted standalone-function name -> the method ident span that produced
    // it, so a second producer of the same name can be reported against itself.
    let mut seen: std::collections::HashMap<String, proc_macro2::Span> =
        std::collections::HashMap::new();

    // Static methods always emit `<ClassName>_<r_method_name>`.
    for m in parsed.static_methods() {
        let name = format!("{}_{}", class_name, m.r_method_name());
        seen.entry(name).or_insert_with(|| m.ident.span());
    }

    // Instance methods (excluding fallback / no_shortcut / property accessors)
    // emit the `<ClassName>_<r_method_name>` shortcut, unless that name is not
    // syntactic R (an operator `r_name`), in which case none is emitted.
    for m in parsed.instance_methods() {
        if m.method_attrs.s7.fallback
            || m.method_attrs.s7.no_shortcut
            || m.method_attrs.s7.getter
            || m.method_attrs.s7.setter
            || m.method_attrs.s7.validate
        {
            continue;
        }
        let Some(name) = s7_class::s7_shortcut_name(&class_name, &m.r_method_name()) else {
            continue;
        };
        if seen.contains_key(&name) {
            return Err(syn::Error::new(
                m.ident.span(),
                format!(
                    "S7 fast-path shortcut `{name}` collides with another generated function \
                     of the same name on `{class_name}`. Rename one (e.g. \
                     `#[miniextendr(s7(r_name = \"...\"))]`) or opt this method out of the \
                     shortcut with `#[miniextendr(s7(no_shortcut))]`."
                ),
            ));
        }
        seen.insert(name, m.ident.span());
    }

    Ok(())
}

/// Env constructors, instance methods and static methods all occupy the same
/// class environment. Check the emitted names before one overwrites another.
fn check_env_member_collisions(parsed: &ParsedImpl) -> syn::Result<()> {
    if parsed.class_system != ClassSystem::Env {
        return Ok(());
    }
    let mut seen = std::collections::HashMap::new();
    if let Some(constructor) = parsed.constructor() {
        seen.insert("new".to_owned(), constructor);
    }
    for method in parsed.instance_methods().chain(parsed.static_methods()) {
        let name = method.r_method_name();
        if let Some(previous) = seen.insert(name.clone(), method) {
            return Err(syn::Error::new_spanned(
                &method.ident,
                format!(
                    "Env member `{}${name}` is defined by both `{}` and `{}`. \
                     Constructors always occupy `new`; instance and static methods \
                     share the class environment. Rename a colliding method with \
                     `#[miniextendr(r_name = \"...\")]`.",
                    parsed.class_name(),
                    previous.ident,
                    method.ident,
                ),
            ));
        }
    }
    Ok(())
}

/// R6 always has an `initialize` member, including classes without a Rust
/// constructor. R6 requires public, private and active names to be distinct;
/// reject collisions before the generated class fails during package loading.
fn check_r6_initialize_collision(parsed: &ParsedImpl) -> syn::Result<()> {
    if parsed.class_system != ClassSystem::R6 {
        return Ok(());
    }
    for method in parsed
        .public_instance_methods()
        .chain(parsed.private_instance_methods())
        .chain(parsed.active_instance_methods())
    {
        let name = if method.is_active() {
            method
                .method_attrs
                .r6
                .prop
                .clone()
                .unwrap_or_else(|| method.r_method_name())
        } else {
            method.r_method_name()
        };
        if name == "initialize" {
            return Err(syn::Error::new_spanned(
                &method.ident,
                "R6 member `initialize` conflicts with the generated constructor. \
                 Rename the method or use `#[miniextendr(r_name = \"...\")]`; \
                 for an active binding, choose a different `r6(prop = \"...\")`.",
            ));
        }
    }
    Ok(())
}

/// The bindings of an `R6::R6Class()` generator object (R6 2.6). The R6
/// generator assigns static methods onto that object (`Class$name <- function`),
/// so a static method under one of these names silently replaces R6's own:
/// `new` stops creating objects, `set` and `lock` are the generated `$set()` /
/// `$lock()` calls, and `$new()` reads the data fields.
const R6_GENERATOR_BINDINGS: &[&str] = &[
    "active",
    "class",
    "classname",
    "clone_method",
    "cloneable",
    "debug",
    "debug_names",
    "get_inherit",
    "has_private",
    "inherit",
    "is_locked",
    "lock",
    "lock_class",
    "lock_objects",
    "new",
    "parent_env",
    "portable",
    "private_fields",
    "private_methods",
    "public_fields",
    "public_methods",
    "self",
    "set",
    "undebug",
    "unlock",
];

/// Reject an R6 static method whose R name is one of the generator object's
/// own bindings ([`R6_GENERATOR_BINDINGS`]), which the static would replace
/// without a warning when the package loads.
fn check_r6_static_collisions(parsed: &ParsedImpl) -> syn::Result<()> {
    if parsed.class_system != ClassSystem::R6 {
        return Ok(());
    }
    for method in parsed.static_methods() {
        let name = method.r_method_name();
        if R6_GENERATOR_BINDINGS.contains(&name.as_str()) {
            return Err(syn::Error::new_spanned(
                &method.ident,
                format!(
                    "R6 static method `{class}${name}` would replace the R6 generator's own \
                     `{name}`: static methods are assigned onto the generator object. Rename \
                     it with `#[miniextendr(r_name = \"...\")]`.",
                    class = parsed.class_name(),
                ),
            ));
        }
    }
    Ok(())
}

/// Two instance methods of one S4 impl block that dispatch through the same
/// generic emit two `setMethod("<generic>", "<Class>", ...)` calls with the
/// same signature, and the second silently replaces the first. The default
/// generic follows the R method name (`s4_<r_name>`), so `r_name = "x"` next to
/// a method `x` is one way to get there; two equal `s4(generic = "...")`
/// overrides are another.
fn check_s4_generic_collisions(parsed: &ParsedImpl) -> syn::Result<()> {
    if parsed.class_system != ClassSystem::S4 {
        return Ok(());
    }
    let mut seen = std::collections::HashMap::new();
    for method in parsed.instance_methods() {
        let generic = s4_class::s4_generic_name(method);
        if let Some(previous) = seen.insert(generic.clone(), method) {
            return Err(syn::Error::new_spanned(
                &method.ident,
                format!(
                    "S4 generic `{generic}` gets a method for `{}` from both `{}` and `{}`; \
                     the second `setMethod()` would replace the first. Rename one with \
                     `#[miniextendr(r_name = \"...\")]` or give it another \
                     `s4(generic = \"...\")`.",
                    parsed.class_name(),
                    previous.ident,
                    method.ident,
                ),
            ));
        }
    }
    Ok(())
}

/// Check the R formals of every generated method
/// ([`check_r_formals`](crate::r_wrapper_builder::check_r_formals)), reserving
/// the names each class system's wrapper binds itself:
///
/// - the receiver of an instance method: `x` for S3 and S4,
///   the first dispatch argument (`x` unless `s7(dispatch)` or an operator
///   says otherwise) for S7; shortcuts use the collision-free `.mx_self`;
/// - `self` and `private`, which R6 binds inside `initialize` and every method
///   of the class: the wrapper reads the pointer from `private$.ptr` and
///   returns `self` from a chaining method.
///
/// Static methods and the other constructors are plain functions and reserve
/// nothing; S7 property accessors have fixed formals; vctrs impls take no
/// instance methods (MXL120).
fn check_method_formals(parsed: &ParsedImpl) -> syn::Result<()> {
    let receiver = |name: &str, method: &str| {
        (
            name.to_string(),
            format!("is the receiver of the generated {method}"),
        )
    };
    for m in parsed.included_methods() {
        let is_ctor = parsed.is_method_constructor(m);
        let instance = m.env.is_instance() && !is_ctor && !m.is_finalizer();
        if (m.method_attrs.receiver.is_some() || m.method_attrs.no_dispatch_dots) && !instance {
            return Err(syn::Error::new_spanned(
                &m.ident,
                "`receiver` and `no_dots` apply only to instance methods",
            ));
        }
        let generic_receiver = m.method_attrs.receiver.as_deref().unwrap_or("x");
        let s7 = &m.method_attrs.s7;
        let reserved = match parsed.class_system {
            ClassSystem::R6 if instance || is_ctor => vec![
                (
                    "self".to_string(),
                    "is R6's `self` binding inside the class, the object the generated \
                     method works on"
                        .to_string(),
                ),
                (
                    "private".to_string(),
                    "is R6's `private` binding inside the class, where the generated method \
                     reads the object's pointer"
                        .to_string(),
                ),
            ],
            ClassSystem::S3 if instance => vec![receiver(generic_receiver, "S3 method")],
            ClassSystem::S4 if instance => vec![receiver(generic_receiver, "S4 method")],
            ClassSystem::S7 if instance && !(s7.getter || s7.setter || s7.validate) => {
                let generic = m
                    .method_attrs
                    .generic
                    .clone()
                    .unwrap_or_else(|| m.r_method_name());
                let dispatch =
                    s7_class::s7_dispatch_args(m, &s7_class::s7_generic_target(&generic));
                vec![receiver(&dispatch[0], "S7 method")]
            }
            _ => Vec::new(),
        };
        crate::r_wrapper_builder::check_r_formals(&m.sig.inputs, &reserved)?;
    }
    Ok(())
}

/// Check that every S3 instance method for a replacement generic
/// (`s3(generic = "[[<-")`, or an `r_name` ending in `<-`) takes the new value
/// last, as `value`
/// ([`check_replacement_value_param`](crate::miniextendr_fn::check_replacement_value_param),
/// #1853). The generator puts the dispatch `...` before that formal
/// ([`s3_method_formals`](crate::r_class_formatter::MethodContext::s3_method_formals)).
/// Other class systems register their methods themselves, and static methods
/// are plain functions.
fn check_s3_replacement_methods(parsed: &ParsedImpl) -> syn::Result<()> {
    if !matches!(parsed.class_system, ClassSystem::S3 | ClassSystem::Vctrs) {
        return Ok(());
    }
    for m in parsed.instance_methods() {
        let generic = m
            .method_attrs
            .generic
            .clone()
            .unwrap_or_else(|| m.r_method_name());
        crate::miniextendr_fn::check_replacement_value_param(
            &generic,
            &m.sig.inputs,
            m.ident.span(),
        )?;
    }
    Ok(())
}

/// Validate the S7 dispatch arguments of every S7 instance method: an explicit
/// `s7(dispatch = "...")`, and the pair S7 fixes for an `Ops` / `%*%`
/// operator generic (`e1, e2` / `x, y`).
///
/// S7 registers a method only when its formals start with the generic's
/// dispatch arguments, in order and without defaults, and it rejects the
/// wrong number of classes in the signature; any violation stops the package
/// from loading. The generator names the receiver after the first dispatch
/// argument and takes the others from the method's leading parameters, so the
/// rules are checked here, where the error can name the fix. `!` is rejected
/// outright because S7 does not dispatch it. Non-S7 class systems, static
/// methods and property accessors are a no-op.
fn check_s7_dispatch(parsed: &ParsedImpl) -> syn::Result<()> {
    if parsed.class_system != ClassSystem::S7 {
        return Ok(());
    }
    for m in parsed.instance_methods() {
        if m.method_attrs.s7.getter || m.method_attrs.s7.setter || m.method_attrs.s7.validate {
            continue;
        }
        let ctx = crate::r_class_formatter::MethodContext::new(m, &parsed.type_ident);
        check_s7_method_dispatch(m, &ctx.params)
            .map_err(|message| syn::Error::new(m.ident.span(), message))?;
    }
    Ok(())
}

/// [`check_s7_dispatch`] for one method; `params` is its R formals string
/// (normalized names, with defaults).
fn check_s7_method_dispatch(m: &ParsedMethod, params: &str) -> Result<(), String> {
    use s7_class::{S7GenericTarget, s7_dispatch_args, s7_generic_target};

    let rust_name = crate::naming::ident_name(&m.ident);
    let generic = m
        .method_attrs
        .generic
        .clone()
        .unwrap_or_else(|| m.r_method_name());
    let target = s7_generic_target(&generic);
    let op = target.name();
    if op == "!" {
        return Err(
            "S7 methods cannot be generated for the `!` operator: S7 does not dispatch \
                    `!` (its Ops handling covers the binary operators only). Keep this method \
                    under an ordinary name and call it directly."
                .to_string(),
        );
    }

    let dispatch = s7_dispatch_args(m, &target);
    let spec = m
        .method_attrs
        .s7
        .dispatch
        .clone()
        .unwrap_or_else(|| dispatch.join(", "));
    let formals = crate::roxygen::split_r_formals(params);
    let names: Vec<&str> = formals
        .iter()
        .map(|f| crate::roxygen::formal_name(f))
        .collect();
    let listed = if names.is_empty() {
        format!("`{rust_name}` has no parameters besides the receiver")
    } else {
        format!(
            "the R parameters of `{rust_name}` are {}",
            names
                .iter()
                .map(|n| format!("`{n}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let mut seen = std::collections::HashSet::new();
    for d in &dispatch {
        // `...` / `..1` pass R's syntactic-name rule but are not arguments.
        if !crate::naming::is_syntactic_r_name(d) || d.starts_with("..") {
            return Err(format!(
                "`s7(dispatch = \"{spec}\")` on `{rust_name}`: `{d}` is not a syntactic R \
                 argument name (S7 dispatch arguments are plain names, and `...` cannot be one)"
            ));
        }
        if !seen.insert(d.as_str()) {
            return Err(format!(
                "`s7(dispatch = \"{spec}\")` on `{rust_name}` names `{d}` twice"
            ));
        }
    }

    let is_base_operator = matches!(target, S7GenericTarget::BaseOperator(_));
    if let Some(pair) = s7_class::s7_operator_dispatch_args(op).filter(|_| is_base_operator) {
        let [left, right] = pair;
        if dispatch != pair {
            return Err(format!(
                "S7 dispatches `{op}` on `{left}` and `{right}`, so `s7(dispatch = \"{spec}\")` \
                 cannot be used on `{rust_name}`: drop it (the pair is the default for `{op}`)"
            ));
        }
        if m.method_attrs.s7.fallback {
            return Err(format!(
                "`s7(fallback)` cannot be combined with the `{op}` operator on `{rust_name}`: \
                 it would register the method for every pair of operands, not for this class"
            ));
        }
        let operands: Vec<&str> = names.iter().copied().filter(|n| *n != "...").collect();
        if operands.len() != 1 {
            return Err(format!(
                "`{rust_name}` implements the `{op}` operator, so it takes exactly one parameter \
                 besides the receiver: the right operand, named `{right}` (S7 calls the method \
                 as `{left} {op} {right}`); {listed}"
            ));
        }
        if operands[0] != right {
            return Err(format!(
                "S7 dispatches `{op}` on `{left}` and `{right}`: the receiver becomes `{left}` and \
                 the right operand of `{rust_name}` must be named `{right}`. Rename the parameter \
                 `{}` to `{right}`.",
                operands[0]
            ));
        }
    }
    if is_base_operator && s7_class::s7_operator_dispatch_args(op).is_none() && dispatch.len() > 1 {
        return Err(format!(
            "S7 dispatches `{op}` through the base primitive on its first argument only, so \
             `s7(dispatch = \"{spec}\")` cannot be used on `{rust_name}`: drop it"
        ));
    }

    // A parameter named like the receiver (`dispatch[0]`) is rejected earlier,
    // by `check_method_formals`.
    for (i, d) in dispatch.iter().enumerate().skip(1) {
        let Some(&found) = names.get(i - 1) else {
            return Err(format!(
                "`s7(dispatch = \"{spec}\")` on `{rust_name}`: dispatch argument `{d}` must be \
                 parameter {i} of the method (S7 needs a method's formals to start with the \
                 generic's dispatch arguments, the first naming the receiver), but {listed}"
            ));
        };
        if found != d {
            return Err(format!(
                "`s7(dispatch = \"{spec}\")` on `{rust_name}`: dispatch argument `{d}` must be \
                 parameter {i} of the method, found `{found}` (S7 needs a method's formals to \
                 start with the generic's dispatch arguments, in order, the first naming the \
                 receiver); {listed}"
            ));
        }
        if formals[i - 1].contains('=') {
            return Err(format!(
                "`s7(dispatch = \"{spec}\")` on `{rust_name}`: dispatch argument `{d}` has a \
                 default value (`{}`), which S7 does not allow on dispatch arguments",
                formals[i - 1].trim()
            ));
        }
    }
    if m.method_attrs.s7.no_dots
        && matches!(target, S7GenericTarget::Local(_))
        && let Some(extra) = names.get(dispatch.len() - 1)
    {
        return Err(format!(
            "`s7(no_dots)` on `{rust_name}` makes a generic without `...`, and S7 then requires \
             every method's formals to equal its dispatch arguments, but `{extra}` is not one. \
             Add it to `s7(dispatch = \"...\")` or drop `no_dots`."
        ));
    }
    Ok(())
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

    let mut parsed = match ParsedImpl::parse(attrs, item_impl) {
        Ok(p) => p,
        Err(e) => return e.into_compile_error().into(),
    };

    if let Err(e) = check_class_members(&parsed) {
        return e.into_compile_error().into();
    }
    // A block that infers its constructor is written in one of two
    // renderings, decided per type by the wrapper writer: with that
    // constructor, or (when another block declares one) with it as a static
    // method. Both must hold.
    let infers_constructor = parsed.inferred_constructor().is_some();
    let demoted_class = if infers_constructor {
        parsed.constructor_demoted = true;
        let demoted = parsed
            .reject_unsupported_describe_in()
            .and_then(|()| check_class_members(&parsed))
            .map(|()| (generate_class_wrapper(&parsed), parsed.class_definitions()));
        parsed.constructor_demoted = false;
        match demoted {
            Ok(class) => Some(class),
            Err(e) => return e.into_compile_error().into(),
        }
    } else {
        None
    };

    let type_ident = &parsed.type_ident;
    let cfg_attrs = &parsed.cfg_attrs;

    // Generate C wrappers for all included methods
    let c_wrappers: Vec<TokenStream> = parsed
        .included_methods()
        .map(|m| generate_method_c_wrapper(&parsed, m))
        .collect();

    // The R wrapper, in the parts the wrapper writer merges per type (#1776).
    let class = generate_class_wrapper(&parsed);

    let original_impl = &parsed.original_impl;

    // Generate forwarding trait impls for as.<class>() coercion methods
    let trait_impls = generate_as_coercion_trait_impls(&parsed);

    // Every part is qualified against the formals pooled over the whole block
    // (the constructor included).
    let formals =
        crate::r_shadowing::formal_names(parsed.included_methods().map(|m| &m.sig.inputs));
    let r_part = |lines: &[String]| crate::r_wrapper_raw_literal(&lines.join("\n"), &formals);
    let class_parts = |class: &ClassWrapper, defines: &[String]| {
        let head = r_part(&class.head);
        let class_docs = r_part(&class.class_docs);
        let class_doc_lines = r_part(&class.class_doc_lines);
        let members = r_part(&class.members);
        let tail = r_part(&class.tail);
        let member_names = &class.member_names;
        quote! {
            defines: &[#(#defines),*],
            head: #head,
            class_docs: #class_docs,
            class_doc_lines: #class_doc_lines,
            members: #members,
            member_names: &[#(#member_names),*],
            tail: #tail,
        }
    };

    let source_loc_doc = crate::source_location_doc(type_ident.span());
    let source_start = type_ident.span().start();
    let source_line_lit = syn::LitInt::new(&source_start.line.to_string(), type_ident.span());
    let source_col_lit =
        syn::LitInt::new(&(source_start.column + 1).to_string(), type_ident.span());

    let param_warnings = &parsed.param_warnings;

    // r_class_name is the R-visible name (may differ from type_ident when
    // `class = "Override"` was set on the impl block).
    let r_class_name_str = parsed.class_name();
    let class_system_str = parsed.class_system.to_ident().to_string();
    // `$` / `[[` field syntax (`s3/s4/env(r_data_accessors)`): the wrapper
    // writer then keeps the type's standalone sidecar accessors internal
    // (#1900). R6 bindings and S7 properties leave them exported.
    let field_syntax = parsed.r_data_accessors.enabled()
        && matches!(
            parsed.class_system,
            ClassSystem::S3 | ClassSystem::S4 | ClassSystem::Env
        );
    let settings = parsed.class_settings();
    let parts = class_parts(&class, &parsed.class_definitions());
    let constructor = parsed
        .constructor()
        .map(|m| crate::naming::ident_name(&m.ident))
        .unwrap_or_default();
    let candidates: Vec<String> = parsed
        .constructor_candidates()
        .map(|m| crate::naming::ident_name(&m.ident))
        .collect();
    let block_entry = |constructor: &str, inferred: bool, demoted, parts| {
        quote! {
            ::miniextendr_api::registry::ClassBlockEntry {
                rust_type: ::core::stringify!(#type_ident),
                type_path: || ::core::any::type_name::<#type_ident>(),
                r_class_name: #r_class_name_str,
                source_file: ::core::file!(),
                source_line: #source_line_lit,
                source_column: #source_col_lit,
                settings: #settings,
                constructor: #constructor,
                constructor_inferred: #inferred,
                constructor_candidates: &[#(#candidates),*],
                demoted: #demoted,
                #parts
            }
        }
    };
    // A block that infers its constructor also registers the rendering in
    // which that method is a static method (`demoted`).
    let (demoted_static, demoted_ref) = match &demoted_class {
        Some((demoted, definitions)) => {
            let entry = block_entry(
                "",
                false,
                quote!(::core::option::Option::None),
                class_parts(demoted, definitions),
            );
            (
                quote! {
                    static DEMOTED: ::miniextendr_api::registry::ClassBlockEntry = #entry;
                },
                quote!(::core::option::Option::Some(&DEMOTED)),
            )
        }
        None => (quote!(), quote!(::core::option::Option::None)),
    };
    let class_block = block_entry(&constructor, infers_constructor, demoted_ref, parts);
    let constructor_check = constructor_check(&parsed);

    // The registrations sit in an anonymous `const _` so that any number of
    // impl blocks of one type can register theirs; the wrapper writer merges
    // them per type (#1776).
    let expanded = quote! {
        #[doc = #source_loc_doc]
        #[doc = ::core::concat!("Generated from source file `", ::core::file!(), "`.")]
        #original_impl

        // Warnings for @param tags on impl blocks
        #param_warnings

        // C wrappers and call method defs
        #(#c_wrappers)*

        // Forwarding trait impls for as.<class>() coercion methods
        #trait_impls

        #constructor_check

        #(#cfg_attrs)*
        const _: () = {
            #demoted_static

            // The impl block's share of the R class, merged with the type's
            // other impl blocks by the wrapper writer.
            #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CLASS_BLOCKS), linkme(crate = ::miniextendr_api::linkme))]
            static CLASS_BLOCK: ::miniextendr_api::registry::ClassBlockEntry = #class_block;

            // Class name registration for cross-reference placeholder resolution.
            // Maps the Rust type name to the R-visible class name at link time.
            #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CLASS_NAMES), linkme(crate = ::miniextendr_api::linkme))]
            static CLASS_NAME: ::miniextendr_api::registry::ClassNameEntry =
                ::miniextendr_api::registry::ClassNameEntry {
                    rust_type: ::core::stringify!(#type_ident),
                    r_class_name: #r_class_name_str,
                    class_system: #class_system_str,
                    field_syntax: #field_syntax,
                };
        };
    };

    expanded.into()
}

/// The checks of the members a block generates, which `expand_impl` runs on
/// each rendering of the block.
fn check_class_members(parsed: &ParsedImpl) -> syn::Result<()> {
    // Every R formal must parse, be distinct, and leave the names the class
    // system's wrapper binds itself (receiver, R6 `self` / `private`) alone.
    check_method_formals(parsed)?;
    check_env_member_collisions(parsed)?;
    check_r6_initialize_collision(parsed)?;
    check_r6_static_collisions(parsed)?;
    check_s4_generic_collisions(parsed)?;
    // A replacement method (`s3(generic = "[[<-")`) takes `value` last.
    check_s3_replacement_methods(parsed)?;
    // Detect S7 fast-path shortcut name collisions (#986). The shortcut
    // `<ClassName>_<method>` shares a namespace with the standalone functions
    // emitted for static methods — an `r_name` override (or a static method
    // literally named like an instance method) can make two definitions clash,
    // with the last one silently winning. Fail loudly at compile time instead.
    check_s7_shortcut_collisions(parsed)?;
    // S7 only registers a method whose formals start with the generic's
    // dispatch arguments (`s7(dispatch)`, the `e1, e2` of an Ops operator);
    // anything else would stop the package from loading.
    check_s7_dispatch(parsed)
}

/// The block's R wrapper, in the parts the wrapper writer merges per type
/// (#1776), with the `as.<class>()` coercion methods (any class system).
fn generate_class_wrapper(parsed: &ParsedImpl) -> ClassWrapper {
    let mut class = match parsed.class_system {
        ClassSystem::Env => generate_env_class(parsed),
        ClassSystem::R6 => generate_r6_class(parsed),
        ClassSystem::S3 => generate_s3_class(parsed),
        ClassSystem::S7 => generate_s7_class(parsed),
        ClassSystem::S4 => generate_s4_class(parsed),
        ClassSystem::Vctrs => generate_vctrs_class(parsed),
    };
    let as_coercion_wrappers = generate_as_coercion_methods(parsed);
    if !as_coercion_wrappers.is_empty() {
        class.members.push(String::new());
        class.members.push(as_coercion_wrappers);
    }
    class
}

/// The compile-time half of the constructor decision. A block that declares
/// the constructor implements the hidden
/// `miniextendr_api::registry::ClassConstructor` for its type, under a
/// type private to the block. A block with several constructor candidates
/// and none declared requires that impl, once per candidate and spanned at
/// it, so the type compiles only when another of its impl blocks declares
/// the constructor; the error names each candidate. Two candidates in two
/// blocks are the wrapper writer's to refuse.
fn constructor_check(parsed: &ParsedImpl) -> TokenStream {
    let cfg_attrs = &parsed.cfg_attrs;
    let (impl_generics, _, where_clause) = parsed.original_impl.generics.split_for_impl();
    let self_ty = &parsed.original_impl.self_ty;
    if parsed.declared_constructor().is_some() {
        return quote! {
            #(#cfg_attrs)*
            const _: () = {
                // Named so that no user type spelled in `#self_ty` is shadowed.
                struct __MxConstructorBlock;
                impl #impl_generics
                    ::miniextendr_api::registry::ClassConstructor<__MxConstructorBlock>
                    for #self_ty #where_clause {}
            };
        };
    }
    if parsed.constructor_candidates().nth(1).is_none() {
        return TokenStream::new();
    }
    // The type spanned at each candidate, so the error points there.
    let requirements = parsed.constructor_candidates().map(|m| {
        let mut ty = parsed.type_ident.clone();
        ty.set_span(m.ident.span());
        quote::quote_spanned! {m.ident.span()=>
            ::miniextendr_api::registry::require_class_constructor::<#ty, _>();
        }
    });
    quote! {
        #(#cfg_attrs)*
        const _: () = {
            #[allow(dead_code)]
            fn constructor_check() {
                #(#requirements)*
            }
        };
    }
}

#[cfg(test)]
mod tests;
// endregion
