//! Derive macros for bidirectional row ↔ dataframe conversions.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Data, DeriveInput, Fields, Type};

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
    let collection_type = parse_collection_type(&input)?;

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

    // Generate DataFrame struct with collection fields
    let df_fields: Vec<_> = field_info
        .iter()
        .map(|(name, ty)| {
            let col_ty = wrap_in_collection(ty, &collection_type);
            quote! { pub #name: #col_ty }
        })
        .collect();

    // Generate the companion DataFrame struct
    let dataframe_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #df_name {
            #(#df_fields),*
        }
    };

    // Extract field names
    let field_names: Vec<_> = field_info.iter().map(|(name, _)| name).collect();

    // Generate IntoDataFrame for DataFrame type
    let first_field = field_names.first().unwrap();
    let df_pairs = field_names.iter().map(|name| {
        let name_str = name.to_string();
        quote! { (#name_str, self.#name) }
    });

    let into_dataframe_impl = quote! {
        impl ::miniextendr_api::convert::IntoDataFrame for #df_name {
            fn into_data_frame(self) -> ::miniextendr_api::List {
                let n_rows = self.#first_field.len();
                ::miniextendr_api::List::from_pairs(vec![
                    #(#df_pairs),*
                ])
                .set_class_str(&["data.frame"])
                .set_row_names_int(n_rows)
            }
        }
    };

    // Generate From<Vec<Row>> for DataFrame
    let field_collections = field_names.iter().map(|name| {
        match collection_type {
            CollectionType::Vec => quote! {
                #name: rows.iter().map(|r| r.#name.clone()).collect()
            },
            CollectionType::BoxedSlice => quote! {
                #name: rows.iter().map(|r| r.#name.clone()).collect::<Vec<_>>().into_boxed_slice()
            },
            CollectionType::Slice => quote! {
                #name: Box::leak(rows.iter().map(|r| r.#name.clone()).collect::<Vec<_>>().into_boxed_slice())
            },
        }
    });

    let from_vec_impl = quote! {
        impl From<Vec<#row_name>> for #df_name {
            fn from(rows: Vec<#row_name>) -> Self {
                #df_name {
                    #(#field_collections),*
                }
            }
        }
    };

    // Generate IntoIterator for DataFrame
    let iterator_name = format_ident!("{}Iterator", df_name);
    let iter_field_names: Vec<_> = field_names.iter().map(|n| format_ident!("{}_iter", n)).collect();

    let iter_fields = iter_field_names.iter().zip(field_info.iter()).map(|(iter_name, (_, ty))| {
        quote! { #iter_name: <Vec<#ty> as IntoIterator>::IntoIter }
    });

    let iter_inits = field_names.iter().zip(iter_field_names.iter()).map(|(field_name, iter_name)| {
        quote! { #iter_name: self.#field_name.into_iter() }
    });

    let next_fields = field_names.iter().zip(iter_field_names.iter()).map(|(field_name, iter_name)| {
        quote! { #field_name: self.#iter_name.next()? }
    });

    let into_iterator_impl = quote! {
        pub struct #iterator_name {
            #(#iter_fields),*
        }

        impl IntoIterator for #df_name {
            type Item = #row_name;
            type IntoIter = #iterator_name;

            fn into_iter(self) -> Self::IntoIter {
                #iterator_name {
                    #(#iter_inits),*
                }
            }
        }

        impl Iterator for #iterator_name {
            type Item = #row_name;

            fn next(&mut self) -> Option<Self::Item> {
                Some(#row_name {
                    #(#next_fields),*
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
fn parse_dataframe_name(input: &DeriveInput) -> syn::Result<syn::Ident> {
    // TODO: Parse #[dataframe(name = "...")] attribute
    // For now, use default naming
    let default_name = format_ident!("{}DataFrame", input.ident);
    Ok(default_name)
}

/// Parse the collection type from attributes, defaulting to `Vec<T>`.
fn parse_collection_type(_input: &DeriveInput) -> syn::Result<CollectionType> {
    // TODO: Parse #[dataframe(collection = "...")] attribute
    // For now, use Vec
    Ok(CollectionType::Vec)
}

#[derive(Debug, Clone, Copy)]
enum CollectionType {
    Vec,
    BoxedSlice,
    Slice,
}

/// Wrap a type in the specified collection type.
fn wrap_in_collection(inner: &Type, collection: &CollectionType) -> Type {
    match collection {
        CollectionType::Vec => parse_quote! { Vec<#inner> },
        CollectionType::BoxedSlice => parse_quote! { Box<[#inner]> },
        CollectionType::Slice => parse_quote! { &'static [#inner] },
    }
}
