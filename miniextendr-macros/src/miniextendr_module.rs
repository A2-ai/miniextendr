//! Module declaration parsing for `miniextendr_module!`.
//!
//! This module handles parsing the `miniextendr_module! { ... }` macro body,
//! which registers functions and ALTREP types with R.
//!
//! # Supported items
//!
//! - `mod <name>;` - Required module name (determines `R_init_<name>_miniextendr` symbol)
//! - `fn <name>;` - Register a `#[miniextendr]` function
//! - `extern "C-unwind" fn <name>;` - Register a raw C ABI function
//! - `struct <name>;` - Register an ALTREP class
//! - `use <submodule>;` - Re-export from a submodule

use crate::{call_method_def_ident_for, r_wrapper_const_ident_for};

/// A single `fn ...;` line inside `miniextendr_module! { ... }`.
///
/// Supported syntaxes:
///
/// ```text
/// fn my_rust_function;
/// extern "C-unwind" fn C_raw_symbol;
/// ```
///
/// Note: To conditionally compile functions, place `#[cfg(...)]` AFTER `#[miniextendr]`
/// on the function definition itself, not in this module declaration.
pub(crate) struct MiniextendrModuleFunction {
    pub _abi: Option<syn::Abi>,
    _fn_token: syn::Token![fn],
    pub ident: syn::Ident,
}

impl syn::parse::Parse for MiniextendrModuleFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _abi = if input.peek(syn::Token![extern]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            _abi,
            _fn_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl MiniextendrModuleFunction {
    pub(crate) fn call_method_def_ident(&self) -> syn::Ident {
        call_method_def_ident_for(&self.ident)
    }

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

/// A `use <module>;` line inside `miniextendr_module! { ... }`.
///
/// Only the simple `use name;` form is supported. This is intentionally restrictive so the
/// generated init/wrapper symbol names are predictable:
/// - `name::R_init_<name>_miniextendr(dll)`
/// - `name::R_WRAPPERS_PARTS_<NAME_UPPER>`
pub(crate) struct MiniextendrModuleUse {
    _use_token: syn::Token![use],
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
/// ```
pub(crate) struct MiniextendrModule {
    pub(crate) module_name: MiniextendrModuleName,
    pub(crate) uses: Vec<MiniextendrModuleUse>,
    pub(crate) functions: Vec<MiniextendrModuleFunction>,
    pub(crate) structs: Vec<MiniextendrModuleStruct>,
}

/// Internal: one semicolon-terminated item in a `miniextendr_module!` body.
enum MiniextendrModuleItem {
    Module(MiniextendrModuleName),
    Use(MiniextendrModuleUse),
    Struct(MiniextendrModuleStruct),
    Func(MiniextendrModuleFunction),
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
        } else if look_ahead.peek(syn::Token![fn]) || look_ahead.peek(syn::Token![extern]) {
            Ok(Self::Func(input.parse()?))
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

        for it in items {
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
            }
        }

        let module_name =
            name.ok_or_else(|| syn::Error::new(input.span(), "missing `mod <name>`"))?;

        Ok(Self {
            module_name,
            uses,
            functions: funs,
            structs,
        })
    }
}
