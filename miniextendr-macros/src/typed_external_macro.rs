//! `impl_typed_external!` — explicit monomorphization of `TypedExternal` for generic types.
//!
//! Since `#[derive(ExternalPtr)]` rejects generic types (the generated .Call entrypoints
//! and R wrapper code cannot be generic), this macro provides an alternative for users
//! who want to use `ExternalPtr<T>` with a specific monomorphization of a generic type.
//!
//! # Usage
//!
//! ```ignore
//! impl_typed_external!(MyWrapper<i32>);
//! impl_typed_external!(MyWrapper<String>);
//! impl_typed_external!(TreeNode<String, Vec<u8>>);
//! ```
//!
//! # Generated Code
//!
//! For `impl_typed_external!(MyWrapper<i32>)`:
//!
//! ```ignore
//! impl TypedExternal for MyWrapper<i32> {
//!     const TYPE_NAME: &'static str = "MyWrapper<i32>";
//!     const TYPE_NAME_CSTR: &'static [u8] = b"MyWrapper<i32>\0";
//!     const TYPE_ID_CSTR: &'static [u8] =
//!         concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"),
//!                 "::", module_path!(), "::MyWrapper<i32>\0").as_bytes();
//! }
//!
//! impl IntoExternalPtr for MyWrapper<i32> {}
//! ```

use proc_macro2::TokenStream;

/// Implementation of `impl_typed_external!`.
///
/// Accepts a concrete type path with generic arguments filled in
/// (e.g., `MyWrapper<i32>`, `TreeNode<String, Vec<u8>>`).
///
/// Returns a `TokenStream` containing:
/// - `impl TypedExternal for <type> { ... }`
/// - `impl IntoExternalPtr for <type> {}`
pub fn impl_typed_external(input: TokenStream) -> syn::Result<TokenStream> {
    // Parse the input as a Type (supports paths with generic args)
    let ty: syn::Type = syn::parse2(input)?;

    // Extract a display name for the type. We use the token representation
    // which naturally includes angle brackets and generic args.
    let type_display = quote::quote!(#ty).to_string();
    // Clean up spacing: `MyWrapper < i32 >` → `MyWrapper<i32>`
    let type_display = type_display
        .replace(" < ", "<")
        .replace(" > ", ">")
        .replace("< ", "<")
        .replace(" >", ">");

    let name_lit = syn::LitStr::new(&type_display, proc_macro2::Span::call_site());
    let name_cstr_bytes = format!("{}\0", type_display);
    let name_cstr =
        syn::LitByteStr::new(name_cstr_bytes.as_bytes(), proc_macro2::Span::call_site());

    Ok(quote::quote! {
        impl ::miniextendr_api::externalptr::TypedExternal for #ty {
            const TYPE_NAME: &'static str = #name_lit;
            const TYPE_NAME_CSTR: &'static [u8] = #name_cstr;
            const TYPE_ID_CSTR: &'static [u8] =
                concat!(
                    env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"),
                    "::", module_path!(), "::", #name_lit, "\0"
                ).as_bytes();
        }

        impl ::miniextendr_api::externalptr::IntoExternalPtr for #ty {}
    })
}
