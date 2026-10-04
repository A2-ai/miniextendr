//! Hint for a file that fails to parse because a Rust `...` parameter is not the
//! last one (#1737).
//!
//! Rust's `...` parameter parses only in last position, so a file holding
//! `fn f(sources: ..., dosing_type: Strs)` fails `syn::parse_file` and no rule
//! checks it. [`hint`] tokenizes the source (which still works, and drops
//! comments) and names the cause and the `&Dots` spelling, for the lint to
//! append to its "failed to parse" warning.
//!
//! Mirror: `miniextendr-macros/src/misplaced_dots.rs` gives `#[miniextendr]` the
//! same error. Keep the detector and [`message`] in sync.

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};

/// A Rust `...` parameter followed by another parameter.
struct MisplacedDots {
    /// The parameter's name; `None` for unnamed dots (a bare `...` or `_: ...`).
    name: Option<String>,
    /// The three `.` tokens.
    dots: TokenStream,
}

/// The diagnostic for a `...` parameter that is not last, named `name` (`None` for unnamed dots).
///
/// Unnamed dots get `_dots`: `_: &Dots` would work on a function (its `_`
/// parameters are renamed) but not on a method, where the `&Dots` parameter
/// needs a plain name. No spelling of Rust `...` parses before another
/// parameter, so the hint names `&Dots`, never `_: ...`.
fn message(name: Option<&str>) -> String {
    let param = name.unwrap_or("_dots");
    format!(
        "Rust's `...` is only valid as the last parameter; write a dots parameter that is not \
         last as `{param}: &Dots` (`miniextendr_api::dots::Dots`), which is R's `...` at that \
         position. See https://a2-ai.github.io/miniextendr/manual/dots-typed-list/#formals-after"
    )
}

/// The hint for a source file that holds a `...` parameter with another parameter
/// after it: `line N: <message>`. `None` when it holds none or does not tokenize.
pub(crate) fn hint(src: &str) -> Option<String> {
    let found = find(src.parse().ok()?)?;
    let line = found
        .dots
        .into_iter()
        .next()
        .map_or(0, |dot| dot.span().start().line);
    Some(format!("line {line}: {}", message(found.name.as_deref())))
}

/// The first `...` parameter that another parameter follows, in any `fn` signature in `tokens`.
fn find(tokens: TokenStream) -> Option<MisplacedDots> {
    let tts: Vec<TokenTree> = tokens.into_iter().collect();
    for (i, tt) in tts.iter().enumerate() {
        match tt {
            TokenTree::Ident(kw) if kw == "fn" => {
                if let Some(TokenTree::Ident(_)) = tts.get(i + 1)
                    && let Some(params) = params_group(&tts[i + 2..])
                    && let Some(found) = in_params(params)
                {
                    return Some(found);
                }
            }
            // Module, impl and trait bodies hold more functions.
            TokenTree::Group(group) => {
                if let Some(found) = find(group.stream()) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

/// The parameter list after `fn name`: the first parenthesized group outside the
/// generics' angle brackets.
fn params_group(after_name: &[TokenTree]) -> Option<TokenStream> {
    let mut depth = 0usize;
    let mut after_minus = false;
    for tt in after_name {
        match tt {
            TokenTree::Group(group) if depth == 0 => {
                return (group.delimiter() == Delimiter::Parenthesis).then(|| group.stream());
            }
            TokenTree::Punct(p) if p.as_char() == '<' => depth += 1,
            // The `>` of `->` (in a `Fn() -> T` bound) closes nothing.
            TokenTree::Punct(p) if p.as_char() == '>' && !after_minus => {
                depth = depth.checked_sub(1)?;
            }
            _ if depth == 0 => return None,
            _ => {}
        }
        after_minus = matches!(tt, TokenTree::Punct(p) if p.as_char() == '-' && p.spacing() == Spacing::Joint);
    }
    None
}

/// A `...` parameter in `params` with another parameter after it (a trailing comma is fine).
fn in_params(params: TokenStream) -> Option<MisplacedDots> {
    let tts: Vec<TokenTree> = params.into_iter().collect();
    (0..tts.len()).find_map(|i| {
        let dots = dots_at(&tts, i)?;
        let comma_next = matches!(tts.get(i + 3), Some(TokenTree::Punct(p)) if p.as_char() == ',');
        if !comma_next || tts.len() <= i + 4 {
            return None;
        }
        let name = match (
            i.checked_sub(2).map(|j| &tts[j]),
            i.checked_sub(1).map(|j| &tts[j]),
        ) {
            (Some(TokenTree::Ident(name)), Some(TokenTree::Punct(colon)))
                if colon.as_char() == ':' && name != "_" =>
            {
                Some(name.to_string())
            }
            _ => None,
        };
        Some(MisplacedDots { name, dots })
    })
}

/// The three `.` tokens of a `...` that starts at `tts[i]`.
fn dots_at(tts: &[TokenTree], i: usize) -> Option<TokenStream> {
    let three = tts.get(i..i + 3)?;
    // The first two dots are joint (`...`, not `. . .`); the last one's spacing
    // depends on what follows it.
    let dot = |tt: &TokenTree, joint: bool| matches!(tt, TokenTree::Punct(p) if p.as_char() == '.' && (!joint || p.spacing() == Spacing::Joint));
    (dot(&three[0], true) && dot(&three[1], true) && dot(&three[2], false))
        .then(|| three.iter().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_names_the_parameter_and_line() {
        let src = "use x::Strs;\n\n#[miniextendr]\npub fn f(sources: ..., dosing_type: Strs) {}\n";
        let hint = hint(src).expect("hint");
        assert!(hint.starts_with("line 4: Rust's `...` is only valid as the last parameter"));
        assert!(hint.contains("`sources: &Dots`"));
    }

    #[test]
    fn bare_dots_get_a_placeholder_name() {
        let hint = hint("mod m { fn f(x: i32, ..., y: i32) {} }").expect("hint");
        assert!(hint.contains("`_dots: &Dots`"));
    }

    /// `_: ...` is unnamed dots too: `_dots: &Dots` works on a method, where
    /// `_: &Dots` needs a plain name.
    #[test]
    fn wild_dots_get_the_placeholder_name() {
        let hint = hint("impl T { fn m(&self, _: ..., n: i32) {} }").expect("hint");
        assert!(hint.contains("`_dots: &Dots`"), "got: {hint}");
        assert!(!hint.contains("`_: &Dots`"), "got: {hint}");
    }

    #[test]
    fn method_in_an_impl_body() {
        assert!(hint("impl T { fn m(&self, rest: ..., n: i32) {} }").is_some());
    }

    #[test]
    fn last_dots_and_comments_get_no_hint() {
        assert_eq!(hint("fn f(x: i32, rest: ...) {}"), None);
        assert_eq!(hint("fn f(x: i32, rest: ...,) {}"), None);
        assert_eq!(hint("fn f(x: i32, _: ...) {}"), None);
        assert_eq!(hint("// fn f(rest: ..., x: i32) {}\nfn g() {}"), None);
        assert_eq!(hint("/// fn f(rest: ..., x: i32) {}\nfn g() {}"), None);
    }

    #[test]
    fn generics_before_the_parameters() {
        assert!(hint("fn f<'a, F: Fn(i32) -> i32>(rest: ..., f: &'a F) {}").is_some());
    }
}
