//! Lowering of the call-shaped `r!()` subset to `RCall`-based Rf_lang* construction.
//!
//! # Strategy
//!
//! After grammar validation, the macro attempts to classify the tail token
//! stream as a **lowerable call**: a single top-level call of the form
//! `ident(args…)` or `pkg::fn(args…)` whose every argument belongs to the
//! lowerable atom set (string literals, integer/float literals, symbolic
//! constants, bare identifiers, nested lowerable calls, or `name = value`
//! named args).
//!
//! When lowerable, the macro emits code that builds the call tree with
//! `RCall` + literal→SEXP constructors and evaluates it. This avoids the
//! parse-then-eval round-trip through `r_eval_str`.
//!
//! When NOT lowerable (operators, assignments, control flow, `[`/`$`/`@`,
//! any ambiguity), it falls back to the original `r_eval_str` string path.
//! The fallback is the safety guarantee — no accepted input can start failing.
//!
//! # Lowerable subset (exhaustive)
//!
//! ## Top-level shape
//! - `ident(args…)` — simple function call
//! - `pkg::fn(args…)` — namespaced call
//!
//! ## Arg atoms
//! - String literal (`"hello"`)
//! - Integer literal (`42L` — one proc_macro2 token with the `L` suffix).
//!   An unsuffixed `42` lowers as a **double**, matching R's parser (R only
//!   produces INTSXP for `L`-suffixed literals; `typeof(42)` is `"double"`)
//! - Float literal (`1.5`, `1e3`)
//! - Symbolic constants: `TRUE`, `FALSE`, `NULL`, `NA`, `NA_integer_`,
//!   `NA_real_`, `NA_character_`, `NA_complex_`, `Inf`, `NaN`
//! - Bare identifier (symbol lookup at eval time)
//! - Nested lowerable call
//! - Named arg: `name = <atom>`
//!
//! Everything else → fallback.

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::quote;

// region: Public entry point

/// Attempt to lower a validated R tail token stream to a `RCall`-based
/// `TokenStream`.
///
/// Returns `Some(ts)` when the tail is fully lowerable; `None` to signal
/// the caller should emit the `r_eval_str` string path instead.
pub(crate) fn try_lower(tail: &TokenStream, env_expr: &TokenStream) -> Option<TokenStream> {
    let tokens: Vec<TokenTree> = tail.clone().into_iter().collect();
    let call = classify_call(&tokens)?;
    Some(emit_call(&call, env_expr))
}

// endregion

// region: AST for lowerable calls

/// A lowerable R call expression.
#[derive(Debug)]
pub(crate) struct LowerCall {
    /// The function to call. Either `plain::name` or namespaced `pkg::fun`.
    fun: LowerFun,
    /// The arguments.
    args: Vec<LowerArg>,
}

/// Function designator.
#[derive(Debug)]
enum LowerFun {
    /// Simple `ident(args…)` — `Rf_install("ident")`.
    Simple(String),
    /// `pkg::fun(args…)`.
    Namespaced { pkg: String, fun: String },
}

/// A single argument (positional or named).
#[derive(Debug)]
struct LowerArg {
    name: Option<String>,
    value: LowerAtom,
}

/// An atom that maps directly to an R SEXP constructor.
#[derive(Debug)]
enum LowerAtom {
    /// `"hello"` — `SEXP::scalar_string_from_str(…)`
    StringLit(String),
    /// `42L` — `SEXP::scalar_integer(…)`
    IntLit(i32),
    /// `1.5`, `1e3`, or unsuffixed `42` (R parses unsuffixed numeric
    /// literals as double) — `SEXP::scalar_real(…)`
    RealLit(f64),
    /// `TRUE` / `FALSE` — `SEXP::scalar_logical(…)`
    Bool(bool),
    /// `NULL` — `SEXP::nil()`
    Null,
    /// Bare `NA` — **logical** NA (`typeof(NA)` is `"logical"` in R)
    Na,
    /// `NA_integer_` — `SEXP::scalar_integer(i32::MIN)`
    NaInteger,
    /// `NA_real_` — `SEXP::scalar_real(NA_REAL)`
    NaReal,
    /// `NA_character_` — `SEXP::scalar_string(SEXP::na_string())`
    NaCharacter,
    /// `Inf` — `SEXP::scalar_real(f64::INFINITY)`
    Inf,
    /// `NaN` — `SEXP::scalar_real(f64::NAN)`
    NaN,
    /// Bare identifier (symbol) — `Rf_install` at eval time via RSymbol
    Symbol(String),
    /// Nested lowerable call
    Call(Box<LowerCall>),
}

// endregion

// region: Classifier

/// Classify a flat token slice as a lowerable top-level call.
///
/// Returns `None` for anything that's not a lowerable call.
///
/// Supported patterns:
/// - `ident(args…)` — simple call, 2 tokens
/// - `ident.ident.…(args…)` — R dot-name call (e.g. `is.null`), multiple tokens + group
/// - `pkg::fn(args…)` — namespaced call, 5 tokens
fn classify_call(tokens: &[TokenTree]) -> Option<LowerCall> {
    // We need at minimum an Ident + a paren group.
    if tokens.len() < 2 {
        return None;
    }

    // The last token must be a parenthesis group.
    let group = match tokens.last() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => g,
        _ => return None,
    };

    let head = &tokens[..tokens.len() - 1];

    // Case 1: single ident `ident(args…)`
    if let [TokenTree::Ident(name)] = head {
        let fun = LowerFun::Simple(name.to_string());
        let args = classify_args(group)?;
        return Some(LowerCall { fun, args });
    }

    // Case 2: `pkg::fn(args…)` — five tokens total (4 in head)
    // Tokenises as: Ident("pkg") Punct(':', Joint) Punct(':') Ident("fn") Group(…)
    // The first `:` must be Joint-spaced: `pkg : : fn(x)` (spaced) is an R
    // parse error and must fall back so the string path rejects it at runtime.
    if let [
        TokenTree::Ident(pkg),
        TokenTree::Punct(c1),
        TokenTree::Punct(c2),
        TokenTree::Ident(fun),
    ] = head
        && c1.as_char() == ':'
        && c1.spacing() == proc_macro2::Spacing::Joint
        && c2.as_char() == ':'
    {
        let fun = LowerFun::Namespaced {
            pkg: pkg.to_string(),
            fun: fun.to_string(),
        };
        let args = classify_args(group)?;
        return Some(LowerCall { fun, args });
    }

    // Case 3: R dot-name `ident.ident.…(args…)` (e.g. `is.null`, `as.character`)
    // Head alternates: Ident Punct('.') Ident Punct('.') … Ident
    // Accept only if every separator is '.' and no other punct.
    if let Some(name) = try_parse_r_dotname(head) {
        let fun = LowerFun::Simple(name);
        let args = classify_args(group)?;
        return Some(LowerCall { fun, args });
    }

    // Anything else (operators, assignments, control flow, etc.) → fallback
    None
}

/// Try to parse a token sequence as an R dot-separated function name.
///
/// Accepts alternating `Ident Punct('.') Ident Punct('.') … Ident` (odd
/// number of tokens, starting and ending with Ident). Returns the
/// reconstructed name string (e.g. `"is.null"`) or `None`.
fn try_parse_r_dotname(tokens: &[TokenTree]) -> Option<String> {
    if tokens.is_empty() || tokens.len().is_multiple_of(2) {
        return None;
    }
    // Must start and end with Ident, alternating with '.' puncts.
    let mut name = String::new();
    for (i, tt) in tokens.iter().enumerate() {
        if i % 2 == 0 {
            // Ident position.
            match tt {
                TokenTree::Ident(id) => {
                    if i > 0 {
                        name.push('.');
                    }
                    name.push_str(&id.to_string());
                }
                _ => return None,
            }
        } else {
            // Separator position: must be '.'.
            match tt {
                TokenTree::Punct(p) if p.as_char() == '.' => {}
                _ => return None,
            }
        }
    }
    if name.is_empty() { None } else { Some(name) }
}

/// Parse the argument group of a call into `LowerArg` list.
/// Returns `None` if any argument is not lowerable.
fn classify_args(g: &Group) -> Option<Vec<LowerArg>> {
    let inner: Vec<TokenTree> = g.stream().into_iter().collect();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    // Split by top-level commas.
    let segments = split_by_comma_owned(&inner);
    let mut args = Vec::with_capacity(segments.len());
    for seg in segments {
        args.push(classify_one_arg(&seg)?);
    }
    Some(args)
}

/// Split a flat `Vec<TokenTree>` by top-level commas.
fn split_by_comma_owned(tokens: &[TokenTree]) -> Vec<Vec<TokenTree>> {
    let mut result: Vec<Vec<TokenTree>> = Vec::new();
    let mut current: Vec<TokenTree> = Vec::new();
    for tt in tokens {
        if matches!(tt, TokenTree::Punct(p) if p.as_char() == ',') {
            result.push(current.clone());
            current.clear();
        } else {
            current.push(tt.clone());
        }
    }
    result.push(current);
    result
}

/// Classify one argument slot (possibly `name = atom`).
fn classify_one_arg(tokens: &[TokenTree]) -> Option<LowerArg> {
    // Skip leading/trailing whitespace by trimming empty.
    // Check for `name = value` shape:
    //   tokens[0] = Ident, tokens[1] = Punct('='), tokens[2..] = value
    if tokens.len() >= 3
        && let (TokenTree::Ident(name), TokenTree::Punct(eq)) = (&tokens[0], &tokens[1])
        && eq.as_char() == '='
    {
        let name_str = name.to_string();
        let value_tokens = &tokens[2..];
        let atom = classify_atom(value_tokens)?;
        return Some(LowerArg {
            name: Some(name_str),
            value: atom,
        });
    }

    // Positional arg.
    let atom = classify_atom(tokens)?;
    Some(LowerArg {
        name: None,
        value: atom,
    })
}

/// Classify a token slice as a lowerable atom.
fn classify_atom(tokens: &[TokenTree]) -> Option<LowerAtom> {
    match tokens {
        // String literal: `"hello"`
        [TokenTree::Literal(lit)] => {
            let s = lit.to_string();
            if s.starts_with('"') {
                // syn::LitStr can parse the Rust literal to get the unescaped string.
                let ts: TokenStream = std::iter::once(TokenTree::Literal(lit.clone())).collect();
                let lit_str: syn::LitStr = syn::parse2(ts).ok()?;
                return Some(LowerAtom::StringLit(lit_str.value()));
            }
            // Integer literal with `L` suffix: `42L` — ONE token in proc_macro2.
            // The string representation ends with 'L' after the digits.
            if s.ends_with('L') {
                let digits = &s[..s.len() - 1];
                // Must be a plain decimal integer (no dot/exponent).
                if !digits.contains('.')
                    && !digits.contains('e')
                    && !digits.contains('E')
                    && let Ok(v) = digits.parse::<i64>()
                    && v > i32::MIN as i64
                    && v <= i32::MAX as i64
                {
                    return Some(LowerAtom::IntLit(v as i32));
                }
                return None;
            }
            // Float literal: has `.` or `e`/`E`
            if s.contains('.') || s.to_lowercase().contains('e') {
                let v: f64 = s.parse().ok()?;
                return Some(LowerAtom::RealLit(v));
            }
            // Plain integer literal (no suffix) — R parses unsuffixed numeric
            // literals as DOUBLE (`typeof(42)` is "double"; only `42L` is
            // integer), so lower to a real to match the string path.
            if let Ok(v) = s.parse::<i64>() {
                // i64 → f64 is exact up to 2^53; anything a user writes as a
                // plain literal in R code is far below that, but guard anyway.
                let real = v as f64;
                if real as i64 == v {
                    return Some(LowerAtom::RealLit(real));
                }
            }
            // Out of range or unrecognised form → not lowerable.
            None
        }

        // Unary minus before a literal: `-42L`, `-1.5`
        [TokenTree::Punct(minus), rest @ ..] if minus.as_char() == '-' && !rest.is_empty() => {
            match classify_atom(rest)? {
                LowerAtom::IntLit(v) => {
                    // Negation must not produce NA (i32::MIN).
                    let neg = v.checked_neg()?;
                    Some(LowerAtom::IntLit(neg))
                }
                LowerAtom::RealLit(v) => Some(LowerAtom::RealLit(-v)),
                // -Inf is a common R idiom.
                LowerAtom::Inf => Some(LowerAtom::RealLit(f64::NEG_INFINITY)),
                _ => None,
            }
        }

        // Bare identifiers and symbolic constants
        [TokenTree::Ident(id)] => {
            Some(match id.to_string().as_str() {
                "TRUE" => LowerAtom::Bool(true),
                "FALSE" => LowerAtom::Bool(false),
                "NULL" => LowerAtom::Null,
                "NA" => LowerAtom::Na,
                "NA_integer_" => LowerAtom::NaInteger,
                "NA_real_" => LowerAtom::NaReal,
                "NA_character_" => LowerAtom::NaCharacter,
                "NA_complex_" => return None, // no constructor available
                "Inf" => LowerAtom::Inf,
                "NaN" => LowerAtom::NaN,
                sym => LowerAtom::Symbol(sym.to_string()),
            })
        }

        // Nested call: ident(args…) or pkg::fn(args…)
        _ => {
            let call = classify_call(tokens)?;
            Some(LowerAtom::Call(Box::new(call)))
        }
    }
}

// endregion

// region: Code emitter

/// Emit a `TokenStream` that builds + evaluates the lowered call.
///
/// The emitted code is:
/// ```rust,ignore
/// unsafe {
///     let __r_scope = ::miniextendr_api::gc_protect::ProtectScope::new();
///     let __r_arg_0 = __r_scope.protect_raw(<atom>);
///     ...
///     ::miniextendr_api::expression::RCall::new("ident")
///         .arg(__r_arg_0)
///         ...
///         .eval(#env_expr)
/// }
/// ```
///
/// For namespaced calls (`pkg::fn(args…)`), the emitted code first resolves
/// the function via `RCall::namespaced`, propagating errors early:
/// ```rust,ignore
/// unsafe {
///     let __r_scope = ::miniextendr_api::gc_protect::ProtectScope::new();
///     let __r_ns_call = ::miniextendr_api::expression::RCall::namespaced("pkg", "fn")?;
///     ...
///     __r_ns_call.arg(...).eval(#env_expr)
/// }
/// ```
fn emit_call(call: &LowerCall, env_expr: &TokenStream) -> TokenStream {
    // Collect all arg values up front, protecting each one.
    let mut scope_lets: Vec<TokenStream> = Vec::new();
    let mut rcall_chain: Vec<TokenStream> = Vec::new();

    for (i, arg) in call.args.iter().enumerate() {
        let var = quote::format_ident!("__r_arg_{}", i);
        let val_ts = emit_atom(&arg.value, &mut scope_lets);
        scope_lets.push(quote! {
            let #var = __r_scope.protect_raw(#val_ts);
        });
        if let Some(ref name) = arg.name {
            rcall_chain.push(quote! {
                .named_arg(#name, #var)
            });
        } else {
            rcall_chain.push(quote! {
                .arg(#var)
            });
        }
    }

    // Wrap the expansion in an immediately-invoked closure that returns
    // `Result<SEXP, String>`. This lets `?` inside the body (from
    // `RCall::namespaced()?` and nested calls) propagate to the closure's
    // return type rather than to the enclosing function (which may return `()`).
    //
    // All FFI calls live inside `unsafe {}` blocks within the closure body.
    match &call.fun {
        LowerFun::Simple(name) => {
            quote! {
                (|| -> ::std::result::Result<::miniextendr_api::SEXP, ::std::string::String> {
                    unsafe {
                        let __r_scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                        #(#scope_lets)*
                        ::miniextendr_api::expression::RCall::new(#name)
                            #(#rcall_chain)*
                            .eval(#env_expr)
                    }
                })()
            }
        }
        LowerFun::Namespaced { pkg, fun } => {
            quote! {
                (|| -> ::std::result::Result<::miniextendr_api::SEXP, ::std::string::String> {
                    let __r_ns_call = unsafe {
                        ::miniextendr_api::expression::RCall::namespaced(#pkg, #fun)?
                    };
                    unsafe {
                        let __r_scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                        #(#scope_lets)*
                        __r_ns_call
                            #(#rcall_chain)*
                            .eval(#env_expr)
                    }
                })()
            }
        }
    }
}

/// Emit the SEXP-producing expression for an atom.
///
/// Nested calls may themselves introduce new `protect_raw` lines; those are
/// pushed into `scope_lets`.
fn emit_atom(atom: &LowerAtom, scope_lets: &mut Vec<TokenStream>) -> TokenStream {
    match atom {
        LowerAtom::StringLit(s) => {
            quote! { ::miniextendr_api::SEXP::scalar_string_from_str(#s) }
        }
        LowerAtom::IntLit(v) => {
            let v = *v;
            quote! { ::miniextendr_api::SEXP::scalar_integer(#v) }
        }
        LowerAtom::RealLit(v) => {
            // f64 literals in quote must avoid NaN/Inf special forms.
            let bits = v.to_bits();
            quote! {
                ::miniextendr_api::SEXP::scalar_real(f64::from_bits(#bits))
            }
        }
        LowerAtom::Bool(b) => {
            quote! { ::miniextendr_api::SEXP::scalar_logical(#b) }
        }
        LowerAtom::Null => {
            quote! { ::miniextendr_api::SEXP::nil() }
        }
        LowerAtom::Na => {
            // R's bare NA is LOGICAL NA (`typeof(NA)` is "logical").
            quote! {
                ::miniextendr_api::SEXP::scalar_logical_raw(
                    ::miniextendr_api::altrep_traits::NA_LOGICAL
                )
            }
        }
        LowerAtom::NaInteger => {
            quote! { ::miniextendr_api::SEXP::scalar_integer(i32::MIN) }
        }
        LowerAtom::NaReal => {
            quote! {
                ::miniextendr_api::SEXP::scalar_real(
                    ::miniextendr_api::altrep_traits::NA_REAL
                )
            }
        }
        LowerAtom::NaCharacter => {
            quote! {
                ::miniextendr_api::SEXP::scalar_string(
                    ::miniextendr_api::SEXP::na_string()
                )
            }
        }
        LowerAtom::Inf => {
            let bits = f64::INFINITY.to_bits();
            quote! { ::miniextendr_api::SEXP::scalar_real(f64::from_bits(#bits)) }
        }
        LowerAtom::NaN => {
            // Use R's canonical NaN bit pattern.
            let bits = f64::NAN.to_bits();
            quote! { ::miniextendr_api::SEXP::scalar_real(f64::from_bits(#bits)) }
        }
        LowerAtom::Symbol(name) => {
            // Look up the symbol in the eval environment via Rf_install.
            // Symbols are never GC'd so no OwnedProtect needed.
            quote! {
                ::miniextendr_api::SEXP::symbol(#name)
            }
        }
        LowerAtom::Call(nested) => {
            // Nested calls: build inline using a fresh inner call structure.
            // We push the sub-scope-lets into the parent scope; all
            // protect_raw calls share the same __r_scope handle (LIFO order
            // is fine — ProtectScope drops all at once).
            emit_nested_call(nested, scope_lets)
        }
    }
}

/// Emit a nested call expression (returns the SEXP, adds protect_raw lines
/// to the parent scope via a temporary).
///
/// For namespaced nested calls (`pkg::fn(args…)`), the namespace resolution
/// is inlined via `RCall::namespaced(pkg, fun)?.build()`. The `?` propagates
/// resolution errors to the outer `Result<SEXP, String>` return.
fn emit_nested_call(call: &LowerCall, scope_lets: &mut Vec<TokenStream>) -> TokenStream {
    // Allocate vars for this nested call's args.
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let idx = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let mut arg_names: Vec<proc_macro2::Ident> = Vec::new();
    let mut arg_named: Vec<bool> = Vec::new();
    let mut arg_names_str: Vec<String> = Vec::new();

    for (i, arg) in call.args.iter().enumerate() {
        let var = quote::format_ident!("__r_nested_{}_{}", idx, i);
        let val_ts = emit_atom(&arg.value, scope_lets);
        scope_lets.push(quote! {
            let #var = __r_scope.protect_raw(#val_ts);
        });
        arg_names.push(var);
        arg_named.push(arg.name.is_some());
        arg_names_str.push(arg.name.clone().unwrap_or_default());
    }

    // Build the RCall builder init depending on whether this is a simple or
    // namespaced call. Namespaced resolution uses `?` on the Result.
    let builder_init: TokenStream = match &call.fun {
        LowerFun::Simple(name) => {
            quote! { ::miniextendr_api::expression::RCall::new(#name) }
        }
        LowerFun::Namespaced { pkg, fun } => {
            quote! { ::miniextendr_api::expression::RCall::namespaced(#pkg, #fun)? }
        }
    };

    // Build the full chain.
    let mut chain = builder_init;
    for (i, var) in arg_names.iter().enumerate() {
        if arg_named[i] {
            let name = &arg_names_str[i];
            chain = quote! { #chain.named_arg(#name, #var) };
        } else {
            chain = quote! { #chain.arg(#var) };
        }
    }

    // `.build()` returns an unprotected LANGSXP; we protect it in the caller's
    // scope_lets before passing to the outer `.arg()`.
    let call_var = quote::format_ident!("__r_call_sexp_{}", idx);
    scope_lets.push(quote! {
        let #call_var = __r_scope.protect_raw(#chain.build());
    });

    quote! { #call_var }
}

// endregion

// region: Unit tests for the classifier

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Vec<TokenTree> {
        let ts: TokenStream = s.parse().expect("failed to parse");
        ts.into_iter().collect()
    }

    fn is_lowerable(s: &str) -> bool {
        classify_call(&parse(s)).is_some()
    }

    // --- Positive cases: all should be lowerable ---

    #[test]
    fn simple_call_no_args() {
        assert!(is_lowerable("noop()"));
    }

    #[test]
    fn simple_call_integer_l_suffix() {
        assert!(is_lowerable("identity(42L)"));
    }

    #[test]
    fn simple_call_negative_integer() {
        assert!(is_lowerable("identity(-1L)"));
    }

    #[test]
    fn simple_call_string() {
        assert!(is_lowerable(r#"paste("hello", "world")"#));
    }

    #[test]
    fn simple_call_float() {
        assert!(is_lowerable("identity(3.14)"));
    }

    #[test]
    fn simple_call_true_false() {
        assert!(is_lowerable("identity(TRUE)"));
        assert!(is_lowerable("identity(FALSE)"));
    }

    #[test]
    fn simple_call_null() {
        assert!(is_lowerable("is.null(NULL)"));
    }

    #[test]
    fn simple_call_na_forms() {
        assert!(is_lowerable("identity(NA)"));
        assert!(is_lowerable("identity(NA_integer_)"));
        assert!(is_lowerable("identity(NA_real_)"));
        assert!(is_lowerable("identity(NA_character_)"));
    }

    #[test]
    fn simple_call_inf_nan() {
        assert!(is_lowerable("identity(Inf)"));
        assert!(is_lowerable("identity(NaN)"));
    }

    #[test]
    fn simple_call_neg_inf() {
        assert!(is_lowerable("identity(-Inf)"));
    }

    #[test]
    fn simple_call_bare_symbol() {
        assert!(is_lowerable("identity(x)"));
    }

    #[test]
    fn named_arg() {
        assert!(is_lowerable("seq(1L, 10L, by = 2L)"));
    }

    #[test]
    fn nested_call() {
        assert!(is_lowerable("c(1L, c(2L, 3L))"));
    }

    #[test]
    fn namespaced_call() {
        assert!(is_lowerable("base::sum(1L, 2L)"));
    }

    #[test]
    fn plain_integer_lowers_as_double() {
        // R parses unsuffixed numeric literals as double: typeof(42) is
        // "double". Only `42L` is INTSXP.
        let call = classify_call(&parse("identity(42)")).unwrap();
        assert!(matches!(call.args[0].value, LowerAtom::RealLit(v) if v == 42.0));
        let call = classify_call(&parse("identity(42L)")).unwrap();
        assert!(matches!(call.args[0].value, LowerAtom::IntLit(42)));
    }

    #[test]
    fn bare_na_is_logical_na() {
        // typeof(NA) is "logical", not integer.
        let call = classify_call(&parse("identity(NA)")).unwrap();
        assert!(matches!(call.args[0].value, LowerAtom::Na));
    }

    // --- Negative cases: all should fall back ---

    #[test]
    fn arithmetic_not_lowerable() {
        assert!(!is_lowerable("1L + 2L"));
    }

    #[test]
    fn assignment_not_lowerable() {
        assert!(!is_lowerable("x <- 5L"));
    }

    #[test]
    fn semicolon_sequence_not_lowerable() {
        // Two-statement sequence — only a call is lowerable.
        assert!(!is_lowerable("x <- 5L; x"));
    }

    #[test]
    fn indexing_not_lowerable() {
        assert!(!is_lowerable("x[1L]"));
    }

    #[test]
    fn dollar_not_lowerable() {
        assert!(!is_lowerable("x$y"));
    }

    #[test]
    fn bare_symbol_not_lowerable() {
        // A bare symbol at top level is not a call — fallback.
        assert!(!is_lowerable("x"));
    }

    #[test]
    fn na_complex_arg_not_lowerable() {
        // NA_complex_ has no constructor → not lowerable.
        assert!(!is_lowerable("identity(NA_complex_)"));
    }

    #[test]
    fn if_not_lowerable() {
        assert!(!is_lowerable("if (TRUE) 1L else 2L"));
    }

    #[test]
    fn empty_arg_slot_not_lowerable() {
        // `matrix(, 2, 2)` is valid R (missing arg) and passes the grammar
        // validator — it must fall back to the string path, never lower with
        // the empty slot silently dropped.
        assert!(!is_lowerable("matrix(, 2, 2)"));
        assert!(!is_lowerable("f(x,,y)"));
    }

    #[test]
    fn spaced_colons_not_namespaced() {
        // `pkg : : fn(x)` is an R parse error — must not lower to a working
        // namespaced call; fallback lets the string path reject it.
        assert!(!is_lowerable("pkg : : fn(1L)"));
    }
}

// endregion
