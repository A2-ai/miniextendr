//! Derive macros for bidirectional row ↔ dataframe conversions.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

/// Derive `DataFrameRow`: generates a companion DataFrame type with collection fields.
///
/// # Requirements
///
/// The type must implement `IntoList` (either manually or via `Serialize` with serde feature).
/// `IntoList` ensures all fields are convertible to R values.
///
/// # Generated Items
///
/// For a struct `Measurement { time: f64, value: f64 }`:
/// - Struct `MeasurementDataFrame { time: Vec<f64>, value: Vec<f64> }`
/// - `impl IntoList for Measurement` (if not already implemented)
/// - `impl IntoDataFrame for MeasurementDataFrame`
/// - `impl From<Vec<Measurement>> for MeasurementDataFrame`
/// - `impl IntoIterator for MeasurementDataFrame`
/// - Associated methods on `Measurement`:
///   - `to_dataframe(Vec<Self>) -> MeasurementDataFrame`
///   - `from_dataframe(MeasurementDataFrame) -> Vec<Self>`
///
/// # Attributes
///
/// - `#[dataframe(name = "CustomName")]` - Custom name for generated DataFrame type
/// - `#[dataframe(collection = "Box<[T]>")]` - Collection type (default: `Vec<T>`)
pub fn derive_dataframe_row(input: DeriveInput) -> syn::Result<TokenStream> {
    let row_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse attributes
    let df_name = parse_dataframe_name(&input)?;

    // Extract fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "DataFrameRow only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "DataFrameRow only supports structs",
            ))
        }
    };

    // Collect field info
    let field_info: Vec<_> = fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            (name, ty)
        })
        .collect();

    // Build DataFrame struct with one field at a time
    let df_fields_tokens: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, ty)| {
            quote! { pub #name: Vec<#ty> }
        })
        .collect();

    // Generate the companion DataFrame struct
    let dataframe_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #df_name {
            #(#df_fields_tokens),*
        }
    };

    // Extract field names
    let field_names: Vec<_> = field_info.iter().map(|(name, _)| name).collect();

    // Generate IntoDataFrame for DataFrame type
    let first_field = field_names.first().unwrap();
    let df_pairs = field_names.iter().map(|name| {
        let name_str = name.to_string();
        quote! { (#name_str, ::miniextendr_api::IntoR::into_sexp(self.#name)) }
    });

    let into_dataframe_impl = quote! {
        impl ::miniextendr_api::convert::IntoDataFrame for #df_name {
            fn into_data_frame(self) -> ::miniextendr_api::List {
                let n_rows = self.#first_field.len();
                ::miniextendr_api::list::List::from_raw_pairs(vec![
                    #(#df_pairs),*
                ])
                .set_class_str(&["data.frame"])
                .set_row_names_int(n_rows)
            }
        }
    };

    // Build From impl struct initialization explicitly
    let mut from_struct_tokens = TokenStream::new();
    for (i, (name, ty)) in field_info.iter().enumerate() {
        if i > 0 {
            from_struct_tokens.extend(quote! { , });
        }
        from_struct_tokens.extend(quote! {
            #name: rows.iter().map(|r| r.#name.clone()).collect::<Vec<#ty>>()
        });
    }

    let from_vec_impl = quote! {
        impl From<Vec<#row_name>> for #df_name {
            fn from(rows: Vec<#row_name>) -> Self {
                #df_name {
                    #from_struct_tokens
                }
            }
        }
    };

    // Generate IntoIterator for DataFrame - use concrete types for simplicity
    let iterator_name = format_ident!("{}Iterator", df_name);

    // Generate iterator fields paired with their types to ensure correct correspondence
    let iter_field_decls: Vec<_> = field_info
        .iter()
        .map(|(name, ty)| {
            quote! { #name: std::vec::IntoIter<#ty> }
        })
        .collect();

    // Generate destructuring pattern
    let destruct_pattern = field_info
        .iter()
        .map(|(name, _ty)| quote! { #name })
        .collect::<Vec<_>>();

    // Build struct initialization tokens explicitly with explicit types
    let mut iter_init_tokens = TokenStream::new();
    for (i, (name, ty)) in field_info.iter().enumerate() {
        if i > 0 {
            iter_init_tokens.extend(quote! { , });
        }
        iter_init_tokens.extend(quote! { #name: <Vec<#ty>>::into_iter(#name) });
    }

    // Build Iterator::next() return struct tokens explicitly
    let mut next_struct_tokens = TokenStream::new();
    for (i, (name, _ty)) in field_info.iter().enumerate() {
        if i > 0 {
            next_struct_tokens.extend(quote! { , });
        }
        next_struct_tokens.extend(quote! { #name: self.#name.next()? });
    }

    let into_iterator_impl = quote! {
        pub struct #iterator_name {
            #(#iter_field_decls),*
        }

        impl IntoIterator for #df_name {
            type Item = #row_name;
            type IntoIter = #iterator_name;

            fn into_iter(self) -> Self::IntoIter {
                let #df_name { #(#destruct_pattern),* } = self;
                #iterator_name {
                    #iter_init_tokens
                }
            }
        }

        impl Iterator for #iterator_name {
            type Item = #row_name;

            fn next(&mut self) -> Option<Self::Item> {
                Some(#row_name {
                    #next_struct_tokens
                })
            }
        }
    };

    // Generate associated methods on row type for working with DataFrame
    let row_methods = quote! {
        impl #impl_generics #row_name #ty_generics #where_clause {
            /// Name of the generated DataFrame companion type.
            pub const DATAFRAME_TYPE_NAME: &'static str = stringify!(#df_name);

            /// Convert a vector of rows into the companion DataFrame type.
            ///
            /// This transposes row-oriented data into column-oriented format.
            pub fn to_dataframe(rows: Vec<Self>) -> #df_name {
                rows.into()
            }

            /// Convert a DataFrame back into a vector of rows.
            ///
            /// This transposes column-oriented data back into row-oriented format.
            pub fn from_dataframe(df: #df_name) -> Vec<Self> {
                df.into_iter().collect()
            }
        }
    };

    // Compile-time assertion: row type must implement IntoList
    // (either via manual impl, #[derive(IntoList)], or Serialize with serde feature)
    let trait_check = quote! {
        const _: () = {
            fn _assert_into_list #impl_generics () #where_clause {
                fn _check<T: ::miniextendr_api::list::IntoList>() {}
                _check::<#row_name #ty_generics>();
            }
        };
    };

    Ok(quote! {
        #dataframe_struct
        #into_dataframe_impl
        #from_vec_impl
        #into_iterator_impl
        #row_methods
        #trait_check
    })
}

/// Parse the dataframe name from attributes, defaulting to `{StructName}DataFrame`.
///
/// Supports `#[dataframe(name = "CustomName")]` to override the generated type name.
fn parse_dataframe_name(input: &DeriveInput) -> syn::Result<syn::Ident> {
    for attr in &input.attrs {
        if !attr.path().is_ident("dataframe") {
            continue;
        }

        let nested = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        )?;

        for meta in &nested {
            if let syn::Meta::NameValue(nv) = meta
                && nv.path.is_ident("name")
            {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &nv.value
                {
                    return Ok(format_ident!("{}", lit_str.value(), span = lit_str.span()));
                } else {
                    return Err(syn::Error::new_spanned(
                        &nv.value,
                        "expected string literal for `name`",
                    ));
                }
            }
        }
    }

    let default_name = format_ident!("{}DataFrame", input.ident);
    Ok(default_name)
}

