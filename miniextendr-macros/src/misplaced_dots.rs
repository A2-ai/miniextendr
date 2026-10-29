//! Diagnostic for a Rust `...` parameter that is not the last one (#1737).
//!
//! Rust's `...` parameter parses only in last position, so
//! `fn f(sources: ..., dosing_type: Strs)` is not a function to `syn`. The item
//! fails `ItemFn` (and, as a method, `ItemImpl` / `ItemTrait`), which would leave
//! `#[miniextendr]` with its struct/enum message. [`find`] recognises that shape
//! at token level so the fallback can name the cause and the `&Dots` spelling.
//!
//! Mirror: `miniextendr-lint/src/misplaced_dots.rs` appends the same text to the
//! lint's "failed to parse" warning. Keep the detector and [`message`] in sync.

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};

/// A Rust `...` parameter followed by another parameter.
pub(crate) struct MisplacedDots {
    /// The parameter's name; `None` for unnamed dots (a bare `...` or `_: ...`).
    pub(crate) name: Option<String>,
    /// The three `.` tokens, so a diagnostic can span the whole `...`.
    pub(crate) dots: TokenStream,
}

/// The diagnostic for a `...` parameter that is not last, named `name` (`None` for unnamed dots).
///
/// Unnamed dots get `_dots`: `_: &Dots` would work on a function (its `_`
/// parameters are renamed) but not on a method, where the `&Dots` parameter
/// needs a plain name. No spelling of Rust `...` parses before another
/// parameter, so the hint names `&Dots`, never `_: ...`.
pub(crate) fn message(name: Option<&str>) -> String {
    let param = name.unwrap_or("_dots");
    format!(
        "Rust's `...` is only valid as the last parameter; write a dots parameter that is not \
         last as `{param}: &Dots` (`miniextendr_api::dots::Dots`), which is R's `...` at that \
         position. See https://a2-ai.github.io/miniextendr/manual/dots-typed-list/#formals-after"
    )
}

/// The first `...` parameter that another parameter follows, in any `fn` signature in `tokens`.
pub(crate) fn find(tokens: TokenStream) -> Option<MisplacedDots> {
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
            // Impl and trait bodies hold methods.
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
mod tests;
