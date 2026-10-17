//! Fieldwise R comparisons for pointer-backed structs (#2149).
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{DeriveInput, spanned::Spanned};

const SETTINGS: [&str; 4] = ["tolerance", "scale", "check_attributes", "check_names"];

#[derive(Default)]
struct Options {
    settings: [Option<String>; 4],
    skip: bool,
    label: Option<String>,
    with: Option<syn::Path>,
}

fn r_expression(expr: &syn::Expr) -> syn::Result<String> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(value) => Ok(value.value()),
            syn::Lit::Float(value) => Ok(value.base10_digits().to_string()),
            syn::Lit::Int(value) => Ok(value.base10_digits().to_string()),
            syn::Lit::Bool(value) => Ok(if value.value { "TRUE" } else { "FALSE" }.to_string()),
            _ => Err(syn::Error::new_spanned(
                expr,
                "expected a number, boolean or R expression string",
            )),
        },
        syn::Expr::Unary(value) if matches!(value.op, syn::UnOp::Neg(_)) => {
            Ok(format!("-{}", r_expression(&value.expr)?))
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            "expected a number, boolean or R expression string",
        )),
    }
}

fn options(attrs: &[syn::Attribute], field: bool) -> syn::Result<Options> {
    let mut options = Options::default();
    let mut seen = std::collections::HashSet::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("all_equal")) {
        attr.parse_nested_meta(|meta| {
            let key = meta.path.get_ident().map(ToString::to_string).unwrap_or_default();
            if !seen.insert(key.clone()) {
                return Err(meta.error(format!("`{key}` is given twice")));
            }
            if let Some(index) = SETTINGS.iter().position(|s| *s == key) {
                options.settings[index] = Some(r_expression(&meta.value()?.parse()?)?);
            } else {
                match key.as_str() {
                    "skip" if field => options.skip = true,
                    "label" if field => options.label = Some(meta.value()?.parse::<syn::LitStr>()?.value()),
                    "with" if field => {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        options.with = Some(value.parse().map_err(|_| {
                            syn::Error::new_spanned(&value, "`with` must be a function path, e.g. \"Self::value\"")
                        })?);
                    }
                    _ => return Err(meta.error("unknown all_equal option; expected tolerance, scale, check_attributes, check_names, or field-only skip, label, with")),
                }
            }
            Ok(())
        })?;
    }
    if options.skip && seen.len() != 1 {
        return Err(syn::Error::new_spanned(
            attrs
                .iter()
                .find(|a| a.path().is_ident("all_equal"))
                .unwrap(),
            "`skip` cannot be combined with other all_equal options",
        ));
    }
    Ok(options)
}

fn option_tokens(value: &Option<String>) -> TokenStream {
    match value {
        Some(value) => quote!(::core::option::Option::Some(#value)),
        None => quote!(::core::option::Option::None),
    }
}

fn resolve_getter(mut path: syn::Path, name: &syn::Ident) -> syn::Path {
    if path.segments[0].ident == "Self" {
        path.segments[0].ident = name.clone();
    }
    path
}

pub(crate) fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let syn::Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input,
            "AllEqual requires a struct",
        ));
    };
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "AllEqual requires a concrete pointer-backed struct",
        ));
    }
    let defaults = options(&input.attrs, false)?.settings;
    let defaults = defaults.iter().map(option_tokens);
    let sidecars = crate::externalptr_derive::comparison_fields(&input)?;
    let name = &input.ident;
    let mut fields = Vec::new();
    let mut reads = Vec::new();
    for (position, field) in data.fields.iter().enumerate() {
        let opts = options(&field.attrs, true)?;
        let member = field
            .ident
            .clone()
            .map(syn::Member::Named)
            .unwrap_or_else(|| syn::Member::Unnamed(syn::Index::from(position)));
        let field_name = field
            .ident
            .as_ref()
            .map(crate::naming::ident_name)
            .unwrap_or_else(|| (position + 1).to_string());
        let sidecar = sidecars.get(&field_name);
        if opts.skip
            || matches!(
                sidecar,
                Some(crate::externalptr_derive::ComparisonField::Selector)
            )
        {
            continue;
        }
        let ty = &field.ty;
        let getter = opts
            .with
            .map(|path| resolve_getter(path, name))
            .or_else(|| match sidecar {
                Some(crate::externalptr_derive::ComparisonField::Computed(path)) => {
                    Some(path.clone())
                }
                _ => None,
            });
        let (value, type_path) = if let Some(getter) = getter {
            (
                quote!(::miniextendr_api::IntoR::into_sexp(#getter(&__mx_data))),
                quote!(|| ::miniextendr_api::registry::all_equal_getter_type::<#name, _>(#getter)),
            )
        } else if let Some(crate::externalptr_derive::ComparisonField::Slot(index)) = sidecar {
            (
                quote!(unsafe { ::miniextendr_api::externalptr::sidecar_r_get::<#name>(x, #index, true) }),
                quote!(::core::any::type_name::<::miniextendr_api::SEXP>),
            )
        } else {
            let clone = quote_spanned!(ty.span()=> <#ty as ::core::clone::Clone>::clone);
            (
                quote_spanned!(ty.span()=>
                    <#ty as ::miniextendr_api::IntoR>::into_sexp(#clone(&__mx_data.#member))
                ),
                quote!(::core::any::type_name::<#ty>),
            )
        };
        let index = isize::try_from(reads.len()).expect("field count fits R length");
        reads.push(quote! {
            let __mx_field = unsafe { __mx_roots.protect_raw(#value) };
            unsafe { ::miniextendr_api::list::ListBuilder::set(&__mx_fields, #index, __mx_field); }
        });
        let label = option_tokens(&opts.label);
        let settings = opts.settings.iter().map(option_tokens);
        fields.push(quote! {
            ::miniextendr_api::registry::AllEqualField {
                name: #field_name,
                label: #label,
                settings: [#(#settings),*],
                type_path: #type_path,
            }
        });
    }
    let len = fields.len();
    let c_name = crate::naming::all_equal_c_ident(name);
    let c_text = c_name.to_string();
    let def = quote::format_ident!("__MX_CALL_DEF_{}", c_name);
    let registration = crate::call_method_def::call_method_def(&def, &c_name, 1);
    let line = u32::try_from(name.span().start().line).expect("source line fits u32");
    Ok(quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        // The field contract is Clone + IntoR, including aliases of Copy types.
        #[allow(clippy::clone_on_copy)]
        pub unsafe extern "C-unwind" fn #c_name(x: ::miniextendr_api::SEXP) -> ::miniextendr_api::SEXP {
            let __mx_mark = ::miniextendr_api::deferred_condition::mark();
            let __mx_value = ::miniextendr_api::unwind_protect::with_r_unwind_protect(|| {
                let __mx_data = unsafe { ::miniextendr_api::externalptr::sidecar_r_struct::<#name>(x) };
                let __mx_roots = unsafe { ::miniextendr_api::ProtectScope::new() };
                // Allocate before any getter or conversion returns an unrooted SEXP.
                let __mx_fields = unsafe { ::miniextendr_api::list::ListBuilder::new(&__mx_roots, #len) };
                #(#reads)*
                ::miniextendr_api::list::ListBuilder::into_sexp(__mx_fields)
            }, ::core::option::Option::None);
            unsafe { ::miniextendr_api::deferred_condition::finish(
                __mx_mark, __mx_value, ::core::option::Option::None
            ) }
        }
        #registration
        const _: () = {
            #[cfg(not(target_arch = "wasm32"))]
            #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_ALL_EQUAL)]
            #[linkme(crate = ::miniextendr_api::linkme)]
            static COMPARISON: ::miniextendr_api::registry::AllEqualEntry =
                ::miniextendr_api::registry::AllEqualEntry {
                    type_path: ::core::any::type_name::<#name>,
                    rust_type: ::core::stringify!(#name),
                    c_name: #c_text,
                    defaults: [#(#defaults),*],
                    fields: &[#(#fields),*],
                    source_file: ::core::file!(),
                    source_line: #line,
                };
        };
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reject_invalid_options() {
        for (source, message) in [
            ("#[all_equal(nope)] struct T;", "unknown all_equal"),
            ("#[all_equal(skip)] struct T;", "unknown all_equal"),
            (
                "struct T { #[all_equal(skip, label = \"x\")] x: i32 }",
                "cannot be combined",
            ),
            (
                "struct T { #[all_equal(with = \"Self::f()\")] x: i32 }",
                "function path",
            ),
            (
                "#[all_equal(scale = 1, scale = 2)] struct T;",
                "given twice",
            ),
            ("enum T { A }", "requires a struct"),
        ] {
            let error = derive(syn::parse_str(source).unwrap())
                .unwrap_err()
                .to_string();
            assert!(error.contains(message), "{error}");
        }
    }

    #[test]
    fn fields_use_clones_getters_and_sidecar_slots() {
        let output = derive(syn::parse_quote! {
            #[all_equal(tolerance = 1e-6, check_attributes = false)]
            struct T {
                #[r_data] _r: RSidecar,
                value: f64,
                #[all_equal(skip)] cache: NoClone,
                #[all_equal(with = "Self::read")] raw: NoClone,
                #[r_data] stored: Sidecar<List>,
            }
        })
        .unwrap()
        .to_string();
        assert!(
            output.contains("Clone > :: clone (& __mx_data . value)"),
            "{output}"
        );
        assert!(!output.contains("__mx_data . cache"));
        assert!(output.contains("T :: read (& __mx_data)"), "{output}");
        assert!(
            output.contains("sidecar_r_get :: < T > (x , 0usize , true)"),
            "{output}"
        );
        assert!(output.contains("\"1e-6\""), "{output}");
        assert!(output.contains("\"FALSE\""), "{output}");
    }
}
