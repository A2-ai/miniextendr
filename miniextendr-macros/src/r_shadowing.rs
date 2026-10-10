//! Calls to base functions that a formal of the same name would shadow.
//!
//! Generated R wrappers call base functions by their bare names: the
//! precondition guards call `isTRUE()` and `length()`, a choice formal's
//! default is `c("a", "b")`, a `Missing<T>` argument is forwarded as
//! `if (missing(x)) quote(expr=) else x`, the check after `.Call()` calls
//! `inherits()` and `attr()`, and dots are forwarded as `list(...)` (or, for
//! `LazyDots`, the frame as `environment()`). R looks
//! up the name of a called function starting in the wrapper's own frame, so a
//! formal with the same name comes first: forcing it fails when the argument
//! was omitted, and a function passed there is called in place of the base
//! function.
//!
//! The rule: inside a generated wrapper, a call to one of these base functions
//! always means the base function, even when a formal has that name.
//! [`qualify_shadowed_calls`] writes `base::length(...)` for each call whose
//! name is also a formal of the wrapper, and leaves every other call alone. It
//! runs where every wrapper is finalized
//! ([`r_wrapper_raw_literal`](crate::r_wrapper_raw_literal)), so no emitter
//! can leave it out, and the author R code spliced into a wrapper (`r_entry`,
//! `r_on_exit`, `r_post_checks`, parameter defaults) is qualified the same way.
//!
//! Only undotted names can collide. A formal is a Rust parameter name without
//! its leading underscores and without `r#`, and a Rust identifier cannot
//! contain `.`, so no formal is named `is.character`, `match.call`, `on.exit`,
//! `.Call` or `.miniextendr_arg_error`. Those calls are never qualified.
//!
//! Only the colliding calls are qualified because `::` costs time on every
//! call: the byte compiler compiles `base::f(x)` as a call to the `::`
//! primitive instead of inlining `f`. On a byte-compiled one-argument wrapper,
//! qualifying the guards added about 200 ns per call (+19%), and qualifying
//! every call about 475 ns (+44%). When no formal collides, the text is
//! returned unchanged.
//!
//! Two generators do not finalize through here: the sidecar accessors of
//! `#[derive(ExternalPtr)]` and the functions `#[derive(Vctrs)]` emits. Their
//! formals are fixed (`x`, `value`, `to`, `...`), and none of them names a base
//! function.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::ops::Range;

/// The undotted base functions that generated wrapper text calls by their bare
/// names.
///
/// A call to one of these is qualified when a formal of the wrapper has the
/// same name. The `generated_callees_are_listed` test checks the generated
/// wrappers in the macro snapshots and the tracked cross-package wrapper
/// files, and fails on an undotted callee that is missing here.
pub(crate) const BASE_CALLEES: &[&str] = &[
    "NextMethod",
    "UseMethod",
    "all",
    "anyNA",
    "attr",
    "c",
    "class",
    "double",
    "emptyenv",
    "environment",
    "exists",
    "get0",
    "getOption",
    "grep",
    "identical",
    "inherits",
    "invisible",
    "isTRUE",
    "lapply",
    "length",
    "list",
    "local",
    "ls",
    "match",
    "missing",
    "names",
    // The argument count an `NArgs` parameter receives (#1860).
    "nargs",
    "paste0",
    // The list lookup of the S3 field methods (#1891).
    "pmatch",
    "quote",
    "return",
    "seq_along",
    "simpleWarning",
    "sprintf",
    "standardGeneric",
    "stop",
    "structure",
    "substitute",
    "switch",
    "topenv",
    "trunc",
    "typeof",
    "unclass",
    "unique",
    "vector",
    "warning",
];

/// The R formals of every wrapper built from `signatures`: the union of each
/// signature's [`r_formal_names`](crate::r_wrapper_builder::r_formal_names).
///
/// An impl block or a trait impl is finalized as one text, so its formals are
/// pooled across its methods. A method whose sibling has a colliding formal is
/// then qualified too, which calls the same function.
pub(crate) fn formal_names<'a>(
    signatures: impl IntoIterator<Item = &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>>,
) -> BTreeSet<String> {
    signatures
        .into_iter()
        .flat_map(|inputs| crate::r_wrapper_builder::r_formal_names(inputs).map(|(name, _)| name))
        .collect()
}

/// Write `base::` before every call in `text` to a [`BASE_CALLEES`] function
/// that shares its name with one of `formals`.
///
/// A call is a name followed by optional spaces and `(`. Strings, raw strings,
/// comments (roxygen blocks included) and backtick names are left as they are,
/// as is a name after `::`, `:::`, `$` or `@`. A replacement call
/// (`class(x) <- v`) is qualified too: R resolves `base::class(x) <- v`
/// through `base::"class<-"`. The match_arg choices placeholder, which the
/// wrapper writer replaces with `c(<choices>)`, counts
/// as a call to `c`.
///
/// Returns `text` borrowed when nothing is qualified.
pub(crate) fn qualify_shadowed_calls<'t>(
    text: &'t str,
    formals: &BTreeSet<String>,
) -> Cow<'t, str> {
    let hits: Vec<&str> = BASE_CALLEES
        .iter()
        .copied()
        .filter(|name| formals.contains(*name))
        .collect();
    if hits.is_empty() {
        return Cow::Borrowed(text);
    }
    let mut out = String::new();
    let mut copied = 0;
    for ident in identifiers(text) {
        let name = &text[ident.clone()];
        let callee = if name.starts_with(crate::match_arg_keys::CHOICES_PLACEHOLDER_PREFIX) {
            "c"
        } else if is_call(text, ident.end) {
            name
        } else {
            continue;
        };
        if hits.contains(&callee) && !is_accessed(text, ident.start) {
            out.push_str(&text[copied..ident.start]);
            out.push_str("base::");
            copied = ident.start;
        }
    }
    if out.is_empty() {
        return Cow::Borrowed(text);
    }
    out.push_str(&text[copied..]);
    Cow::Owned(out)
}

// region: lexer

/// Byte ranges of the R identifiers in `text`, skipping strings (`"..."`,
/// `'...'`), raw strings (`r"(...)"`, `R'[...]'`, `r"-{...}-"`), comments,
/// backtick names, `%op%` operators and numbers.
fn identifiers(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(c) = text[i..].chars().next() {
        match c {
            '"' | '\'' | '`' => i = quoted_end(text, i, c),
            '#' => i = text[i..].find('\n').map_or(text.len(), |n| i + n),
            '%' => i = infix_end(text, i),
            c if c.is_ascii_digit() => i = ident_end(text, i),
            c if c.is_alphabetic() || c == '.' => {
                let end = ident_end(text, i);
                if matches!(&text[i..end], "r" | "R")
                    && let Some(raw_end) = raw_string_end(text, end)
                {
                    i = raw_end;
                    continue;
                }
                out.push(i..end);
                i = end;
            }
            c => i += c.len_utf8(),
        }
    }
    out
}

/// End of the identifier (or number) starting at `start`.
fn ident_end(text: &str, start: usize) -> usize {
    text[start..]
        .char_indices()
        .find(|&(_, c)| !(c.is_alphanumeric() || c == '.' || c == '_'))
        .map_or(text.len(), |(n, _)| start + n)
}

/// End of the string or backtick name opened by `quote` at `start`, after its
/// closing quote. A backslash escapes the next character.
fn quoted_end(text: &str, start: usize, quote: char) -> usize {
    let mut chars = text[start + 1..].char_indices();
    while let Some((n, c)) = chars.next() {
        if c == '\\' {
            chars.next();
        } else if c == quote {
            return start + 1 + n + c.len_utf8();
        }
    }
    text.len()
}

/// End of the `%op%` operator starting at `start`, or just past the `%` when
/// the line has no closing `%`.
fn infix_end(text: &str, start: usize) -> usize {
    let rest = &text[start + 1..];
    let line = &rest[..rest.find('\n').unwrap_or(rest.len())];
    line.find('%').map_or(start + 1, |n| start + 1 + n + 1)
}

/// End of the raw string whose quote is at `quote_at`, right after an `r` or
/// `R`: the quote, any dashes, and an opening `(`, `[` or `{`, closed by the
/// matching bracket, the same dashes and the same quote. `None` when the
/// quote does not open a raw string.
fn raw_string_end(text: &str, quote_at: usize) -> Option<usize> {
    let rest = &text[quote_at..];
    let quote = rest.chars().next().filter(|c| matches!(c, '"' | '\''))?;
    let dashes = rest[1..].chars().take_while(|&c| c == '-').count();
    let close = match rest[1 + dashes..].chars().next()? {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => return None,
    };
    let terminator = format!("{close}{}{quote}", "-".repeat(dashes));
    let body = 1 + dashes + 1;
    Some(
        rest[body..]
            .find(&terminator)
            .map_or(text.len(), |n| quote_at + body + n + terminator.len()),
    )
}

/// Whether the identifier ending at `end` is called: followed by optional
/// spaces and `(`.
fn is_call(text: &str, end: usize) -> bool {
    text[end..].trim_start_matches([' ', '\t']).starts_with('(')
}

/// Whether the identifier starting at `start` follows `::`, `:::`, `$` or `@`,
/// so it names a namespace export or a member, not a function in scope.
fn is_accessed(text: &str, start: usize) -> bool {
    let before = text[..start].trim_end();
    before.ends_with("::") || before.ends_with('$') || before.ends_with('@')
}

// endregion

#[cfg(test)]
mod tests;
