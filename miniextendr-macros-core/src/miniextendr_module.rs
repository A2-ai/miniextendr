//! Module declaration parsing for `miniextendr_module!`.
//!
//! This module handles parsing the `miniextendr_module! { ... }` macro body,
//! which registers functions and ALTREP types with R.
//!
//! # Supported items
//!
//! - `mod <name>;` - Required module name (determines `R_init_<name>_miniextendr` symbol)
//! - `fn <name>;` - Register a `#[miniextendr]` function
//! - `struct <name>;` - Register an ALTREP class
//! - `impl <name>;` - Register a `#[miniextendr(env|r6|s7|s3|s4)]` impl block
//! - `impl <name> as "label";` - Register a labeled impl block (see below)
//! - `impl <Trait> for <Type>;` - Register a trait impl for cross-package dispatch
//! - `use <submodule>;` - Re-export from a submodule
//!
//! Note: `extern "C-unwind" fn <name>;` syntax is accepted for parsing but
//! treated identically to `fn <name>;`. The ABI distinction is handled by
//! `#[miniextendr]` at the function definition site.
//!
//! # Multiple impl blocks with labels
//!
//! When a type has multiple `#[miniextendr]` impl blocks (e.g., to organize methods
//! into logical groups), each block must have a distinct label:
//!
//! ```rust,ignore
//! #[miniextendr(label = "constructors")]
//! impl MyType {
//!     fn new() -> Self { ... }
//!     fn from_value(x: i32) -> Self { ... }
//! }
//!
//! #[miniextendr(label = "methods")]
//! impl MyType {
//!     fn get_value(&self) -> i32 { ... }
//!     fn set_value(&mut self, x: i32) { ... }
//! }
//!
//! miniextendr_module! {
//!     mod mymod;
//!     impl MyType as "constructors";
//!     impl MyType as "methods";
//! }
//! ```
//!
//! **Rules for labeled impl blocks:**
//! - If a type has only one impl block, no label is required
//! - If a type has 2+ impl blocks, ALL must have distinct labels
//! - Each labeled impl block must be registered separately in `miniextendr_module!`
//! - Labels can be combined with class systems: `#[miniextendr(r6, label = "methods")]`
//!
//! The `miniextendr-lint` crate validates these rules at build time.

use crate::{call_method_def_ident_for, match_arg_call_defs_ident_for, r_wrapper_const_ident_for};

/// A single `fn ...;` line inside `miniextendr_module! { ... }`.
///
/// Registers a function that has the `#[miniextendr]` attribute.
///
/// ```text
/// fn my_function;
/// ```
///
/// Note: `extern "C-unwind" fn <name>;` syntax is accepted for backwards
/// compatibility but treated identically to `fn <name>;`.
///
/// To conditionally compile functions, place `#[cfg(...)]` AFTER `#[miniextendr]`
/// on the function definition itself, not in this module declaration.
pub struct MiniextendrModuleFunction {
    /// Attributes attached to the module entry (e.g., cfg/doc mirrors from the function).
    pub attrs: Vec<syn::Attribute>,
    /// Optional extern ABI when the declaration uses `extern "C-unwind"`.
    pub _abi: Option<syn::Abi>,
    /// Token for the `fn` keyword kept for accurate span reporting.
    _fn_token: syn::Token![fn],
    /// Identifier of the `#[miniextendr]` function being registered.
    pub ident: syn::Ident,
}

impl syn::parse::Parse for MiniextendrModuleFunction {
    /// Parses one `fn` (or `extern "C-unwind" fn`) module entry.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let _abi = if input.peek(syn::Token![extern]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            attrs,
            _abi,
            _fn_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl MiniextendrModuleFunction {
    /// Returns the identifier for the generated `R_CallMethodDef` entry for this function.
    ///
    /// Delegates to [`call_method_def_ident_for`],
    /// producing `call_method_def_{ident}`.
    pub fn call_method_def_ident(&self) -> syn::Ident {
        call_method_def_ident_for(&self.ident)
    }

    /// Returns the identifier for the generated `const &str` holding R wrapper source.
    ///
    /// Delegates to [`r_wrapper_const_ident_for`],
    /// producing `R_WRAPPER_{IDENT}` (uppercased).
    pub fn r_wrapper_const_ident(&self) -> syn::Ident {
        r_wrapper_const_ident_for(&self.ident)
    }

    /// Returns the identifier for the match_arg choices helper call defs array.
    ///
    /// Delegates to [`match_arg_call_defs_ident_for`],
    /// producing `MATCH_ARG_CALL_DEFS_{IDENT}` (uppercased).
    pub fn match_arg_call_defs_ident(&self) -> syn::Ident {
        match_arg_call_defs_ident_for(&self.ident)
    }
}

/// A single `struct ...;` line inside `miniextendr_module! { ... }`.
///
/// This is used to request ALTREP class registration at `R_init_*` time:
///
/// ```text
/// struct MyAltrepClass;
/// ```
///
/// The struct must implement `miniextendr_api::altrep_registration::RegisterAltrep`.
pub struct MiniextendrModuleStruct {
    _struct_token: syn::Token![struct],
    /// Name of the ALTREP struct to register.
    pub ident: syn::Ident,
}

impl syn::parse::Parse for MiniextendrModuleStruct {
    /// Parses one `struct Type;` module entry.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _struct_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

/// The required `mod <name>;` header inside `miniextendr_module! { ... }`.
///
/// This determines the generated init symbol: `R_init_<name>_miniextendr`.
pub struct MiniextendrModuleName {
    _mod_token: syn::Token![mod],
    /// Base name that drives `R_init_<name>_miniextendr` symbol generation.
    pub ident: syn::Ident,
}

impl syn::parse::Parse for MiniextendrModuleName {
    /// Parses the required `mod name;` module header.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _mod_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

/// An `impl <Type>;` or `impl <Type> as "label";` line inside `miniextendr_module! { ... }`.
///
/// Registers an impl block that has `#[miniextendr(env|r6|s7|s3|s4)]` attribute.
///
/// ```text
/// impl Counter;                    // Single impl block (no label)
/// impl Counter as "constructors";  // Labeled impl block
/// ```
///
/// When a type has multiple `#[miniextendr]` impl blocks, each must be registered
/// with its distinct label using the `as "label"` syntax.
pub struct MiniextendrModuleImpl {
    /// Attributes on the impl entry (passed through for cfg/doc parity).
    pub attrs: Vec<syn::Attribute>,
    _impl_token: syn::Token![impl],
    /// Type that has a `#[miniextendr(...)]` impl block.
    pub ident: syn::Ident,
    /// Optional label for distinguishing multiple impl blocks of the same type.
    pub label: Option<String>,
}

impl syn::parse::Parse for MiniextendrModuleImpl {
    /// Parses one `impl Type;` or `impl Type as "label";` module entry.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let _impl_token = input.parse()?;
        let ident = input.parse()?;

        // Check for optional `as "label"` suffix
        let label = if input.peek(syn::Token![as]) {
            let _: syn::Token![as] = input.parse()?;
            let label_lit: syn::LitStr = input.parse()?;
            Some(label_lit.value())
        } else {
            None
        };

        Ok(Self {
            attrs,
            _impl_token,
            ident,
            label,
        })
    }
}

/// An `impl <Trait> for <Type>;` line inside `miniextendr_module! { ... }`.
///
/// Registers a trait implementation for cross-package trait dispatch.
/// This generates the type-erased wrapper infrastructure (vtable, query function, etc.).
///
/// ```text
/// impl Counter for SimpleCounter;
/// impl AltIntegerData for Vec<i32>;  // Generic types supported
/// ```
///
/// Requirements:
/// - The trait must have `#[miniextendr]` applied (generates TAG, VTable, etc.)
/// - The type must have `#[miniextendr] impl Trait for Type` (generates vtable static)
/// - The type should have `#[derive(ExternalPtr)]`
pub struct MiniextendrModuleTraitImpl {
    /// Attributes on the trait impl entry (for cfg propagation).
    pub attrs: Vec<syn::Attribute>,
    /// Token span for `impl` retained for diagnostics.
    pub _impl_token: syn::Token![impl],
    /// Trait being exposed for cross-package dispatch.
    pub trait_path: syn::Path,
    /// Token span for `for` retained for diagnostics.
    pub _for_token: syn::Token![for],
    /// Concrete type providing the trait implementation.
    /// Supports simple types (`MyType`) and generic types (`Vec<i32>`, `Range<i32>`).
    pub impl_type: syn::Type,
}

impl syn::parse::Parse for MiniextendrModuleTraitImpl {
    /// Parses one `impl Trait for Type;` module entry.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        Ok(Self {
            attrs,
            _impl_token: input.parse()?,
            trait_path: input.parse()?,
            _for_token: input.parse()?,
            impl_type: input.parse()?,
        })
    }
}

/// ALTREP base type, determined from the trait name.
///
/// Used by `altrep_module.rs` to select which `impl_alt*_from_data!` macro to invoke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltrepBase {
    /// ALTREP class over `INTSXP`.
    Integer,
    /// ALTREP class over `REALSXP`.
    Real,
    /// ALTREP class over `LGLSXP`.
    Logical,
    /// ALTREP class over `RAWSXP`.
    Raw,
    /// ALTREP class over `STRSXP`.
    String,
    /// ALTREP class over `CPLXSXP`.
    Complex,
    /// ALTREP class over `VECSXP`.
    List,
}

impl MiniextendrModuleTraitImpl {
    /// Returns a sanitized string name for the implementing type, suitable for use in
    /// generated Rust identifiers.
    ///
    /// Replaces angle brackets, spaces, colons, and commas with underscores, then
    /// collapses runs of underscores and trims leading/trailing underscores.
    ///
    /// # Examples
    ///
    /// - `Vec<i32>` becomes `"Vec_i32"`
    /// - `Range<i32>` becomes `"Range_i32"`
    /// - `MyType` becomes `"MyType"`
    pub fn type_name_sanitized(&self) -> String {
        use quote::ToTokens;
        let type_str = self.impl_type.to_token_stream().to_string();
        // Replace special characters with underscores for valid identifiers
        type_str
            .replace(['<', '>', ' ', ':'], "_")
            .replace(',', "_")
            .replace("__", "_")
            .trim_matches('_')
            .to_string()
    }

    /// Returns the identifier for the `R_CallMethodDef` array const for this trait impl.
    ///
    /// Format: `{TYPE}_{TRAIT}_CALL_DEFS` (both uppercased). Used by the module macro to
    /// collect all `.Call` entry points for R routine registration.
    pub fn call_defs_const_ident(&self) -> syn::Ident {
        let type_upper = self.type_name_sanitized().to_uppercase();
        let trait_name = self
            .trait_path
            .segments
            .last()
            .map(|s| s.ident.to_string().to_uppercase())
            .unwrap_or_default();
        quote::format_ident!("{}_{}_CALL_DEFS", type_upper, trait_name)
    }

    /// Returns the identifier for the `const &str` holding the R wrapper source for this trait impl.
    ///
    /// Format: `R_WRAPPERS_{TYPE}_{TRAIT}_IMPL` (both uppercased). The module macro
    /// concatenates these fragments to produce the final R wrapper file.
    pub fn r_wrappers_const_ident(&self) -> syn::Ident {
        let type_upper = self.type_name_sanitized().to_uppercase();
        let trait_name = self
            .trait_path
            .segments
            .last()
            .map(|s| s.ident.to_string().to_uppercase())
            .unwrap_or_default();
        quote::format_ident!("R_WRAPPERS_{}_{}_IMPL", type_upper, trait_name)
    }

    /// Returns the trait name as a string (last path segment only).
    ///
    /// For `crate::MyTrait`, returns `"MyTrait"`. Returns an empty string if the
    /// path has no segments (which should not happen in practice).
    pub fn trait_name(&self) -> String {
        self.trait_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default()
    }

    /// Checks if this trait impl is for one of the built-in ALTREP data traits.
    ///
    /// Returns `Some(AltrepBase)` if the trait name matches an ALTREP data trait
    /// (e.g., `AltIntegerData`, `AltRealData`), or `None` for non-ALTREP traits.
    /// Used to determine whether the module macro should invoke `impl_alt*_from_data!`.
    pub fn altrep_base(&self) -> Option<AltrepBase> {
        match self.trait_name().as_str() {
            "AltIntegerData" => Some(AltrepBase::Integer),
            "AltRealData" => Some(AltrepBase::Real),
            "AltLogicalData" => Some(AltrepBase::Logical),
            "AltRawData" => Some(AltrepBase::Raw),
            "AltStringData" => Some(AltrepBase::String),
            "AltComplexData" => Some(AltrepBase::Complex),
            "AltListData" => Some(AltrepBase::List),
            _ => None,
        }
    }

    /// For simple types (non-generic), returns the type identifier.
    ///
    /// Returns `Some(ident)` for types like `MyType` or `Counter`.
    /// Returns `None` for generic types like `Vec<i32>` or `Range<i32>`.
    ///
    /// This is used for cross-package trait dispatch which requires simple types.
    pub fn simple_type_ident(&self) -> Option<&syn::Ident> {
        if let syn::Type::Path(type_path) = &self.impl_type {
            // Simple type: single segment with no generic arguments
            if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
                let segment = &type_path.path.segments[0];
                if matches!(segment.arguments, syn::PathArguments::None) {
                    return Some(&segment.ident);
                }
            }
        }
        None
    }
}

impl MiniextendrModuleImpl {
    /// Returns the identifier for the `R_CallMethodDef` array const for this impl block.
    ///
    /// Format: `{TYPE}_CALL_DEFS` (unlabeled) or `{TYPE}_{LABEL}_CALL_DEFS` (labeled),
    /// all uppercased. Used by the module macro to collect `.Call` entry points.
    pub fn call_defs_const_ident(&self) -> syn::Ident {
        let type_upper = self.ident.to_string().to_uppercase();
        if let Some(ref label) = self.label {
            let label_upper = label.to_uppercase();
            quote::format_ident!("{}_{}_CALL_DEFS", type_upper, label_upper)
        } else {
            quote::format_ident!("{}_CALL_DEFS", type_upper)
        }
    }

    /// Returns the identifier for the `const &str` holding R wrapper source for this impl block.
    ///
    /// Format: `R_WRAPPERS_IMPL_{TYPE}` (unlabeled) or `R_WRAPPERS_IMPL_{TYPE}_{LABEL}` (labeled),
    /// all uppercased. The module macro concatenates these to produce the final R wrapper file.
    pub fn r_wrappers_const_ident(&self) -> syn::Ident {
        let type_upper = self.ident.to_string().to_uppercase();
        if let Some(ref label) = self.label {
            let label_upper = label.to_uppercase();
            quote::format_ident!("R_WRAPPERS_IMPL_{}_{}", type_upper, label_upper)
        } else {
            quote::format_ident!("R_WRAPPERS_IMPL_{}", type_upper)
        }
    }

    /// Returns the label string if this is a labeled impl block, or `None` for unlabeled.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the identifier for sidecar call defs const (from #[derive(ExternalPtr)]).
    ///
    /// Format: `RDATA_CALL_DEFS_{TYPE}`
    ///
    /// This constant is generated by `#[derive(ExternalPtr)]` when the struct has
    /// `#[r_data]` fields with `RSidecar` and `RData` markers.
    pub fn rdata_call_defs_const_ident(&self) -> syn::Ident {
        let type_upper = self.ident.to_string().to_uppercase();
        quote::format_ident!("RDATA_CALL_DEFS_{}", type_upper)
    }

    /// Returns the identifier for sidecar R wrappers const (from #[derive(ExternalPtr)]).
    ///
    /// Format: `R_WRAPPERS_RDATA_{TYPE}`
    pub fn rdata_r_wrappers_const_ident(&self) -> syn::Ident {
        let type_upper = self.ident.to_string().to_uppercase();
        quote::format_ident!("R_WRAPPERS_RDATA_{}", type_upper)
    }
}

/// A `vctrs <Type>;` line inside `miniextendr_module! { ... }`.
///
/// Registers a type that has `#[derive(Vctrs)]` attribute, including its R S3 method wrappers.
///
/// ```text
/// vctrs Percent;
/// ```
pub struct MiniextendrModuleVctrs {
    /// Attributes on the vctrs entry (for cfg propagation).
    pub attrs: Vec<syn::Attribute>,
    /// Identifier for the custom `vctrs` keyword.
    pub _vctrs_ident: syn::Ident,
    /// Type that has `#[derive(Vctrs)]` attribute.
    pub ident: syn::Ident,
}

impl syn::parse::Parse for MiniextendrModuleVctrs {
    /// Parses one `vctrs Type;` module entry.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vctrs_ident: syn::Ident = input.parse()?;
        if vctrs_ident != "vctrs" {
            return Err(syn::Error::new(vctrs_ident.span(), "expected `vctrs`"));
        }
        Ok(Self {
            attrs,
            _vctrs_ident: vctrs_ident,
            ident: input.parse()?,
        })
    }
}

impl MiniextendrModuleVctrs {
    /// Returns the identifier for the `const &str` holding R wrapper source (S3 method wrappers).
    ///
    /// Format: `R_WRAPPERS_VCTRS_{TYPE}` (uppercased). The module macro concatenates
    /// these fragments to produce the final R wrapper file.
    pub fn r_wrappers_const_ident(&self) -> syn::Ident {
        let type_upper = self.ident.to_string().to_uppercase();
        quote::format_ident!("R_WRAPPERS_VCTRS_{}", type_upper)
    }
}

/// A `use <module>;` line inside `miniextendr_module! { ... }`.
///
/// Supports both simple and path forms:
/// - `use submodule;` — simple form
/// - `use crate::submodule;` — path form (leaf segment used as module name)
/// - `use super::submodule;` — path form (leaf segment used as module name)
///
/// The leaf segment determines the generated init/wrapper symbol names:
/// - `name::R_init_<name>_miniextendr(dll)`
/// - `name::R_WRAPPERS_PARTS_<NAME_UPPER>`
///
/// Rejected forms:
/// - `use x as y;` — renamed imports would break R wrapper name correspondence
/// - `use x::*;` — glob imports can't be enumerated statically
/// - `use x::{a, b};` — grouped imports must be listed separately
pub struct MiniextendrModuleUse {
    /// Attributes on the use entry (e.g., `#[cfg(feature = "x")]`).
    pub attrs: Vec<syn::Attribute>,
    _use_token: syn::Token![use],
    /// Target module to re-export wrappers from (leaf segment of the path).
    pub use_name: syn::UseName,
}

impl syn::parse::Parse for MiniextendrModuleUse {
    /// Parses one `use name;` or `use path::to::name;` module entry,
    /// with optional leading attributes (e.g., `#[cfg(feature = "x")] use submod;`).
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let _use_token = input.parse()?;
        let use_tree: syn::UseTree = input.parse()?;
        let use_name = Self::extract_leaf_name(&use_tree)?;
        Ok(Self {
            attrs,
            _use_token,
            use_name,
        })
    }
}

impl MiniextendrModuleUse {
    /// Walks a `UseTree` to extract the leaf `UseName`.
    ///
    /// Accepts `UseTree::Name` (simple) and `UseTree::Path` (walks to leaf).
    /// Rejects rename, glob, and group forms with descriptive errors.
    fn extract_leaf_name(tree: &syn::UseTree) -> syn::Result<syn::UseName> {
        match tree {
            syn::UseTree::Name(use_name) => Ok(use_name.clone()),
            syn::UseTree::Path(use_path) => {
                // Recurse into the nested tree to find the leaf
                Self::extract_leaf_name(&use_path.tree)
            }
            syn::UseTree::Rename(_) => Err(syn::Error::new_spanned(
                tree,
                "renamed imports (`use x as y`) are not supported in miniextendr_module!; \
                     the module name determines R wrapper names",
            )),
            syn::UseTree::Glob(_) => Err(syn::Error::new_spanned(
                tree,
                "glob imports (`use x::*`) are not supported in miniextendr_module!; \
                     modules must be listed explicitly",
            )),
            syn::UseTree::Group(_) => Err(syn::Error::new_spanned(
                tree,
                "grouped imports (`use x::{a, b}`) are not supported in miniextendr_module!; \
                     list each module separately",
            )),
        }
    }
}

/// Parsed body of a `miniextendr_module! { ... }` invocation.
///
/// The body is a semicolon-terminated list of items in any order, with exactly one
/// `mod <name>;` header:
///
/// ```text
/// mod mypkg;
/// use submodule;
/// fn exported_fn;
/// struct MyAltrep;
/// impl Counter;
/// impl MyTrait for Counter;
/// vctrs Percent;
/// ```
pub struct MiniextendrModule {
    /// The module header (`mod <name>;`) that drives symbol generation.
    pub module_name: MiniextendrModuleName,
    /// Submodules to re-export wrappers from (`use foo;`).
    pub uses: Vec<MiniextendrModuleUse>,
    /// Functions registered via `fn name;`.
    pub functions: Vec<MiniextendrModuleFunction>,
    /// ALTREP structs registered via `struct Name;`.
    pub structs: Vec<MiniextendrModuleStruct>,
    /// Impl blocks registered via `impl Type;`.
    pub impls: Vec<MiniextendrModuleImpl>,
    /// Trait impls registered via `impl Trait for Type;`.
    pub trait_impls: Vec<MiniextendrModuleTraitImpl>,
    /// Vctrs types registered via `vctrs Type;`.
    pub vctrs: Vec<MiniextendrModuleVctrs>,
}

/// Internal: one semicolon-terminated item in a `miniextendr_module!` body.
///
/// Used as an intermediate representation during parsing. The [`MiniextendrModule`]
/// parser collects these items, then sorts them into typed vectors.
enum MiniextendrModuleItem {
    /// `mod <name>;` — the required module name header.
    Module(MiniextendrModuleName),
    /// `use <submodule>;` — re-export from a child module.
    Use(MiniextendrModuleUse),
    /// `struct <Name>;` — ALTREP class registration.
    Struct(MiniextendrModuleStruct),
    /// `fn <name>;` — `#[miniextendr]` function registration.
    Func(MiniextendrModuleFunction),
    /// `impl <Type>;` or `impl <Type> as "label";` — impl block registration.
    Impl(MiniextendrModuleImpl),
    /// `impl <Trait> for <Type>;` — trait impl registration (boxed due to larger size).
    TraitImpl(Box<MiniextendrModuleTraitImpl>),
    /// `vctrs <Type>;` — vctrs S3 type registration.
    Vctrs(MiniextendrModuleVctrs),
}

impl MiniextendrModuleItem {
    /// Returns a representative [`Span`](proc_macro2::Span) for this item, used to
    /// attach error messages to the correct source location (e.g., for "missing `mod`" errors).
    fn span(&self) -> proc_macro2::Span {
        use syn::spanned::Spanned;
        match self {
            Self::Module(m) => m.ident.span(),
            Self::Use(u) => u.use_name.ident.span(),
            Self::Struct(s) => s.ident.span(),
            Self::Func(f) => f.ident.span(),
            Self::Impl(i) => i.ident.span(),
            Self::TraitImpl(t) => t.impl_type.span(),
            Self::Vctrs(v) => v.ident.span(),
        }
    }
}

impl syn::parse::Parse for MiniextendrModuleItem {
    /// Parses one semicolon-terminated item inside `miniextendr_module!`.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Skip past attributes to peek at the actual item keyword
        let fork = input.fork();
        let _ = fork.call(syn::Attribute::parse_outer)?;

        let look_ahead = fork.lookahead1();

        if look_ahead.peek(syn::Token![mod]) {
            Ok(Self::Module(input.parse()?))
        } else if look_ahead.peek(syn::Token![use]) {
            Ok(Self::Use(input.parse()?))
        } else if look_ahead.peek(syn::Token![struct]) {
            Ok(Self::Struct(input.parse()?))
        } else if look_ahead.peek(syn::Token![impl]) {
            // Distinguish between `impl Type;` and `impl Trait for Type;`
            // Fork to look ahead: parse `impl Path` and check for `for`
            let fork2 = input.fork();
            let _: syn::Token![impl] = fork2.parse()?;
            let _: syn::Path = fork2.parse()?;
            if fork2.peek(syn::Token![for]) {
                // This is `impl Trait for Type;`
                Ok(Self::TraitImpl(Box::new(input.parse()?)))
            } else {
                // This is `impl Type;`
                Ok(Self::Impl(input.parse()?))
            }
        } else if look_ahead.peek(syn::Token![fn]) || look_ahead.peek(syn::Token![extern]) {
            Ok(Self::Func(input.parse()?))
        } else if look_ahead.peek(syn::Ident) {
            // Check for custom keywords like `vctrs`
            let ident: syn::Ident = fork.parse()?;
            if ident == "vctrs" {
                Ok(Self::Vctrs(input.parse()?))
            } else {
                Err(syn::Error::new(
                    ident.span(),
                    format!(
                        "unknown module item keyword `{}`; expected: mod, use, fn, struct, impl, vctrs",
                        ident
                    ),
                ))
            }
        } else {
            Err(syn::Error::new(
                fork.span(),
                "unrecognized item in miniextendr_module!; expected: mod name;, use submodule;, fn name;, struct Name;, impl Type;, vctrs Name;",
            ))
        }
    }
}

impl syn::parse::Parse for MiniextendrModule {
    /// Parses the full `miniextendr_module!` body.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let items: syn::punctuated::Punctuated<MiniextendrModuleItem, syn::Token![;]> =
            syn::punctuated::Punctuated::parse_terminated_with(
                input,
                MiniextendrModuleItem::parse,
            )?;

        let mut name = None;
        let mut uses = Vec::new();
        let mut funs = Vec::new();
        let mut structs = Vec::new();
        let mut impls = Vec::new();
        let mut trait_impls = Vec::new();
        let mut vctrs_items = Vec::new();
        let mut first_item_span = None::<proc_macro2::Span>;

        for it in items {
            // Capture span of first non-mod item for error reporting
            if first_item_span.is_none() && !matches!(it, MiniextendrModuleItem::Module(_)) {
                first_item_span = Some(it.span());
            }
            match it {
                MiniextendrModuleItem::Module(m) => {
                    if name.is_some() {
                        return Err(syn::Error::new(m._mod_token.span, "duplicate `mod <name>`"));
                    }
                    name = Some(m);
                }
                MiniextendrModuleItem::Use(u) => uses.push(u),
                MiniextendrModuleItem::Struct(s) => structs.push(s),
                MiniextendrModuleItem::Func(f) => funs.push(f),
                MiniextendrModuleItem::Impl(i) => impls.push(i),
                MiniextendrModuleItem::TraitImpl(ti) => trait_impls.push(*ti),
                MiniextendrModuleItem::Vctrs(v) => vctrs_items.push(v),
            }
        }

        let module_name = name.ok_or_else(|| {
            syn::Error::new(
                first_item_span.unwrap_or_else(|| input.span()),
                "missing `mod <name>;` declaration in miniextendr_module!",
            )
        })?;

        Ok(Self {
            module_name,
            uses,
            functions: funs,
            structs,
            impls,
            trait_impls,
            vctrs: vctrs_items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_not_first_item() {
        // mod can appear at any position, not just first
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            fn f;
            mod mypkg;
        };
        let parsed = syn::parse2::<MiniextendrModule>(tokens);
        assert!(parsed.is_ok(), "mod at non-first position should parse");
        assert_eq!(parsed.unwrap().module_name.ident, "mypkg");
    }

    #[test]
    fn missing_mod_errors() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            fn f;
        };
        let parsed = syn::parse2::<MiniextendrModule>(tokens);
        let err = match parsed {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected parse error for missing mod"),
        };
        assert!(
            err.contains("missing `mod <name>;`"),
            "error message should mention missing mod: {err}"
        );
        assert!(
            !err.contains("first item"),
            "error message should not say 'first item': {err}"
        );
    }

    #[test]
    fn use_simple_name() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            mod mypkg;
            use submod;
        };
        let parsed = syn::parse2::<MiniextendrModule>(tokens).unwrap();
        assert_eq!(parsed.uses.len(), 1);
        assert_eq!(parsed.uses[0].use_name.ident, "submod");
    }

    #[test]
    fn use_path_extracts_leaf() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            mod mypkg;
            use crate::submod;
        };
        let parsed = syn::parse2::<MiniextendrModule>(tokens).unwrap();
        assert_eq!(parsed.uses.len(), 1);
        assert_eq!(parsed.uses[0].use_name.ident, "submod");
    }

    #[test]
    fn use_deep_path_extracts_leaf() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            mod mypkg;
            use super::nested::deep::submod;
        };
        let parsed = syn::parse2::<MiniextendrModule>(tokens).unwrap();
        assert_eq!(parsed.uses.len(), 1);
        assert_eq!(parsed.uses[0].use_name.ident, "submod");
    }

    #[test]
    fn use_rename_rejected() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            mod mypkg;
            use submod as renamed;
        };
        let err = match syn::parse2::<MiniextendrModule>(tokens) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected parse error for renamed use"),
        };
        assert!(
            err.contains("renamed imports"),
            "error should mention renamed imports: {err}"
        );
    }

    #[test]
    fn use_glob_rejected() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            mod mypkg;
            use foo::*;
        };
        let err = match syn::parse2::<MiniextendrModule>(tokens) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected parse error for glob use"),
        };
        assert!(
            err.contains("glob imports"),
            "error should mention glob imports: {err}"
        );
    }

    #[test]
    fn use_group_rejected() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            mod mypkg;
            use foo::{bar, baz};
        };
        let err = match syn::parse2::<MiniextendrModule>(tokens) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected parse error for grouped use"),
        };
        assert!(
            err.contains("grouped imports"),
            "error should mention grouped imports: {err}"
        );
    }

    #[test]
    fn use_with_cfg_attribute() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            mod mypkg;
            #[cfg(feature = "rayon")]
            use parallel_mod;
        };
        let parsed = syn::parse2::<MiniextendrModule>(tokens).unwrap();
        assert_eq!(parsed.uses.len(), 1);
        assert_eq!(parsed.uses[0].use_name.ident, "parallel_mod");
        assert_eq!(parsed.uses[0].attrs.len(), 1);
        assert!(parsed.uses[0].attrs[0].path().is_ident("cfg"));
    }

    #[test]
    fn use_with_cfg_and_path() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! {
            mod mypkg;
            #[cfg(feature = "rayon")]
            use crate::parallel_mod;
        };
        let parsed = syn::parse2::<MiniextendrModule>(tokens).unwrap();
        assert_eq!(parsed.uses.len(), 1);
        assert_eq!(parsed.uses[0].use_name.ident, "parallel_mod");
        assert_eq!(parsed.uses[0].attrs.len(), 1);
    }
}
