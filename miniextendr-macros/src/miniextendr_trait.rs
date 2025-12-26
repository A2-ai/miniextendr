//! # Trait Support for `#[miniextendr]`
//!
//! This module handles `#[miniextendr]` applied to trait definitions,
//! generating the ABI infrastructure for cross-package trait dispatch.
//!
//! ## Overview
//!
//! When `#[miniextendr]` is applied to a trait, it generates:
//!
//! 1. **Type tag constant** (`TAG_<TraitName>`) - 128-bit identifier for runtime type checking
//! 2. **Vtable struct** (`<TraitName>VTable`) - Function pointer table for method dispatch
//! 3. **View struct** (`<TraitName>View`) - Runtime wrapper combining data pointer and vtable
//! 4. **Method shims** - `extern "C"` functions that convert SEXP arguments and call methods
//! 5. **Vtable builder** - `__<trait>_build_vtable::<T>()` for impl blocks
//!
//! ## Usage
//!
//! ```ignore
//! #[miniextendr]
//! pub trait Counter {
//!     fn value(&self) -> i32;
//!     fn increment(&mut self);
//!     fn add(&mut self, n: i32);
//! }
//! ```
//!
//! Generates (conceptually):
//!
//! ```ignore
//! // Original trait (passed through)
//! pub trait Counter {
//!     fn value(&self) -> i32;
//!     fn increment(&mut self);
//!     fn add(&mut self, n: i32);
//! }
//!
//! // Type tag for runtime identification
//! pub const TAG_COUNTER: mx_tag = mx_tag::new(0x..., 0x...);
//!
//! // Vtable with one entry per method
//! #[repr(C)]
//! pub struct CounterVTable {
//!     pub value: mx_meth,
//!     pub increment: mx_meth,
//!     pub add: mx_meth,
//! }
//!
//! // View combining data pointer and vtable
//! #[repr(C)]
//! pub struct CounterView {
//!     pub data: *mut std::ffi::c_void,
//!     pub vtable: *const CounterVTable,
//! }
//!
//! // Shim for each method
//! unsafe extern "C" fn __counter_value_shim<T: Counter>(
//!     data: *mut c_void, argc: i32, argv: *const SEXP
//! ) -> SEXP {
//!     // 1. Check arity
//!     // 2. Cast data to &T
//!     // 3. Call method
//!     // 4. Convert result to SEXP
//!     // 5. Catch panics
//! }
//!
//! // Builder to create vtable for a concrete type
//! pub const fn __counter_build_vtable<T: Counter>() -> CounterVTable {
//!     CounterVTable {
//!         value: __counter_value_shim::<T>,
//!         increment: __counter_increment_shim::<T>,
//!         add: __counter_add_shim::<T>,
//!     }
//! }
//! ```
//!
//! ## Supported Method Signatures
//!
//! Methods must follow these constraints:
//!
//! - **Receiver**: `&self` or `&mut self` only (no `self`, no generics)
//! - **Arguments**: Types that implement `TryFromSexp`
//! - **Return**: Types that implement `IntoR`, or `()`
//! - **No generics**: Methods cannot have generic type parameters
//! - **No async**: Async methods are not supported
//!
//! ## Default Methods
//!
//! Default method implementations are supported. The vtable builder will
//! use the default implementation if the concrete type doesn't override it.
//!
//! ```ignore
//! #[miniextendr]
//! pub trait Counter {
//!     fn value(&self) -> i32;
//!
//!     // Default implementation - included in vtable
//!     fn is_zero(&self) -> bool {
//!         self.value() == 0
//!     }
//! }
//! ```
//!
//! ## Error Handling
//!
//! Method shims handle errors as follows:
//!
//! - **Arity mismatch**: Calls `r_stop("expected N arguments, got M")`
//! - **Type conversion failure**: Calls `r_stop` with the error message
//! - **Panic**: Caught via `catch_unwind`, converted to `r_stop`
//!
//! ## Thread Safety
//!
//! All generated shims are **main-thread only**. They do not route through
//! `with_r_thread` because R invokes `.Call` on the main thread.

use proc_macro2::TokenStream;
use syn::ItemTrait;

/// Expand `#[miniextendr]` applied to a trait definition.
///
/// # Arguments
///
/// * `attr` - Attribute arguments (currently unused, reserved for future options)
/// * `item` - The trait definition token stream
///
/// # Returns
///
/// Expanded token stream containing:
/// - Original trait definition
/// - Type tag constant
/// - Vtable struct
/// - View struct
/// - Method shims
/// - Vtable builder function
///
/// # Errors
///
/// Returns a compile error if:
/// - Trait has generic type parameters
/// - Methods have unsupported signatures
/// - Methods are async
pub fn expand_trait(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let trait_item = syn::parse_macro_input!(item as ItemTrait);

    // Validate trait constraints
    if let Err(e) = validate_trait(&trait_item) {
        return e.into_compile_error().into();
    }

    // Generate the expanded code
    let expanded = generate_trait_abi(&trait_item);

    expanded.into()
}

/// Validate that the trait meets requirements for ABI generation.
///
/// # Constraints
///
/// - No generic type parameters on the trait itself
/// - All methods must have `&self` or `&mut self` receiver
/// - Methods cannot be async
/// - Methods cannot have generic parameters
fn validate_trait(trait_item: &ItemTrait) -> syn::Result<()> {
    let trait_name = &trait_item.ident;

    // Check for generic parameters on trait
    if !trait_item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &trait_item.generics,
            "#[miniextendr] traits cannot have generic parameters",
        ));
    }

    // Validate each method
    for item in &trait_item.items {
        if let syn::TraitItem::Fn(method) = item {
            validate_method(method, trait_name)?;
        }
    }

    Ok(())
}

/// Validate a single trait method.
fn validate_method(method: &syn::TraitItemFn, trait_name: &syn::Ident) -> syn::Result<()> {
    let method_name = &method.sig.ident;

    // Check for async
    if method.sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            &method.sig.asyncness,
            format!(
                "#[miniextendr] trait `{}::{}` cannot be async",
                trait_name, method_name
            ),
        ));
    }

    // Check for generics on method
    if !method.sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &method.sig.generics,
            format!(
                "#[miniextendr] trait method `{}::{}` cannot have generic parameters",
                trait_name, method_name
            ),
        ));
    }

    // Check receiver - must be &self or &mut self
    let has_valid_receiver = method.sig.inputs.first().is_some_and(|arg| {
        matches!(
            arg,
            syn::FnArg::Receiver(r) if r.reference.is_some() && r.colon_token.is_none()
        )
    });

    if !has_valid_receiver {
        return Err(syn::Error::new_spanned(
            &method.sig,
            format!(
                "#[miniextendr] trait method `{}::{}` must have `&self` or `&mut self` receiver",
                trait_name, method_name
            ),
        ));
    }

    Ok(())
}

/// Generate the ABI infrastructure for a trait.
///
/// This is the main code generation function that produces:
/// - Type tag constant
/// - Vtable struct
/// - View struct
/// - Method shims
/// - Vtable builder
fn generate_trait_abi(trait_item: &ItemTrait) -> TokenStream {
    let trait_name = &trait_item.ident;
    let _vis = &trait_item.vis;

    // Generate names for generated items
    let tag_name = quote::format_ident!("TAG_{}", trait_name.to_string().to_uppercase());
    let vtable_name = quote::format_ident!("{}VTable", trait_name);
    let view_name = quote::format_ident!("{}View", trait_name);
    let build_vtable_fn = quote::format_ident!(
        "__{}_build_vtable",
        trait_name.to_string().to_lowercase()
    );

    // TODO: Implement full code generation
    //
    // For now, generate a compile_error! to indicate this is scaffolding.
    // The actual implementation will:
    //
    // 1. Generate TAG constant using mx_tag_from_path or hash
    // 2. Generate VTable struct with mx_meth fields for each method
    // 3. Generate View struct with data pointer and vtable pointer
    // 4. Generate shim functions for each method
    // 5. Generate build_vtable function

    let trait_name_str = trait_name.to_string();

    quote::quote! {
        // Pass through the original trait
        #trait_item

        // Scaffolding - not yet implemented
        //
        // TODO: Generate these items:
        //
        // #vis const #tag_name: ::miniextendr_api::abi::mx_tag = ...;
        //
        // #[repr(C)]
        // #vis struct #vtable_name { ... }
        //
        // #[repr(C)]
        // #vis struct #view_name { ... }
        //
        // #vis const fn #build_vtable_fn<T: #trait_name>() -> #vtable_name { ... }

        compile_error!(concat!(
            "#[miniextendr] on traits is not yet implemented.\n",
            "Trait: ", #trait_name_str, "\n",
            "This is scaffolding for the trait ABI system.\n",
            "\n",
            "Expected generated items:\n",
            "  - const ", stringify!(#tag_name), ": mx_tag\n",
            "  - struct ", stringify!(#vtable_name), "\n",
            "  - struct ", stringify!(#view_name), "\n",
            "  - fn ", stringify!(#build_vtable_fn), "<T>()\n",
        ));
    }
}

/// Information extracted from a trait method for code generation.
#[derive(Debug)]
#[allow(dead_code)]
struct MethodInfo {
    /// Method name
    name: syn::Ident,
    /// Whether receiver is `&mut self` (vs `&self`)
    is_mut: bool,
    /// Parameter types (excluding self)
    param_types: Vec<syn::Type>,
    /// Parameter names (excluding self)
    param_names: Vec<syn::Ident>,
    /// Return type (None for `()`)
    return_type: Option<syn::Type>,
    /// Whether method has a default implementation
    has_default: bool,
}

/// Extract method information from a trait method.
#[allow(dead_code)]
fn extract_method_info(method: &syn::TraitItemFn) -> MethodInfo {
    let name = method.sig.ident.clone();

    // Check if receiver is &mut self
    let is_mut = method.sig.inputs.first().is_some_and(|arg| {
        matches!(arg, syn::FnArg::Receiver(r) if r.mutability.is_some())
    });

    // Extract parameters (skip self)
    let mut param_types = Vec::new();
    let mut param_names = Vec::new();
    for (i, arg) in method.sig.inputs.iter().skip(1).enumerate() {
        if let syn::FnArg::Typed(pat_type) = arg {
            param_types.push((*pat_type.ty).clone());
            if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                param_names.push(pat_ident.ident.clone());
            } else {
                param_names.push(quote::format_ident!("arg{}", i));
            }
        }
    }

    // Extract return type
    let return_type = match &method.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => {
            // Check if it's unit type ()
            if matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty()) {
                None
            } else {
                Some((**ty).clone())
            }
        }
    };

    // Check for default implementation
    let has_default = method.default.is_some();

    MethodInfo {
        name,
        is_mut,
        param_types,
        param_names,
        return_type,
        has_default,
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add tests for trait validation and code generation
}
