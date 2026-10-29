//! Code generation for `#[derive(RCondition)]`.
//!
//! Generates `impl miniextendr_api::condition::RConditionData` from the
//! type's structure, and `From<T> for miniextendr_api::condition::RCondition`,
//! so a package keeps one Rust type per condition family and gets the R side
//! without hand-written glue:
//!
//! - `class()`: `c(<member>, <family>)` for enum variants, `c(<family>)` for
//!   structs. `#[condition(class("…", "…"))]` on a struct is its whole class
//!   vector, and on a variant it replaces the member. The family is
//!   `#[condition(class = "…")]` on the type, else the
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
//!   slots (`message`, `call`, `kind`) are rejected.
//!   `#[condition(skip_none)]` on an `Option` field leaves its entry out when
//!   it is `None`. `#[condition(data(name = value, …))]` on the type or a
//!   variant adds entries after the fields', each `RValue::from(value)`. With
//!   no entries, `data()` is `None`.
//! - `call()`: `#[condition(call = none)]` on the type (every variant) or on
//!   one variant raises that condition without a call (`conditionCall()` is
//!   `NULL`, R's `call. = FALSE`). Without it the method keeps the trait's
//!   default, the call the transport captured.
//! - `argument_message()`: `#[condition(argument_message = "…")]` on the
//!   struct or on a variant, a `format!` string like `message`. It is the
//!   whole message when the error refuses an argument in a conversion
//!   (#1833); a variant without it keeps the trait's default, `None`.
//!
//! - `From<T> for RCondition`: the kind is `#[condition(kind = …)]` on the
//!   variant, else on the type, else `error`. Only `RCondition::from(x)`
//!   (`.defer()` / `.raise()`) reads it: a returned `Err(x)` is always an R
//!   error, and `defer_warning(x)` / `defer_message(x)` choose their own kind.
//!
//! The payload feeds `defer_warning` & co. (a condition accompanying a value)
//! and `Result<T, E>` returns (a classed error): both go through the trait.
//! Generic types are rejected, like the other derives in this crate.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields};

use crate::naming::{ident_name, to_snake_case};

/// The condition object's own slots. Mirrors
/// `miniextendr_api::condition::RESERVED_CONDITION_FIELDS`; keep in sync.
const RESERVED_FIELDS: [&str; 3] = ["message", "call", "kind"];

// region: Attribute parsing

/// `class = "…"` or `class("…", "…")`.
enum ClassSpec {
    /// A struct's only class, an enum's family, or a variant's member.
    One(syn::LitStr),
    /// A struct's whole class vector, or a variant's classes before the
    /// family; most specific first.
    List(Vec<syn::LitStr>, Span),
}

/// One `data(name = value)` entry.
struct DataEntry {
    name: String,
    value: syn::Expr,
    span: Span,
}

/// `#[condition(...)]` options on the type or on one variant.
#[derive(Default)]
struct ContainerAttrs {
    class: Option<ClassSpec>,
    message: Option<syn::LitStr>,
    /// `argument_message = "…"`: the whole message of an argument error.
    argument_message: Option<syn::LitStr>,
    /// `call = none`: the condition carries no call.
    call_none: bool,
    /// `kind = error | warning | message | condition`: the kind
    /// `From<T> for RCondition` gives the value.
    kind: Option<syn::Ident>,
    /// `data(name = value, …)`: entries after the fields'.
    data: Vec<DataEntry>,
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
                out.class = Some(if meta.input.peek(syn::token::Paren) {
                    let span = meta.path.span();
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let list =
                        Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated(&content)?;
                    if list.is_empty() {
                        return Err(meta.error("`class(...)` needs at least one class"));
                    }
                    ClassSpec::List(list.into_iter().collect(), span)
                } else {
                    ClassSpec::One(meta.value()?.parse()?)
                });
            } else if meta.path.is_ident("data") {
                meta.parse_nested_meta(|entry| {
                    let ident = entry
                        .path
                        .get_ident()
                        .ok_or_else(|| entry.error("a `data` entry is `name = value`"))?
                        .clone();
                    let value: syn::Expr = entry.value()?.parse()?;
                    out.data.push(DataEntry {
                        name: ident_name(&ident),
                        value,
                        span: ident.span(),
                    });
                    Ok(())
                })?;
            } else if meta.path.is_ident("message") {
                if out.message.is_some() {
                    return Err(meta.error("`message` is given twice"));
                }
                out.message = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("argument_message") {
                if out.argument_message.is_some() {
                    return Err(meta.error("`argument_message` is given twice"));
                }
                out.argument_message = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("kind") {
                if out.kind.is_some() {
                    return Err(meta.error("`kind` is given twice"));
                }
                let kind: syn::Ident = meta.value()?.parse()?;
                if !matches!(
                    kind.to_string().as_str(),
                    "error" | "warning" | "message" | "condition"
                ) {
                    return Err(syn::Error::new(
                        kind.span(),
                        "condition kind must be error, warning, message or condition",
                    ));
                }
                out.kind = Some(kind);
            } else if meta.path.is_ident("call") {
                if out.call_none {
                    return Err(meta.error("`call` is given twice"));
                }
                let value: syn::Ident = meta.value()?.parse().map_err(|err| {
                    syn::Error::new(
                        err.span(),
                        "`call` takes `none`: `#[condition(call = none)]` raises the condition \
                         without a call, like R's `call. = FALSE`",
                    )
                })?;
                if value != "none" {
                    return Err(syn::Error::new(
                        value.span(),
                        format!(
                            "unknown `call` value `{value}`; the only one is `none` \
                             (`#[condition(call = none)]`: no call, like R's `call. = FALSE`)"
                        ),
                    ));
                }
                out.call_none = true;
            } else {
                return Err(meta.error(format!(
                    "unknown `condition` option on {on}; expected `class = \"…\"`, \
                     `class(\"…\", \"…\")`, `message = \"…\"`, `argument_message = \"…\"`, \
                     `data(name = value)`, `call = none` or `kind = warning` \
                     (error/message/condition also accepted)"
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
    /// Leave the entry out when the `Option` field is `None`.
    skip_none: bool,
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
            } else if meta.path.is_ident("skip_none") {
                out.skip_none = true;
            } else if meta.path.is_ident("rename") {
                if out.rename.is_some() {
                    return Err(meta.error("`rename` is given twice"));
                }
                out.rename = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error(
                    "unknown `condition` option on a field; expected `skip`, `skip_none`, `debug` \
                     or `rename = \"…\"`",
                ));
            }
            Ok(())
        })?;
    }
    if out.skip && (out.rename.is_some() || out.debug || out.skip_none) {
        return Err(syn::Error::new(
            attrs
                .iter()
                .find(|a| a.path().is_ident("condition"))
                .map_or_else(Span::call_site, Spanned::span),
            "`skip` cannot be combined with `rename`, `debug` or `skip_none`",
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
    /// An `Option` field whose entry is left out when it is `None`.
    skip_none: bool,
}

/// Whether `ty` is written as `Option<_>` (any path ending in `Option`).
fn is_option(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.qself.is_none()
        && path.path.segments.last().is_some_and(|segment| segment.ident == "Option"))
}

/// Refuse a name the condition object owns, or one already in `seen`.
fn check_data_name(name: &str, seen: &[String], span: Span, context: &str) -> syn::Result<()> {
    if RESERVED_FIELDS.contains(&name) {
        return Err(syn::Error::new(
            span,
            format!(
                "`{name}` is one of the condition's own slots (`message`, `call`, `kind`); \
                 rename the field with `#[condition(rename = \"…\")]` or exclude it with `#[condition(skip)]`"
            ),
        ));
    }
    if seen.iter().any(|s| s == name) {
        return Err(syn::Error::new(
            span,
            format!("duplicate `data` field name `{name}` in {context}"),
        ));
    }
    Ok(())
}

enum Style {
    Named,
    Tuple,
    Unit,
}

/// The fields of a struct or of one enum variant, validated, with the
/// `data(...)` entries that follow them.
struct Shape<'a> {
    style: Style,
    fields: Vec<FieldInfo>,
    extra: Vec<&'a DataEntry>,
}

impl<'a> Shape<'a> {
    fn from_fields(
        fields: &Fields,
        extra: Vec<&'a DataEntry>,
        context: &str,
    ) -> syn::Result<Shape<'a>> {
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
                check_data_name(&name, &seen, field.span(), context)?;
                seen.push(name.clone());
                Some(name)
            };
            if attrs.skip_none && !is_option(&field.ty) {
                return Err(syn::Error::new(
                    field.ty.span(),
                    "`skip_none` needs an `Option<_>` field: it leaves the entry out when the field is `None`",
                ));
            }
            infos.push(FieldInfo {
                binding,
                data_name,
                debug: attrs.debug,
                skip_none: attrs.skip_none,
            });
        }
        for entry in &extra {
            check_data_name(&entry.name, &seen, entry.span, context)?;
            seen.push(entry.name.clone());
        }
        Ok(Shape {
            style,
            fields: infos,
            extra,
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
    /// [`Self::pattern_data`]: the fields' entries, then the `data(...)` ones.
    fn data_expr(&self) -> TokenStream {
        let value = |f: &FieldInfo, binding: TokenStream| {
            if f.debug {
                quote! { ::miniextendr_api::RValue::debug(#binding) }
            } else {
                quote! { ::miniextendr_api::RValue::from(::core::clone::Clone::clone(#binding)) }
            }
        };
        let extra = self.extra.iter().map(|entry| {
            let name = &entry.name;
            let expr = &entry.value;
            quote! { (::std::string::String::from(#name), ::miniextendr_api::RValue::from(#expr)) }
        });
        if !self
            .fields
            .iter()
            .any(|f| f.skip_none && f.data_name.is_some())
        {
            let entries: Vec<TokenStream> = self
                .fields
                .iter()
                .filter_map(|f| {
                    let name = f.data_name.as_ref()?;
                    let binding = &f.binding;
                    let value = value(f, quote! { #binding });
                    Some(quote! { (::std::string::String::from(#name), #value) })
                })
                .chain(extra)
                .collect();
            return if entries.is_empty() {
                quote! { ::core::option::Option::None }
            } else {
                quote! { ::core::option::Option::Some(::std::vec![#(#entries),*]) }
            };
        }
        // A `None` in a `skip_none` field leaves its entry out, so the
        // entries are pushed one by one, and none at all is `None`.
        let pushes = self.fields.iter().filter_map(|f| {
            let name = f.data_name.as_ref()?;
            let binding = &f.binding;
            Some(if f.skip_none {
                let value = value(f, quote! { __value });
                quote! {
                    if let ::core::option::Option::Some(__value) = #binding {
                        __data.push((::std::string::String::from(#name), #value));
                    }
                }
            } else {
                let value = value(f, quote! { #binding });
                quote! { __data.push((::std::string::String::from(#name), #value)); }
            })
        });
        let extra = extra.map(|entry| quote! { __data.push(#entry); });
        quote! {{
            let mut __data: ::miniextendr_api::condition::ConditionData = ::std::vec::Vec::new();
            #(#pushes)*
            #(#extra)*
            (!__data.is_empty()).then_some(__data)
        }}
    }
}

// endregion

// region: Entry point

/// Main entry point for `#[derive(RCondition)]`: the `RConditionData`
/// metadata and `From<T> for RCondition` with the declared kind.
pub fn derive_r_condition(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    if let Some(param) = input
        .generics
        .params
        .iter()
        .find(|param| !matches!(param, syn::GenericParam::Lifetime(_)))
    {
        return Err(syn::Error::new(
            param.span(),
            "`#[derive(RCondition)]` does not support type or const parameters; \
             lifetime parameters are fine",
        ));
    }
    let container = parse_container_attrs(&input.attrs, "the type")?;
    derive_condition(&input, name, container)
}

/// The `RCondition` variant for a declared kind; no kind is an error.
fn condition_expr(kind: Option<&syn::Ident>) -> TokenStream {
    let variant = match kind.map(ToString::to_string).as_deref() {
        Some("error") | None => quote! { Error },
        Some("warning") => quote! { Warning },
        Some("message") => quote! { Message },
        Some("condition") => quote! { Condition },
        _ => unreachable!("kind was validated while parsing"),
    };
    quote! { Self::#variant { message, class, data, call } }
}

fn derive_condition(
    input: &DeriveInput,
    name: &syn::Ident,
    container: ContainerAttrs,
) -> syn::Result<TokenStream> {
    let default_family = || to_snake_case(&ident_name(name));

    let condition_body;
    let display = quote! { ::std::string::ToString::to_string(self) };
    let call_none = quote! { ::miniextendr_api::condition::ConditionCall::None };

    // `call_body` is `None` when no part of the type asks for `call = none`,
    // and `argument_message_body` when none gives an `argument_message`: the
    // impl then keeps the trait's default.
    let (message_body, class_body, data_body, call_body, argument_message_body) = match &input.data
    {
        Data::Struct(data) => {
            let shape = Shape::from_fields(
                &data.fields,
                container.data.iter().collect(),
                &format!("`{name}`"),
            )?;
            condition_body = condition_expr(container.kind.as_ref());
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
            let classes: Vec<String> = match &container.class {
                Some(ClassSpec::One(lit)) => vec![lit.value()],
                Some(ClassSpec::List(list, _)) => list.iter().map(syn::LitStr::value).collect(),
                None => vec![default_family()],
            };
            let class = quote! { ::std::vec![#(::std::string::String::from(#classes)),*] };
            let pattern = shape.pattern_data(&self_path);
            let data_expr = shape.data_expr();
            let data = quote! {
                let #pattern = self;
                #data_expr
            };
            let call = container.call_none.then(|| call_none.clone());
            let argument_message = container.argument_message.as_ref().map(|fmt| {
                let pattern = shape.pattern_all(&self_path);
                quote! {
                    #[allow(unused_variables)]
                    let #pattern = self;
                    ::core::option::Option::Some(::std::format!(#fmt))
                }
            });
            (message, class, data, call, argument_message)
        }
        Data::Enum(data) => {
            if let Some(message) = &container.message {
                return Err(syn::Error::new(
                    message.span(),
                    "`message` goes on the variants of an enum, not on the enum itself",
                ));
            }
            if let Some(message) = &container.argument_message {
                return Err(syn::Error::new(
                    message.span(),
                    "`argument_message` goes on the variants of an enum, not on the enum itself",
                ));
            }
            if data.variants.is_empty() {
                return Err(syn::Error::new(
                    name.span(),
                    "`#[derive(RCondition)]` needs at least one variant",
                ));
            }
            let family = match &container.class {
                Some(ClassSpec::One(lit)) => lit.value(),
                Some(ClassSpec::List(_, span)) => {
                    return Err(syn::Error::new(
                        *span,
                        "on an enum, `class(...)` goes on a variant; the enum's \
                         `class = \"…\"` names the family every variant ends with",
                    ));
                }
                None => default_family(),
            };
            let variant_count = data.variants.len();
            let mut condition_arms = Vec::new();
            let mut message_arms = Vec::new();
            let mut class_arms = Vec::new();
            let mut data_arms = Vec::new();
            let mut call_none_arms = Vec::new();
            let mut argument_message_arms = Vec::new();
            let mut needs_display = false;
            for variant in &data.variants {
                let variant_ident = &variant.ident;
                let attrs = parse_container_attrs(&variant.attrs, "a variant")?;
                let members: Vec<String> = match &attrs.class {
                    Some(ClassSpec::One(lit)) => vec![lit.value()],
                    Some(ClassSpec::List(list, _)) => list.iter().map(syn::LitStr::value).collect(),
                    None => vec![format!(
                        "{family}_{}",
                        to_snake_case(&ident_name(variant_ident))
                    )],
                };
                let shape = Shape::from_fields(
                    &variant.fields,
                    container.data.iter().chain(&attrs.data).collect(),
                    &format!("variant `{name}::{variant_ident}`"),
                )?;
                let pattern = shape.pattern_wild(&quote! { #name::#variant_ident });
                let expr = condition_expr(attrs.kind.as_ref().or(container.kind.as_ref()));
                condition_arms.push(quote! { #pattern => #expr, });
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
                        #(::std::string::String::from(#members),)*
                        ::std::string::String::from(#family),
                    ],
                });
                let pattern = shape.pattern_data(&path);
                let data_expr = shape.data_expr();
                data_arms.push(quote! { #pattern => #data_expr, });
                if let Some(fmt) = &attrs.argument_message {
                    let pattern = shape.pattern_all(&path);
                    argument_message_arms.push(quote! {
                        #[allow(unused_variables)]
                        #pattern => ::core::option::Option::Some(::std::format!(#fmt)),
                    });
                }
                if attrs.call_none {
                    call_none_arms.push(wild);
                }
            }
            condition_body = quote! { match value { #(#condition_arms)* } };
            let message = if message_arms.is_empty() {
                display
            } else if needs_display {
                quote! { match self { #(#message_arms)* _ => #display } }
            } else {
                quote! { match self { #(#message_arms)* } }
            };
            let class = quote! { match self { #(#class_arms)* } };
            let data = quote! { match self { #(#data_arms)* } };
            let call = if container.call_none || call_none_arms.len() == variant_count {
                Some(call_none.clone())
            } else if call_none_arms.is_empty() {
                None
            } else {
                Some(quote! {
                    match self {
                        #(#call_none_arms)|* => #call_none,
                        _ => ::miniextendr_api::condition::ConditionCall::Inherit,
                    }
                })
            };
            let argument_message = if argument_message_arms.is_empty() {
                None
            } else if argument_message_arms.len() == variant_count {
                Some(quote! { match self { #(#argument_message_arms)* } })
            } else {
                Some(quote! {
                    #[allow(unreachable_patterns)]
                    match self {
                        #(#argument_message_arms)*
                        _ => ::core::option::Option::None,
                    }
                })
            };
            (message, class, data, call, argument_message)
        }
        Data::Union(_) => {
            return Err(syn::Error::new(
                name.span(),
                "`#[derive(RCondition)]` supports structs and enums, not unions",
            ));
        }
    };

    let call_method = call_body.map(|body| {
        quote! {
            fn call(&self) -> ::miniextendr_api::condition::ConditionCall {
                #body
            }
        }
    });
    let argument_message_method = argument_message_body.map(|body| {
        quote! {
            fn argument_message(&self) -> ::core::option::Option<::std::string::String> {
                #body
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::core::convert::From<#name #ty_generics>
            for ::miniextendr_api::condition::RCondition #where_clause
        {
            fn from(value: #name #ty_generics) -> Self {
                let message = ::miniextendr_api::condition::RConditionData::message(&value);
                let class = ::miniextendr_api::condition::RConditionData::class(&value);
                let data = ::miniextendr_api::condition::RConditionData::data(&value);
                let call = ::miniextendr_api::condition::RConditionData::call(&value);
                #condition_body
            }
        }

        #[automatically_derived]
        impl #impl_generics ::miniextendr_api::condition::RConditionData
            for #name #ty_generics #where_clause
        {
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

            #call_method

            #argument_message_method
        }
    })
}

// endregion

// region: Tests

#[cfg(test)]
mod tests;

// endregion
