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
//! ```text
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
//! - **Receiver**: `&self` or `&mut self` for instance methods, or none for static methods
//! - **Arguments**: Types that implement `TryFromSexp`
//! - **Return**: Types that implement `IntoR`, or `()`
//! - **No generics**: Methods cannot have generic type parameters
//! - **No async**: Async methods are not supported
//! - **Static methods**: Methods without a receiver are allowed and resolved at compile time
//!   (they don't go through the vtable)
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

    // Check receiver - must be &self, &mut self, or no receiver (static method)
    // Static methods are allowed but won't be included in the vtable
    // (they're resolved at compile time via <Type as Trait>::method())
    let receiver = method.sig.inputs.first();
    if let Some(syn::FnArg::Receiver(r)) = receiver {
        // If there's a receiver, it must be &self or &mut self (not self by value)
        if r.reference.is_none() {
            return Err(syn::Error::new_spanned(
                &method.sig,
                format!(
                    "#[miniextendr] trait method `{}::{}` receiver must be `&self` or `&mut self`, not `self` by value",
                    trait_name, method_name
                ),
            ));
        }
        if r.colon_token.is_some() {
            return Err(syn::Error::new_spanned(
                &method.sig,
                format!(
                    "#[miniextendr] trait method `{}::{}` receiver cannot have explicit type annotation",
                    trait_name, method_name
                ),
            ));
        }
    }
    // If receiver is None or FnArg::Typed (no self), it's a static method - allowed

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
    let vis = &trait_item.vis;

    // Generate names for generated items
    let tag_name = quote::format_ident!("TAG_{}", trait_name.to_string().to_uppercase());
    let vtable_name = quote::format_ident!("{}VTable", trait_name);
    let view_name = quote::format_ident!("{}View", trait_name);
    let build_vtable_fn =
        quote::format_ident!("__{}_build_vtable", trait_name.to_string().to_lowercase());

    // Collect method information
    // Filter to only include instance methods (with &self or &mut self) for vtable
    // Static methods are resolved at compile time and don't need vtable dispatch
    let methods: Vec<_> = trait_item
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(method) = item {
                let info = extract_method_info(method);
                // Only include instance methods in vtable
                if info.has_self { Some(info) } else { None }
            } else {
                None
            }
        })
        .collect();

    // Generate tag path string for hashing
    // Use module_path!() at expansion site for proper namespacing
    let tag_path = format!("{{}}::{}", trait_name);

    // Generate vtable fields
    let vtable_fields: Vec<_> = methods
        .iter()
        .map(|m| {
            let name = &m.name;
            quote::quote! {
                pub #name: ::miniextendr_api::abi::mx_meth
            }
        })
        .collect();

    // Generate shim functions and vtable field initializers
    let shim_fns: Vec<_> = methods
        .iter()
        .map(|m| generate_method_shim(trait_name, m))
        .collect();

    let vtable_inits: Vec<_> = methods
        .iter()
        .map(|m| {
            let name = &m.name;
            let shim_name =
                quote::format_ident!("__{}_{}_shim", trait_name.to_string().to_lowercase(), name);
            quote::quote! {
                #name: #shim_name::<T>
            }
        })
        .collect();

    quote::quote! {
        // Pass through the original trait
        #trait_item

        #[doc = concat!(
            "Type tag for runtime identification of the `",
            stringify!(#trait_name),
            "` trait."
        )]
        #vis const #tag_name: ::miniextendr_api::abi::mx_tag =
            ::miniextendr_api::abi::mx_tag_from_path(concat!(module_path!(), #tag_path));

        #[doc = concat!("Vtable for the `", stringify!(#trait_name), "` trait.")]
        ///
        /// Contains one `mx_meth` function pointer per trait method.
        #[repr(C)]
        struct #vtable_name {
            #(#vtable_fields),*
        }

        #[doc = concat!(
            "Runtime view for objects implementing `",
            stringify!(#trait_name),
            "`."
        )]
        ///
        /// Combines a data pointer with a vtable pointer for method dispatch.
        #[repr(C)]
        #vis struct #view_name {
            /// Pointer to the concrete object data.
            pub data: *mut ::std::os::raw::c_void,
            /// Pointer to the vtable for this trait.
            pub vtable: *const #vtable_name,
        }

        // Method shims
        #(#shim_fns)*

        #[doc = concat!(
            "Build a vtable for a concrete type implementing `",
            stringify!(#trait_name),
            "`."
        )]
        #vis const fn #build_vtable_fn<T: #trait_name>() -> #vtable_name {
            #vtable_name {
                #(#vtable_inits),*
            }
        }
    }
}

/// Generate a method shim function for a trait method.
///
/// The shim is an `extern "C"` function that:
/// 1. Checks argument arity
/// 2. Wraps everything in `catch_unwind` to prevent unwinding across FFI
/// 3. Converts SEXP arguments to Rust types
/// 4. Calls the actual method on the concrete type
/// 5. Converts the result back to SEXP
/// 6. On panic, converts to R error via `r_stop`
fn generate_method_shim(trait_name: &syn::Ident, method: &MethodInfo) -> TokenStream {
    let method_name = &method.name;
    let shim_name = quote::format_ident!(
        "__{}_{}_shim",
        trait_name.to_string().to_lowercase(),
        method_name
    );

    let param_count = method.param_types.len();
    let expected_argc = param_count as i32;

    // Generate argument extraction
    let arg_extractions: Vec<_> = method
        .param_names
        .iter()
        .zip(method.param_types.iter())
        .enumerate()
        .map(|(i, (name, ty))| {
            let name_str = name.to_string();
            quote::quote! {
                let #name: #ty = unsafe {
                    ::miniextendr_api::trait_abi::extract_arg(argc, argv, #i, #name_str)
                };
            }
        })
        .collect();

    // Generate method call
    let param_names = &method.param_names;
    let method_call = if method.is_mut {
        quote::quote! {
            let self_ref = unsafe { &mut *(data as *mut T) };
            self_ref.#method_name(#(#param_names),*)
        }
    } else {
        quote::quote! {
            let self_ref = unsafe { &*(data as *const T) };
            self_ref.#method_name(#(#param_names),*)
        }
    };

    // Generate result conversion
    let result_conversion = if method.return_type.is_some() {
        quote::quote! {
            unsafe { ::miniextendr_api::trait_abi::to_sexp(result) }
        }
    } else {
        quote::quote! {
            let _ = result;
            unsafe { ::miniextendr_api::trait_abi::nil() }
        }
    };

    let method_name_str = format!("{}::{}", trait_name, method_name);

    quote::quote! {
        #[doc = concat!(
            "Method shim for `",
            stringify!(#trait_name),
            "::",
            stringify!(#method_name),
            "`."
        )]
        ///
        /// Converts SEXP arguments, calls the method, and returns SEXP result.
        /// Panics are caught via `catch_unwind` and converted to R errors.
        #[doc(hidden)]
        unsafe extern "C" fn #shim_name<T: #trait_name>(
            data: *mut ::std::os::raw::c_void,
            argc: i32,
            argv: *const ::miniextendr_api::ffi::SEXP,
        ) -> ::miniextendr_api::ffi::SEXP {
            // Check arity (before catch_unwind - uses r_stop which doesn't return)
            unsafe {
                ::miniextendr_api::trait_abi::check_arity(argc, #expected_argc, #method_name_str);
            }

            // Wrap everything in catch_unwind to prevent unwinding across FFI
            let panic_result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                // Extract arguments
                #(#arg_extractions)*

                // Call method
                let result = { #method_call };

                // Convert result
                #result_conversion
            }));

            match panic_result {
                Ok(sexp) => sexp,
                Err(payload) => {
                    // Convert panic to R error
                    let msg = ::miniextendr_api::worker::panic_payload_to_string(&payload);
                    ::miniextendr_api::worker::panic_message_to_r_error(msg)
                }
            }
        }
    }
}

/// Information extracted from a trait method for code generation.
#[derive(Debug)]
struct MethodInfo {
    /// Method name
    name: syn::Ident,
    /// Whether the method has a self receiver (instance method)
    has_self: bool,
    /// Whether receiver is `&mut self` (vs `&self`) - only meaningful if has_self is true
    is_mut: bool,
    /// Parameter types (excluding self)
    param_types: Vec<syn::Type>,
    /// Parameter names (excluding self)
    param_names: Vec<syn::Ident>,
    /// Return type (None for `()`)
    return_type: Option<syn::Type>,
    /// Whether method has a default implementation
    #[allow(dead_code)]
    has_default: bool,
}

/// Extract method information from a trait method.
fn extract_method_info(method: &syn::TraitItemFn) -> MethodInfo {
    let name = method.sig.ident.clone();

    // Check for receiver
    let (has_self, is_mut) = method.sig.inputs.first().map_or((false, false), |arg| {
        if let syn::FnArg::Receiver(r) = arg {
            (true, r.mutability.is_some())
        } else {
            (false, false)
        }
    });

    // Extract parameters (skip self if present)
    let skip_count = if has_self { 1 } else { 0 };
    let mut param_types = Vec::new();
    let mut param_names = Vec::new();
    for (i, arg) in method.sig.inputs.iter().skip(skip_count).enumerate() {
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
        has_self,
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
