//! Shared `.Call` registration for generated SEXP wrappers.

use proc_macro2::TokenStream;
use quote::quote;

/// Emit a registration with exactly the wrapper's C arity. The caller owns
/// cfg/doc attributes and decides whether its signature includes a call slot.
/// The registered name is also the linker symbol used by the wasm writer.
pub(crate) fn call_method_def(
    def_ident: &syn::Ident,
    c_ident: &syn::Ident,
    num_args: usize,
) -> TokenStream {
    let c_name = syn::LitCStr::new(
        std::ffi::CString::new(crate::naming::ident_name(c_ident))
            .expect("valid C symbol")
            .as_c_str(),
        c_ident.span(),
    );
    let arity = syn::LitInt::new(&num_args.to_string(), c_ident.span());
    let params = std::iter::repeat_n(quote!(::miniextendr_api::SEXP), num_args);
    quote! {
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals, non_snake_case)]
        static #def_ident: ::miniextendr_api::sys::R_CallMethodDef =
            ::miniextendr_api::sys::R_CallMethodDef {
                name: #c_name.as_ptr(),
                fun: ::core::option::Option::Some(unsafe {
                    ::core::mem::transmute::<
                        unsafe extern "C-unwind" fn(#(#params),*) -> ::miniextendr_api::SEXP,
                        unsafe extern "C-unwind" fn() -> *mut ::core::ffi::c_void,
                    >(#c_ident)
                }),
                numArgs: #arity,
            };
    }
}

#[cfg(test)]
mod tests {
    use super::call_method_def;

    #[test]
    fn registers_the_linker_symbol_with_the_given_arity() {
        let def: syn::Ident = syn::parse_quote!(__MX_CALL_DEF_RDATA_SET_Engine_rate);
        let c_ident: syn::Ident = syn::parse_quote!(C_Engine__set_rate);
        let code = call_method_def(&def, &c_ident, 3).to_string();
        assert!(
            code.contains(r#"name : c"C_Engine__set_rate" . as_ptr ()"#),
            "{code}"
        );
        assert!(code.contains("numArgs : 3"), "{code}");
        // The transmute names the arity in its source type, so a wrapper with
        // another parameter count fails to compile instead of registering.
        assert!(
            code.contains(
                "fn (:: miniextendr_api :: SEXP , :: miniextendr_api :: SEXP , :: miniextendr_api :: SEXP) -> :: miniextendr_api :: SEXP"
            ),
            "{code}"
        );
        assert!(code.contains("MX_CALL_DEFS"), "{code}");
    }
}
