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
//!
//! # IMPORTANT: Duplicated in miniextendr-lint
//!
//! **This file is copied to `miniextendr-lint/src/miniextendr_module.rs`.**
//!
//! Changes to this file should be manually copied to the lint crate to keep
//! the parser in sync. The duplication exists to allow independent publishing
//! to crates.io (cross-crate `#[path]` includes don't work in published packages).
//!
//! **When modifying this file:**
//! 1. Make your changes here (in miniextendr-macros)
//! 2. Copy the updated file to `miniextendr-lint/src/miniextendr_module.rs`
//! 3. Update lint's helper functions if you add new imports from `crate::`
//! 4. Test both crates build successfully
//!
//! **Constraints:**
//! - Keep imports from `crate::` minimal (lint must provide stubs)
//! - Parser should be as self-contained as possible
//!
//! See `miniextendr-lint/src/lib.rs` module docs for more details.

use crate::{call_method_def_ident_for, r_wrapper_const_ident_for};

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
pub(crate) struct MiniextendrModuleFunction {
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
    /// Identifier for the generated `R_CallMethodDef` entry for this function.
    pub(crate) fn call_method_def_ident(&self) -> syn::Ident {
        call_method_def_ident_for(&self.ident)
    }

    /// Identifier for the generated R wrapper source string const.
    pub(crate) fn r_wrapper_const_ident(&self) -> syn::Ident {
        r_wrapper_const_ident_for(&self.ident)
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
pub(crate) struct MiniextendrModuleStruct {
    _struct_token: syn::Token![struct],
    #[allow(dead_code)]
    /// Name of the ALTREP struct to register.
    pub ident: syn::Ident,
}

impl syn::parse::Parse for MiniextendrModuleStruct {
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
pub(crate) struct MiniextendrModuleName {
    _mod_token: syn::Token![mod],
    /// Base name that drives `R_init_<name>_miniextendr` symbol generation.
    pub ident: syn::Ident,
}

impl syn::parse::Parse for MiniextendrModuleName {
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
pub(crate) struct MiniextendrModuleImpl {
    /// Attributes on the impl entry (passed through for cfg/doc parity).
    pub attrs: Vec<syn::Attribute>,
    _impl_token: syn::Token![impl],
    /// Type that has a `#[miniextendr(...)]` impl block.
    pub ident: syn::Ident,
    /// Optional label for distinguishing multiple impl blocks of the same type.
    pub label: Option<String>,
}

impl syn::parse::Parse for MiniextendrModuleImpl {
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
pub(crate) struct MiniextendrModuleTraitImpl {
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
pub(crate) enum AltrepBase {
    Integer,
    Real,
    Logical,
    Raw,
    String,
    Complex,
    List,
}

impl MiniextendrModuleTraitImpl {
    /// Returns a sanitized string name for the type, suitable for identifier generation.
    ///
    /// Converts generic types like `Vec<i32>` to `Vec_i32`, `Range<i32>` to `Range_i32`, etc.
    pub(crate) fn type_name_sanitized(&self) -> String {
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

    /// Returns the identifier for the call defs const.
    /// Format: `{TYPE}_{TRAIT}_CALL_DEFS`
    pub(crate) fn call_defs_const_ident(&self) -> syn::Ident {
        let type_upper = self.type_name_sanitized().to_uppercase();
        let trait_name = self
            .trait_path
            .segments
            .last()
            .map(|s| s.ident.to_string().to_uppercase())
            .unwrap_or_default();
        quote::format_ident!("{}_{}_CALL_DEFS", type_upper, trait_name)
    }

    /// Returns the identifier for the R wrappers const.
    /// Format: `R_WRAPPERS_{TYPE}_{TRAIT}_IMPL`
    pub(crate) fn r_wrappers_const_ident(&self) -> syn::Ident {
        let type_upper = self.type_name_sanitized().to_uppercase();
        let trait_name = self
            .trait_path
            .segments
            .last()
            .map(|s| s.ident.to_string().to_uppercase())
            .unwrap_or_default();
        quote::format_ident!("R_WRAPPERS_{}_{}_IMPL", type_upper, trait_name)
    }

    /// Returns the trait name (last segment of the path).
    pub(crate) fn trait_name(&self) -> String {
        self.trait_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default()
    }

    /// Checks if this trait impl is for an ALTREP data trait.
    /// Returns the ALTREP base type if so.
    pub(crate) fn altrep_base(&self) -> Option<AltrepBase> {
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
    pub(crate) fn simple_type_ident(&self) -> Option<&syn::Ident> {
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
    /// Returns the identifier for the call defs const.
    ///
    /// Format: `{TYPE}_CALL_DEFS` or `{TYPE}_{LABEL}_CALL_DEFS` if labeled.
    pub(crate) fn call_defs_const_ident(&self) -> syn::Ident {
        let type_upper = self.ident.to_string().to_uppercase();
        if let Some(ref label) = self.label {
            let label_upper = label.to_uppercase();
            quote::format_ident!("{}_{}_CALL_DEFS", type_upper, label_upper)
        } else {
            quote::format_ident!("{}_CALL_DEFS", type_upper)
        }
    }

    /// Returns the identifier for the R wrappers const.
    ///
    /// Format: `R_WRAPPERS_IMPL_{TYPE}` or `R_WRAPPERS_IMPL_{TYPE}_{LABEL}` if labeled.
    pub(crate) fn r_wrappers_const_ident(&self) -> syn::Ident {
        let type_upper = self.ident.to_string().to_uppercase();
        if let Some(ref label) = self.label {
            let label_upper = label.to_uppercase();
            quote::format_ident!("R_WRAPPERS_IMPL_{}_{}", type_upper, label_upper)
        } else {
            quote::format_ident!("R_WRAPPERS_IMPL_{}", type_upper)
        }
    }

    /// Returns the label if present.
    #[allow(dead_code)]
    pub(crate) fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the identifier for sidecar call defs const (from #[derive(ExternalPtr)]).
    ///
    /// Format: `RDATA_CALL_DEFS_{TYPE}`
    ///
    /// This constant is generated by `#[derive(ExternalPtr)]` when the struct has
    /// `#[r_data]` fields with `RSidecar` and `RData` markers.
    pub(crate) fn rdata_call_defs_const_ident(&self) -> syn::Ident {
        let type_upper = self.ident.to_string().to_uppercase();
        quote::format_ident!("RDATA_CALL_DEFS_{}", type_upper)
    }

    /// Returns the identifier for sidecar R wrappers const (from #[derive(ExternalPtr)]).
    ///
    /// Format: `R_WRAPPERS_RDATA_{TYPE}`
    pub(crate) fn rdata_r_wrappers_const_ident(&self) -> syn::Ident {
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
pub(crate) struct MiniextendrModuleVctrs {
    /// Attributes on the vctrs entry (for cfg propagation).
    #[cfg_attr(not(feature = "vctrs"), allow(dead_code))]
    pub attrs: Vec<syn::Attribute>,
    /// Identifier for the custom `vctrs` keyword.
    pub _vctrs_ident: syn::Ident,
    /// Type that has `#[derive(Vctrs)]` attribute.
    pub ident: syn::Ident,
}

impl syn::parse::Parse for MiniextendrModuleVctrs {
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
    /// Returns the identifier for the R wrappers const.
    ///
    /// Format: `R_WRAPPERS_VCTRS_{TYPE}`
    #[cfg(feature = "vctrs")]
    pub(crate) fn r_wrappers_const_ident(&self) -> syn::Ident {
        let type_upper = self.ident.to_string().to_uppercase();
        quote::format_ident!("R_WRAPPERS_VCTRS_{}", type_upper)
    }
}

/// A `use <module>;` line inside `miniextendr_module! { ... }`.
///
/// Only the simple `use name;` form is supported. This is intentionally restrictive so the
/// generated init/wrapper symbol names are predictable:
/// - `name::R_init_<name>_miniextendr(dll)`
/// - `name::R_WRAPPERS_PARTS_<NAME_UPPER>`
pub(crate) struct MiniextendrModuleUse {
    _use_token: syn::Token![use],
    /// Target module to re-export wrappers from.
    pub use_name: syn::UseName,
}

impl syn::parse::Parse for MiniextendrModuleUse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::spanned::Spanned;
        let _use_token = input.parse()?;
        let use_name: syn::UseTree = input.parse()?;
        let use_name = match use_name {
            syn::UseTree::Name(use_name) => use_name,
            syn::UseTree::Rename(use_rename) => {
                return Err(syn::Error::new(
                    use_rename.span(),
                    "it is not possible to rename wrappers in `miniextendr_module`",
                ));
            }
            syn::UseTree::Path(_) | syn::UseTree::Glob(_) | syn::UseTree::Group(_) => {
                return Err(syn::Error::new(use_name.span(), "syntax not supported"));
            }
        };
        Ok(Self {
            _use_token,
            use_name,
        })
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
pub(crate) struct MiniextendrModule {
    /// The module header (`mod <name>;`) that drives symbol generation.
    pub(crate) module_name: MiniextendrModuleName,
    /// Submodules to re-export wrappers from (`use foo;`).
    pub(crate) uses: Vec<MiniextendrModuleUse>,
    /// Functions registered via `fn name;`.
    pub(crate) functions: Vec<MiniextendrModuleFunction>,
    /// ALTREP structs registered via `struct Name;`.
    pub(crate) structs: Vec<MiniextendrModuleStruct>,
    /// Impl blocks registered via `impl Type;`.
    pub(crate) impls: Vec<MiniextendrModuleImpl>,
    /// Trait impls registered via `impl Trait for Type;`.
    pub(crate) trait_impls: Vec<MiniextendrModuleTraitImpl>,
    /// Vctrs types registered via `vctrs Type;`.
    #[cfg_attr(not(feature = "vctrs"), allow(dead_code))]
    pub(crate) vctrs: Vec<MiniextendrModuleVctrs>,
}

/// Internal: one semicolon-terminated item in a `miniextendr_module!` body.
enum MiniextendrModuleItem {
    Module(MiniextendrModuleName),
    Use(MiniextendrModuleUse),
    Struct(MiniextendrModuleStruct),
    Func(MiniextendrModuleFunction),
    Impl(MiniextendrModuleImpl),
    TraitImpl(Box<MiniextendrModuleTraitImpl>),
    Vctrs(MiniextendrModuleVctrs),
}

impl MiniextendrModuleItem {
    /// Get a representative span for this item (for error reporting).
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
            Err(look_ahead.error())
        }
    }
}

impl syn::parse::Parse for MiniextendrModule {
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
                "missing `mod <name>;` declaration (required as first item in miniextendr_module!)",
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
