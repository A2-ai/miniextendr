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
use quote::{format_ident, quote};

/// Strip miniextendr attributes (and roxygen tags) from an impl block and its items.
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
}

impl ReceiverKind {
    /// Returns true if this is an instance method (has self).
    pub fn is_instance(&self) -> bool {
        matches!(self, ReceiverKind::Ref | ReceiverKind::RefMut)
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
    /// Method identifier
    pub ident: syn::Ident,
    /// Receiver kind
    pub env: ReceiverKind,
    /// Method signature (without env)
    pub sig: syn::Signature,
    /// Visibility
    pub vis: syn::Visibility,
    /// Roxygen tag lines extracted from Rust doc comments
    pub doc_tags: Vec<String>,
    /// Per-method attributes for class system overrides
    pub method_attrs: MethodAttrs,
    /// Parameter default values from `#[miniextendr(default = "...")]`
    pub param_defaults: std::collections::HashMap<String, String>,
}

/// Per-method attributes for class system customization.
#[derive(Debug, Default)]
pub struct MethodAttrs {
    /// Skip this method
    pub ignore: bool,
    /// Mark as constructor
    pub constructor: bool,
    /// Mark as finalizer (R6)
    pub finalize: bool,
    /// Mark as private (R6)
    pub private: bool,
    /// Mark as active binding getter (R6)
    pub active: bool,
    /// R6 active binding setter marker.
    ///
    /// Use `#[miniextendr(r6(setter, prop = "name"))]` to mark a method as an R6 active
    /// binding setter. The property name must match a getter to create a combined binding.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(r6(active))]  // or r6(active, prop = "len")
    /// fn length(&self) -> i32 { self.data.len() as i32 }
    ///
    /// #[miniextendr(r6(setter, prop = "length"))]
    /// fn set_length(&mut self, value: i32) { self.data.resize(value as usize, 0); }
    /// // Generates combined active binding:
    /// // length = function(value) { if (missing(value)) get_length() else set_length(value) }
    /// ```
    pub r6_setter: bool,
    /// R6 property name for active bindings (defaults to method name).
    ///
    /// When specified via `#[miniextendr(r6(active, prop = "name"))]`, overrides the
    /// default property name which is derived from the method name.
    pub r6_prop: Option<String>,
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
    /// Span of `active` for error reporting.
    pub active_span: Option<proc_macro2::Span>,
    /// S7 property getter marker.
    ///
    /// Use `#[miniextendr(s7(getter))]` or `#[miniextendr(s7(getter, prop = "name"))]` to mark
    /// a method as an S7 property getter. The generated S7 class will include a computed
    /// property with this getter.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(getter))]
    /// fn length(&self) -> i32 { self.data.len() as i32 }
    /// // Generates: length = new_property(getter = function(self) ...)
    /// ```
    pub s7_getter: bool,
    /// S7 property setter marker.
    ///
    /// Use `#[miniextendr(s7(setter, prop = "name"))]` to mark a method as an S7 property
    /// setter. The property name must match a getter to create a dynamic property.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(getter, prop = "len"))]
    /// fn length(&self) -> i32 { self.data.len() as i32 }
    ///
    /// #[miniextendr(s7(setter, prop = "len"))]
    /// fn set_length(&mut self, value: i32) { self.data.resize(value as usize, 0); }
    /// // Generates: len = new_property(getter = ..., setter = ...)
    /// ```
    pub s7_setter: bool,
    /// S7 property name (defaults to method name).
    ///
    /// When specified via `#[miniextendr(s7(getter, prop = "name"))]`, overrides the
    /// default property name which is derived from the method name.
    pub s7_prop: Option<String>,
    /// S7 property default value (R expression).
    ///
    /// Use `#[miniextendr(s7(getter, default = "0.0"))]` to set a default value.
    /// The value is an R expression that will be used as the `default` parameter
    /// in `new_property()`.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(getter, default = "0.0"))]
    /// fn score(&self) -> f64 { self.score }
    /// // Generates: score = new_property(class = class_double, default = 0.0, getter = ...)
    /// ```
    pub s7_default: Option<String>,
    /// S7 property validator marker.
    ///
    /// Use `#[miniextendr(s7(validate, prop = "name"))]` to mark a method as a property
    /// validator. The method should take a value and return `Result<(), String>` or
    /// return nothing and panic on invalid input.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(validate, prop = "score"))]
    /// fn validate_score(value: f64) -> Result<(), String> {
    ///     if value < 0.0 || value > 100.0 {
    ///         Err("score must be between 0 and 100".into())
    ///     } else {
    ///         Ok(())
    ///     }
    /// }
    /// ```
    pub s7_validate: bool,
    /// S7 property required marker.
    ///
    /// Use `#[miniextendr(s7(getter, required))]` to mark a property as required.
    /// This generates `default = quote(stop("@name is required"))` in R.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(getter, required))]
    /// fn id(&self) -> String { self.id.clone() }
    /// // Generates: id = new_property(default = quote(stop("@id is required")), ...)
    /// ```
    pub s7_required: bool,
    /// S7 property frozen marker.
    ///
    /// Use `#[miniextendr(s7(getter, frozen))]` to mark a property that can only
    /// be set once. After the initial value is set, attempts to change it will error.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(getter, frozen))]
    /// fn created_at(&self) -> f64 { self.created_at }
    /// ```
    pub s7_frozen: bool,
    /// S7 property deprecated marker.
    ///
    /// Use `#[miniextendr(s7(getter, deprecated = "message"))]` to mark a property
    /// as deprecated. Getter and setter will emit deprecation warnings.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(getter, deprecated = "Use 'value' instead"))]
    /// fn old_value(&self) -> i32 { self.value }
    /// ```
    pub s7_deprecated: Option<String>,
    // =========================================================================
    // S7 Phase 3: Generic dispatch control
    // =========================================================================
    /// S7 no_dots marker - removes `...` from generic signature.
    ///
    /// Use for strict generics like `length()` that don't accept extra args.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(no_dots))]
    /// fn length(&self) -> i32 { self.data.len() as i32 }
    /// // Generates: new_generic("length", "x", function(x) S7_dispatch())
    /// // Instead of: new_generic("length", "x", function(x, ...) S7_dispatch())
    /// ```
    pub s7_no_dots: bool,
    /// S7 multiple dispatch - specifies dispatch arguments.
    ///
    /// Use `#[miniextendr(s7(dispatch = "x,y"))]` to enable double dispatch.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(dispatch = "x,y"))]
    /// fn compare(&self, other: &OtherType) -> i32 { ... }
    /// // Generates: new_generic("compare", c("x", "y"), function(x, y, ...) S7_dispatch())
    /// ```
    pub s7_dispatch: Option<String>,
    /// S7 fallback marker - register method for class_any.
    ///
    /// Use for fallback implementations that handle any type.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(fallback))]
    /// fn describe(&self) -> String { "unknown".to_string() }
    /// // Registers method for S7::class_any
    /// ```
    pub s7_fallback: bool,
    // =========================================================================
    // S7 Phase 4: Conversion support
    // =========================================================================
    /// S7 convert_from - marks a method that converts FROM another type.
    ///
    /// Use `#[miniextendr(s7(convert_from = "OtherType"))]` on a static method
    /// that takes OtherType and returns Self. This generates an S7 convert() method.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(convert_from = "Point2D"))]
    /// fn from_2d(p: &Point2D) -> Self {
    ///     Point3D { x: p.x, y: p.y, z: 0.0 }
    /// }
    /// // Generates: S7::method(convert, list(Point2D, Point3D)) <- function(from, to) ...
    /// ```
    pub s7_convert_from: Option<String>,
    /// S7 convert_to - marks a method that converts TO another type.
    ///
    /// Use `#[miniextendr(s7(convert_to = "OtherType"))]` on an instance method
    /// that returns OtherType. This generates an S7 convert() method.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s7(convert_to = "Point2D"))]
    /// fn to_2d(&self) -> Point2D {
    ///     Point2D { x: self.x, y: self.y }
    /// }
    /// // Generates: S7::method(convert, list(Point3D, Point2D)) <- function(from, to) ...
    /// ```
    pub s7_convert_to: Option<String>,
    // =========================================================================
    // Lifecycle support
    // =========================================================================
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
    /// Mark as R6 deep_clone method.
    ///
    /// Use `#[miniextendr(r6(deep_clone))]` to mark a method as the R6 deep clone handler.
    /// This method will be wired into `private$deep_clone` in the R6Class definition.
    pub deep_clone: bool,
    /// vctrs protocol method override.
    ///
    /// Use `#[miniextendr(vctrs(format))]` to mark a method as implementing a vctrs
    /// protocol S3 generic. The method will be generated as `format.<class>` instead
    /// of the default Rust method name.
    ///
    /// Supported protocols: format, vec_proxy, vec_proxy_equal, vec_proxy_compare,
    /// vec_proxy_order, vec_restore, obj_print_data, obj_print_header, obj_print_footer.
    pub vctrs_protocol: Option<String>,
}

/// Parsed impl block with all methods.
#[derive(Debug)]
pub struct ParsedImpl {
    /// Type being implemented
    pub type_ident: syn::Ident,
    /// Class system to use
    pub class_system: ClassSystem,
    /// Override class name (else type name)
    pub class_name: Option<String>,
    /// Optional label for distinguishing multiple impl blocks of the same type.
    pub label: Option<String>,
    /// Roxygen tag lines extracted from impl doc comments
    pub doc_tags: Vec<String>,
    /// All parsed methods
    pub methods: Vec<ParsedMethod>,
    /// Original impl item for re-emission
    pub original_impl: syn::ItemImpl,
    /// cfg attributes to propagate
    pub cfg_attrs: Vec<syn::Attribute>,
    /// vctrs-specific attributes (only used when class_system is Vctrs)
    pub vctrs_attrs: VctrsAttrs,
    // R6-specific configuration (propagated from ImplAttrs)
    pub r6_inherit: Option<String>,
    pub r6_portable: Option<bool>,
    pub r6_cloneable: Option<bool>,
    pub r6_lock_objects: Option<bool>,
    pub r6_lock_class: Option<bool>,
    // S7-specific configuration (propagated from ImplAttrs)
    pub s7_parent: Option<String>,
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
}

/// Attributes on the impl block itself.
#[derive(Debug)]
pub struct ImplAttrs {
    pub class_system: ClassSystem,
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
    // =========================================================================
    // R6-specific configuration
    // =========================================================================
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
    // =========================================================================
    // S7-specific configuration
    // =========================================================================
    /// S7 parent class for inheritance.
    /// Use `#[miniextendr(s7(parent = "ParentClass"))]` to specify the parent.
    pub s7_parent: Option<String>,
    /// S7 abstract class flag. Abstract classes cannot be instantiated.
    pub s7_abstract: bool,
    // =========================================================================
    // Sidecar integration
    // =========================================================================
    /// When true, auto-include `#[r_data]` field accessors in the class definition.
    /// For R6: active bindings via `$set("active", ...)` post-creation.
    /// For S7: properties spliced from `.rdata_properties_{Type}`.
    pub r_data_accessors: bool,
    // =========================================================================
    // Strict conversion mode
    // =========================================================================
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
        // #[...(active)] is only meaningful for R6
        if attrs.active && class_system != ClassSystem::R6 {
            return Err(syn::Error::new(
                attrs.active_span.unwrap_or(span),
                "#[r6(active)] is only valid for R6 class systems",
            ));
        }

        // convert_from and convert_to are mutually exclusive on the same method
        // - convert_from expects a static method (no &self, takes source type)
        // - convert_to expects an instance method (&self, returns target type)
        if attrs.s7_convert_from.is_some() && attrs.s7_convert_to.is_some() {
            return Err(syn::Error::new(
                span,
                "cannot specify both `convert_from` and `convert_to` on the same method; \
                 convert_from is for static methods, convert_to is for instance methods",
            ));
        }

        // Worker attribute is now supported on methods
        // (validation happens during wrapper generation based on return type)

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
                            method_attrs.finalize = true;
                        } else if inner.path.is_ident("private") {
                            method_attrs.private = true;
                        } else if inner.path.is_ident("active") {
                            use syn::spanned::Spanned;
                            method_attrs.active = true;
                            method_attrs.active_span = Some(inner.path.span());
                        } else if inner.path.is_ident("setter") {
                            // Active binding setter: works for both R6 and S7
                            method_attrs.r6_setter = true;
                            method_attrs.s7_setter = true;
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
                            method_attrs.s7_getter = true;
                        } else if inner.path.is_ident("validate") {
                            method_attrs.s7_validate = true;
                        } else if inner.path.is_ident("prop") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            let prop_value = value.value();
                            // Set both S7 and R6 prop - the class system will use the appropriate one
                            method_attrs.s7_prop = Some(prop_value.clone());
                            method_attrs.r6_prop = Some(prop_value);
                        } else if inner.path.is_ident("default") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7_default = Some(value.value());
                        } else if inner.path.is_ident("required") {
                            method_attrs.s7_required = true;
                        } else if inner.path.is_ident("frozen") {
                            method_attrs.s7_frozen = true;
                        } else if inner.path.is_ident("deprecated") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7_deprecated = Some(value.value());
                        } else if inner.path.is_ident("no_dots") {
                            method_attrs.s7_no_dots = true;
                        } else if inner.path.is_ident("dispatch") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7_dispatch = Some(value.value());
                        } else if inner.path.is_ident("fallback") {
                            method_attrs.s7_fallback = true;
                        } else if inner.path.is_ident("convert_from") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7_convert_from = Some(value.value());
                        } else if inner.path.is_ident("convert_to") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.s7_convert_to = Some(value.value());
                        } else if inner.path.is_ident("deep_clone") {
                            method_attrs.deep_clone = true;
                        } else {
                            return Err(inner.error(
                                "unknown method option; expected one of: ignore, constructor, finalize, private, active, worker, no_worker, main_thread, no_main_thread, check_interrupt, coerce, no_coerce, rng, unwrap_in_r, error_in_r, no_error_in_r, generic, class, getter, setter, validate, prop, default, required, frozen, deprecated, no_dots, dispatch, fallback, convert_from, convert_to, deep_clone"
                            ));
                        }
                        Ok(())
                    })?;
                } else if meta.path.is_ident("defaults") {
                    // Capture span for error reporting
                    use syn::spanned::Spanned;
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
                    use syn::spanned::Spanned;
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
                } else {
                    return Err(meta.error(
                        "unknown attribute; expected one of: env, r6, s3, s4, s7, vctrs, defaults, unsafe, check_interrupt, coerce, no_coerce, rng, unwrap_in_r, error_in_r, no_error_in_r, as, lifecycle"
                    ));
                }
                Ok(())
            })?;
        }

        // Resolve feature defaults for fields not explicitly set
        method_attrs.worker = worker.unwrap_or(cfg!(feature = "default-worker"));
        method_attrs.unsafe_main_thread =
            unsafe_main_thread.unwrap_or(cfg!(feature = "default-main-thread"));
        method_attrs.coerce = coerce.unwrap_or(cfg!(feature = "default-coerce"));
        let resolved_error_in_r = error_in_r.unwrap_or(cfg!(feature = "default-error-in-r"));
        if resolved_error_in_r && method_attrs.unwrap_in_r {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`error_in_r` (from `default-error-in-r` feature) and `unwrap_in_r` are mutually exclusive; use `no_error_in_r` to opt out",
            ));
        }
        method_attrs.error_in_r = resolved_error_in_r;

        Ok(method_attrs)
    }

    /// Detect env kind from function signature.
    ///
    /// Handles both standard (`&self`, `&mut self`) and typed (`self: &Self`, `self: &mut Self`) receivers.
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
                    // Check for typed receiver (self: &Self, self: &mut Self)
                    if let syn::Type::Reference(type_ref) = r.ty.as_ref() {
                        if type_ref.mutability.is_some() {
                            ReceiverKind::RefMut
                        } else {
                            ReceiverKind::Ref
                        }
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

    /// Create signature without env (for C wrapper generation).
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
        let env = Self::detect_env(&item.sig);
        let mut method_attrs = Self::parse_method_attrs(&item.attrs)?;

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

        // Validate: `self` by value (consuming) methods are not fully supported
        // They're either: constructor (returns Self), finalizer (marked or inferred), or error
        if env == ReceiverKind::Value {
            let returns_self = matches!(&item.sig.output, syn::ReturnType::Type(_, ty)
                if matches!(ty.as_ref(), syn::Type::Path(p)
                    if p.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false)));

            // Allow if: constructor (returns Self) or explicitly marked as finalize
            let is_allowed = returns_self || method_attrs.constructor || method_attrs.finalize;

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
        if self.method_attrs.private {
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
        self.method_attrs.finalize || (self.env == ReceiverKind::Value && !self.returns_self())
    }

    /// Returns true if this method should be an R6 active binding.
    /// Active bindings provide property-like access (obj$name instead of obj$name()).
    pub fn is_active(&self) -> bool {
        self.method_attrs.active
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

    /// Call method def identifier for registration.
    ///
    /// Format: `call_method_def_{type}_{method}` or `call_method_def_{type}_{label}_{method}` if labeled.
    pub fn call_method_def_ident(
        &self,
        type_ident: &syn::Ident,
        label: Option<&str>,
    ) -> syn::Ident {
        if let Some(label) = label {
            format_ident!("call_method_def_{}_{}_{}", type_ident, label, self.ident)
        } else {
            format_ident!("call_method_def_{}_{}", type_ident, self.ident)
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
        let doc_tags = crate::roxygen::roxygen_tags_from_attrs(&item_impl.attrs);

        Ok(ParsedImpl {
            type_ident,
            class_system: attrs.class_system,
            class_name: attrs.class_name,
            label: attrs.label,
            doc_tags,
            methods,
            // Strip miniextendr attributes (and roxygen tags) before re-emitting.
            original_impl: strip_miniextendr_attrs_from_impl(item_impl),
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
            .find(|m| m.should_include() && m.is_constructor())
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
                && !m.method_attrs.r6_setter // Exclude setters
        })
    }

    /// Get active binding setter methods for R6 (have env, marked as r6_setter).
    pub fn active_setter_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods
            .iter()
            .filter(|m| m.should_include() && m.env.is_instance() && m.method_attrs.r6_setter)
    }

    /// Find the setter method for a given property name.
    pub fn find_setter_for_prop(&self, prop_name: &str) -> Option<&ParsedMethod> {
        self.active_setter_methods().find(|m| {
            // Match by explicit prop name or by method name with "set_" prefix removed
            if let Some(ref explicit_prop) = m.method_attrs.r6_prop {
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
                && !m.is_constructor()
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

    /// Module constant identifier for all call method defs.
    ///
    /// Format: `{TYPE}_CALL_DEFS` or `{TYPE}_{LABEL}_CALL_DEFS` if labeled.
    pub fn call_defs_const_ident(&self) -> syn::Ident {
        let type_upper = self.type_ident.to_string().to_uppercase();
        if let Some(ref label) = self.label {
            let label_upper = label.to_uppercase();
            format_ident!("{}_{}_CALL_DEFS", type_upper, label_upper)
        } else {
            format_ident!("{}_CALL_DEFS", type_upper)
        }
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

/// Generate C wrapper for a method.
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
    // Static methods use worker thread by default, main thread only when explicitly requested
    let thread_strategy = if method.method_attrs.unsafe_main_thread || method.env.is_instance() {
        ThreadStrategy::MainThread
    } else {
        ThreadStrategy::WorkerThread
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
        let self_extraction = if method.env == ReceiverKind::RefMut {
            quote! {
                let mut self_ptr = unsafe {
                    ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(self_sexp)
                };
                let self_ref = self_ptr.downcast_mut::<#type_ident>()
                    .expect(concat!("expected ExternalPtr<", stringify!(#type_ident), ">"));
            }
        } else {
            quote! {
                let self_ptr = unsafe {
                    ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(self_sexp)
                };
                let self_ref = self_ptr.downcast_ref::<#type_ident>()
                    .expect(concat!("expected ExternalPtr<", stringify!(#type_ident), ">"));
            }
        };
        vec![self_extraction]
    } else {
        vec![]
    };

    // Generate call expression
    let call_expr = if method.env.is_instance() {
        quote! { self_ref.#method_ident(#(#rust_args),*) }
    } else {
        quote! { #type_ident::#method_ident(#(#rust_args),*) }
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

    builder.build().generate()
}

/// Returns true when the return type is syntactically `Result<_, _>`.
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

/// Generate R wrapper string for env-style class.
pub fn generate_env_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    // Check if class has @noRd - if so, skip method documentation
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(&parsed_impl.doc_tags, "noRd");

    let mut lines = Vec::new();

    // Class environment documentation and definition
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, &parsed_impl.doc_tags, "")
            .with_export_control(parsed_impl.internal, parsed_impl.noexport)
            .build(),
    );
    // Inject lifecycle imports from methods into class-level roxygen block
    if let Some(lc_import) = crate::lifecycle::collect_lifecycle_imports(
        parsed_impl
            .methods
            .iter()
            .filter_map(|m| m.method_attrs.lifecycle.as_ref()),
    ) {
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(insert_pos, format!("#' {}", lc_import));
    }
    lines.push(format!("{} <- new.env(parent = emptyenv())", class_name));
    lines.push(String::new());

    // Constructor
    if let Some(ctx) = parsed_impl.constructor_context() {
        // Skip method documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, "new", type_ident, &ctx.method.doc_tags)
                    .with_name_prefix("$")
                    .with_params_as_details();
            lines.extend(method_doc.build());
        }
        lines.push(format!("{}$new <- function({}) {{", class_name, ctx.params));
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        lines.push(format!("  self <- {}", ctx.static_call()));
        lines.push(format!("  class(self) <- \"{}\"", class_name));
        lines.push("  self".to_string());
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods
    for ctx in parsed_impl.instance_method_contexts() {
        let method_name = ctx.method.ident.to_string();
        // Skip method documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_name_prefix("$")
                    .with_params_as_details();
            lines.extend(method_doc.build());
        }

        lines.push(format!(
            "{}${} <- function({}) {{",
            class_name, method_name, ctx.params
        ));

        // Inject lifecycle prelude if present
        let what = format!("{}${}", class_name, method_name);
        if let Some(prelude) = ctx.method.lifecycle_prelude(&what) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }

        let call = ctx.instance_call("self");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Static methods
    for ctx in parsed_impl.static_method_contexts() {
        let method_name = ctx.method.ident.to_string();
        // Skip method documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_name_prefix("$")
                    .with_params_as_details();
            lines.extend(method_doc.build());
        }

        lines.push(format!(
            "{}${} <- function({}) {{",
            class_name, method_name, ctx.params
        ));

        // Inject lifecycle prelude if present
        let what = format!("{}${}", class_name, method_name);
        if let Some(prelude) = ctx.method.lifecycle_prelude(&what) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // $ dispatch - export as S3 methods
    // Handles both functions (inherent methods) and environments (trait namespaces)
    let has_internal = crate::roxygen::has_roxygen_tag(&parsed_impl.doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !has_internal && !parsed_impl.noexport;

    // Generate roxygen tags for dispatch methods
    if class_has_no_rd {
        // For internal classes, add @noRd to suppress roxygen2 S3 method detection
        lines.push("#' @noRd".to_string());
    } else {
        lines.push(format!("#' @rdname {}", class_name));
        lines.push("#' @param self The object instance.".to_string());
        lines.push("#' @param name Method name for dispatch.".to_string());
        if should_export {
            lines.push("#' @export".to_string());
        }
    }
    lines.push(format!("`$.{}` <- function(self, name) {{", class_name));
    lines.push(format!("  obj <- {}[[name]]", class_name));
    lines.push("  if (is.environment(obj)) {".to_string());
    lines.push("    # Trait namespace - wrap instance methods to prepend self".to_string());
    lines.push("    bound <- new.env(parent = emptyenv())".to_string());
    lines.push("    for (method_name in names(obj)) {".to_string());
    lines.push("      method <- obj[[method_name]]".to_string());
    lines.push("      if (is.function(method)) {".to_string());
    lines.push("        if (isTRUE(attr(method, \".__mx_instance__\"))) {".to_string());
    lines.push("          local({".to_string());
    lines.push("            m <- method".to_string());
    lines.push("            bound[[method_name]] <<- function(...) m(self, ...)".to_string());
    lines.push("          })".to_string());
    lines.push("        } else {".to_string());
    lines.push("          bound[[method_name]] <- method".to_string());
    lines.push("        }".to_string());
    lines.push("      }".to_string());
    lines.push("    }".to_string());
    lines.push("    bound".to_string());
    lines.push("  } else if (is.null(obj)) {".to_string());
    lines.push("    # Not found at top level — search trait namespace environments".to_string());
    lines.push(format!("    for (ns_name in names({})) {{", class_name));
    lines.push(format!("      ns <- {}[[ns_name]]", class_name));
    lines.push(
        "      if (is.environment(ns) && exists(name, envir = ns, inherits = FALSE)) {".to_string(),
    );
    lines.push("        method <- ns[[name]]".to_string());
    lines.push(
        "        if (is.function(method) && isTRUE(attr(method, \".__mx_instance__\"))) {"
            .to_string(),
    );
    lines.push("          # Instance method — bind self as first arg".to_string());
    lines.push("          m <- method".to_string());
    lines.push("          s <- self".to_string());
    lines.push("          return(function(...) m(s, ...))".to_string());
    lines.push("        } else if (is.function(method)) {".to_string());
    lines.push("          return(method)".to_string());
    lines.push("        }".to_string());
    lines.push("      }".to_string());
    lines.push("    }".to_string());
    lines.push("    NULL".to_string());
    lines.push("  } else {".to_string());
    lines.push("    environment(obj) <- environment()".to_string());
    lines.push("    obj".to_string());
    lines.push("  }".to_string());
    lines.push("}".to_string());
    if class_has_no_rd {
        lines.push("#' @noRd".to_string());
    } else {
        lines.push(format!("#' @rdname {}", class_name));
        if should_export {
            lines.push("#' @export".to_string());
        }
    }
    lines.push(format!("`[[.{}` <- `$.{}`", class_name, class_name));

    lines.join("\n")
}

/// Generate R wrapper string for R6-style class.
///
/// Creates an R6::R6Class with:
/// - `initialize` method that calls the Rust `new` function (or accepts `.ptr` directly)
/// - Public methods for all instance methods
/// - Private `.ptr` field holding the ExternalPtr
pub fn generate_r6_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;

    // Check if .ptr parameter will be added to initialize (for static methods returning Self)
    let has_self_returning_methods = parsed_impl
        .methods
        .iter()
        .filter(|m| m.should_include())
        .any(|m| m.returns_self());

    let mut lines = Vec::new();

    // Start R6Class definition with documentation
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, class_doc_tags, "R6")
            .with_imports("@importFrom R6 R6Class")
            .with_export_control(parsed_impl.internal, parsed_impl.noexport)
            .build(),
    );
    // Inject lifecycle imports from methods into class-level roxygen block
    if let Some(lc_import) = crate::lifecycle::collect_lifecycle_imports(
        parsed_impl
            .methods
            .iter()
            .filter_map(|m| m.method_attrs.lifecycle.as_ref()),
    ) {
        // Insert before @export (which is last)
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(insert_pos, format!("#' {}", lc_import));
    }

    // Document .ptr param if initialize will have it (for static methods returning Self)
    if has_self_returning_methods && !crate::roxygen::has_roxygen_tag(class_doc_tags, "param .ptr")
    {
        // Insert before @export (which is last)
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(
            insert_pos,
            "#' @param .ptr Internal pointer (used by static methods, not for direct use)."
                .to_string(),
        );
    }
    // R6Class definition — optionally include inherit
    if let Some(ref parent) = parsed_impl.r6_inherit {
        lines.push(format!(
            "{} <- R6::R6Class(\"{}\", inherit = {},",
            class_name, class_name, parent
        ));
    } else {
        lines.push(format!("{} <- R6::R6Class(\"{}\",", class_name, class_name));
    }

    // Portable flag (only emit if explicitly set to FALSE, since TRUE is default)
    if parsed_impl.r6_portable == Some(false) {
        lines.push("  portable = FALSE,".to_string());
    }

    // Public list
    lines.push("  public = list(".to_string());

    // Public instance methods (collect first to know if we need trailing comma on initialize)
    let public_method_contexts: Vec<_> = parsed_impl.public_instance_method_contexts().collect();
    let has_public_methods = !public_method_contexts.is_empty();

    // Constructor (initialize) - accepts either normal params or a pre-made .ptr
    if let Some(ctx) = parsed_impl.constructor_context() {
        // Add inline roxygen documentation for initialize method
        // Note: @title is replaced with @description for R6 inline docs (roxygen requirement)
        for tag in &ctx.method.doc_tags {
            for line in tag.lines() {
                let line = if line.starts_with("@title ") {
                    line.replacen("@title ", "@description ", 1)
                } else {
                    line.to_string()
                };
                lines.push(format!("    #' {}", line));
            }
        }

        // Only add trailing comma if there are public methods after initialize
        let comma = if has_public_methods { "," } else { "" };

        // Precondition checks for constructor parameters
        let ctor_preconditions = ctx.precondition_checks();

        if has_self_returning_methods {
            let full_params = if ctx.params.is_empty() {
                ".ptr = NULL".to_string()
            } else {
                format!("{}, .ptr = NULL", ctx.params)
            };
            lines.push(format!("    initialize = function({}) {{", full_params));
            // Only check preconditions when not using .ptr shortcut
            if !ctor_preconditions.is_empty() {
                lines.push("      if (is.null(.ptr)) {".to_string());
                for check in &ctor_preconditions {
                    lines.push(format!("        {}", check));
                }
                lines.push("      }".to_string());
            }
            lines.push("      if (!is.null(.ptr)) {".to_string());
            lines.push("        private$.ptr <- .ptr".to_string());
            lines.push("      } else {".to_string());
            lines.push(format!("        private$.ptr <- {}", ctx.static_call()));
            lines.push("      }".to_string());
            lines.push(format!("    }}{}", comma));
        } else {
            lines.push(format!("    initialize = function({}) {{", ctx.params));
            for check in &ctor_preconditions {
                lines.push(format!("      {}", check));
            }
            lines.push(format!("      private$.ptr <- {}", ctx.static_call()));
            lines.push(format!("    }}{}", comma));
        }
    }

    // Public instance methods
    for (i, ctx) in public_method_contexts.iter().enumerate() {
        let comma = if i < public_method_contexts.len() - 1 {
            ","
        } else {
            ""
        };

        // Add inline roxygen documentation for this method
        // Note: @title is replaced with @description for R6 inline docs (roxygen requirement)
        for tag in &ctx.method.doc_tags {
            for line in tag.lines() {
                let line = if line.starts_with("@title ") {
                    line.replacen("@title ", "@description ", 1)
                } else {
                    line.to_string()
                };
                lines.push(format!("    #' {}", line));
            }
        }

        lines.push(format!(
            "    {} = function({}) {{",
            ctx.method.ident, ctx.params
        ));

        // Inject lifecycle prelude if present
        let what = format!("{}${}", class_name, ctx.method.ident);
        if let Some(prelude) = ctx.method.lifecycle_prelude(&what) {
            lines.push(format!("      {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("      {}", check));
        }

        let call = ctx.instance_call("private$.ptr");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r)
            .with_indent(6); // R6 methods have 6-space indent
        lines.extend(return_builder.build_r6_body());

        lines.push(format!("    }}{}", comma));
    }

    lines.push("  ),".to_string());

    // Private list - includes .ptr and any private methods
    lines.push("  private = list(".to_string());

    // Private instance methods
    for ctx in parsed_impl.private_instance_method_contexts() {
        lines.push(format!(
            "    {} = function({}) {{",
            ctx.method.ident, ctx.params
        ));

        let call = ctx.instance_call("private$.ptr");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r)
            .with_indent(6);
        lines.extend(return_builder.build_r6_body());

        lines.push("    },".to_string());
    }

    // Finalizer (if any)
    if let Some(finalizer) = parsed_impl.finalizer() {
        let c_ident = finalizer.c_wrapper_ident(type_ident, parsed_impl.label());
        lines.push(format!(
            "    finalize = function() .Call({}, .call = match.call(), private$.ptr),",
            c_ident
        ));
    }

    // deep_clone (if any method marked with #[miniextendr(r6(deep_clone))])
    if let Some(dc_method) = parsed_impl
        .methods
        .iter()
        .find(|m| m.method_attrs.deep_clone && m.should_include())
    {
        let c_ident = dc_method.c_wrapper_ident(type_ident, parsed_impl.label());
        lines.push(format!(
            "    deep_clone = function(name, value) .Call({}, .call = match.call(), private$.ptr, name, value),",
            c_ident
        ));
    }

    // .ptr field (always last, no trailing comma)
    lines.push("    .ptr = NULL".to_string());
    lines.push("  ),".to_string());

    // Active bindings list (for property-like access)
    let active_method_contexts: Vec<_> = parsed_impl.active_instance_method_contexts().collect();
    if !active_method_contexts.is_empty() {
        lines.push("  active = list(".to_string());

        for (i, ctx) in active_method_contexts.iter().enumerate() {
            let comma = if i < active_method_contexts.len() - 1 {
                ","
            } else {
                ""
            };

            // Add inline @field documentation for active bindings
            // roxygen2 requires @field tags (not @description) for active bindings
            let method_name = ctx.method.ident.to_string();
            for tag in &ctx.method.doc_tags {
                for (line_idx, line) in tag.lines().enumerate() {
                    // Convert @description/@title to @field on first line only
                    let line = if line_idx == 0 {
                        if let Some(desc) = line.strip_prefix("@description ") {
                            format!("@field {} {}", method_name, desc)
                        } else if let Some(desc) = line.strip_prefix("@title ") {
                            format!("@field {} {}", method_name, desc)
                        } else if !line.starts_with('@') {
                            // Plain doc comment - treat as field description
                            format!("@field {} {}", method_name, line)
                        } else {
                            line.to_string()
                        }
                    } else {
                        // Continuation lines stay as-is
                        line.to_string()
                    };
                    lines.push(format!("    #' {}", line));
                }
            }

            // Determine the property name (from r6_prop or method name)
            let prop_name = ctx
                .method
                .method_attrs
                .r6_prop
                .clone()
                .unwrap_or_else(|| ctx.method.ident.to_string());

            // Check if there's a matching setter for this property
            let setter = parsed_impl.find_setter_for_prop(&prop_name);

            if let Some(setter_method) = setter {
                // Combined getter/setter active binding
                // Format: name = function(value) { if (missing(value)) getter else setter }
                lines.push(format!("    {} = function(value) {{", prop_name));
                lines.push("      if (missing(value)) {".to_string());

                // Getter call
                let getter_call = ctx.instance_call("private$.ptr");
                lines.push(format!("        {}", getter_call));

                lines.push("      } else {".to_string());

                // Setter call - construct directly
                let setter_c_ident =
                    setter_method.c_wrapper_ident(type_ident, parsed_impl.label.as_deref());
                let setter_call = format!(
                    ".Call({}, .call = match.call(), private$.ptr, value)",
                    setter_c_ident
                );
                lines.push(format!("        {}", setter_call));
                lines.push("        invisible(self)".to_string());

                lines.push("      }".to_string());
                lines.push(format!("    }}{}", comma));
            } else {
                // Getter-only active binding (no parameters besides self)
                // Format: name = function() { ... }
                lines.push(format!("    {} = function() {{", prop_name));

                let call = ctx.instance_call("private$.ptr");
                let strategy = crate::ReturnStrategy::for_method(ctx.method);
                let return_builder = crate::MethodReturnBuilder::new(call)
                    .with_strategy(strategy)
                    .with_class_name(class_name.clone())
                    .with_error_in_r(ctx.method.method_attrs.error_in_r)
                    .with_indent(6); // R6 active bindings have 6-space indent
                lines.extend(return_builder.build_r6_body());

                lines.push(format!("    }}{}", comma));
            }
        }

        lines.push("  ),".to_string());
    }

    // Class options
    let lock_objects = parsed_impl.r6_lock_objects.unwrap_or(true);
    let lock_class = parsed_impl.r6_lock_class.unwrap_or(false);
    let cloneable = parsed_impl.r6_cloneable.unwrap_or(false);
    lines.push(format!(
        "  lock_objects = {},",
        if lock_objects { "TRUE" } else { "FALSE" }
    ));
    lines.push(format!(
        "  lock_class = {},",
        if lock_class { "TRUE" } else { "FALSE" }
    ));
    lines.push(format!(
        "  cloneable = {}",
        if cloneable { "TRUE" } else { "FALSE" }
    ));
    lines.push(")".to_string());

    // If r_data_accessors is set, apply sidecar active bindings from #[derive(ExternalPtr)]
    if parsed_impl.r_data_accessors {
        let type_name = type_ident.to_string();
        lines.push(format!(
            ".rdata_active_bindings_{}({})",
            type_name, class_name
        ));
    }

    // Check if class has @noRd
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");

    // Static methods as separate functions on the class object
    for ctx in parsed_impl.static_method_contexts() {
        let method_name = ctx.method.ident.to_string();
        let static_method_name = format!("{}${}", class_name, method_name);
        lines.push(String::new());

        let method_doc =
            MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                .with_name_prefix("$")
                .with_class_no_rd(class_has_no_rd);
        lines.extend(method_doc.build());

        lines.push(format!(
            "{} <- function({}) {{",
            static_method_name, ctx.params
        ));

        // Inject lifecycle prelude if present
        let what = format!("{}${}", class_name, method_name);
        if let Some(prelude) = ctx.method.lifecycle_prelude(&what) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build_r6_body());

        lines.push("}".to_string());
    }

    lines.join("\n")
}

/// Generate R wrapper string for S3-style class.
///
/// Creates:
/// - Constructor function `new_<class>()` that returns an ExternalPtr with class attribute
/// - S3 generic methods `<method>.<class>` for each instance method
pub fn generate_s3_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    // S3 convention: lowercase constructor name
    let ctor_name = format!("new_{}", class_name.to_lowercase());
    let class_doc_tags = &parsed_impl.doc_tags;
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let class_has_internal = crate::roxygen::has_roxygen_tag(class_doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !class_has_internal && !parsed_impl.noexport;

    let mut lines = Vec::new();

    // Constructor with combined class and constructor documentation
    if let Some(ctx) = parsed_impl.constructor_context() {
        let mut ctor_doc_tags = Vec::new();
        ctor_doc_tags.extend(class_doc_tags.iter().cloned());
        ctor_doc_tags.extend(ctx.method.doc_tags.iter().cloned());

        lines.extend(
            ClassDocBuilder::new(&class_name, type_ident, &ctor_doc_tags, "S3")
                .with_export_control(parsed_impl.internal, parsed_impl.noexport)
                .build(),
        );
        // Inject lifecycle imports from methods into class-level roxygen block
        if let Some(lc_import) = crate::lifecycle::collect_lifecycle_imports(
            parsed_impl
                .methods
                .iter()
                .filter_map(|m| m.method_attrs.lifecycle.as_ref()),
        ) {
            let insert_pos = lines.len().saturating_sub(1);
            lines.insert(insert_pos, format!("#' {}", lc_import));
        }
        lines.push(format!("{} <- function({}) {{", ctor_name, ctx.params));
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        lines.push(format!(
            "  structure({}, class = \"{}\")",
            ctx.static_call(),
            class_name
        ));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods as S3 generics + methods
    for ctx in parsed_impl.instance_method_contexts() {
        let generic_name = ctx.generic_name();
        // Use custom class suffix if provided (for double-dispatch patterns like vec_ptype2.a.b)
        let method_class_suffix = ctx
            .class_suffix()
            .map(|s| s.to_string())
            .unwrap_or_else(|| class_name.clone());
        let s3_method_name = format!("{}.{}", generic_name, method_class_suffix);
        let full_params = ctx.instance_formals(true); // adds x, ..., params

        // Only create the S3 generic if no generic/class override was provided
        // (custom class suffix implies using an existing generic)
        if !ctx.has_generic_override() && !ctx.has_class_override() {
            // Create the S3 generic (only for custom generics, not base R overrides)
            lines.push(format!("#' @title S3 generic for `{}`", generic_name));
            lines.push(format!("#' S3 generic for `{}`", generic_name));
            lines.push(format!("#' @rdname {}", class_name));
            lines.push(format!("#' @name {}", generic_name));
            lines.push("#' @param x An object".to_string());
            lines.push("#' @param ... Additional arguments passed to methods".to_string());
            lines.push(format!(
                "#' @source Generated by miniextendr from `{}::{}`",
                type_ident, ctx.method.ident
            ));
            if should_export {
                lines.push("#' @export".to_string());
            }
            lines.push(format!(
                "if (!exists(\"{generic_name}\", mode = \"function\")) {{
  {generic_name} <- function(x, ...) UseMethod(\"{generic_name}\")
}}"
            ));
            lines.push(String::new());
        }

        // Then create the S3 method
        let method_doc =
            MethodDocBuilder::new(&class_name, &generic_name, type_ident, &ctx.method.doc_tags);
        lines.extend(method_doc.build());
        lines.push(format!("#' @method {} {}", generic_name, class_name));
        if should_export {
            lines.push("#' @export".to_string());
        }
        lines.push(format!(
            "{} <- function({}) {{",
            s3_method_name, full_params
        ));

        // Inject lifecycle prelude if present
        let what = format!("{}.{}", generic_name, class_name);
        if let Some(prelude) = ctx.method.lifecycle_prelude(&what) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }

        let call = ctx.instance_call("x");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_chain_var("x".to_string())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Static methods as regular functions
    for ctx in parsed_impl.static_method_contexts() {
        // Static methods get a prefix to avoid naming conflicts
        let fn_name = format!("{}_{}", class_name.to_lowercase(), ctx.method.ident);
        let method_name = ctx.method.ident.to_string();

        let method_doc =
            MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                .with_r_name(fn_name.clone());
        lines.extend(method_doc.build());
        // Export static methods so users can call them
        lines.push("#' @export".to_string());

        lines.push(format!("{} <- function({}) {{", fn_name, ctx.params));

        // Inject lifecycle prelude if present
        if let Some(prelude) = ctx.method.lifecycle_prelude(&fn_name) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Create class environment for static methods and trait namespace compatibility
    // Check if class should be exported
    let has_no_rd = crate::roxygen::has_roxygen_tag(&parsed_impl.doc_tags, "noRd");
    let has_internal = crate::roxygen::has_roxygen_tag(&parsed_impl.doc_tags, "keywords internal")
        || parsed_impl.internal;
    let export_line = if !has_no_rd && !has_internal && !parsed_impl.noexport {
        "#' @export\n"
    } else {
        ""
    };
    lines.push(format!(
        "#' @rdname {}
{}{} <- new.env(parent = emptyenv())",
        class_name, export_line, class_name
    ));
    lines.push(String::new());

    // Add $new binding to class environment (for Class$new() syntax)
    if parsed_impl.constructor_context().is_some() {
        lines.push(format!("{}$new <- {}", class_name, ctor_name));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Map a Rust return type to an S7 class name.
///
/// Returns `None` if the type doesn't map to a specific S7 class (uses class_any).
///
/// # S7 Class Mapping
///
/// | Rust Type | S7 Class |
/// |-----------|----------|
/// | `i32`, `i16`, `i8` | `class_integer` |
/// | `f64`, `f32` | `class_double` |
/// | `bool` | `class_logical` |
/// | `u8` | `class_raw` |
/// | `String`, `&str` | `class_character` |
/// | `Vec<i32>` | `class_integer` |
/// | `Vec<f64>` | `class_double` |
/// | `Vec<bool>` | `class_logical` |
/// | `Vec<String>` | `class_character` |
/// | `Option<T>` | `NULL | class_T` (union) |
fn rust_type_to_s7_class(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(type_path) => {
            let seg = type_path.path.segments.last()?;
            let ident = seg.ident.to_string();

            match ident.as_str() {
                // Scalar types
                "i32" | "i16" | "i8" | "isize" => Some("S7::class_integer".to_string()),
                "f64" | "f32" => Some("S7::class_double".to_string()),
                "bool" => Some("S7::class_logical".to_string()),
                "u8" => Some("S7::class_raw".to_string()),
                "String" => Some("S7::class_character".to_string()),

                // Vec types - check inner type
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                    {
                        // Recursively get the inner type's class
                        return rust_type_to_s7_class(inner);
                    }
                    None
                }

                // Option types - create union with NULL
                "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                        && let Some(inner_class) = rust_type_to_s7_class(inner)
                    {
                        return Some(format!("NULL | {}", inner_class));
                    }
                    None
                }

                // Result types - use the Ok type
                "Result" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                    {
                        return rust_type_to_s7_class(inner);
                    }
                    None
                }

                _ => None,
            }
        }
        syn::Type::Reference(type_ref) => {
            // Handle &str
            if let syn::Type::Path(type_path) = type_ref.elem.as_ref()
                && let Some(seg) = type_path.path.segments.last()
                && seg.ident == "str"
            {
                return Some("S7::class_character".to_string());
            }
            // Recurse for other reference types
            rust_type_to_s7_class(&type_ref.elem)
        }
        _ => None,
    }
}

/// Generate R wrapper string for S7-style class.
///
/// Creates:
/// - S7::new_class with constructor and .ptr property
/// - S7::new_property for computed/dynamic properties (from #[s7(getter)]/setter)
/// - S7::new_generic + S7::method for each instance method
pub fn generate_s7_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{
        ClassDocBuilder, MethodContext, MethodDocBuilder, ParsedImplExt,
    };

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    // Check if class has @noRd - if so, skip method documentation and exports
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let class_has_internal = crate::roxygen::has_roxygen_tag(class_doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !class_has_internal && !parsed_impl.noexport;

    let mut lines = Vec::new();

    // Collect S7 property getters, setters, and validators
    // Property name is: s7_prop if specified, else method name
    // We store method idents so we can look them up later
    struct S7Property {
        name: String,
        getter_method_ident: Option<String>,
        setter_method_ident: Option<String>,
        validator_method_ident: Option<String>,
        /// S7 class type inferred from getter return type (e.g., "S7::class_double")
        class_type: Option<String>,
        /// Default value (R expression)
        default_value: Option<String>,
        /// Property is required (error if not provided)
        required: bool,
        /// Property is frozen (can only be set once)
        frozen: bool,
        /// Deprecation message
        deprecated: Option<String>,
    }

    let mut properties: std::collections::BTreeMap<String, S7Property> =
        std::collections::BTreeMap::new();
    let mut property_method_idents: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // First pass: collect all property methods (getters, setters, validators)
    for method in &parsed_impl.methods {
        if !method.should_include() {
            continue;
        }
        let attrs = &method.method_attrs;

        if attrs.s7_getter || attrs.s7_setter || attrs.s7_validate {
            let method_ident = method.ident.to_string();
            let prop_name = attrs
                .s7_prop
                .clone()
                .unwrap_or_else(|| method_ident.clone());

            property_method_idents.insert(method_ident.clone());

            let entry = properties.entry(prop_name.clone()).or_insert(S7Property {
                name: prop_name,
                getter_method_ident: None,
                setter_method_ident: None,
                validator_method_ident: None,
                class_type: None,
                default_value: None,
                required: false,
                frozen: false,
                deprecated: None,
            });

            if attrs.s7_getter {
                entry.getter_method_ident = Some(method_ident.clone());
                // Extract S7 class type from getter's return type
                if let syn::ReturnType::Type(_, ret_type) = &method.sig.output {
                    entry.class_type = rust_type_to_s7_class(ret_type);
                }
                // Capture property attributes from getter
                if let Some(ref default) = attrs.s7_default {
                    entry.default_value = Some(default.clone());
                }
                if attrs.s7_required {
                    entry.required = true;
                }
                if attrs.s7_frozen {
                    entry.frozen = true;
                }
                if let Some(ref msg) = attrs.s7_deprecated {
                    entry.deprecated = Some(msg.clone());
                }
            }
            if attrs.s7_setter {
                entry.setter_method_ident = Some(method_ident.clone());
            }
            if attrs.s7_validate {
                entry.validator_method_ident = Some(method_ident);
            }
        }
    }

    // Helper to find method by ident
    let find_method = |ident: &str| -> Option<&ParsedMethod> {
        parsed_impl.methods.iter().find(|m| m.ident == ident)
    };

    // Constructor - check if .ptr param will be added (for static methods returning Self)
    let has_self_returning_methods = parsed_impl
        .methods
        .iter()
        .filter(|m| m.should_include())
        .any(|m| m.returns_self());

    // Determine imports based on whether we have properties and what class types are used
    let base_imports = "new_class class_any new_object S7_object new_generic method";
    let mut import_parts: Vec<&str> = vec![base_imports];

    if !properties.is_empty() {
        import_parts.push("new_property");
    }

    // Check if any methods use S7 convert (convert_from or convert_to)
    let has_convert_methods = parsed_impl.methods.iter().any(|m| {
        m.should_include()
            && (m.method_attrs.s7_convert_from.is_some() || m.method_attrs.s7_convert_to.is_some())
    });
    if has_convert_methods {
        import_parts.push("convert");
    }

    // Collect unique S7 class types used in properties
    let mut class_imports: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for prop in properties.values() {
        if let Some(ref class_type) = prop.class_type {
            // Extract class name from "S7::class_xxx" or "NULL | S7::class_xxx"
            for part in class_type.split('|') {
                let part = part.trim();
                if let Some(class_name) = part.strip_prefix("S7::") {
                    class_imports.insert(class_name);
                }
            }
        }
    }
    // Sort for deterministic output
    let mut sorted_imports: Vec<&str> = class_imports.into_iter().collect();
    sorted_imports.sort();
    for class_name in sorted_imports {
        import_parts.push(class_name);
    }

    let imports = format!("@importFrom S7 {}", import_parts.join(" "));

    // Class definition with documentation
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, class_doc_tags, "S7")
            .with_imports(&imports)
            .with_export_control(parsed_impl.internal, parsed_impl.noexport)
            .build(),
    );
    // Inject lifecycle imports from methods into class-level roxygen block
    if let Some(lc_import) = crate::lifecycle::collect_lifecycle_imports(
        parsed_impl
            .methods
            .iter()
            .filter_map(|m| m.method_attrs.lifecycle.as_ref()),
    ) {
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(insert_pos, format!("#' {}", lc_import));
    }

    // Document .ptr param - S7::new_class always creates a constructor that accepts
    // all properties as parameters, so .ptr is always a valid parameter
    // Skip if class has @noRd
    if !class_has_no_rd && !crate::roxygen::has_roxygen_tag(class_doc_tags, "param .ptr") {
        lines.push(
            "#' @param .ptr Internal pointer (used by static methods, not for direct use)."
                .to_string(),
        );
    }

    // S7::new_class — optionally include parent and abstract
    if let Some(ref parent) = parsed_impl.s7_parent {
        lines.push(format!(
            "{} <- S7::new_class(\"{}\", parent = {},",
            class_name, class_name, parent
        ));
    } else {
        lines.push(format!(
            "{} <- S7::new_class(\"{}\",",
            class_name, class_name
        ));
    }

    if parsed_impl.s7_abstract {
        lines.push("  abstract = TRUE,".to_string());
    }

    // Properties - .ptr holds the ExternalPtr, plus computed/dynamic properties
    // When r_data_accessors is set, merge with sidecar properties from #[derive(ExternalPtr)]
    // Collect property items into a vec, then join with ",\n" to avoid bare commas on standalone lines
    let mut prop_items: Vec<String> = Vec::new();
    prop_items.push("    .ptr = S7::class_any".to_string());

    // Generate computed/dynamic properties
    for prop in properties.values() {
        // Generate property definition
        let mut prop_parts = Vec::new();

        // Add class constraint if known (inferred from getter return type)
        if let Some(ref class_type) = prop.class_type {
            prop_parts.push(format!("class = {}", class_type));
        }

        // Handle default value or required pattern
        if prop.required {
            // Required pattern: error if not provided
            prop_parts.push(format!(
                "default = quote(stop(\"@{} is required\"))",
                prop.name
            ));
        } else if let Some(ref default) = prop.default_value {
            // Explicit default value (R expression)
            prop_parts.push(format!("default = {}", default));
        }

        // Add validator if present
        if let Some(ref validator_ident) = prop.validator_method_ident
            && let Some(validator_method) = find_method(validator_ident)
        {
            let ctx = MethodContext::new(validator_method, type_ident, parsed_impl.label());
            // Validator is called with just the value, not self
            // Generate: validator = function(value) .Call(C_Type__validate_prop, value)
            prop_parts.push(format!(
                "validator = function(value) .Call({}, .call = match.call(), value)",
                ctx.c_ident
            ));
        }

        // Generate getter (with optional deprecation warning)
        if let Some(ref getter_ident) = prop.getter_method_ident
            && let Some(getter_method) = find_method(getter_ident)
        {
            let ctx = MethodContext::new(getter_method, type_ident, parsed_impl.label());
            let getter_call = ctx.instance_call("self@.ptr");
            if let Some(ref msg) = prop.deprecated {
                // Deprecated getter: emit warning then return value
                prop_parts.push(format!(
                    "getter = function(self) {{ warning(\"Property @{} is deprecated: {}\"); {} }}",
                    prop.name, msg, getter_call
                ));
            } else {
                prop_parts.push(format!("getter = function(self) {}", getter_call));
            }
        }

        // Generate setter (with optional frozen/deprecation handling)
        if let Some(ref setter_ident) = prop.setter_method_ident
            && let Some(setter_method) = find_method(setter_ident)
        {
            let ctx = MethodContext::new(setter_method, type_ident, parsed_impl.label());
            let setter_call = ctx.instance_call("self@.ptr");

            if prop.frozen {
                // Frozen pattern: error if property was already set (non-NULL)
                // Note: This is a simplified check; true frozen behavior would need
                // a separate flag in the object to track if ever set
                if let Some(ref msg) = prop.deprecated {
                    prop_parts.push(format!(
                        "setter = function(self, value) {{ warning(\"Property @{} is deprecated: {}\"); if (!is.null(self@{})) stop(\"Property @{} is frozen and cannot be modified\"); {}; self }}",
                        prop.name, msg, prop.name, prop.name, setter_call
                    ));
                } else {
                    prop_parts.push(format!(
                        "setter = function(self, value) {{ if (!is.null(self@{})) stop(\"Property @{} is frozen and cannot be modified\"); {}; self }}",
                        prop.name, prop.name, setter_call
                    ));
                }
            } else if let Some(ref msg) = prop.deprecated {
                // Deprecated setter: emit warning then set value
                prop_parts.push(format!(
                    "setter = function(self, value) {{ warning(\"Property @{} is deprecated: {}\"); {}; self }}",
                    prop.name, msg, setter_call
                ));
            } else {
                // Normal setter
                prop_parts.push(format!(
                    "setter = function(self, value) {{ {}; self }}",
                    setter_call
                ));
            }
        }

        if prop_parts.is_empty() {
            // This shouldn't happen, but handle gracefully
            prop_items.push(format!("    {} = S7::new_property()", prop.name));
        } else {
            prop_items.push(format!(
                "    {} = S7::new_property({})",
                prop.name,
                prop_parts.join(", ")
            ));
        }
    }

    if parsed_impl.r_data_accessors {
        lines.push("  properties = c(list(".to_string());
    } else {
        lines.push("  properties = list(".to_string());
    }
    lines.push(prop_items.join(",\n"));

    // Close the properties list (or merge with sidecar properties)
    if parsed_impl.r_data_accessors {
        let type_name = type_ident.to_string();
        lines.push(format!("  ), .rdata_properties_{}),", type_name));
    } else {
        lines.push("  ),".to_string());
    }

    if let Some(ctx) = parsed_impl.constructor_context() {
        let ctor_preconditions = ctx.precondition_checks();
        if has_self_returning_methods {
            let params_with_ptr = if ctx.params.is_empty() {
                ".ptr = NULL".to_string()
            } else {
                format!("{}, .ptr = NULL", ctx.params)
            };
            lines.push(format!("  constructor = function({}) {{", params_with_ptr));
            // Only check preconditions when not using .ptr shortcut
            if !ctor_preconditions.is_empty() {
                lines.push("    if (is.null(.ptr)) {".to_string());
                for check in &ctor_preconditions {
                    lines.push(format!("      {}", check));
                }
                lines.push("    }".to_string());
            }
            lines.push("    if (!is.null(.ptr)) {".to_string());
            lines.push("      S7::new_object(S7::S7_object(), .ptr = .ptr)".to_string());
            lines.push("    } else {".to_string());
            lines.push(format!(
                "      S7::new_object(S7::S7_object(), .ptr = {})",
                ctx.static_call()
            ));
            lines.push("    }".to_string());
            lines.push("  }".to_string());
        } else {
            lines.push(format!("  constructor = function({}) {{", ctx.params));
            for check in &ctor_preconditions {
                lines.push(format!("    {}", check));
            }
            lines.push(format!(
                "    S7::new_object(S7::S7_object(), .ptr = {})",
                ctx.static_call()
            ));
            lines.push("  }".to_string());
        }
    }

    lines.push(")".to_string());
    lines.push(String::new());

    // Instance methods as S7 generics + methods
    // Skip methods that are property getters/setters (they're handled as S7 properties)
    for ctx in parsed_impl.instance_method_contexts() {
        let method_ident = ctx.method.ident.to_string();
        if property_method_idents.contains(&method_ident) {
            continue;
        }

        let generic_name = ctx.generic_name();
        let full_params = ctx.instance_formals(true); // adds x, ..., params
        let method_attrs = &ctx.method.method_attrs;

        // For fallback methods, use tryCatch to avoid slot-access errors on non-S7 objects
        let self_expr = if method_attrs.s7_fallback {
            "tryCatch(x@.ptr, error = function(e) x)"
        } else {
            "x@.ptr"
        };
        let call = ctx.instance_call(self_expr);

        // Determine dispatch class (fallback -> class_any, normal -> class_name)
        let method_class = if method_attrs.s7_fallback {
            "S7::class_any".to_string()
        } else {
            class_name.clone()
        };

        // Documentation - skip if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &generic_name, type_ident, &ctx.method.doc_tags);
            lines.extend(method_doc.build());
        }

        if ctx.has_generic_override() {
            // Parse "pkg::name" format for external generics
            let (pkg, gen_name) = if generic_name.contains("::") {
                let parts: Vec<&str> = generic_name.split("::").collect();
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("base".to_string(), generic_name.clone())
            };

            // Use S7::new_external_generic for existing generics from other packages
            lines.push(format!(
                "if (!exists(\"{gen_name}\", mode = \"function\")) {{"
            ));
            lines.push(format!(
                "  {gen_name} <- S7::new_external_generic(\"{pkg}\", \"{gen_name}\")"
            ));
            lines.push("}".to_string());

            // Define method using the resolved generic name
            let strategy = crate::ReturnStrategy::for_method(ctx.method);
            let return_expr = crate::MethodReturnBuilder::new(call.clone())
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .with_error_in_r(ctx.method.method_attrs.error_in_r)
                .build_s7_inline();

            // Inject lifecycle prelude and precondition checks if present
            let what = format!("{}.{}", generic_name, class_name);
            let lifecycle = ctx.method.lifecycle_prelude(&what);
            let preconditions = ctx.precondition_checks();
            if lifecycle.is_some() || !preconditions.is_empty() {
                lines.push(format!(
                    "S7::method({gen_name}, {method_class}) <- function({full_params}) {{"
                ));
                if let Some(prelude) = lifecycle {
                    lines.push(format!("  {prelude}"));
                }
                for check in &preconditions {
                    lines.push(format!("  {check}"));
                }
                lines.push(format!("  {return_expr}"));
                lines.push("}".to_string());
            } else {
                lines.push(format!(
                    "S7::method({gen_name}, {method_class}) <- function({full_params}) {return_expr}"
                ));
            }
        } else {
            // Create new S7 generic if it doesn't exist
            // Add @export so roxygen generates export() in NAMESPACE (if class should be exported)
            if should_export {
                lines.push("#' @export".to_string());
            }

            // Determine dispatch arguments (default: "x", or custom via dispatch = "x,y")
            let dispatch_args = if let Some(ref dispatch) = method_attrs.s7_dispatch {
                // Multiple dispatch: "x,y" -> c("x", "y")
                let args: Vec<&str> = dispatch.split(',').map(|s| s.trim()).collect();
                if args.len() == 1 {
                    format!("\"{}\"", args[0])
                } else {
                    format!(
                        "c({})",
                        args.iter()
                            .map(|a| format!("\"{}\"", a))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            } else {
                "\"x\"".to_string()
            };

            // Determine function signature (with or without ...)
            let generic_sig = if method_attrs.s7_no_dots {
                // no_dots: strict generic without ...
                if let Some(ref dispatch) = method_attrs.s7_dispatch {
                    let args: Vec<&str> = dispatch.split(',').map(|s| s.trim()).collect();
                    format!("function({}) S7::S7_dispatch()", args.join(", "))
                } else {
                    "function(x) S7::S7_dispatch()".to_string()
                }
            } else {
                // Default: include ... for extra args
                if let Some(ref dispatch) = method_attrs.s7_dispatch {
                    let args: Vec<&str> = dispatch.split(',').map(|s| s.trim()).collect();
                    format!("function({}, ...) S7::S7_dispatch()", args.join(", "))
                } else {
                    "function(x, ...) S7::S7_dispatch()".to_string()
                }
            };

            lines.push(format!(
                "if (!exists(\"{generic_name}\", mode = \"function\")) {{"
            ));
            lines.push(format!(
                "  {generic_name} <- S7::new_generic(\"{generic_name}\", {dispatch_args}, {generic_sig})"
            ));
            lines.push("}".to_string());

            // Define method
            let strategy = crate::ReturnStrategy::for_method(ctx.method);
            let return_expr = crate::MethodReturnBuilder::new(call)
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .with_error_in_r(ctx.method.method_attrs.error_in_r)
                .build_s7_inline();

            // Use matching formals for method (with or without ...)
            let method_formals = ctx.instance_formals_with_dots(true, !method_attrs.s7_no_dots);

            // Inject lifecycle prelude and precondition checks if present
            let what = format!("{}.{}", generic_name, class_name);
            let lifecycle = ctx.method.lifecycle_prelude(&what);
            let preconditions = ctx.precondition_checks();
            if lifecycle.is_some() || !preconditions.is_empty() {
                lines.push(format!(
                    "S7::method({generic_name}, {method_class}) <- function({method_formals}) {{"
                ));
                if let Some(prelude) = lifecycle {
                    lines.push(format!("  {prelude}"));
                }
                for check in &preconditions {
                    lines.push(format!("  {check}"));
                }
                lines.push(format!("  {return_expr}"));
                lines.push("}".to_string());
            } else {
                lines.push(format!(
                    "S7::method({generic_name}, {method_class}) <- function({method_formals}) {return_expr}"
                ));
            }
        }
        lines.push(String::new());
    }

    // Static methods as regular functions
    for ctx in parsed_impl.static_method_contexts() {
        let fn_name = format!("{}_{}", class_name, ctx.method.ident);
        let method_name = ctx.method.ident.to_string();

        // Skip documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_r_name(fn_name.clone());
            lines.extend(method_doc.build());
        }
        // Export static methods so users can call them (if class should be exported)
        if should_export {
            lines.push("#' @export".to_string());
        }

        lines.push(format!("{} <- function({}) {{", fn_name, ctx.params));

        // Inject lifecycle prelude if present
        if let Some(prelude) = ctx.method.lifecycle_prelude(&fn_name) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_expr = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r)
            .build_s7_inline();
        lines.push(format!("  {}", return_expr));

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Phase 4: S7 convert() methods from Rust From/TryFrom patterns
    // Convert methods enable type coercion between S7 classes using S7::convert()
    //
    // Two patterns:
    // 1. convert_from = "OtherType" on static method: converts FROM OtherType TO this class
    //    Rust: fn from_other(other: OtherType) -> Self
    //    R: S7::method(S7::convert, list(OtherType, ThisClass)) <- function(from, to) ...
    //
    // 2. convert_to = "OtherType" on instance method: converts FROM this class TO OtherType
    //    Rust: fn to_other(&self) -> OtherType
    //    R: S7::method(S7::convert, list(ThisClass, OtherType)) <- function(from, to) ...

    for method in &parsed_impl.methods {
        if !method.should_include() {
            continue;
        }
        let attrs = &method.method_attrs;

        // Handle convert_from (static method pattern)
        // S7 convert signature is function(from, to) - one parameter for the source object
        if let Some(ref from_type) = attrs.s7_convert_from {
            let ctx = MethodContext::new(method, type_ident, parsed_impl.label());

            // Documentation for convert method (skip if class has @noRd)
            if !class_has_no_rd {
                lines.push(format!("#' @name convert-{}-to-{}", from_type, class_name));
                lines.push(format!("#' @rdname {}", class_name));
                lines.push(format!(
                    "#' @source Generated by miniextendr from `{}::{}`",
                    type_ident, method.ident
                ));
            }

            // Generate: S7::method(S7::convert, list(FromType, ThisClass)) <- function(from, to) ...
            // The convert_from method takes the source object as its sole parameter
            // We pass from@.ptr to extract the ExternalPtr from the S7 object
            let call_with_from = format!(".Call({}, .call = match.call(), from@.ptr)", ctx.c_ident);

            let strategy = crate::ReturnStrategy::for_method(method);
            let return_expr = crate::MethodReturnBuilder::new(call_with_from)
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .with_error_in_r(method.method_attrs.error_in_r)
                .build_s7_inline();

            // Use imported `convert` - requires `@importFrom S7 convert` in package
            lines.push(format!(
                "S7::method(convert, list({}, {})) <- function(from, to) {}",
                from_type, class_name, return_expr
            ));
            lines.push(String::new());
        }

        // Handle convert_to (instance method pattern)
        // S7 convert signature is function(from, to) - self becomes from
        if let Some(ref to_type) = attrs.s7_convert_to {
            let ctx = MethodContext::new(method, type_ident, parsed_impl.label());

            // Documentation for convert method (skip if class has @noRd)
            if !class_has_no_rd {
                lines.push(format!("#' @name convert-{}-to-{}", class_name, to_type));
                lines.push(format!("#' @rdname {}", class_name));
                lines.push(format!(
                    "#' @source Generated by miniextendr from `{}::{}`",
                    type_ident, method.ident
                ));
            }

            // Generate: S7::method(convert, list(ThisClass, ToType)) <- function(from, to) ...
            // The convert_to method is an instance method where self is mapped to from@.ptr
            let call = format!(".Call({}, .call = match.call(), from@.ptr)", ctx.c_ident);

            // Force ReturnSelf strategy for convert methods since they return S7 class types
            // that need to be wrapped: ToType(.ptr = <result>)
            let return_expr = crate::MethodReturnBuilder::new(call)
                .with_strategy(crate::ReturnStrategy::ReturnSelf)
                .with_class_name(to_type.clone())
                .with_error_in_r(method.method_attrs.error_in_r)
                .build_s7_inline();

            // Use imported `convert` - requires `@importFrom S7 convert` in package
            lines.push(format!(
                "S7::method(convert, list({}, {})) <- function(from, to) {}",
                class_name, to_type, return_expr
            ));
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

/// Generate R wrapper string for S4-style class.
///
/// Creates:
/// - setClass with ptr slot
/// - Constructor function
/// - setMethod for each instance method
pub fn generate_s4_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    // Check if class has @noRd - if so, skip method documentation and exports
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let class_has_internal = crate::roxygen::has_roxygen_tag(class_doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !class_has_internal && !parsed_impl.noexport;

    let mut lines = Vec::new();

    // Class definition with documentation (S4 uses setClass, no @export on class definition)
    let has_export = crate::roxygen::has_roxygen_tag(class_doc_tags, "export");
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, class_doc_tags, "S4")
            .with_imports("@importFrom methods setClass setGeneric setMethod new")
            .with_export_control(parsed_impl.internal, parsed_impl.noexport)
            .build(),
    );
    // Inject lifecycle imports from methods into class-level roxygen block
    if let Some(lc_import) = crate::lifecycle::collect_lifecycle_imports(
        parsed_impl
            .methods
            .iter()
            .filter_map(|m| m.method_attrs.lifecycle.as_ref()),
    ) {
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(insert_pos, format!("#' {}", lc_import));
    }
    // Remove the @export that ClassDocBuilder adds (S4 doesn't export the class definition)
    if !has_export {
        lines.pop();
    }
    if !class_has_no_rd {
        lines.push(format!(
            "#' @slot ptr External pointer to Rust `{}` struct",
            type_ident
        ));
    }
    lines.push(format!(
        "methods::setClass(\"{}\", slots = c(ptr = \"externalptr\"))",
        class_name
    ));
    lines.push(String::new());

    // Constructor function
    if let Some(ctx) = parsed_impl.constructor_context() {
        // Skip documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, "new", type_ident, &ctx.method.doc_tags);
            lines.extend(method_doc.build());
        }
        // Export the constructor function so users can create instances (if class should be exported)
        if should_export {
            lines.push("#' @export".to_string());
        }

        lines.push(format!("{} <- function({}) {{", class_name, ctx.params));
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        lines.push(format!(
            "  methods::new(\"{}\", ptr = {})",
            class_name,
            ctx.static_call()
        ));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods as S4 methods
    // Note: S4 uses empty param_defaults for method signatures (different from other systems)
    for method in parsed_impl.instance_methods() {
        let c_ident = method.c_wrapper_ident(type_ident, parsed_impl.label());
        let method_name = if let Some(ref generic) = method.method_attrs.generic {
            generic.clone()
        } else {
            format!("s4_{}", method.ident)
        };
        // S4 methods use empty defaults for consistency with setMethod
        let params = crate::r_wrapper_builder::build_r_formals_from_sig(
            &method.sig,
            &std::collections::HashMap::new(),
        );
        let args = crate::r_wrapper_builder::build_r_call_args_from_sig(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call(), x@ptr)", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), x@ptr, {})", c_ident, args)
        };
        let full_params = if params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", params)
        };

        // Documentation for the generic - skip if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &method.doc_tags);
            lines.extend(method_doc.build());
        }

        // Define generic unconditionally - setGeneric() is idempotent and handles
        // re-definition correctly. The conditional `if (!isGeneric())` pattern fails
        // during package reload because isGeneric() can return TRUE from stale cache
        // entries while the actual generic no longer exists in the namespace.
        lines.push(format!(
            "methods::setGeneric(\"{}\", function(x, ...) standardGeneric(\"{}\"))",
            method_name, method_name
        ));

        // Define method with @exportMethod for proper S4 dispatch (if class should be exported)
        if should_export {
            lines.push(format!("#' @exportMethod {}", method_name));
        }

        let strategy = crate::ReturnStrategy::for_method(method);
        let return_expr = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(method.method_attrs.error_in_r)
            .build_s4_inline();

        // Inject lifecycle prelude and precondition checks if present
        let what = format!("{}.{}", method_name, class_name);
        let lifecycle = method.lifecycle_prelude(&what);
        let preconditions = crate::r_preconditions::build_precondition_checks(
            &method.sig.inputs,
            &std::collections::HashSet::new(),
        )
        .static_checks;
        if lifecycle.is_some() || !preconditions.is_empty() {
            lines.push(format!(
                "methods::setMethod(\"{}\", \"{}\", function({}) {{",
                method_name, class_name, full_params
            ));
            if let Some(prelude) = lifecycle {
                lines.push(format!("  {}", prelude));
            }
            for check in &preconditions {
                lines.push(format!("  {}", check));
            }
            lines.push(format!("  {}", return_expr));
            lines.push("})".to_string());
        } else {
            lines.push(format!(
                "methods::setMethod(\"{}\", \"{}\", function({}) {})",
                method_name, class_name, full_params, return_expr
            ));
        }
        lines.push(String::new());
    }

    // Static methods as regular functions
    for ctx in parsed_impl.static_method_contexts() {
        let fn_name = format!("{}_{}", class_name, ctx.method.ident);
        let method_name = ctx.method.ident.to_string();

        // Skip documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_r_name(fn_name.clone());
            lines.extend(method_doc.build());
        }
        // Export static methods so users can call them (if class should be exported)
        if should_export {
            lines.push("#' @export".to_string());
        }

        lines.push(format!("{} <- function({}) {{", fn_name, ctx.params));

        // Inject lifecycle prelude if present
        if let Some(prelude) = ctx.method.lifecycle_prelude(&fn_name) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_expr = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r)
            .build_s4_inline();
        lines.push(format!("  {}", return_expr));

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate R wrapper string for vctrs-style class.
///
/// Creates a vctrs-compatible S3 class with:
/// - Constructor using `vctrs::new_vctr()`, `vctrs::new_rcrd()`, or `vctrs::new_list_of()`
/// - `vec_ptype2.<class>.<class>` and `vec_cast.<class>.<class>` for same-type coercion
/// - `vec_ptype_abbr.<class>` for compact printing (if `abbr` is specified)
/// - Instance methods as regular S3 methods
pub fn generate_vctrs_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    let vctrs_attrs = &parsed_impl.vctrs_attrs;
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd");
    let class_has_internal = crate::roxygen::has_roxygen_tag(class_doc_tags, "keywords internal")
        || parsed_impl.internal;
    let should_export = !class_has_no_rd && !class_has_internal && !parsed_impl.noexport;

    // Constructor name follows vctrs convention: new_<class>
    let ctor_name = format!("new_{}", class_name.to_lowercase());

    let mut lines = Vec::new();

    // Constructor with combined class and constructor documentation
    if let Some(ctx) = parsed_impl.constructor_context() {
        let mut ctor_doc_tags = Vec::new();
        ctor_doc_tags.extend(class_doc_tags.iter().cloned());
        ctor_doc_tags.extend(ctx.method.doc_tags.iter().cloned());

        lines.extend(
            ClassDocBuilder::new(&class_name, type_ident, &ctor_doc_tags, "vctrs S3")
                .with_imports("@importFrom vctrs new_vctr new_rcrd new_list_of vec_ptype2 vec_cast vec_ptype_abbr")
                .with_export_control(parsed_impl.internal, parsed_impl.noexport)
                .build(),
        );
        // Inject lifecycle imports from methods into class-level roxygen block
        if let Some(lc_import) = crate::lifecycle::collect_lifecycle_imports(
            parsed_impl
                .methods
                .iter()
                .filter_map(|m| m.method_attrs.lifecycle.as_ref()),
        ) {
            let insert_pos = lines.len().saturating_sub(1);
            lines.insert(insert_pos, format!("#' {}", lc_import));
        }

        // Generate constructor body based on vctrs kind
        lines.push(format!("{} <- function({}) {{", ctor_name, ctx.params));
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        lines.push(format!("  data <- {}", ctx.static_call()));

        match vctrs_attrs.kind {
            VctrsKind::Vctr => {
                // Build new_vctr call with optional inherit_base_type
                let inherit_arg = match vctrs_attrs.inherit_base_type {
                    Some(true) => ", inherit_base_type = TRUE",
                    Some(false) => ", inherit_base_type = FALSE",
                    None => "",
                };
                lines.push(format!(
                    "  vctrs::new_vctr(data, class = \"{}\"{})",
                    class_name, inherit_arg
                ));
            }
            VctrsKind::Rcrd => {
                // Record type - data should be a list
                lines.push(format!(
                    "  vctrs::new_rcrd(data, class = \"{}\")",
                    class_name
                ));
            }
            VctrsKind::ListOf => {
                // list_of - needs ptype
                let ptype_arg = vctrs_attrs
                    .ptype
                    .as_ref()
                    .map(|p| format!(", ptype = {}", p))
                    .unwrap_or_default();
                lines.push(format!(
                    "  vctrs::new_list_of(data, class = \"{}\"{})",
                    class_name, ptype_arg
                ));
            }
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // vec_ptype_abbr for compact printing (if abbr is specified)
    if let Some(abbr) = &vctrs_attrs.abbr {
        lines.push(format!("#' @rdname {}", class_name));
        lines.push(format!("#' @method vec_ptype_abbr {}", class_name));
        if should_export {
            lines.push("#' @export".to_string());
        }
        lines.push(format!(
            "vec_ptype_abbr.{} <- function(x, ...) \"{}\"",
            class_name, abbr
        ));
        lines.push(String::new());
    }

    // Self-coercion methods (required for vctrs to work properly)
    // vec_ptype2.<class>.<class> - returns prototype for combining same types
    lines.push(format!("#' @rdname {}", class_name));
    lines.push(format!(
        "#' @method vec_ptype2 {}.{}",
        class_name, class_name
    ));
    lines.push(format!("#' @param x A {} vector.", class_name));
    lines.push(format!("#' @param y A {} vector.", class_name));
    lines.push("#' @param ... Additional arguments (unused).".to_string());
    if should_export {
        lines.push("#' @export".to_string());
    }
    match vctrs_attrs.kind {
        VctrsKind::Vctr => {
            let base_type = vctrs_attrs
                .base
                .as_ref()
                .map(|b| format!("{}()", b))
                .unwrap_or_else(|| "double()".to_string());
            let inherit_arg = match vctrs_attrs.inherit_base_type {
                Some(true) => ", inherit_base_type = TRUE",
                Some(false) => ", inherit_base_type = FALSE",
                None => "",
            };
            lines.push(format!(
                "vec_ptype2.{c}.{c} <- function(x, y, ...) vctrs::new_vctr({base}, class = \"{c}\"{inherit})",
                c = class_name,
                base = base_type,
                inherit = inherit_arg
            ));
        }
        VctrsKind::Rcrd => {
            // For records, return empty record with same field structure
            lines.push(format!(
                "vec_ptype2.{c}.{c} <- function(x, y, ...) x[0]",
                c = class_name
            ));
        }
        VctrsKind::ListOf => {
            let ptype_arg = vctrs_attrs
                .ptype
                .as_ref()
                .map(|p| format!(", ptype = {}", p))
                .unwrap_or_default();
            lines.push(format!(
                "vec_ptype2.{c}.{c} <- function(x, y, ...) vctrs::new_list_of(list(), class = \"{c}\"{ptype})",
                c = class_name,
                ptype = ptype_arg
            ));
        }
    }
    lines.push(String::new());

    // vec_cast.<class>.<class> - identity cast (no-op for same type)
    lines.push(format!("#' @rdname {}", class_name));
    lines.push(format!("#' @method vec_cast {}.{}", class_name, class_name));
    lines.push(format!("#' @param x A {} vector to cast.", class_name));
    lines.push(format!("#' @param to A {} prototype.", class_name));
    lines.push("#' @param ... Additional arguments (unused).".to_string());
    if should_export {
        lines.push("#' @export".to_string());
    }
    lines.push(format!(
        "vec_cast.{c}.{c} <- function(x, to, ...) x",
        c = class_name
    ));
    lines.push(String::new());

    // Instance methods as S3 generics + methods
    for ctx in parsed_impl.instance_method_contexts() {
        // vctrs protocol override: use the protocol name as the S3 generic
        let is_protocol = ctx.method.method_attrs.vctrs_protocol.is_some();
        let generic_name = if let Some(ref proto) = ctx.method.method_attrs.vctrs_protocol {
            proto.clone()
        } else {
            ctx.generic_name()
        };
        // Use custom class suffix if provided (for double-dispatch patterns like vec_ptype2.a.b)
        let method_class_suffix = ctx
            .class_suffix()
            .map(|s| s.to_string())
            .unwrap_or_else(|| class_name.clone());
        let s3_method_name = format!("{}.{}", generic_name, method_class_suffix);
        let full_params = ctx.instance_formals(true); // adds x, ..., params

        // Only create the S3 generic if no generic/class override was provided
        // vctrs protocol methods use existing generics from the vctrs package
        if !is_protocol && !ctx.has_generic_override() && !ctx.has_class_override() {
            lines.push(format!("#' @title S3 generic for `{}`", generic_name));
            lines.push(format!("#' S3 generic for `{}`", generic_name));
            lines.push(format!("#' @rdname {}", class_name));
            lines.push(format!("#' @name {}", generic_name));
            lines.push("#' @param x An object".to_string());
            lines.push("#' @param ... Additional arguments passed to methods".to_string());
            lines.push(format!(
                "#' @source Generated by miniextendr from `{}::{}`",
                type_ident, ctx.method.ident
            ));
            if should_export {
                lines.push("#' @export".to_string());
            }
            lines.push(format!(
                "if (!exists(\"{generic_name}\", mode = \"function\")) {{
  {generic_name} <- function(x, ...) UseMethod(\"{generic_name}\")
}}"
            ));
            lines.push(String::new());
        }

        // Then create the S3 method
        let method_doc =
            MethodDocBuilder::new(&class_name, &generic_name, type_ident, &ctx.method.doc_tags);
        lines.extend(method_doc.build());
        lines.push(format!(
            "#' @method {} {}",
            generic_name, method_class_suffix
        ));
        lines.push(format!(
            "{} <- function({}) {{",
            s3_method_name, full_params
        ));

        // Inject lifecycle prelude if present
        let what = format!("{}.{}", generic_name, class_name);
        if let Some(prelude) = ctx.method.lifecycle_prelude(&what) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }

        let call = ctx.instance_call("x");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_chain_var("x".to_string())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Static methods as regular functions
    for ctx in parsed_impl.static_method_contexts() {
        let fn_name = format!("{}_{}", class_name.to_lowercase(), ctx.method.ident);
        let method_name = ctx.method.ident.to_string();

        let method_doc =
            MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                .with_r_name(fn_name.clone());
        lines.extend(method_doc.build());

        lines.push(format!("{} <- function({}) {{", fn_name, ctx.params));

        // Inject lifecycle prelude if present
        if let Some(prelude) = ctx.method.lifecycle_prelude(&fn_name) {
            lines.push(format!("  {}", prelude));
        }
        // Inject precondition checks
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_error_in_r(ctx.method.method_attrs.error_in_r);
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

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

/// Expand a #[miniextendr(env|r6|s7|s3|s4|vctrs)] impl block.
///
/// This handles two cases:
/// 1. **Inherent impls** (`impl Type`): Generate class-system wrappers
/// 2. **Trait impls** (`impl Trait for Type`): Generate vtable static for trait ABI
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

    let call_defs_const = parsed.call_defs_const_ident();

    let label = parsed.label();
    let call_def_idents: Vec<syn::Ident> = parsed
        .included_methods()
        .map(|m| m.call_method_def_ident(type_ident, label))
        .collect();

    let call_defs_len = call_def_idents.len();
    let call_defs_len_lit =
        syn::LitInt::new(&call_defs_len.to_string(), proc_macro2::Span::call_site());

    let original_impl = &parsed.original_impl;

    let r_wrapper_str: TokenStream = {
        use std::str::FromStr;
        let indented = r_wrapper_string.replace('\n', "\n  ");
        let raw = format!("r#\"\n  {}\n\"#", indented);
        TokenStream::from_str(&raw).expect("valid raw string literal")
    };

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

    let expanded = quote! {
        // Original impl block with doc link to R wrapper
        #[doc = #r_wrapper_doc]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #original_impl

        // C wrappers and call method defs
        #(#c_wrappers)*

        // R wrapper constant
        #(#cfg_attrs)*
        #[doc = concat!(
            "R wrapper code for impl block on `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        const #r_wrappers_const: &str =
            concat!(
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
            );

        // Call method def array for module registration
        #(#cfg_attrs)*
        #[doc = concat!(
            "Call method definition array for impl block on `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        const #call_defs_const: [::miniextendr_api::ffi::R_CallMethodDef; #call_defs_len_lit] =
            [#(#call_def_idents),*];
    };

    expanded.into()
}

#[cfg(test)]
mod tests;
