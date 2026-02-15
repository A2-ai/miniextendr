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
            method.sig.asyncness,
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

    // Check receiver - must be &self, &mut self, self: &Self, self: &mut Self, or no receiver
    // Static methods are allowed but won't be included in the vtable
    // (they're resolved at compile time via <Type as Trait>::method())
    let receiver = method.sig.inputs.first();
    if let Some(syn::FnArg::Receiver(r)) = receiver {
        // Accept either:
        // - `&self` / `&mut self` (r.reference is Some)
        // - `self: &Self` / `self: &mut Self` (r.colon_token is Some with reference type)
        let is_ref = if r.reference.is_some() {
            true
        } else if r.colon_token.is_some() {
            // Check if the type is a reference type (&Self or &mut Self)
            matches!(r.ty.as_ref(), syn::Type::Reference(_))
        } else {
            false
        };

        if !is_ref {
            return Err(syn::Error::new_spanned(
                r,
                format!(
                    "#[miniextendr] trait method `{}::{}` receiver must be `&self` or `&mut self`, not `self` by value",
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
    // IMPORTANT: For cross-package trait dispatch, the tag must NOT include module_path!()
    // Different packages defining the same trait signature should get the same tag.
    // We use just the trait name - in practice, trait names + methods should be unique enough.
    let tag_path = trait_name.to_string();

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

    // Compute extra bounds needed for shims (Self returns → IntoR, &Self params → TypedExternal)
    let extra_bounds = compute_extra_bounds(&methods);

    // Generate shim functions and vtable field initializers
    let shim_fns: Vec<_> = methods
        .iter()
        .map(|m| generate_method_shim(trait_name, m, &extra_bounds))
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

    // Generate method wrappers for the View struct
    // Methods with Self in return type or &Self in params are skipped
    let view_methods: Vec<_> = methods.iter().filter_map(generate_view_method).collect();

    let trait_name_str = trait_name.to_string();
    let source_loc_doc = crate::source_location_doc(trait_name.span());

    quote::quote! {
        // Pass through the original trait
        #trait_item

        #[doc = concat!(
            "Type tag for runtime identification of the `",
            stringify!(#trait_name),
            "` trait."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #vis const #tag_name: ::miniextendr_api::abi::mx_tag =
            ::miniextendr_api::abi::mx_tag_from_path(#tag_path);

        #[doc = concat!("Vtable for the `", stringify!(#trait_name), "` trait.")]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        ///
        /// Contains one `mx_meth` function pointer per trait method.
        #[repr(C)]
        #[doc(hidden)]
        #vis struct #vtable_name {
            #(#vtable_fields),*
        }

        #[doc = concat!(
            "Runtime view for objects implementing `",
            stringify!(#trait_name),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        ///
        /// Combines a data pointer with a vtable pointer for method dispatch.
        /// Use `try_from_sexp` to create a view from an R external pointer.
        #[repr(C)]
        #vis struct #view_name {
            /// Pointer to the concrete object data.
            pub data: *mut ::std::os::raw::c_void,
            /// Pointer to the vtable for this trait.
            pub vtable: *const #vtable_name,
        }

        // TraitView implementation
        impl ::miniextendr_api::TraitView for #view_name {
            const TAG: ::miniextendr_api::abi::mx_tag = #tag_name;

            #[inline]
            unsafe fn from_raw_parts(
                data: *mut ::std::os::raw::c_void,
                vtable: *const ::std::os::raw::c_void,
            ) -> Self {
                Self {
                    data,
                    vtable: vtable as *const #vtable_name,
                }
            }
        }

        // Method wrappers on View
        impl #view_name {
            /// Try to create a view from an R SEXP.
            ///
            /// Returns `Some(Self)` if the object implements this trait,
            /// `None` otherwise.
            ///
            /// # Safety
            ///
            /// - `sexp` must be a valid R external pointer (EXTPTRSXP)
            /// - Must be called on R's main thread
            #[inline]
            pub unsafe fn try_from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> Option<Self> {
                <Self as ::miniextendr_api::TraitView>::try_from_sexp(sexp)
            }

            /// Try to create a view, panicking with error message on failure.
            ///
            /// # Safety
            ///
            /// Same as `try_from_sexp`.
            #[inline]
            pub unsafe fn from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> Self {
                Self::try_from_sexp(sexp)
                    .expect(concat!("Object does not implement ", #trait_name_str, " trait"))
            }

            #(#view_methods)*
        }

        // Method shims
        #(#shim_fns)*

        #[doc = concat!(
            "Build a vtable for a concrete type implementing `",
            stringify!(#trait_name),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #vis const fn #build_vtable_fn<T: #trait_name #(+ #extra_bounds)*>() -> #vtable_name {
            #vtable_name {
                #(#vtable_inits),*
            }
        }
    }
}

/// Generate a method wrapper for the View struct.
///
/// This creates a method on the View that calls through the vtable.
/// Returns None for methods with `Self` in return types or `&Self` in parameters,
/// since these can't be meaningfully expressed on the type-erased View.
fn generate_view_method(method: &MethodInfo) -> Option<TokenStream> {
    // Skip methods where Self appears in return type or parameters.
    // In the View context, Self refers to the View struct, not the concrete type,
    // so these methods can't work through the type-erased vtable dispatch.
    if method
        .return_type
        .as_ref()
        .is_some_and(type_contains_self)
    {
        return None;
    }
    if method.param_types.iter().any(|ty| type_contains_self(ty)) {
        return None;
    }

    let method_name = &method.name;
    let param_names = &method.param_names;
    let param_types = &method.param_types;

    // Generate function parameters
    let params: Vec<_> = param_names
        .iter()
        .zip(param_types.iter())
        .map(|(name, ty)| {
            quote::quote! { #name: #ty }
        })
        .collect();

    // Generate self receiver
    let self_param = if method.is_mut {
        quote::quote! { &mut self }
    } else {
        quote::quote! { &self }
    };

    // Generate argument array for vtable call
    let argc = param_types.len() as i32;
    let arg_conversions: Vec<_> = param_names
        .iter()
        .map(|name| {
            quote::quote! {
                ::miniextendr_api::trait_abi::to_sexp(#name)
            }
        })
        .collect();

    // Generate vtable call
    let vtable_call = if argc > 0 {
        quote::quote! {
            let args: [::miniextendr_api::ffi::SEXP; #argc as usize] = [#(#arg_conversions),*];
            ((*self.vtable).#method_name)(self.data, #argc, args.as_ptr())
        }
    } else {
        quote::quote! {
            ((*self.vtable).#method_name)(self.data, 0, ::std::ptr::null())
        }
    };

    // Generate return type handling
    let return_type = &method.return_type;
    let (return_sig, result_conversion) = if let Some(ret_ty) = return_type {
        (
            quote::quote! { -> #ret_ty },
            quote::quote! {
                ::miniextendr_api::trait_abi::from_sexp::<#ret_ty>(result)
            },
        )
    } else {
        (
            quote::quote! {},
            quote::quote! {
                let _ = result;
            },
        )
    };

    Some(quote::quote! {
        #[doc = concat!("Call `", stringify!(#method_name), "` through the vtable.")]
        #[inline]
        pub fn #method_name(#self_param #(, #params)*) #return_sig {
            unsafe {
                let result = { #vtable_call };
                #result_conversion
            }
        }
    })
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
fn generate_method_shim(
    trait_name: &syn::Ident,
    method: &MethodInfo,
    extra_bounds: &[TokenStream],
) -> TokenStream {
    let method_name = &method.name;
    let shim_name = quote::format_ident!(
        "__{}_{}_shim",
        trait_name.to_string().to_lowercase(),
        method_name
    );

    let param_count = method.param_types.len();
    let expected_argc = param_count as i32;

    // Generate argument extraction
    // For &Self params, extract ExternalPtr<T> and borrow from it
    let arg_extractions: Vec<_> = method
        .param_names
        .iter()
        .zip(method.param_types.iter())
        .enumerate()
        .map(|(i, (name, ty))| {
            let name_str = name.to_string();
            let (is_self_ref, is_mut) = param_is_self_ref(ty);
            if is_self_ref {
                // &Self or &mut Self: extract ExternalPtr<T> and deref
                let extptr_name = quote::format_ident!("__extptr_{}", name);
                if is_mut {
                    quote::quote! {
                        let mut #extptr_name: ::miniextendr_api::ExternalPtr<T> = unsafe {
                            ::miniextendr_api::trait_abi::extract_arg(argc, argv, #i, #name_str)
                        };
                        let #name: &mut T = &mut *#extptr_name;
                    }
                } else {
                    quote::quote! {
                        let #extptr_name: ::miniextendr_api::ExternalPtr<T> = unsafe {
                            ::miniextendr_api::trait_abi::extract_arg(argc, argv, #i, #name_str)
                        };
                        let #name: &T = &*#extptr_name;
                    }
                }
            } else {
                quote::quote! {
                    let #name: #ty = unsafe {
                        ::miniextendr_api::trait_abi::extract_arg(argc, argv, #i, #name_str)
                    };
                }
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
        /// Both Rust panics and R longjmps are caught via `with_r_unwind_protect`.
        #[doc(hidden)]
        unsafe extern "C" fn #shim_name<T: #trait_name #(+ #extra_bounds)*>(
            data: *mut ::std::os::raw::c_void,
            argc: i32,
            argv: *const ::miniextendr_api::ffi::SEXP,
        ) -> ::miniextendr_api::ffi::SEXP {
            // Check arity (before unwind protect - uses r_stop which doesn't return)
            unsafe {
                ::miniextendr_api::trait_abi::check_arity(argc, #expected_argc, #method_name_str);
            }

            // Wrap in with_r_unwind_protect to catch both Rust panics and R longjmps.
            // This is safer than catch_unwind alone because extract_arg and user code
            // may call R API functions that error via longjmp.
            ::miniextendr_api::unwind_protect::with_r_unwind_protect(|| {
                // Extract arguments
                #(#arg_extractions)*

                // Call method
                let result = { #method_call };

                // Convert result
                #result_conversion
            }, None)
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

// =============================================================================
// Self-type detection helpers
// =============================================================================

/// Check if a type syntactically contains `Self`.
///
/// Used to detect when a method returns `Self` (or `Option<Self>`, `Vec<Self>`, etc.)
/// so the generated shim can add `IntoR` bounds.
fn type_contains_self(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => {
            for seg in &tp.path.segments {
                if seg.ident == "Self" {
                    return true;
                }
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner) = arg {
                            if type_contains_self(inner) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
        syn::Type::Reference(r) => type_contains_self(&r.elem),
        syn::Type::Tuple(t) => t.elems.iter().any(type_contains_self),
        syn::Type::Slice(s) => type_contains_self(&s.elem),
        syn::Type::Array(a) => type_contains_self(&a.elem),
        syn::Type::Paren(p) => type_contains_self(&p.elem),
        _ => false,
    }
}

/// Check if a parameter type is `&Self` or `&mut Self`.
///
/// Returns `(is_self_ref, is_mut)`. When true, the generated shim extracts
/// an `ExternalPtr<T>` from the SEXP and borrows from it instead of trying
/// to extract `&T` directly (which doesn't implement `TryFromSexp`).
fn param_is_self_ref(ty: &syn::Type) -> (bool, bool) {
    if let syn::Type::Reference(r) = ty {
        if let syn::Type::Path(tp) = r.elem.as_ref() {
            if tp.path.is_ident("Self") {
                return (true, r.mutability.is_some());
            }
        }
    }
    (false, false)
}

/// Compute extra trait bounds needed for the shim and build_vtable functions.
///
/// - Methods returning `Self` need `T: IntoR` for SEXP conversion
/// - Methods with `&Self` params need `T: TypedExternal + 'static` for ExternalPtr extraction
fn compute_extra_bounds(methods: &[MethodInfo]) -> Vec<proc_macro2::TokenStream> {
    let mut needs_into_r = false;
    let mut needs_typed_external = false;

    for method in methods {
        if method
            .return_type
            .as_ref()
            .is_some_and(type_contains_self)
        {
            needs_into_r = true;
        }
        if method.param_types.iter().any(|ty| param_is_self_ref(ty).0) {
            needs_typed_external = true;
        }
    }

    let mut bounds = Vec::new();
    if needs_into_r {
        bounds.push(quote::quote! { ::miniextendr_api::IntoR });
    }
    if needs_typed_external {
        bounds.push(quote::quote! { ::miniextendr_api::TypedExternal + 'static });
    }
    bounds
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
mod tests;
