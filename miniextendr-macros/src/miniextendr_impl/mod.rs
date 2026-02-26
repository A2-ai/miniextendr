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

// =============================================================================
// Class-system R wrapper generators (sub-modules)
// =============================================================================

mod env_class;
mod r6_class;
mod s3_class;
mod s4_class;
mod s7_class;
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

    let expanded = quote! {
        // Original impl block with doc link to R wrapper
        #[doc = #r_wrapper_doc]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #original_impl

        // C wrappers and call method defs
        #(#c_wrappers)*

        // Forwarding trait impls for as.<class>() coercion methods
        #trait_impls

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
