//! `#[derive(RConditionError)]`: the R-facing shape of an error or warning
//! payload type (#1448).
//!
//! Generates `impl miniextendr_api::condition::RConditionError` from the
//! type's structure, so a package keeps one Rust type per condition family
//! and gets the R side without hand-written glue:
//!
//! - `class()`: `c(<member>, <family>)` for enum variants, `c(<family>)` for
//!   structs. The family is `#[condition(class = "…")]` on the type, else the
//!   type name in snake_case; the member is the same attribute on the
//!   variant, else `<family>_<variant in snake_case>`.
//! - `message()`: `#[condition(message = "…")]` on the struct or on a variant
//!   is a `format!` string with the fields in scope (tuple fields as `_0`,
//!   `_1`, …). Without it the message is the type's `Display` rendering.
//! - `data()`: every field as a named entry built with
//!   `RValue::from(field.clone())`. `#[condition(rename = "…")]` changes the
//!   name, `#[condition(skip)]` drops the field, `#[condition(debug)]`
//!   attaches the `Debug` rendering (`RValue::debug`) instead of a
//!   conversion. Tuple fields need `rename` or `skip`; the condition's own
//!   slots (`message`, `call`, `kind`) are rejected; a unit shape gives
//!   `None`.
//!
//! The payload feeds `defer_warning` & co. (a condition accompanying a value)
//! and `Result<T, E>` returns (a classed error): both go through the trait.
//! Generic types are rejected, like the other derives in this crate.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields};

use crate::naming::{ident_name, to_snake_case};

/// The condition object's own slots. Mirrors
/// `miniextendr_api::condition::RESERVED_CONDITION_FIELDS`; keep in sync.
const RESERVED_FIELDS: [&str; 3] = ["message", "call", "kind"];

// region: Attribute parsing

/// `#[condition(...)]` options on the type or on one variant.
#[derive(Default)]
struct ContainerAttrs {
    class: Option<syn::LitStr>,
    message: Option<syn::LitStr>,
}

fn parse_container_attrs(attrs: &[syn::Attribute], on: &str) -> syn::Result<ContainerAttrs> {
    let mut out = ContainerAttrs::default();
    for attr in attrs {
        if !attr.path().is_ident("condition") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("class") {
                if out.class.is_some() {
                    return Err(meta.error("`class` is given twice"));
                }
                out.class = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("message") {
                if out.message.is_some() {
                    return Err(meta.error("`message` is given twice"));
                }
                out.message = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error(format!(
                    "unknown `condition` option on {on}; expected `class = \"…\"` or `message = \"…\"`"
                )));
            }
            Ok(())
        })?;
    }
    Ok(out)
}

/// `#[condition(...)]` options on one field.
#[derive(Default)]
struct FieldAttrs {
    skip: bool,
    rename: Option<syn::LitStr>,
    debug: bool,
}

fn parse_field_attrs(attrs: &[syn::Attribute]) -> syn::Result<FieldAttrs> {
    let mut out = FieldAttrs::default();
    for attr in attrs {
        if !attr.path().is_ident("condition") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                out.skip = true;
            } else if meta.path.is_ident("debug") {
                out.debug = true;
            } else if meta.path.is_ident("rename") {
                if out.rename.is_some() {
                    return Err(meta.error("`rename` is given twice"));
                }
                out.rename = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error(
                    "unknown `condition` option on a field; expected `skip`, `debug` or `rename = \"…\"`",
                ));
            }
            Ok(())
        })?;
    }
    if out.skip && (out.rename.is_some() || out.debug) {
        return Err(syn::Error::new(
            attrs
                .iter()
                .find(|a| a.path().is_ident("condition"))
                .map_or_else(Span::call_site, Spanned::span),
            "`skip` cannot be combined with `rename` or `debug`",
        ));
    }
    Ok(out)
}

// endregion

// region: Shape

/// One field of a struct or variant, as the derive sees it.
struct FieldInfo {
    /// The binding used in patterns: the field name, or `_0` / `_1` / … for
    /// tuple fields.
    binding: syn::Ident,
    /// The `data` entry name; `None` when the field is skipped.
    data_name: Option<String>,
    /// Attach `RValue::debug(field)` instead of `RValue::from(field.clone())`.
    debug: bool,
}

enum Style {
    Named,
    Tuple,
    Unit,
}

/// The fields of a struct or of one enum variant, validated.
struct Shape {
    style: Style,
    fields: Vec<FieldInfo>,
}

impl Shape {
    fn from_fields(fields: &Fields, context: &str) -> syn::Result<Shape> {
        let (style, iter): (Style, Vec<&syn::Field>) = match fields {
            Fields::Named(named) => (Style::Named, named.named.iter().collect()),
            Fields::Unnamed(unnamed) => (Style::Tuple, unnamed.unnamed.iter().collect()),
            Fields::Unit => (Style::Unit, Vec::new()),
        };
        let mut infos = Vec::with_capacity(iter.len());
        let mut seen: Vec<String> = Vec::new();
        for (index, field) in iter.iter().enumerate() {
            let attrs = parse_field_attrs(&field.attrs)?;
            let binding = match &field.ident {
                Some(ident) => ident.clone(),
                None => format_ident!("_{index}", span = field.span()),
            };
            let data_name = if attrs.skip {
                None
            } else {
                let name = match (&attrs.rename, &field.ident) {
                    (Some(lit), _) => lit.value(),
                    (None, Some(ident)) => ident_name(ident),
                    (None, None) => {
                        return Err(syn::Error::new(
                            field.span(),
                            format!(
                                "tuple field {index} of {context} needs a `data` name: add \
                                 `#[condition(rename = \"name\")]`, or `#[condition(skip)]` to leave it out"
                            ),
                        ));
                    }
                };
                if RESERVED_FIELDS.contains(&name.as_str()) {
                    return Err(syn::Error::new(
                        field.span(),
                        format!(
                            "`{name}` is one of the condition's own slots (`message`, `call`, `kind`); \
                             rename the field with `#[condition(rename = \"…\")]` or exclude it with `#[condition(skip)]`"
                        ),
                    ));
                }
                if seen.contains(&name) {
                    return Err(syn::Error::new(
                        field.span(),
                        format!("duplicate `data` field name `{name}` in {context}"),
                    ));
                }
                seen.push(name.clone());
                Some(name)
            };
            infos.push(FieldInfo {
                binding,
                data_name,
                debug: attrs.debug,
            });
        }
        Ok(Shape {
            style,
            fields: infos,
        })
    }

    /// A pattern binding every field (for `message` format strings).
    fn pattern_all(&self, path: &TokenStream) -> TokenStream {
        let bindings = self.fields.iter().map(|f| &f.binding);
        match self.style {
            Style::Named => quote! { #path { #(#bindings),* } },
            Style::Tuple => quote! { #path ( #(#bindings),* ) },
            Style::Unit => quote! { #path },
        }
    }

    /// A pattern binding only the fields that produce `data` entries.
    fn pattern_data(&self, path: &TokenStream) -> TokenStream {
        match self.style {
            Style::Named => {
                let bindings = self
                    .fields
                    .iter()
                    .filter(|f| f.data_name.is_some())
                    .map(|f| &f.binding);
                quote! { #path { #(#bindings,)* .. } }
            }
            Style::Tuple => {
                let slots = self.fields.iter().map(|f| {
                    if f.data_name.is_some() {
                        let b = &f.binding;
                        quote! { #b }
                    } else {
                        quote! { _ }
                    }
                });
                quote! { #path ( #(#slots),* ) }
            }
            Style::Unit => quote! { #path },
        }
    }

    /// A pattern that matches without binding anything.
    fn pattern_wild(&self, path: &TokenStream) -> TokenStream {
        match self.style {
            Style::Unit => quote! { #path },
            Style::Named | Style::Tuple => quote! { #path { .. } },
        }
    }

    /// The `Option<ConditionData>` expression over the bindings of
    /// [`Self::pattern_data`].
    fn data_expr(&self) -> TokenStream {
        let entries: Vec<TokenStream> = self
            .fields
            .iter()
            .filter_map(|f| {
                let name = f.data_name.as_ref()?;
                let binding = &f.binding;
                let value = if f.debug {
                    quote! { ::miniextendr_api::RValue::debug(#binding) }
                } else {
                    quote! {
                        ::miniextendr_api::RValue::from(::core::clone::Clone::clone(#binding))
                    }
                };
                Some(quote! { (::std::string::String::from(#name), #value) })
            })
            .collect();
        if entries.is_empty() {
            quote! { ::core::option::Option::None }
        } else {
            quote! { ::core::option::Option::Some(::std::vec![#(#entries),*]) }
        }
    }
}

// endregion

// region: Entry point

/// Main entry point for `#[derive(RConditionError)]`.
pub fn derive_r_condition_error(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "`#[derive(RConditionError)]` does not support generic types",
        ));
    }
    let container = parse_container_attrs(&input.attrs, "the type")?;
    let family = container
        .class
        .as_ref()
        .map_or_else(|| to_snake_case(&ident_name(name)), syn::LitStr::value);

    let display = quote! { ::std::string::ToString::to_string(self) };

    let (message_body, class_body, data_body) = match &input.data {
        Data::Struct(data) => {
            let shape = Shape::from_fields(&data.fields, &format!("`{name}`"))?;
            let self_path = quote! { Self };
            let message = match &container.message {
                Some(fmt) => {
                    let pattern = shape.pattern_all(&self_path);
                    quote! {
                        #[allow(unused_variables)]
                        let #pattern = self;
                        ::std::format!(#fmt)
                    }
                }
                None => display.clone(),
            };
            let class = quote! { ::std::vec![::std::string::String::from(#family)] };
            let pattern = shape.pattern_data(&self_path);
            let data_expr = shape.data_expr();
            let data = quote! {
                let #pattern = self;
                #data_expr
            };
            (message, class, data)
        }
        Data::Enum(data) => {
            if let Some(message) = &container.message {
                return Err(syn::Error::new(
                    message.span(),
                    "`message` goes on the variants of an enum, not on the enum itself",
                ));
            }
            if data.variants.is_empty() {
                return Err(syn::Error::new(
                    name.span(),
                    "`#[derive(RConditionError)]` needs at least one variant",
                ));
            }
            let mut message_arms = Vec::new();
            let mut class_arms = Vec::new();
            let mut data_arms = Vec::new();
            let mut needs_display = false;
            for variant in &data.variants {
                let variant_ident = &variant.ident;
                let attrs = parse_container_attrs(&variant.attrs, "a variant")?;
                let member = attrs.class.as_ref().map_or_else(
                    || format!("{family}_{}", to_snake_case(&ident_name(variant_ident))),
                    syn::LitStr::value,
                );
                let shape = Shape::from_fields(
                    &variant.fields,
                    &format!("variant `{name}::{variant_ident}`"),
                )?;
                let path = quote! { Self::#variant_ident };
                match &attrs.message {
                    Some(fmt) => {
                        let pattern = shape.pattern_all(&path);
                        message_arms.push(quote! {
                            #[allow(unused_variables)]
                            #pattern => ::std::format!(#fmt),
                        });
                    }
                    None => needs_display = true,
                }
                let wild = shape.pattern_wild(&path);
                class_arms.push(quote! {
                    #wild => ::std::vec![
                        ::std::string::String::from(#member),
                        ::std::string::String::from(#family),
                    ],
                });
                let pattern = shape.pattern_data(&path);
                let data_expr = shape.data_expr();
                data_arms.push(quote! { #pattern => #data_expr, });
            }
            let message = if message_arms.is_empty() {
                display
            } else if needs_display {
                quote! { match self { #(#message_arms)* _ => #display } }
            } else {
                quote! { match self { #(#message_arms)* } }
            };
            let class = quote! { match self { #(#class_arms)* } };
            let data = quote! { match self { #(#data_arms)* } };
            (message, class, data)
        }
        Data::Union(_) => {
            return Err(syn::Error::new(
                name.span(),
                "`#[derive(RConditionError)]` supports structs and enums, not unions",
            ));
        }
    };

    Ok(quote! {
        #[automatically_derived]
        impl ::miniextendr_api::condition::RConditionError for #name {
            fn message(&self) -> ::std::string::String {
                #message_body
            }

            fn class(&self) -> ::std::vec::Vec<::std::string::String> {
                #class_body
            }

            fn data(
                &self,
            ) -> ::core::option::Option<::miniextendr_api::condition::ConditionData> {
                #data_body
            }
        }
    })
}

// endregion

// region: Tests

#[cfg(test)]
mod tests {
    use super::*;

    fn derive(input: DeriveInput) -> String {
        derive_r_condition_error(input).unwrap().to_string()
    }

    fn derive_err(input: DeriveInput) -> String {
        derive_r_condition_error(input).unwrap_err().to_string()
    }

    #[test]
    fn enum_classes_default_to_snake_case_family_and_member() {
        let code = derive(syn::parse_quote! {
            enum PkgError {
                MissingField { field: String },
                OutOfRange { value: f64, max: f64 },
                Plain,
            }
        });
        assert!(code.contains("\"pkg_error_missing_field\""));
        assert!(code.contains("\"pkg_error_out_of_range\""));
        assert!(code.contains("\"pkg_error_plain\""));
        assert!(code.contains("\"pkg_error\""));
        // No `message` anywhere: Display is the message.
        assert!(code.contains("ToString :: to_string (self)"));
        assert!(!code.contains("format !"));
        // Fields become data entries under their own names.
        assert!(code.contains("\"field\""));
        assert!(code.contains("\"max\""));
        assert!(code.contains("Clone :: clone (max)"));
    }

    #[test]
    fn explicit_classes_messages_and_field_options() {
        let code = derive(syn::parse_quote! {
            #[condition(class = "pkg_warning")]
            enum PkgWarning {
                #[condition(message = "dropped {dropped} of {total} rows")]
                Truncated { dropped: i32, total: i32 },
                #[condition(class = "pkg_warning_slow_path", message = "slow")]
                Slow,
                Other {
                    #[condition(rename = "n_attempts")]
                    attempts: i32,
                    #[condition(debug)]
                    range: std::ops::Range<i32>,
                    #[condition(skip)]
                    detail: Vec<u8>,
                },
            }
        });
        assert!(code.contains("\"pkg_warning_truncated\""));
        assert!(code.contains("\"pkg_warning_slow_path\""));
        assert!(code.contains("\"pkg_warning_other\""));
        assert!(code.contains("format ! (\"dropped {dropped} of {total} rows\")"));
        // A variant without `message` keeps the Display fallback arm.
        assert!(code.contains("_ => :: std :: string :: ToString :: to_string (self)"));
        assert!(code.contains("\"n_attempts\""));
        assert!(!code.contains("\"attempts\""));
        assert!(code.contains("RValue :: debug (range)"));
        assert!(!code.contains("\"detail\""));
        // The skipped field is not bound in the data pattern.
        assert!(code.contains("Self :: Other { attempts , range , .. } =>"));
    }

    #[test]
    fn struct_shapes() {
        let named = derive(syn::parse_quote! {
            #[condition(message = "took {attempts} attempts")]
            struct RetryNotice {
                attempts: i32,
            }
        });
        assert!(named.contains("\"retry_notice\""));
        assert!(named.contains("let Self { attempts } = self"));
        assert!(named.contains("format ! (\"took {attempts} attempts\")"));

        let tuple = derive(syn::parse_quote! {
            struct Pair(#[condition(rename = "left")] i32, #[condition(skip)] String);
        });
        assert!(tuple.contains("\"left\""));
        assert!(tuple.contains("Self (_0 , _)"));

        let unit = derive(syn::parse_quote! {
            struct Nothing;
        });
        assert!(unit.contains("Option :: None"));
        assert!(unit.contains("\"nothing\""));
    }

    #[test]
    fn rejects_generic_types_and_unions() {
        let err = derive_err(syn::parse_quote! {
            struct Generic<T> {
                value: T,
            }
        });
        assert!(err.contains("generic"));

        let err = derive_err(syn::parse_quote! {
            union U {
                a: u32,
                b: f32,
            }
        });
        assert!(err.contains("not unions"));
    }

    #[test]
    fn rejects_unnamed_tuple_fields_and_reserved_or_duplicate_names() {
        let err = derive_err(syn::parse_quote! {
            enum E {
                Raw(i32),
            }
        });
        assert!(err.contains("tuple field 0 of variant `E::Raw` needs a `data` name"));

        let err = derive_err(syn::parse_quote! {
            struct S {
                kind: String,
            }
        });
        assert!(err.contains("`kind` is one of the condition's own slots"));

        let err = derive_err(syn::parse_quote! {
            struct S {
                #[condition(rename = "message")]
                text: String,
            }
        });
        assert!(err.contains("`message` is one of the condition's own slots"));

        let err = derive_err(syn::parse_quote! {
            struct S {
                a: i32,
                #[condition(rename = "a")]
                b: i32,
            }
        });
        assert!(err.contains("duplicate `data` field name `a`"));
    }

    #[test]
    fn rejects_misplaced_or_unknown_options() {
        let err = derive_err(syn::parse_quote! {
            #[condition(message = "nope")]
            enum E {
                A,
            }
        });
        assert!(err.contains("goes on the variants"));

        let err = derive_err(syn::parse_quote! {
            #[condition(prefix = "x")]
            struct S {
                a: i32,
            }
        });
        assert!(err.contains("unknown `condition` option on the type"));

        let err = derive_err(syn::parse_quote! {
            struct S {
                #[condition(class = "x")]
                a: i32,
            }
        });
        assert!(err.contains("unknown `condition` option on a field"));

        let err = derive_err(syn::parse_quote! {
            struct S {
                #[condition(skip, rename = "b")]
                a: i32,
            }
        });
        assert!(err.contains("`skip` cannot be combined"));

        let err = derive_err(syn::parse_quote! {
            enum Empty {}
        });
        assert!(err.contains("at least one variant"));
    }
}

// endregion
