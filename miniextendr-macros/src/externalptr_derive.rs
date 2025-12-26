//! # `#[derive(ExternalPtr)]` - ExternalPtr and Trait ABI Generation
//!
//! This module implements the `#[derive(ExternalPtr)]` macro which:
//!
//! 1. **Basic case**: Generates `TypedExternal` impl for use with `ExternalPtr<T>`
//! 2. **With traits**: Additionally generates the type-erased wrapper infrastructure
//!    for cross-package trait dispatch.
//!
//! ## Usage
//!
//! ### Basic (no traits)
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! struct MyData {
//!     value: i32,
//! }
//! // Generates: impl TypedExternal for MyData { ... }
//! ```
//!
//! ### With trait support
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! #[externalptr(traits = [Counter])]
//! struct MyCounter {
//!     value: i32,
//! }
//! // Generates:
//! // - impl TypedExternal for MyCounter
//! // - __MX_WRAPPER_MYCOUNTER (erased wrapper struct)
//! // - __MX_BASE_VTABLE_MYCOUNTER (base vtable)
//! // - __mx_wrap_mycounter() -> *mut mx_erased
//! ```
//!
//! ## Generated Types (with traits)
//!
//! ### Wrapper Struct
//!
//! ```ignore
//! #[repr(C)]
//! struct __MxWrapperMyCounter {
//!     erased: mx_erased,  // Must be first field
//!     data: MyCounter,
//! }
//! ```
//!
//! ### Base Vtable
//!
//! ```ignore
//! static __MX_BASE_VTABLE_MYCOUNTER: mx_base_vtable = mx_base_vtable {
//!     drop: __mx_drop_mycounter,
//!     concrete_tag: TAG_MYCOUNTER,
//!     query: __mx_query_mycounter,
//! };
//! ```
//!
//! ### Query Function
//!
//! The query function maps trait tags to vtable pointers:
//!
//! ```ignore
//! unsafe extern "C" fn __mx_query_mycounter(
//!     ptr: *mut mx_erased,
//!     trait_tag: mx_tag,
//! ) -> *const c_void {
//!     if trait_tag == TAG_COUNTER {
//!         return &__VTABLE_COUNTER_FOR_MYCOUNTER as *const _ as *const c_void;
//!     }
//!     std::ptr::null()
//! }
//! ```

use proc_macro2::TokenStream;
use syn::{DeriveInput, Path};

/// Parse the `#[externalptr(...)]` attribute to extract trait list.
fn parse_externalptr_attrs(input: &DeriveInput) -> syn::Result<Vec<Path>> {
    let mut traits = Vec::new();

    for attr in &input.attrs {
        if !attr.path().is_ident("externalptr") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("traits") {
                // Parse: traits = [Trait1, Trait2, ...]
                // First consume the `=` token
                let _eq: syn::Token![=] = meta.input.parse()?;
                // Then parse the bracketed list
                let content;
                syn::bracketed!(content in meta.input);
                let paths =
                    syn::punctuated::Punctuated::<Path, syn::Token![,]>::parse_terminated(&content)?;
                traits.extend(paths);
                Ok(())
            } else {
                Err(meta.error("unknown externalptr attribute"))
            }
        })?;
    }

    Ok(traits)
}

/// Generate the basic `TypedExternal` implementation.
fn generate_typed_external(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let name_str = name.to_string();
    let name_lit = syn::LitStr::new(&name_str, name.span());
    let name_cstr = syn::LitByteStr::new(format!("{}\0", name_str).as_bytes(), name.span());

    quote::quote! {
        impl #impl_generics ::miniextendr_api::externalptr::TypedExternal for #name #ty_generics #where_clause {
            const TYPE_NAME: &'static str = #name_lit;
            const TYPE_NAME_CSTR: &'static [u8] = #name_cstr;
        }
    }
}

/// Generate the trait ABI wrapper infrastructure.
fn generate_trait_wrapper(input: &DeriveInput, traits: &[Path]) -> TokenStream {
    let name = &input.ident;
    let name_upper = name.to_string().to_uppercase();
    let name_lower = name.to_string().to_lowercase();

    // Generate identifiers
    let wrapper_name = quote::format_ident!("__MxWrapper{}", name);
    let base_vtable_name = quote::format_ident!("__MX_BASE_VTABLE_{}", name_upper);
    let concrete_tag_name = quote::format_ident!("__MX_TAG_{}", name_upper);
    let drop_fn_name = quote::format_ident!("__mx_drop_{}", name_lower);
    let query_fn_name = quote::format_ident!("__mx_query_{}", name_lower);
    let wrap_fn_name = quote::format_ident!("__mx_wrap_{}", name_lower);

    // Generate tag path string for hashing
    let tag_path = format!("{{}}::{}", name);

    // Generate query branches for each trait
    let query_branches: Vec<_> = traits
        .iter()
        .map(|trait_path| {
            let trait_name = trait_path
                .segments
                .last()
                .map(|s| &s.ident)
                .expect("trait path has at least one segment");
            let trait_name_upper = trait_name.to_string().to_uppercase();
            let _trait_name_lower = trait_name.to_string().to_lowercase();

            // Build TAG path - same module as trait
            let mut tag_path = trait_path.clone();
            if let Some(last) = tag_path.segments.last_mut() {
                last.ident = quote::format_ident!("TAG_{}", trait_name_upper);
            }

            // Build vtable static name: __VTABLE_{TRAIT}_FOR_{TYPE}
            let vtable_name =
                quote::format_ident!("__VTABLE_{}_FOR_{}", trait_name_upper, name_upper);

            // Build vtable type path - same module as trait
            let mut vtable_type_path = trait_path.clone();
            if let Some(last) = vtable_type_path.segments.last_mut() {
                last.ident = quote::format_ident!("{}VTable", trait_name);
            }

            // If trait path has module prefix, vtable is local (no prefix)
            // If trait path is bare (e.g., `Counter`), vtable is also local
            let vtable_ref = if trait_path.segments.len() > 1 {
                // External trait - vtable is local
                quote::quote! { &#vtable_name }
            } else {
                // Local trait - vtable is local
                quote::quote! { &#vtable_name }
            };

            quote::quote! {
                if trait_tag == #tag_path {
                    return #vtable_ref as *const _ as *const ::std::os::raw::c_void;
                }
            }
        })
        .collect();

    quote::quote! {
        /// Type-erased wrapper for `#name` with trait dispatch support.
        ///
        /// Generated by `#[derive(ExternalPtr)]`.
        #[repr(C)]
        #[doc(hidden)]
        pub struct #wrapper_name {
            /// Type-erased header. Must be first field for `mx_erased` compatibility.
            pub erased: ::miniextendr_api::abi::mx_erased,
            /// The actual data.
            pub data: #name,
        }

        /// Concrete type tag for `#name`.
        ///
        /// Used for type-safe downcasts via `mx_base_vtable::concrete_tag`.
        #[doc(hidden)]
        pub const #concrete_tag_name: ::miniextendr_api::abi::mx_tag =
            ::miniextendr_api::abi::mx_tag_from_path(concat!(module_path!(), #tag_path));

        /// Drop function for `#name` wrapper.
        ///
        /// Called by R's GC finalizer via `mx_base_vtable::drop`.
        #[doc(hidden)]
        unsafe extern "C" fn #drop_fn_name(ptr: *mut ::miniextendr_api::abi::mx_erased) {
            if ptr.is_null() {
                return;
            }
            // Cast to wrapper and drop
            let wrapper = ptr as *mut #wrapper_name;
            unsafe { drop(Box::from_raw(wrapper)); }
        }

        /// Query function for `#name` trait dispatch.
        ///
        /// Maps trait tags to vtable pointers.
        #[doc(hidden)]
        unsafe extern "C" fn #query_fn_name(
            _ptr: *mut ::miniextendr_api::abi::mx_erased,
            trait_tag: ::miniextendr_api::abi::mx_tag,
        ) -> *const ::std::os::raw::c_void {
            #(#query_branches)*
            ::std::ptr::null()
        }

        /// Base vtable for `#name`.
        ///
        /// Provides drop, concrete_tag, and query for the type-erased wrapper.
        #[doc(hidden)]
        pub static #base_vtable_name: ::miniextendr_api::abi::mx_base_vtable =
            ::miniextendr_api::abi::mx_base_vtable {
                drop: #drop_fn_name,
                concrete_tag: #concrete_tag_name,
                query: #query_fn_name,
            };

        /// Create a type-erased wrapper for `#name`.
        ///
        /// Returns a pointer suitable for passing to `mx_wrap()`.
        #[doc(hidden)]
        pub fn #wrap_fn_name(data: #name) -> *mut ::miniextendr_api::abi::mx_erased {
            let wrapper = Box::new(#wrapper_name {
                erased: ::miniextendr_api::abi::mx_erased {
                    base: &#base_vtable_name,
                },
                data,
            });
            Box::into_raw(wrapper) as *mut ::miniextendr_api::abi::mx_erased
        }
    }
}

/// Main entry point for `#[derive(ExternalPtr)]`.
pub fn derive_external_ptr(input: DeriveInput) -> syn::Result<TokenStream> {
    let traits = parse_externalptr_attrs(&input)?;

    let typed_external = generate_typed_external(&input);

    let trait_wrapper = if traits.is_empty() {
        TokenStream::new()
    } else {
        generate_trait_wrapper(&input, &traits)
    };

    Ok(quote::quote! {
        #typed_external
        #trait_wrapper
    })
}
