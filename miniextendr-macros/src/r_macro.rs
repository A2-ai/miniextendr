//! Implementation of the `r!` proc-macro.
//!
//! Parses the optional `env: <expr> ;` head, validates the R token tail with
//! the conservative grammar checker in [`grammar`], then either:
//!
//! 1. **Lowers** the tail to `RCall`-based `Rf_lang*` construction when the
//!    tail is a single call of the lowerable subset (see [`lowering`]), or
//! 2. **Falls back** to the `r_eval_str` string path for everything else.
//!
//! The fallback path produces code byte-identical to the old `macro_rules! r`
//! expansion — `::core::stringify!` rather than `TokenStream::to_string()` so
//! spacing normalisation is identical.
//!
//! ```rust,ignore
//! // Lowered: r!(c(1L, 2L, 3L))
//! unsafe {
//!     let __r_scope = ::miniextendr_api::gc_protect::ProtectScope::new();
//!     // ... protect each arg ...
//!     ::miniextendr_api::expression::RCall::new("c")
//!         .arg(__r_arg_0)
//!         ...
//!         .eval(::miniextendr_api::sys::R_GlobalEnv)
//! }
//!
//! // Fallback: r!(1L + 2L)
//! unsafe {
//!     ::miniextendr_api::expression::r_eval_str(
//!         ::core::stringify!(1L + 2L),
//!         ::miniextendr_api::sys::R_GlobalEnv,
//!     )
//! }
//! ```

pub(crate) mod grammar;
pub(crate) mod lowering;

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

    // Build the environment expression used in both paths.
    let env_ts: TokenStream = match env_expr {
        Some(ref e) => quote! { #e },
        None => quote! { ::miniextendr_api::sys::R_GlobalEnv },
    };

    // Attempt to lower the tail to an RCall-based expansion.
    // Falls back to the stringify + r_eval_str path when not applicable.
    let expanded = if let Some(lowered) = lowering::try_lower(&tail_tokens, &env_ts) {
        lowered
    } else {
        quote! {
            unsafe {
                ::miniextendr_api::expression::r_eval_str(
                    ::core::stringify!(#tail_tokens),
                    #env_ts,
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
    let has_env_prefix = tokens.len() >= 2
        && matches!(&tokens[0], proc_macro2::TokenTree::Ident(id) if id == "env")
        && matches!(&tokens[1], proc_macro2::TokenTree::Punct(p) if p.as_char() == ':');

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
