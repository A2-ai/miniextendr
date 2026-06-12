//! Conservative R-grammar validation for the `r!` proc-macro.
//!
//! This module implements a **reject-only-known-bad** strategy: walk the
//! `proc_macro2::TokenStream` and emit a `syn::Error` for constructs that R's
//! parser is guaranteed to reject.  Anything that cannot be confidently
//! classified as an error is accepted silently.
//!
//! # Non-goals
//!
//! A complete R grammar over Rust tokens is not achievable:
//! - Single-quoted strings (`'hello'`) and backtick-quoted names (\`foo\`)
//!   already die at the Rust lexer — nothing to validate.
//! - `%op%` tokenises as `%`, ident, `%` — looks like two leading `%` ops;
//!   we accept it rather than risk false positives.
//! - R formulas (`~`), `?`, unary operators and other niche constructs may
//!   look syntactically ambiguous — we accept those too.
//!
//! # Accepted forms (must never be rejected)
//!
//! - `;`-statement sequences (`a <- 1L; b <- 2L; a + b`)
//! - `<-` assignment (tokenises as `<` + `-` joint punct)
//! - `<<-` assignment (tokenises as `<` + `<` + `-`)
//! - `->` assignment (tokenises as `-` + `>` joint punct)
//! - `%in%`, `%*%`, and all other `%op%` operators
//! - Empty groups in `[`-index position (`x[,1]`, `x[1,]`)
//! - `~` formulas
//! - `\(x) x+1` lambda syntax (R 4.1+; `\` alone doesn't survive Rust
//!   lexing unless inside a string literal, so no R-side validation needed)

use proc_macro2::{Delimiter, Group, Punct, Spacing, Span, TokenStream, TokenTree};
use syn::Error;

/// Returns `Ok(())` if the token stream passes the conservative validator,
/// or an `Err` spanned to the first problematic token pair/group.
pub(crate) fn validate(tokens: &TokenStream) -> Result<(), Error> {
    let flat: Vec<TokenTree> = flatten_top_level(tokens);
    validate_sequence(&flat, SequenceContext::TopLevel)
}

// region: Sequence-level checks

#[derive(Clone, Copy, PartialEq, Eq)]
enum SequenceContext {
    /// Top-level or `;`-delimited statement — empty call argument check
    /// applies to `(` groups here.
    TopLevel,
    /// Inside a `[` or `[[` group — trailing-comma empties are valid R
    /// (`x[,1]`) so suppress the empty-call-arg diagnostic.
    IndexGroup,
}

fn validate_sequence(seq: &[TokenTree], ctx: SequenceContext) -> Result<(), Error> {
    if seq.is_empty() {
        return Ok(());
    }

    // Check for trailing binary operator at end of sequence.
    check_trailing_binary_op(seq)?;

    // Check consecutive binary operators (neither unary-capable).
    check_consecutive_binary_ops(seq)?;

    // Walk each token; recurse into groups.
    let mut i = 0;
    while i < seq.len() {
        let tt = &seq[i];
        if let TokenTree::Group(g) = tt {
            validate_group(g, ctx, seq, i)?;
        }
        i += 1;
    }

    Ok(())
}

// endregion

// region: Group-level checks

fn validate_group(
    g: &Group,
    _outer_ctx: SequenceContext,
    outer_seq: &[TokenTree],
    pos: usize,
) -> Result<(), Error> {
    let inner: Vec<TokenTree> = flatten_top_level(&g.stream());

    match g.delimiter() {
        Delimiter::Parenthesis => {
            // Check whether this parenthesized group is a function call.
            // It is a call if the preceding token is an ident or a closing
            // group (i.e. `f(...)`, `f(...)()`, `obj$field(...)`).
            let is_call = pos > 0 && {
                match &outer_seq[pos - 1] {
                    TokenTree::Ident(_) => true,
                    TokenTree::Group(prev) => matches!(
                        prev.delimiter(),
                        Delimiter::Parenthesis | Delimiter::Bracket
                    ),
                    _ => false,
                }
            };

            if is_call {
                // Validate call arguments: no empty non-trailing position
                // (e.g. `f(, x)` or `f(x,,y)` but `f()` and `f(x,)` are ok).
                check_empty_call_arg(&inner, g.span())?;
            } else {
                // Standalone parenthesised expression — must not be empty after
                // a binary operator (`x + ()` is an error).
                if inner.is_empty()
                    && pos > 0
                    && let Some(op_span) = preceding_binary_op_span(outer_seq, pos - 1)
                {
                    return Err(Error::new(
                        op_span,
                        "R syntax error: binary operator followed by empty parentheses `()` — \
                         expected an operand",
                    ));
                }
            }

            // Recurse into call arguments split by top-level commas.
            let sub_ctx = SequenceContext::TopLevel;
            for arg in split_by_comma(&inner) {
                validate_sequence(arg, sub_ctx)?;
            }

            // Also validate control-flow keywords before this group.
            validate_control_flow_keyword(outer_seq, pos, g.span())?;
        }

        Delimiter::Bracket => {
            // `[` groups: empty comma slots are valid R (matrix/dataframe
            // indexing) — don't fire empty-arg diagnostic.
            let sub_ctx = SequenceContext::IndexGroup;
            for arg in split_by_comma(&inner) {
                validate_sequence(arg, sub_ctx)?;
            }
        }

        Delimiter::Brace => {
            // `{` blocks: validate each `;`-separated or newline-separated
            // statement.  We treat commas and semicolons both as separators
            // here for simplicity (R allows both in limited contexts).
            let sub_ctx = SequenceContext::TopLevel;
            for stmt in split_by_semicolon(&inner) {
                if !stmt.is_empty() {
                    validate_sequence(stmt, sub_ctx)?;
                }
            }
        }

        Delimiter::None => {
            validate_sequence(&inner, _outer_ctx)?;
        }
    }

    Ok(())
}

// endregion

// region: Control-flow keyword validation

/// Validates that control-flow keywords (`if`, `while`, `for`, `function`, `repeat`)
/// are followed by the correct next token.
fn validate_control_flow_keyword(
    seq: &[TokenTree],
    paren_pos: usize,
    paren_span: Span,
) -> Result<(), Error> {
    if paren_pos == 0 {
        return Ok(());
    }
    // The token immediately before the `(` group.
    let kw = match &seq[paren_pos - 1] {
        TokenTree::Ident(id) => id.to_string(),
        _ => return Ok(()),
    };

    match kw.as_str() {
        "if" | "while" => {
            // `if (cond)` / `while (cond)` — must be followed by at least one
            // token (the body), and the parenthesised group must be non-empty.
            let g = match &seq[paren_pos] {
                TokenTree::Group(g) => g,
                _ => return Ok(()),
            };
            let inner: Vec<TokenTree> = flatten_top_level(&g.stream());
            if inner.is_empty() {
                return Err(Error::new(
                    paren_span,
                    format!(
                        "R syntax error: `{kw}` condition is empty — \
                         `{kw} ()` is not valid R"
                    ),
                ));
            }
            // Must be followed by a body token.
            if paren_pos + 1 >= seq.len() {
                return Err(Error::new(
                    paren_span,
                    format!(
                        "R syntax error: `{kw} (...)` has no body — \
                         expected a consequent expression after the condition"
                    ),
                ));
            }
        }
        "for" => {
            // `for (ident in seq)` — the parenthesised group must contain
            // exactly: ident `in` tokens.
            let g = match &seq[paren_pos] {
                TokenTree::Group(g) => g,
                _ => return Ok(()),
            };
            let inner: Vec<TokenTree> = flatten_top_level(&g.stream());
            if inner.is_empty() {
                return Err(Error::new(
                    paren_span,
                    "R syntax error: `for` loop variable list is empty — \
                     expected `for (ident in seq)`",
                ));
            }
            // Check for presence of `in` keyword.
            let has_in = inner
                .iter()
                .any(|t| matches!(t, TokenTree::Ident(id) if id == "in"));
            if !has_in {
                return Err(Error::new(
                    paren_span,
                    "R syntax error: `for` loop missing `in` — \
                     expected `for (ident in seq)`",
                ));
            }
            // Must have a body after.
            if paren_pos + 1 >= seq.len() {
                return Err(Error::new(
                    paren_span,
                    "R syntax error: `for (ident in seq)` has no body — \
                     expected a loop body after the `for` header",
                ));
            }
        }
        "function" => {
            // `function(args)` — the `(` is the argument list; already validated as non-call.
            // No additional rule needed beyond that the group exists.
        }
        _ => {}
    }

    Ok(())
}

// endregion

// region: Trailing binary operator check

/// Returns an error if the sequence ends with a binary operator.
///
/// Token-pair awareness: `<` + `-` joint = `<-` (assignment, not a trailing `<`);
/// `<` + `<` + `-` = `<<-` (also assignment); `-` + `>` joint = `->` (assignment).
fn check_trailing_binary_op(seq: &[TokenTree]) -> Result<(), Error> {
    if seq.is_empty() {
        return Ok(());
    }

    // Find the "logical last token", ignoring `;` (statement separator is fine).
    let logical_end = seq
        .iter()
        .rposition(|t| !is_semicolon(t))
        .unwrap_or(seq.len().saturating_sub(1));

    // For a trailing operator to be an error, the token at logical_end must be
    // a "pure binary" operator (i.e. not unary-capable), and must NOT be the
    // first component of a multi-token assignment form.

    let span = match &seq[logical_end] {
        TokenTree::Punct(p) => {
            let ch = p.as_char();
            // Check for assignment forms ending here that are actually valid:
            //   `<-`: ch=`-`, preceded by `<`
            //   `<<-`: ch=`-`, preceded by two `<`
            //   `->`: ch=`>`, preceded by `-`
            //   `<<=`: well-formed? Not R; skip.
            if ch == '-'
                && logical_end > 0
                && punct_char_at(seq, logical_end - 1) == Some('<')
            {
                return Ok(()); // `<-` assignment
            }
            if ch == '>'
                && logical_end > 0
                && punct_char_at(seq, logical_end - 1) == Some('-')
            {
                return Ok(()); // `->` assignment
            }
            if is_pure_binary_op(p) {
                p.span()
            } else {
                return Ok(());
            }
        }
        _ => return Ok(()),
    };

    Err(Error::new(
        span,
        "R syntax error: expression ends with a binary operator — \
         expected a right-hand operand after the operator",
    ))
}

// endregion

// region: Consecutive binary operators check

/// Returns an error if two consecutive pure-binary operators appear where
/// neither can be unary (R disallows things like `x * + y`... actually
/// `+` and `-` ARE unary so we must be careful).
///
/// Rule: report an error only when BOTH operators are in the non-unary set
/// (`*`, `/`, `^`, `==`, `!=`, `<=`, `>=`, `&&`, `||`, `%`, `@`, `$`,
/// `~`, `?` are only flagged when clearly doubled like `* *` — actually
/// we keep this very conservative: flag only `*`, `/` doubled, since
/// those are the clearest cases, and accept `+ +`, `- -`, etc. which
/// can be chained unary in R.
///
/// To stay conservative we only flag the case of two identical non-unary ops
/// that can't be chained. For now we keep this simple and flag `* *` and
/// `/ /` explicitly (the easiest false-negative-free cases).
///
/// Actually the plan says: "two consecutive binary operators where neither is
/// unary-capable (`+ - ! ~ ?` are unary)". Let's implement that conservatively.
fn check_consecutive_binary_ops(seq: &[TokenTree]) -> Result<(), Error> {
    // Build a simplified view: only keep non-whitespace punct tokens and check
    // for pairs of non-unary binary operators.
    //
    // We are very conservative: only flag when we see two *definitely*
    // non-unary operators in a row, and neither is part of a multi-char
    // operator form (`<=`, `>=`, `==`, `!=`, `->`, `<-`, `<<-`, `**`).

    let puncts: Vec<(usize, char, Spacing, Span)> = seq
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            if let TokenTree::Punct(p) = t {
                Some((i, p.as_char(), p.spacing(), p.span()))
            } else {
                None
            }
        })
        .collect();

    // Look for adjacent pure-binary-non-unary punct pairs where both are
    // at top-level (not separated by non-punct tokens).
    for w in puncts.windows(2) {
        let (i0, c0, spacing0, span0) = w[0];
        let (i1, c1, _spacing1, span1) = w[1];

        // Adjacent: no non-punct tokens between them (i1 == i0 + 1) AND
        // the first token's spacing is Alone (no joint continuation).
        if i1 != i0 + 1 {
            continue;
        }

        // Skip multi-char compound operators that are valid R:
        //   `<-` (`<` then `-` joint), `->` (`-` then `>` joint),
        //   `<<-`, `<=`, `>=`, `==`, `!=`, `**` (R doesn't have this but
        //   still skip to avoid false positives), `&&`, `||`.
        // We skip ALL joint pairs conservatively.
        if spacing0 == Spacing::Joint {
            continue;
        }

        // Both tokens must be purely non-unary binary operators to flag.
        // Unary-capable: `+`, `-`, `!`, `~`, `?`.
        let non_unary_binary = |c: char| matches!(c, '*' | '/' | '^' | '%' | '@' | '$');

        if non_unary_binary(c0) && non_unary_binary(c1) {
            // Double `*` (e.g. `x ** y`) or `/ /` etc. — clearly invalid R.
            let _ = span1; // used in error below
            return Err(Error::new(
                span0,
                format!(
                    "R syntax error: consecutive binary operators `{c0}` and `{c1}` — \
                     expected an operand between them"
                ),
            ));
        }
    }

    Ok(())
}

// endregion

// region: Empty call-argument check

/// Checks that a `f(...)` call argument list has no empty non-trailing slots.
///
/// - `f()` — ok (single empty group).
/// - `f(, x)` — error (leading empty slot).
/// - `f(x,,y)` — error (middle empty slot).
/// - `f(x,)` — ok (trailing empty is fine in R: named calls sometimes end `,`).
fn check_empty_call_arg(args: &[TokenTree], call_span: Span) -> Result<(), Error> {
    let segments: Vec<&[TokenTree]> = split_by_comma(args);

    // A single empty segment is `f()` — valid.
    if segments.len() == 1 {
        return Ok(());
    }

    // Check every segment except the last one (trailing comma is ok).
    for (i, seg) in segments.iter().enumerate() {
        let is_last = i == segments.len() - 1;
        if is_last {
            break; // trailing empty ok
        }
        if seg.is_empty() {
            return Err(Error::new(
                call_span,
                "R syntax error: empty argument in function call — \
                 `f(, x)` / `f(x,,y)` are not valid R (use `f(NULL, x)` or \
                 named arguments if you want a placeholder). \
                 Note: `x[, 1]` (index subscript) is valid; use `[[` or `[` groups.",
            ));
        }
    }

    Ok(())
}

// endregion

// region: Helpers

fn flatten_top_level(ts: &TokenStream) -> Vec<TokenTree> {
    ts.clone().into_iter().collect()
}

fn split_by_comma(seq: &[TokenTree]) -> Vec<&[TokenTree]> {
    let mut parts: Vec<&[TokenTree]> = Vec::new();
    let mut start = 0;
    for (i, tt) in seq.iter().enumerate() {
        if is_comma(tt) {
            parts.push(&seq[start..i]);
            start = i + 1;
        }
    }
    parts.push(&seq[start..]);
    parts
}

fn split_by_semicolon(seq: &[TokenTree]) -> Vec<&[TokenTree]> {
    let mut parts: Vec<&[TokenTree]> = Vec::new();
    let mut start = 0;
    for (i, tt) in seq.iter().enumerate() {
        if is_semicolon(tt) {
            parts.push(&seq[start..i]);
            start = i + 1;
        }
    }
    parts.push(&seq[start..]);
    parts
}

fn is_comma(tt: &TokenTree) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == ',')
}

fn is_semicolon(tt: &TokenTree) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == ';')
}

/// Returns `true` for operators that are clearly binary and NOT unary-capable
/// in R.  `+`, `-`, `!`, `~`, `?` are unary-capable so we return `false`.
fn is_pure_binary_op(p: &Punct) -> bool {
    // We only flag unambiguously binary operators at end-of-sequence.
    // Conservatively keep this list small.
    matches!(p.as_char(), '*' | '/' | '^' | '%' | '@' | '$')
}

fn punct_char_at(seq: &[TokenTree], idx: usize) -> Option<char> {
    match &seq[idx] {
        TokenTree::Punct(p) => Some(p.as_char()),
        _ => None,
    }
}

/// Returns the span of a binary operator token at `pos` if it is a binary op.
/// Used to anchor the "empty paren after binary op" diagnostic.
fn preceding_binary_op_span(seq: &[TokenTree], pos: usize) -> Option<Span> {
    let p = match &seq[pos] {
        TokenTree::Punct(p) => p,
        _ => return None,
    };
    // Full set of binary operators (include `+` and `-` here since `x + ()` is an error
    // even though `+` can be unary — it's not unary when there's already a left operand,
    // but we'd need a parser to know that). Stay conservative: only pure-binary.
    if is_pure_binary_op(p) {
        Some(p.span())
    } else {
        None
    }
}

// endregion

// region: Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;

    fn ok(src: &str) {
        let ts: TokenStream = src.parse().expect("not a valid Rust token stream");
        assert!(
            validate(&ts).is_ok(),
            "expected Ok for {src:?}, got: {:?}",
            validate(&ts)
        );
    }

    fn err(src: &str) {
        let ts: TokenStream = src.parse().expect("not a valid Rust token stream");
        assert!(validate(&ts).is_err(), "expected Err for {src:?}, got Ok");
    }

    // region: Trailing binary operators (should be rejected)

    #[test]
    fn trailing_multiply() {
        err("x *");
    }

    #[test]
    fn trailing_divide() {
        err("x /");
    }

    #[test]
    fn trailing_caret() {
        err("x ^");
    }

    // region: Things that should pass

    #[test]
    fn arithmetic_expression() {
        ok("1L + 2L");
    }

    #[test]
    fn assignment_arrow() {
        // `<-` tokenises as `<` then `-` in proc_macro2 (joint)
        ok(".x <- 41L + 1L");
    }

    #[test]
    fn forward_arrow_assignment() {
        ok("41L + 1L -> .y");
    }

    #[test]
    fn semicolon_sequence() {
        ok("a <- 7L ; a * 6L");
    }

    #[test]
    fn multi_statement_with_env_form() {
        // The env form is stripped before validation, but the body `local_val <- 7L; local_val * 6L`
        // should be valid.
        ok("local_val <- 7L ; local_val * 6L");
    }

    #[test]
    fn trailing_semicolon_ok() {
        ok("x <- 1L ;");
    }

    #[test]
    fn nchar_call() {
        ok(r#"nchar("hello")"#);
    }

    #[test]
    fn index_with_empty_first_arg() {
        // x[,1] — valid R matrix indexing; second comma-slot is non-empty
        // This is a bracket group, not a function call.
        ok("x[,1]");
    }

    #[test]
    fn empty_function_call() {
        // f() is valid
        ok("f()");
    }

    #[test]
    fn percent_in_percent() {
        // %in% tokenises as `%` ident `%` — must pass
        ok("x % in % y");
    }

    #[test]
    fn tilde_formula() {
        ok("y ~ x");
    }

    #[test]
    fn logical_operators() {
        ok("x && y");
        ok("x || y");
    }

    // region: Control flow

    #[test]
    fn if_else_valid() {
        ok("if (x > 0) x else - x");
    }

    #[test]
    fn while_valid() {
        ok("while (i < 10) i <- i + 1L");
    }

    #[test]
    fn for_valid() {
        ok("for (i in 1:10) print(i)");
    }

    // region: Control flow errors

    #[test]
    fn if_empty_condition() {
        err("if () x");
    }

    #[test]
    fn if_no_body() {
        err("if (x)");
    }

    #[test]
    fn for_missing_in() {
        err("for (x) {}");
    }

    #[test]
    fn for_empty() {
        err("for () {}");
    }

    // region: Consecutive non-unary binary operators

    #[test]
    fn double_star() {
        err("x * * y");
    }

    #[test]
    fn double_slash() {
        err("x / / y");
    }

    // region: Empty call args

    #[test]
    fn leading_empty_call_arg() {
        err("f(, x)");
    }

    #[test]
    fn middle_empty_call_arg() {
        err("f(x,,y)");
    }

    #[test]
    fn trailing_empty_call_arg_ok() {
        // R allows `f(x,)` in some contexts (though uncommon)
        ok("f(x,)");
    }

    #[test]
    fn empty_paren_after_binary_at() {
        // `x @ ()` — `@` is a pure binary operator
        err("x @ ()");
    }
}

// endregion
