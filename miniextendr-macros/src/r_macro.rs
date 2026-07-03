//! Implementation of the `r!` proc-macro.
//!
//! Parses the optional `env: <expr> ;` head, validates the R token tail with
//! the conservative grammar checker in [`grammar`], then emits an expansion
//! byte-identical to the old `macro_rules! r` expansion:
//!
//! ```rust,ignore
//! // r!(tokens)
//! unsafe {
//!     ::miniextendr_api::expression::r_eval_str(
//!         ::core::stringify!(tokens),
//!         ::miniextendr_api::sys::R_GlobalEnv,
//!     )
//! }
//!
//! // r!(env: e; tokens)
//! unsafe {
//!     ::miniextendr_api::expression::r_eval_str(
//!         ::core::stringify!(tokens),
//!         e,
//!     )
//! }
//! ```
//!
//! The `::core::stringify!` re-emission (rather than `TokenStream::to_string()`)
//! guarantees byte-identical runtime behaviour: `stringify!` is the same
//! built-in that the old `macro_rules!` used, so spacing normalisation is
//! identical.

pub(crate) mod grammar;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Expr};

// region: Public entry point

/// Parse and validate `r!(...)` input, returning the expanded `TokenStream`.
pub(crate) fn expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match expand_inner(input.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// endregion

// region: Parser + expansion

fn expand_inner(input: TokenStream) -> Result<TokenStream, Error> {
    // Parse the input as `[env: <Expr> ;] <tail_tokens>`.
    let (env_expr, tail_tokens) = parse_r_macro_input(input)?;

    // Conservative grammar validation on the tail.
    grammar::validate(&tail_tokens)?;

    // Emit byte-identical expansion.
    let expanded = if let Some(env) = env_expr {
        quote! {
            unsafe {
                ::miniextendr_api::expression::r_eval_str(
                    ::core::stringify!(#tail_tokens),
                    #env,
                )
            }
        }
    } else {
        quote! {
            unsafe {
                ::miniextendr_api::expression::r_eval_str(
                    ::core::stringify!(#tail_tokens),
                    ::miniextendr_api::sys::R_GlobalEnv,
                )
            }
        }
    };

    Ok(expanded)
}

// endregion

// region: Input parser

/// Parse the `r!(...)` input into an optional `env` expression and the tail token stream.
///
/// Grammar: `[env: <Expr> ;] <tail_tokens...>`
///
/// The `env:` head is identified by the leading `env` ident followed by `:`.
/// The `<Expr>` is parsed greedily up to the first **top-level** `;` token
/// (matching the `macro_rules!` contract: `env: $env:expr; $($code:tt)+` where
/// `$env:expr` consumes greedily until the `;`).
///
/// After the `;`, the remaining tokens form the tail.
fn parse_r_macro_input(input: TokenStream) -> Result<(Option<TokenStream>, TokenStream), Error> {
    // Collect all tokens for lookahead.
    let tokens: Vec<proc_macro2::TokenTree> = input.clone().into_iter().collect();

    // Detect `env:` prefix: first token is `env` ident, second is `:` punct.
    // The `:` must be Alone-spaced — a Joint `:` is the first half of `::`,
    // i.e. R code like `env::foo` (namespace access), not the env head.
    let has_env_prefix = tokens.len() >= 2
        && matches!(&tokens[0], proc_macro2::TokenTree::Ident(id) if id == "env")
        && matches!(&tokens[1], proc_macro2::TokenTree::Punct(p)
            if p.as_char() == ':' && p.spacing() == proc_macro2::Spacing::Alone);

    if !has_env_prefix {
        // No env head — entire input is the R tail.
        if tokens.is_empty() {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "r!() requires at least one R token",
            ));
        }
        return Ok((None, input));
    }

    // Skip `env` + `:` (tokens[0] and tokens[1]); the remainder is `<Expr> ; <tail>`.
    // We need to split at the first TOP-LEVEL `;`.
    let after_env_colon = &tokens[2..];

    // Find the first top-level `;` token.
    let semi_idx = after_env_colon
        .iter()
        .position(|t| matches!(t, proc_macro2::TokenTree::Punct(p) if p.as_char() == ';'));

    let semi_idx = semi_idx.ok_or_else(|| {
        Error::new(
            proc_macro2::Span::call_site(),
            "r!(env: ...) missing `;` separator — expected `r!(env: <expr>; <R code>)`",
        )
    })?;

    let env_tokens: TokenStream = after_env_colon[..semi_idx].iter().cloned().collect();
    let tail_tokens: TokenStream = after_env_colon[semi_idx + 1..].iter().cloned().collect();

    if env_tokens.is_empty() {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "r!(env: ...) has an empty `env` expression — expected `r!(env: <expr>; <R code>)`",
        ));
    }

    if tail_tokens.is_empty() {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "r!(env: ...; <code>) has no R code after the `;` separator",
        ));
    }

    // Parse the env expression string via syn to get a proper `Expr`.
    // We use syn::parse2 on the env_tokens.
    let env_expr: Expr = syn::parse2(env_tokens).map_err(|e| {
        Error::new(
            e.span(),
            format!("r!(env: ...) — invalid env expression: {e}"),
        )
    })?;

    Ok((Some(quote! { #env_expr }), tail_tokens))
}

// endregion

// region: Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> (Option<TokenStream>, TokenStream) {
        parse_r_macro_input(src.parse().unwrap()).unwrap()
    }

    #[test]
    fn env_head_is_split_off() {
        let (env, tail) = parse("env: e; x + 1");
        assert_eq!(env.unwrap().to_string(), "e");
        assert_eq!(tail.to_string(), "x + 1");
    }

    #[test]
    fn double_colon_is_r_code_not_env_head() {
        // `env::foo()` is R namespace access on a package named `env`,
        // not the `env: <expr> ;` head.
        let (env, tail) = parse("env::foo()");
        assert!(env.is_none());
        assert_eq!(tail.to_string(), "env :: foo ()");
    }

    #[test]
    fn env_head_without_semicolon_errors() {
        let err = parse_r_macro_input("env: e".parse().unwrap()).unwrap_err();
        assert!(err.to_string().contains("missing `;`"));
    }
}

// endregion
