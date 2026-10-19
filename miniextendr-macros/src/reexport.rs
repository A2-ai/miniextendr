//! `reexport!(pkg::name, ...)`: R re-exports declared from Rust (#1910).
//!
//! Each item is another package's export that this package exports too. The
//! macro registers one `ReexportEntry` per item in
//! `miniextendr_api::registry::MX_REEXPORTS`; the wrapper writer turns them
//! into roxygen2's re-export blocks (`@importFrom pkg name`, `@export`,
//! `pkg::name`), one per `pkg::name` however many sites declare it, and
//! refuses a name re-exported from two packages or bound by the package
//! itself.
//!
//! The package and a syntactic name are dot-joined identifiers
//! (`data.table::fread`, `stats::na.omit`); a non-syntactic name is a string
//! literal (`magrittr::"%>%"`). Every problem of one invocation is reported at
//! once, each on its own tokens.

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::ext::IdentExt;
use syn::parse::{ParseStream, Parser};

/// One `pkg::name` item.
struct Item {
    package: String,
    /// The tokens of the package, for an error on them.
    package_tokens: TokenStream,
    name: String,
    /// The tokens of the name, for an error on them.
    name_tokens: TokenStream,
    /// Whether the name is a string literal.
    literal: bool,
    /// The item's first token, whose position the entry records.
    span: Span,
}

/// Expand `reexport!(...)`.
pub(crate) fn expand(input: TokenStream) -> TokenStream {
    match parse_items(input) {
        Ok(items) => entries(&items),
        Err(err) => err.into_compile_error(),
    }
}

/// The items of an invocation, or every problem found in it.
fn parse_items(input: TokenStream) -> syn::Result<Vec<Item>> {
    if input.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "`reexport!` takes one or more `pkg::name` items, e.g. `reexport!(generics::tidy)`",
        ));
    }
    let mut errors: Option<syn::Error> = None;
    let mut push = |err: syn::Error| match &mut errors {
        Some(errors) => errors.combine(err),
        None => errors = Some(err),
    };
    let mut items: Vec<Item> = Vec::new();
    let segments = split_items(input);
    let last = segments.len() - 1;
    for (index, (segment, comma)) in segments.into_iter().enumerate() {
        if segment.is_empty() {
            // A trailing comma ends the list; any other empty item is a stray comma.
            if index != last {
                push(syn::Error::new(
                    comma.unwrap_or_else(Span::call_site),
                    "expected a `pkg::name` item before this `,`",
                ));
            }
            continue;
        }
        let item = match parse_item.parse2(segment) {
            Ok(item) => item,
            Err(err) => {
                push(err);
                continue;
            }
        };
        let mut valid = true;
        if let Some(problem) = package_problem(&item.package) {
            push(syn::Error::new_spanned(&item.package_tokens, problem));
            valid = false;
        }
        if let Some(problem) = name_problem(&item.package, &item.name, item.literal) {
            push(syn::Error::new_spanned(&item.name_tokens, problem));
            valid = false;
        }
        if !valid {
            continue;
        }
        if let Some(earlier) = items
            .iter()
            .find(|earlier| earlier.name == item.name && earlier.package != item.package)
        {
            push(syn::Error::new_spanned(
                &item.package_tokens,
                format!(
                    "`{}` is already re-exported from `{}` in this `reexport!`; a package \
                     exports one object under a name",
                    item.name, earlier.package
                ),
            ));
            continue;
        }
        items.push(item);
    }
    match errors {
        Some(errors) => Err(errors),
        None => Ok(items),
    }
}

/// The input split at its top-level commas: each item's tokens and the comma
/// after it (`None` after the last). Delimited groups are single token trees,
/// so a comma inside one does not split.
fn split_items(input: TokenStream) -> Vec<(TokenStream, Option<Span>)> {
    let mut segments = Vec::new();
    let mut current = TokenStream::new();
    for tree in input {
        match &tree {
            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                segments.push((std::mem::take(&mut current), Some(punct.span())));
            }
            _ => current.extend([tree]),
        }
    }
    segments.push((current, None));
    segments
}

/// One item: `<dotted identifiers>::<dotted identifiers | string literal>`.
fn parse_item(input: ParseStream) -> syn::Result<Item> {
    let span = input.span();
    let (package, package_tokens) = dotted_identifiers(input, "the package name of `pkg::name`")?;
    if !input.peek(syn::Token![::]) {
        let message =
            format!("expected `::` after the package `{package}`: an item is `pkg::name`");
        return Err(if input.is_empty() {
            syn::Error::new_spanned(&package_tokens, message)
        } else {
            input.error(message)
        });
    }
    input.parse::<syn::Token![::]>()?;
    let (name, name_tokens, literal) = if input.peek(syn::LitStr) {
        let lit: syn::LitStr = input.parse()?;
        if !lit.suffix().is_empty() {
            return Err(syn::Error::new_spanned(
                &lit,
                "a re-exported name written as a string literal takes no suffix",
            ));
        }
        (lit.value(), lit.to_token_stream(), true)
    } else {
        let (name, tokens) = dotted_identifiers(
            input,
            "the name of `pkg::name`, as identifiers joined by `.` or a string literal",
        )?;
        (name, tokens, false)
    };
    if !input.is_empty() {
        let message = if input.peek(syn::Token![::]) {
            "an item names a package and one of its exports, `pkg::name`; separate items with `,`"
        } else {
            "unexpected token after `pkg::name`; separate items with `,`"
        };
        return Err(input.error(message));
    }
    Ok(Item {
        package,
        package_tokens,
        name,
        name_tokens,
        literal,
        span,
    })
}

/// Identifiers joined by `.` (`data.table`, `na.omit`), read as written:
/// a Rust keyword is an identifier here (`base::as.numeric`), and a raw
/// identifier loses its `r#`.
fn dotted_identifiers(input: ParseStream, what: &str) -> syn::Result<(String, TokenStream)> {
    if !input.peek(syn::Ident::peek_any) {
        return Err(input.error(format!("expected {what}")));
    }
    let mut tokens = TokenStream::new();
    let first = syn::Ident::parse_any(input)?;
    let mut text = first.unraw().to_string();
    first.to_tokens(&mut tokens);
    while input.peek(syn::Token![.]) && !input.peek(syn::Token![..]) {
        let dot: syn::Token![.] = input.parse()?;
        if !input.peek(syn::Ident::peek_any) {
            return Err(input.error(
                "expected an identifier after `.`; write a name that is not identifiers \
                 joined by `.` as a string literal, `pkg::\"name\"`",
            ));
        }
        let part = syn::Ident::parse_any(input)?;
        text.push('.');
        text.push_str(&part.unraw().to_string());
        dot.to_tokens(&mut tokens);
        part.to_tokens(&mut tokens);
    }
    Ok((text, tokens))
}

/// Why `package` is not an R package name, if it isn't.
fn package_problem(package: &str) -> Option<String> {
    (!crate::naming::is_r_package_name(package)).then(|| {
        format!(
            "`{package}` is not an R package name: ASCII letters, digits and `.`, at least two \
             characters, starting with a letter and not ending in `.`"
        )
    })
}

/// Why `name` can't be re-exported, if it can't. Written as identifiers it
/// must be a syntactic R name; a string literal may be any non-empty ASCII
/// text without whitespace, control characters or the quoting characters
/// `` ` ``, `\` and `"`, which the generated R code and roxygen2 tags write
/// verbatim or between quotes.
fn name_problem(package: &str, name: &str, literal: bool) -> Option<String> {
    if !literal {
        return (!crate::naming::is_syntactic_r_name(name)).then(|| {
            format!(
                "`{name}` is not a syntactic R name; write a non-syntactic name as a string \
                 literal, `{package}::\"{name}\"`"
            )
        });
    }
    if name.is_empty() {
        Some("a re-exported name can't be empty".to_string())
    } else if !name.is_ascii() {
        Some(format!(
            "`{name}` is not ASCII; R code in a package must be ASCII"
        ))
    } else if name
        .chars()
        .any(|c| c.is_ascii_whitespace() || c.is_ascii_control() || matches!(c, '`' | '\\' | '"'))
    {
        Some(format!(
            "the re-exported name `{name}` can't hold whitespace, control characters, `` ` ``, \
             `\\` or `\"`"
        ))
    } else {
        None
    }
}

/// One `MX_REEXPORTS` entry per item, in an anonymous `const _` so any number
/// of invocations live side by side. The slice is host-only, so on wasm32 the
/// invocation expands to nothing.
fn entries(items: &[Item]) -> TokenStream {
    let statics = items.iter().enumerate().map(|(index, item)| {
        let ident = quote::format_ident!("REEXPORT_{index}");
        let package = &item.package;
        let name = &item.name;
        let start = item.span.start();
        let source_line = syn::LitInt::new(&start.line.to_string(), item.span);
        let source_column = syn::LitInt::new(&(start.column + 1).to_string(), item.span);
        quote! {
            #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_REEXPORTS)]
            #[linkme(crate = ::miniextendr_api::linkme)]
            static #ident: ::miniextendr_api::registry::ReexportEntry =
                ::miniextendr_api::registry::ReexportEntry {
                    package: #package,
                    name: #name,
                    source_file: ::core::file!(),
                    source_line: #source_line,
                    source_column: #source_column,
                };
        }
    });
    quote! {
        #[cfg(not(target_arch = "wasm32"))]
        const _: () = {
            #(#statics)*
        };
    }
}

#[cfg(test)]
mod tests;
