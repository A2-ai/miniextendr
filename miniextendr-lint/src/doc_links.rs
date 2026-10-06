//! The doc-comment text of `#[miniextendr]` items that reaches roxygen2 as
//! markdown, and the `pkg::topic` links inside it (MXL204).
//!
//! The walk mirrors `miniextendr-macros/src/roxygen.rs`: leading prose comes
//! first, then `@tag` sections. The text of a code tag (`@examples`, ...) is not
//! markdown, and the lines after a single-line tag (`@export`, ...) stay in
//! rustdoc only. Fenced code blocks are skipped. Keep `MULTILINE_TAGS` and
//! `CODE_TAGS` in sync with that file.

use syn::Attribute;
use syn::spanned::Spanned;

/// Tags whose text runs over several lines (`MULTILINE_TAGS` in the macros).
const MULTILINE_TAGS: &[&str] = &[
    "description",
    "details",
    "return",
    "returns",
    "param",
    "note",
    "seealso",
    "section",
    "format",
    "references",
    "slot",
    "field",
    "value",
    "prop",
    "describeIn",
    "family",
    "inherit",
    "inheritParams",
    "inheritSection",
    "source",
    "author",
];

/// One line of doc text that roxygen2 reads as markdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownLine {
    /// 1-based source line.
    pub line: usize,
    /// The text, with the `@tag name` prefix removed on a tag's first line.
    pub text: String,
    /// Whether the line is leading prose (before the first `@tag`).
    pub leading_prose: bool,
}

/// A `pkg::topic` link as roxygen2 reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PkgLink {
    /// 1-based source line.
    pub line: usize,
    /// The link as written, brackets included.
    pub written: String,
    /// Everything before the last `::` of the target.
    pub pkg: String,
    /// Whether it sits in leading prose.
    pub leading_prose: bool,
}

/// Every `pkg::topic` link in the markdown text of `attrs`' doc comments.
pub fn pkg_links(attrs: &[Attribute]) -> Vec<PkgLink> {
    let mut links = Vec::new();
    for md in markdown_lines(attrs) {
        for (written, target) in bracketed_targets(&md.text) {
            if let Some(pkg) = r_package_of(target) {
                links.push(PkgLink {
                    line: md.line,
                    written: written.to_string(),
                    pkg: pkg.to_string(),
                    leading_prose: md.leading_prose,
                });
            }
        }
    }
    links
}

/// Whether the doc comment of `attrs` has a `@tag` line for `tag`: its first
/// word for a one-word `tag` (`noRd`), its whole text for a longer one
/// (`keywords internal`), as `has_roxygen_tag` in the macros matches it.
pub fn has_tag(attrs: &[Attribute], tag: &str) -> bool {
    attrs.iter().any(|attr| {
        let syn::Meta::NameValue(nv) = &attr.meta else {
            return false;
        };
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) = &nv.value
        else {
            return false;
        };
        attr.path().is_ident("doc")
            && lit.value().lines().any(|line| {
                line.trim_start().strip_prefix('@').is_some_and(|rest| {
                    if tag.contains(' ') {
                        rest.trim() == tag
                    } else {
                        rest.split_whitespace().next() == Some(tag)
                    }
                })
            })
    })
}

/// The doc text of `attrs` that roxygen2 reads as markdown.
pub fn markdown_lines(attrs: &[Attribute]) -> Vec<MarkdownLine> {
    let mut out = Vec::new();
    // `None` is leading prose; `Some(true)` a tag whose text is markdown.
    let mut section: Option<bool> = None;
    let mut fence: Option<String> = None;
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) = &nv.value
        else {
            continue;
        };
        let first_line = attr.span().start().line;
        for (offset, raw) in lit.value().lines().enumerate() {
            let trimmed = raw.trim_start();
            if let Some(open) = &fence {
                if trimmed.starts_with(open.as_str()) {
                    fence = None;
                }
                continue;
            }
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                fence = Some(trimmed[..3].to_string());
                continue;
            }
            let mut text = raw;
            if let Some(rest) = trimmed.strip_prefix('@')
                && rest.starts_with(|c: char| c.is_ascii_alphabetic())
            {
                let name_end = rest
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .unwrap_or(rest.len());
                section = Some(MULTILINE_TAGS.contains(&&rest[..name_end]));
                text = &rest[name_end..];
            }
            if section != Some(false) {
                out.push(MarkdownLine {
                    line: first_line + offset,
                    text: text.to_string(),
                    leading_prose: section.is_none(),
                });
            }
        }
    }
    out
}

/// The `[...]` link targets in one line of markdown, as `(written, target)`.
///
/// Mirrors the scanner in `miniextendr-macros/src/roxygen.rs`
/// (`sanitize_roxygen_links`): code spans and `\[` are skipped, an inline
/// `[text](dest)` is no R link, `[text][target]` has its own target and
/// `[target]` is its own.
fn bracketed_targets(s: &str) -> Vec<(&str, &str)> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut in_code = false;
    while i < s.len() {
        match bytes[i] {
            b'`' => in_code = !in_code,
            b'[' if !in_code && !(i > 0 && bytes[i - 1] == b'\\') => {
                if let Some(close_rel) = s[i + 1..].find(']') {
                    let close = i + 1 + close_rel;
                    let inner = &s[i + 1..close];
                    match bytes.get(close + 1) {
                        Some(b'(') => {}
                        Some(b'[') => {
                            if let Some(t_rel) = s[close + 2..].find(']') {
                                let end = close + 2 + t_rel + 1;
                                out.push((&s[i..end], &s[close + 2..end - 1]));
                                i = end;
                                continue;
                            }
                            out.push((&s[i..close + 1], inner));
                        }
                        _ => out.push((&s[i..close + 1], inner)),
                    }
                    i = close + 1;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// The R package roxygen2 links `target` into, when it reads it as `pkg::topic`
/// for a package name R accepts and a Rust path root it does not own.
fn r_package_of(target: &str) -> Option<&str> {
    let target = target.trim();
    let target = target
        .strip_prefix('`')
        .and_then(|t| t.strip_suffix('`'))
        .unwrap_or(target);
    if target.contains(char::is_whitespace) {
        return None;
    }
    let (pkg, topic) = target.rsplit_once("::")?;
    let topic = topic.strip_suffix("()").unwrap_or(topic);
    let name_ok = pkg.len() >= 2
        && pkg.starts_with(|c: char| c.is_ascii_alphabetic())
        && !pkg.ends_with('.')
        && pkg.chars().all(|c| c.is_ascii_alphanumeric() || c == '.');
    let topic_ok = !topic.is_empty()
        && topic
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    (name_ok && topic_ok && !matches!(pkg, "crate" | "self" | "Self")).then_some(pkg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn links(doc: &[&str]) -> Vec<(String, String, bool)> {
        let attrs: Vec<Attribute> = doc.iter().map(|l| syn::parse_quote!(#[doc = #l])).collect();
        pkg_links(&attrs)
            .into_iter()
            .map(|l| (l.written, l.pkg, l.leading_prose))
            .collect()
    }

    #[test]
    fn finds_every_link_form_and_skips_the_rest() {
        let found = links(&[
            " See [`Sources::prepare`] and [Type::method()].",
            " [text][dplyr::bind_rows], [t][crate::x], [`a::b::c`], [x](y::z)",
            " `[code::span]` and \\[esc::aped] and [plain]",
        ]);
        let pkgs: Vec<_> = found.iter().map(|f| f.1.as_str()).collect();
        assert_eq!(pkgs, ["Sources", "Type", "dplyr"]);
        assert!(found.iter().all(|f| f.2), "all leading prose");
    }

    #[test]
    fn skips_code_tags_fences_and_lines_after_single_line_tags() {
        let found = links(&[
            " @details Use [aa::b] here",
            " and [cc::d] there.",
            " ```",
            " [ee::f]",
            " ```",
            " @examples",
            " [gg::h]",
            " @export",
            " Rust callers: [ii::j]",
            " @param x see [kk::l]",
        ]);
        let pkgs: Vec<_> = found.iter().map(|f| f.1.as_str()).collect();
        assert_eq!(pkgs, ["aa", "cc", "kk"]);
        assert!(found.iter().all(|f| !f.2), "all in tags");
    }
}
