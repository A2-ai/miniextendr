//! Roxygen tag extraction and processing for R wrapper generation.
//!
//! This module extracts roxygen2-style tags (e.g., `@param`, `@examples`) from Rust
//! doc comments and propagates them to generated R wrapper code.
//!
//! # Usage
//!
//! In Rust doc comments, use roxygen2 tags:
//!
//! ```rust,ignore
//! /// @param x A numeric input.
//! /// @return The squared value.
//! /// @examples
//! /// square(4)
//! #[miniextendr]
//! pub fn square(x: f64) -> f64 { x * x }
//! ```
//!
//! # How doc lines carry over
//!
//! Every reader here ([`explicit_roxygen_tags_from_attrs`],
//! [`leading_prose_from_attrs`], [`strip_roxygen_from_attrs`]) walks the same
//! lines ([`doc_lines`]) with the same roles ([`classify`]):
//!
//! - Text before the first `@tag` line is leading prose. It becomes
//!   `@description` (when the block has none) with its lines, blank lines and
//!   indentation kept, and it stays in rustdoc. Its `[...]` links lose their
//!   brackets unless the crate sets `roxygen_prose_links = "keep"`
//!   ([`ProseLinks`]). Under `"keep"`, and in the text of every explicit tag
//!   but the code ones ([`CODE_TAGS`]), only a link roxygen2 can never
//!   resolve loses them ([`is_rustdoc_only_target`]).
//! - A tag runs from its `@tag` line to the next one. A multi-line tag
//!   (`@description`, `@return`, `@examples`, ...) keeps its blank lines (a
//!   roxygen2 paragraph break; a blank line in an example) and the indentation
//!   of its continuation lines, less the one space rustdoc's `/// ` puts there.
//!   Trailing blank lines are dropped. roxygen2 markdown reads a continuation
//!   indented 4 or more spaces after a blank line as a code block, as rustdoc
//!   does.
//! - A joined tag (`@title`, `@keywords`, `@concept`, `@aliases`) folds its
//!   wrapped lines onto one line.
//! - A single-line tag (`@export`, `@noRd`, `@rdname topic`, ...) ends at its
//!   line. Lines after it, up to the next tag, are rustdoc only.
//! - Rustdoc keeps the leading prose and the rustdoc-only lines, and drops
//!   every tag with its text.
//!
//! # R Package Configuration
//!
//! For roxygen2 to process multiline tags correctly, add this to your `DESCRIPTION` file:
//!
//! ```text
//! Roxygen: list(markdown = TRUE)
//! ```

use std::collections::HashSet;

/// Tags that allow multi-line content (continuation lines appended).
/// All other tags are treated as single-line.
const MULTILINE_TAGS: &[&str] = &[
    "examples",
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
    "value", // synonym for return
    "prop",  // S7 property documentation (roxygen2 8.0.0+)
    // Tags whose *name* is a single word but whose text may wrap onto the
    // next `///` line. Dropping the wrapped part silently truncated them (#1476).
    "describeIn",
    "family",
    "inherit",
    "inheritParams",
    "inheritSection",
    // roxygen2's other tags whose text runs over several lines (code, raw Rd,
    // NAMESPACE directives, free text).
    "examplesIf",
    "usage",
    "rawRd",
    "rawNamespace",
    "evalRd",
    "evalNamespace",
    "source",
    "author",
];

/// Tags whose text is R code or raw Rd, not markdown: their `[...]` is never
/// a link, so [`neutralize_tag_links`] leaves them as written.
const CODE_TAGS: &[&str] = &[
    "examples",
    "examplesIf",
    "usage",
    "eval",
    "evalRd",
    "evalNamespace",
    "rawRd",
    "rawNamespace",
];

/// Tags whose wrapped continuation lines are joined back onto one line with a
/// space instead of a newline: roxygen2 wants these on a single line (or reads
/// them as one whitespace-separated list), but a long title, keyword list,
/// concept, or alias list in a Rust doc comment can be wrapped by the author.
const JOINED_TAGS: &[&str] = &["title", "keywords", "concept", "aliases"];

/// A bare `@name` / `@rdname` (topic written on the next `///` line, which
/// roxygen2 accepts) takes exactly one continuation line as its topic. Once
/// the tag has a topic it is single-line again, so following prose is not
/// absorbed (see `test_rdname_stays_single_line`). The wrapper registry
/// decides page routing from the single `#' @name <topic>` line, so a dropped
/// topic would silently change which page a function lands on.
fn is_bare_topic_tag(tag: &str) -> bool {
    matches!(tag, "@name" | "@rdname")
}

/// Check if a tag name supports multi-line content.
fn is_multiline_tag(tag: &str) -> bool {
    // Extract the tag name from "@tagname ..." or "@tagname"
    let tag_name = tag
        .strip_prefix('@')
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or("");
    MULTILINE_TAGS.contains(&tag_name)
}

/// Check if a tag's continuation lines are joined with a space (see [`JOINED_TAGS`]).
fn is_joined_tag(tag: &str) -> bool {
    let tag_name = tag
        .strip_prefix('@')
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or("");
    JOINED_TAGS.contains(&tag_name)
}

// region: doc lines

/// One line of a string-literal doc attribute.
#[derive(Debug, PartialEq, Eq)]
struct DocLine {
    /// Index of the doc attribute in the attribute slice it was read from.
    attr: usize,
    /// The line with its rustdoc lead removed (see [`doc_lines`]); empty for
    /// a blank line.
    text: String,
}

/// The lines of every string-literal doc attribute in `attrs`, in order.
///
/// Non-doc attributes are skipped, so an interleaved `#[cfg(...)]` never
/// interrupts a tag (#613). A `#[doc = include_str!(..)]` is not a literal and
/// stays unread. Per attribute:
///
/// - a single-line literal (each `///` line, a `#[doc = "..."]` from
///   `macro_rules!`) loses at most the one leading space rustdoc's `/// `
///   puts there;
/// - a multi-line literal (`/** ... */`, `#[doc = "a\nb"]`) loses the common
///   leading whitespace of its non-blank lines, so a relative indent survives;
/// - a whitespace-only line becomes an empty one.
fn doc_lines(attrs: &[syn::Attribute]) -> Vec<DocLine> {
    let mut out = Vec::new();
    for (attr, a) in attrs.iter().enumerate() {
        if !a.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &a.meta else {
            continue;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let syn::Lit::Str(lit) = &expr_lit.lit else {
            continue;
        };
        let value = lit.value();
        // `"".lines()` yields nothing, but an empty `///` is one blank line.
        let lines: Vec<&str> = if value.is_empty() {
            vec![""]
        } else {
            value.lines().collect()
        };
        let lead = if value.contains('\n') {
            common_indent(&lines)
        } else {
            " "
        };
        for line in lines {
            let text = if line.trim().is_empty() {
                ""
            } else {
                line.strip_prefix(lead).unwrap_or(line)
            };
            out.push(DocLine {
                attr,
                text: text.to_string(),
            });
        }
    }
    out
}

/// The leading whitespace every non-blank line of `lines` starts with.
fn common_indent<'a>(lines: &[&'a str]) -> &'a str {
    lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| &line[..line.len() - line.trim_start().len()])
        .reduce(|common, lead| {
            let shared = common
                .char_indices()
                .zip(lead.chars())
                .take_while(|((_, a), b)| a == b)
                .last()
                .map_or(0, |((i, c), _)| i + c.len_utf8());
            &common[..shared]
        })
        .unwrap_or("")
}

/// What a doc line is to roxygen extraction and rustdoc stripping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineRole {
    /// Before the first tag: leading prose (promoted to `@description`, kept
    /// in rustdoc).
    Prose,
    /// A line starting with `@` (after leading whitespace): a new tag.
    TagStart,
    /// Part of the open tag's text, or a blank line that ends a tag (trailing,
    /// or before the next tag).
    TagBody,
    /// After a single-line tag: rustdoc only, never R. A blank line is
    /// rustdoc when a rustdoc-only line follows it.
    Rustdoc,
}

/// How the tag that is open takes the lines that follow it.
#[derive(Clone, Copy)]
enum OpenTag {
    /// [`MULTILINE_TAGS`]: every line up to the next tag, blank lines included.
    Multi,
    /// [`JOINED_TAGS`], or a bare `@name` / `@rdname` still waiting for its
    /// topic ([`is_bare_topic_tag`]): the non-blank lines, folded onto one
    /// line. A bare topic tag takes one line, then it is `Single`.
    Joined { bare_topic: bool },
    /// Any other tag: its own line only.
    Single,
}

impl OpenTag {
    fn of(tag_line: &str) -> Self {
        if is_multiline_tag(tag_line) {
            Self::Multi
        } else if is_joined_tag(tag_line) {
            Self::Joined { bare_topic: false }
        } else if is_bare_topic_tag(tag_line) {
            Self::Joined { bare_topic: true }
        } else {
            Self::Single
        }
    }
}

/// The role of each of `lines` ([`doc_lines`]): the one rule the tag
/// extractor, the leading-prose reader and the rustdoc strip share.
fn classify(lines: &[DocLine]) -> Vec<LineRole> {
    let mut roles = Vec::with_capacity(lines.len());
    let mut open: Option<OpenTag> = None;
    for line in lines {
        let text = line.text.trim_start();
        let role = if text.starts_with('@') {
            open = Some(OpenTag::of(text));
            LineRole::TagStart
        } else {
            match open {
                None => LineRole::Prose,
                Some(OpenTag::Multi | OpenTag::Joined { bare_topic: false }) => LineRole::TagBody,
                Some(OpenTag::Joined { bare_topic: true }) => {
                    if !text.is_empty() {
                        open = Some(OpenTag::Single);
                    }
                    LineRole::TagBody
                }
                Some(OpenTag::Single) => LineRole::Rustdoc,
            }
        };
        roles.push(role);
    }
    // A blank line after a single-line tag goes with the paragraph after it:
    // rustdoc before a rustdoc-only line, the tag's end otherwise.
    let mut next_is_rustdoc = false;
    for (line, role) in lines.iter().zip(roles.iter_mut()).rev() {
        if !line.text.is_empty() {
            next_is_rustdoc = *role == LineRole::Rustdoc;
        } else if *role == LineRole::Rustdoc && !next_is_rustdoc {
            *role = LineRole::TagBody;
        }
    }
    roles
}

// endregion

/// Extract roxygen tag lines (starting with '@') from Rust doc attributes.
///
/// Most tags capture only a single line. Multi-line tags like `@examples`,
/// `@description`, `@param`, and `@return` append their continuation lines,
/// blank lines and indentation included (see the module docs).
///
/// Leading prose (the paragraphs before the first tag) is promoted to
/// `@description` when the block has none.
pub(crate) fn roxygen_tags_from_attrs(attrs: &[syn::Attribute]) -> Vec<String> {
    roxygen_tags_from_attrs_impl(attrs)
}

/// Extract roxygen tags for an impl-block method.
///
/// Identical to [`roxygen_tags_from_attrs`] — leading prose is promoted to
/// `@description` in every context. Kept as a named alias so the class-system
/// generators read clearly at the call site.
pub(crate) fn roxygen_tags_from_attrs_for_r6_method(attrs: &[syn::Attribute]) -> Vec<String> {
    roxygen_tags_from_attrs_impl(attrs)
}

/// Core implementation of roxygen tag extraction from `#[doc = "..."]` attributes.
///
/// Walks the doc lines ([`doc_lines`]) with their roles ([`classify`]). Lines
/// starting with `@` begin a new tag; continuation lines are appended only if
/// the current tag is multiline-capable (or joined, see [`JOINED_TAGS`]).
///
/// Only doc attributes are read, in order, so interleaved `#[cfg(...)]` or
/// other non-doc attributes never break multiline-tag continuation (#613).
/// This is a pure parse-side transform: the emitted `TokenStream` is
/// unaffected.
///
/// Leading prose (paragraphs before the first `@tag`) is promoted to a
/// `@description` tag — never `@title`. The `@title` is left to the caller
/// (structural name); see [`leading_prose_from_attrs`] for why.
fn roxygen_tags_from_attrs_impl(attrs: &[syn::Attribute]) -> Vec<String> {
    roxygen_tags_with(attrs, crate::crate_config::roxygen_prose_links())
}

/// [`roxygen_tags_from_attrs_impl`] with the crate's [`ProseLinks`] setting
/// passed in (testable without a manifest).
///
/// Every explicit tag loses its rustdoc-only links ([`neutralize_tag_links`])
/// whatever the setting, which governs the leading prose only.
fn roxygen_tags_with(attrs: &[syn::Attribute], links: ProseLinks) -> Vec<String> {
    let mut tags: Vec<String> = explicit_roxygen_tags_from_attrs(attrs)
        .into_iter()
        .map(neutralize_tag_links)
        .collect();

    // Check which tags are present
    let tag_names_set = tag_names(&tags);
    let has_description = tag_names_set.contains("description");

    // Promote leading prose doc comments to `@description` (all paragraphs before
    // the first `@tag`), unless the author already wrote an explicit `@description`.
    //
    // The page `@title` is NOT synthesized from prose. Rustdoc summaries are markdown
    // written for `cargo doc` — intra-doc links (`[`Foo`]`, `[x][crate::y]`) and code
    // spans — which roxygen2's markdown parser tries to resolve as R `\link{}` topics
    // and fails ("could not resolve link to topic" / "refers to un-installed package").
    // Titles come from the structural name instead: the wrapper name for standalone
    // functions (see `lib.rs`) and `@title {Name} Class` for class blocks (see
    // `ClassDocBuilder`). Demoting prose to `@description` keeps it visible while the
    // title stays link-free.
    //
    // `leading_prose_from_attrs` returns `None` for tag-led blocks (no leading prose),
    // so `@inherit`/`@rdname`-only docs never gain a spurious description.
    if !has_description && let Some(desc) = leading_prose_from_attrs(attrs, links) {
        tags.insert(0, format!("@description {}", desc));
    }

    tags
}

/// Parse only the author-written `@tag` lines from doc attributes — no
/// leading-prose promotion.
///
/// A tag line is left-trimmed, so `///   @param` still starts a tag. A
/// multi-line tag keeps its continuation lines as written (less the rustdoc
/// lead, [`doc_lines`]) and its interior blank lines, and drops its trailing
/// ones; a joined or bare-topic tag folds its lines onto one with a space.
///
/// [`roxygen_tags_from_attrs_impl`] layers the leading-prose → `@description`
/// promotion on top of this. [`doc_conflict_warnings`] must use this raw parse
/// instead: comparing the *synthesized* description (all leading prose) against
/// the implicit one (second paragraph) warned on every multi-paragraph doc
/// comment that had no explicit `@description` at all (#1172).
fn explicit_roxygen_tags_from_attrs(attrs: &[syn::Attribute]) -> Vec<String> {
    let lines = doc_lines(attrs);
    let roles = classify(&lines);
    let mut tags: Vec<String> = Vec::new();
    // Blank lines seen inside a multi-line tag, written out only when more of
    // its text follows: a tag's trailing blank lines are dropped.
    let mut pending_blanks = 0;

    for (line, role) in lines.iter().zip(roles) {
        match role {
            LineRole::TagStart => {
                pending_blanks = 0;
                tags.push(line.text.trim_start().to_string());
            }
            LineRole::TagBody => {
                let Some(last) = tags.last_mut() else {
                    continue;
                };
                if is_multiline_tag(last) {
                    if line.text.is_empty() {
                        pending_blanks += 1;
                    } else {
                        for _ in 0..std::mem::take(&mut pending_blanks) {
                            last.push('\n');
                        }
                        last.push('\n');
                        last.push_str(&line.text);
                    }
                } else if !line.text.is_empty() {
                    // Wrapped joined tag or bare topic: fold onto one line.
                    last.push(' ');
                    last.push_str(line.text.trim());
                }
            }
            // Leading prose is read by `leading_prose_from_attrs` and promoted
            // to @description in `roxygen_tags_from_attrs_impl`; rustdoc-only
            // lines never reach R.
            LineRole::Prose | LineRole::Rustdoc => {}
        }
    }

    tags
}

/// One roxygen comment line: `#' text`, or a bare `#'` for a blank `text`
/// (no trailing whitespace in the generated R). Render author tag text
/// through this (or the `push_roxygen_tags*` helpers), never
/// `format!("#' {}", ..)`.
pub(crate) fn roxygen_line(text: &str) -> String {
    if text.trim().is_empty() {
        "#'".to_string()
    } else {
        format!("#' {text}")
    }
}

/// Render roxygen tag lines as "#' ..." comment lines.
///
/// Multiline tags (containing '\n') are split into separate `#'` lines
/// ([`roxygen_line`]).
pub(crate) fn format_roxygen_tags(tags: &[String]) -> String {
    let mut out = String::new();
    for line in tags.iter().flat_map(|tag| tag.lines()) {
        out.push_str(&roxygen_line(line));
        out.push('\n');
    }
    out
}

/// Push roxygen tag lines into a vector of R wrapper lines.
///
/// Multiline tags (containing '\n') are split into separate `#'` lines
/// ([`roxygen_line`]).
pub(crate) fn push_roxygen_tags(lines: &mut Vec<String>, tags: &[String]) {
    lines.extend(tags.iter().flat_map(|tag| tag.lines()).map(roxygen_line));
}

/// Like [`push_roxygen_tags`] but takes `&[&str]` for filtered tag slices.
pub(crate) fn push_roxygen_tags_str(lines: &mut Vec<String>, tags: &[&str]) {
    lines.extend(tags.iter().flat_map(|tag| tag.lines()).map(roxygen_line));
}

/// Return true if the tag list contains a specific roxygen tag.
///
/// Supports both single-word tags (e.g., `"export"`, `"noRd"`) and
/// multi-word tags (e.g., `"keywords internal"`). For single-word tags,
/// matches the first word after `@`. For multi-word tags, matches the
/// full content after `@` (trimmed).
pub(crate) fn has_roxygen_tag(tags: &[String], tag: &str) -> bool {
    if tag.contains(' ') {
        // Multi-word tag: match the full content after @
        tags.iter().any(|t| {
            t.trim_start()
                .strip_prefix('@')
                .is_some_and(|rest| rest.trim() == tag)
        })
    } else {
        tag_names(tags).contains(tag)
    }
}

/// The topic named by the tag list's own `@rdname` tag, if it has one.
///
/// Class generators default every method onto the class page (`@rdname
/// <Class>`); a method-level `/// @rdname other` splits it onto its own page.
/// Every emission path that injects the class default must consult this first
/// so a user-supplied topic is honoured once, not duplicated by the default.
pub(crate) fn rdname_value(tags: &[String]) -> Option<&str> {
    tags.iter().find_map(|t| {
        let rest = t.trim_start().strip_prefix("@rdname")?;
        let value = rest.trim();
        (rest.starts_with(char::is_whitespace) && !value.is_empty()).then_some(value)
    })
}

/// Push `#' @rdname <topic>` for a scaffolding block that follows its method:
/// the method's page ([`method_page`]: its `@describeIn` destination or own
/// `@rdname`), else `default` (the class page).
pub(crate) fn push_rdname_or_default(lines: &mut Vec<String>, tags: &[String], default: &str) {
    let topic = method_page(tags, default);
    lines.push(format!("#' @rdname {topic}"));
}

/// The `@source` provenance line for a generated documentation block, or
/// `None` unless the crate opted in with `[package.metadata.miniextendr]
/// source_tags = true` (#1552). On a shared Rd page the tag collects one
/// paragraph per function, so it is off by default; the `# Generated from
/// Rust fn … (file:line:col)` comment above each wrapper stays as the
/// navigation pointer. Every emitter goes through here (or the two
/// specialisations below), so the knob has one implementation.
pub(crate) fn source_tag(text: impl AsRef<str>) -> Option<String> {
    source_tag_with(crate::crate_config::source_tags_enabled(), text.as_ref())
}

/// [`source_tag`] with the crate setting passed in (testable without a manifest).
pub(crate) fn source_tag_with(enabled: bool, text: &str) -> Option<String> {
    enabled.then(|| format!("#' @source {text}"))
}

/// The `@source` line for a class method: ``Generated by miniextendr from
/// `Type::method` ``, subject to [`source_tag`]'s opt-in.
pub(crate) fn method_source_tag(
    type_ident: &syn::Ident,
    method_ident: &syn::Ident,
) -> Option<String> {
    source_tag(format!(
        "Generated by miniextendr from `{}::{}`",
        type_ident, method_ident
    ))
}

/// The `@source` line for a class definition: ``Generated by miniextendr from
/// Rust type `Type` ``, subject to [`source_tag`]'s opt-in.
pub(crate) fn class_source_tag(type_ident: &syn::Ident) -> Option<String> {
    source_tag(format!(
        "Generated by miniextendr from Rust type `{}`",
        type_ident
    ))
}

/// Extract the set of tag names from a list of roxygen tag strings.
///
/// Each tag string is expected to start with `@tagname`. Returns a set of
/// the tag names (without the `@` prefix).
fn tag_names(tags: &[String]) -> HashSet<&str> {
    let mut names = HashSet::new();
    for tag in tags {
        let trimmed = tag.trim_start();
        let name = trimmed
            .strip_prefix('@')
            .and_then(|rest| rest.split_whitespace().next());
        if let Some(name) = name {
            names.insert(name);
        }
    }
    names
}

/// Find the value of a specific roxygen tag (e.g., "title" for `@title ...`).
///
/// Returns `None` if the tag is not present or has no value.
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
pub(crate) fn find_tag_value<'a>(tags: &'a [String], tag_name: &str) -> Option<&'a str> {
    for tag in tags {
        let trimmed = tag.trim_start();
        if let Some(rest) = trimmed.strip_prefix('@') {
            let mut parts = rest.splitn(2, |c: char| c.is_whitespace());
            if let Some(name) = parts.next()
                && name == tag_name
            {
                // Get the value (everything after the tag name)
                return parts.next().map(|s| s.trim());
            }
        }
    }
    None
}

/// Normalize text for comparison: lowercase, collapse whitespace, strip trailing punctuation.
///
/// Used by `doc_conflict_warnings` to compare explicit `@title`/`@description` values
/// with implicit values derived from the doc comment structure. Normalization ensures
/// minor formatting differences (extra spaces, trailing periods) don't trigger false warnings.
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
fn normalize_for_comparison(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut result = String::new();
    for word in lower.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    }
    result.truncate(
        result
            .trim_end_matches(|c: char| c.is_ascii_punctuation())
            .len(),
    );
    result
}

/// Extract the implicit title from doc attributes (first sentence, up to first `.` or newline).
///
/// Returns `None` if there are no doc comments or if docs start with a `@tag`.
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
pub(crate) fn implicit_title_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let syn::Lit::Str(lit) = &expr_lit.lit else {
            continue;
        };

        let content = lit.value();
        let trimmed = content.trim();

        // If we hit a @tag before any content, there's no implicit title
        if trimmed.starts_with('@') {
            if lines.is_empty() {
                return None;
            }
            break;
        }

        // Empty line ends first sentence for title extraction
        if trimmed.is_empty() {
            break;
        }

        // Check if this line contains a sentence-ending period
        if let Some(pos) = trimmed.find(". ") {
            lines.push(trimmed[..pos].to_string());
            break;
        } else if trimmed.ends_with('.') {
            lines.push(trimmed.trim_end_matches('.').to_string());
            break;
        } else {
            lines.push(trimmed.to_string());
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

/// Extract the implicit description from doc attributes (second paragraph).
///
/// In roxygen2, the first paragraph is the title and the second paragraph is the
/// description. This function skips the first paragraph (up to the first blank line)
/// and returns the second paragraph.
///
/// Returns `None` if there is no second paragraph, no doc comments, or if docs
/// start with a `@tag`.
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
pub(crate) fn implicit_description_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    let mut found_first_paragraph = false;
    let mut in_gap = false;

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let syn::Lit::Str(lit) = &expr_lit.lit else {
            continue;
        };

        let content = lit.value();
        let trimmed = content.trim();

        // If we hit a @tag, stop
        if trimmed.starts_with('@') {
            break;
        }

        if !found_first_paragraph {
            // Still in the first paragraph (title)
            if trimmed.is_empty() {
                // Blank line — first paragraph ended, now in the gap
                found_first_paragraph = true;
                in_gap = true;
            }
            // Non-empty lines before first blank are title — skip
        } else if in_gap {
            // Between paragraphs — skip blank lines
            if !trimmed.is_empty() {
                // Start of second paragraph
                in_gap = false;
                lines.push(trimmed.to_string());
            }
        } else {
            // In second paragraph
            if trimmed.is_empty() {
                // End of second paragraph
                break;
            }
            lines.push(trimmed.to_string());
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

/// What leading prose does with `[...]` link syntax on its way to
/// `@description`: the crate's `roxygen_prose_links` setting in
/// `[package.metadata.miniextendr]`.
///
/// rustdoc and roxygen2 read the same `[name()]` / `[pkg::name()]` / `[Topic]`
/// / `` [`Topic`] `` syntax as a link to a Rust item and to an R help topic
/// respectively, so the two cannot be told apart from the text: rustdoc
/// resolves `[name()]` to an in-scope Rust fn and warns
/// `broken_intra_doc_links` on any it cannot find. The crate says which reader
/// its doc comments are written for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ProseLinks {
    /// `"strip"`, the default: doc comments are written for rustdoc, so a link
    /// loses its brackets ([`Neutralize::AllLinks`]) and roxygen2 never tries
    /// to resolve a Rust item as an R topic.
    #[default]
    Strip,
    /// `"keep"`: doc comments are written for roxygen2, so leading prose keeps
    /// its links, as the text of an explicit tag does. Only the links roxygen2
    /// can never resolve lose their brackets ([`Neutralize::RustdocOnly`]).
    Keep,
}

impl ProseLinks {
    /// Parse the manifest spelling (`"strip"` / `"keep"`).
    pub(crate) fn parse_name(name: &str) -> Option<Self> {
        match name {
            "strip" => Some(Self::Strip),
            "keep" => Some(Self::Keep),
            _ => None,
        }
    }
}

/// Collect the leading prose of a doc comment (the lines before the first
/// `@tag`, [`LineRole::Prose`]) as roxygen `@description` text, its links
/// treated per the crate's [`ProseLinks`] setting (passed in, so the unit tests
/// need no manifest).
///
/// The lines ([`doc_lines`]) keep their line breaks, blank lines (roxygen2
/// paragraph breaks, rendered as bare `#'` lines by `push_roxygen_tags`) and
/// indentation, so a markdown list, a nested item or a fenced block reaches
/// roxygen2 as written. Leading and trailing blank lines are dropped. With
/// [`ProseLinks::Strip`] every link is neutralized; with [`ProseLinks::Keep`]
/// only the rustdoc-only ones are ([`sanitize_prose_links`]).
///
/// Returns `None` when the block has no leading prose (empty, or starts with a `@tag`),
/// so tag-led blocks never gain a spurious `@description`.
fn leading_prose_from_attrs(attrs: &[syn::Attribute], links: ProseLinks) -> Option<String> {
    let lines = doc_lines(attrs);
    let prose: Vec<&str> = lines
        .iter()
        .zip(classify(&lines))
        .take_while(|(_, role)| *role == LineRole::Prose)
        .map(|(line, _)| line.text.as_str())
        .collect();
    let first = prose.iter().position(|line| !line.is_empty())?;
    let last = prose.iter().rposition(|line| !line.is_empty())?;
    let prose = &prose[first..=last];
    Some(match links {
        ProseLinks::Strip => sanitize_prose_links(prose, Neutralize::AllLinks),
        ProseLinks::Keep => sanitize_prose_links(prose, Neutralize::RustdocOnly),
    })
}

/// An explicit tag with its rustdoc-only links neutralized
/// ([`Neutralize::RustdocOnly`]), or as written when its text is code
/// ([`CODE_TAGS`]).
fn neutralize_tag_links(tag: String) -> String {
    if roxygen_tag_name(&tag).is_some_and(|name| CODE_TAGS.contains(&name)) {
        return tag;
    }
    sanitize_prose_links(&tag.lines().collect::<Vec<_>>(), Neutralize::RustdocOnly)
}

/// Which `[...]` links [`sanitize_roxygen_links`] reduces to their text.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Neutralize {
    /// Every link: leading prose under [`ProseLinks::Strip`].
    AllLinks,
    /// Only a link whose target roxygen2 can never resolve
    /// ([`is_rustdoc_only_target`]): leading prose under [`ProseLinks::Keep`],
    /// and explicit tag text ([`neutralize_tag_links`]).
    RustdocOnly,
}

/// [`sanitize_roxygen_links`] over prose lines, one paragraph (the lines
/// between blank lines) at a time, joined back with their line breaks.
///
/// A link whose text and target sit on two lines of one paragraph is still
/// found. CommonMark code spans do not cross a blank line, so an unbalanced
/// backtick cannot switch neutralizing off beyond its paragraph. A fenced
/// block (```` ``` ```` or `~~~`, blank lines included) is code and passes
/// through untouched, so `x[i]` there is never rewritten.
fn sanitize_prose_links(lines: &[&str], links: Neutralize) -> String {
    fn flush(paragraph: &mut Vec<&str>, out: &mut Vec<String>, links: Neutralize) {
        if !paragraph.is_empty() {
            out.push(sanitize_roxygen_links(&paragraph.join("\n"), links));
            paragraph.clear();
        }
    }

    let mut out: Vec<String> = Vec::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut open_fence: Option<&str> = None;
    for &line in lines {
        if let Some(open) = open_fence {
            if fence_marker(line).is_some_and(|close| closes_fence(line, close, open)) {
                open_fence = None;
            }
            out.push(line.to_string());
        } else if let Some(open) = fence_marker(line) {
            flush(&mut paragraph, &mut out, links);
            open_fence = Some(open);
            out.push(line.to_string());
        } else if line.is_empty() {
            flush(&mut paragraph, &mut out, links);
            out.push(String::new());
        } else {
            paragraph.push(line);
        }
    }
    flush(&mut paragraph, &mut out, links);
    out.join("\n")
}

/// The fence run (3 or more `` ` `` or `~`) a CommonMark code-fence line
/// starts with, after at most 3 spaces of indent.
fn fence_marker(line: &str) -> Option<&str> {
    let body = line.trim_start_matches(' ');
    if line.len() - body.len() > 3 {
        return None;
    }
    let fence_char = body.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let run = body.len() - body.trim_start_matches(fence_char).len();
    (run >= 3).then(|| &body[..run])
}

/// Whether `line`, whose fence run is `close`, ends the block `open` began: the
/// same character, at least as long, and nothing after it.
fn closes_fence(line: &str, close: &str, open: &str) -> bool {
    close.starts_with(&open[..1])
        && close.len() >= open.len()
        && line.trim_start_matches(' ')[close.len()..]
            .trim()
            .is_empty()
}

/// Neutralize rustdoc intra-doc link syntax so prose is valid roxygen2 markdown.
///
/// rustdoc `[`Foo`]` / `[Foo]` / `[text][target]` are intra-doc links resolved
/// against *Rust* items by `cargo doc`. roxygen2 (markdown on) reads the same
/// `[...]` as an R `\link{}` to a *help topic*, which can't resolve. We strip the
/// link brackets down to the visible text (keeping any `` `code` `` span), while
/// leaving genuine markdown links `[text](url)` — recognized by the `]( ` that
/// follows — untouched. With [`Neutralize::AllLinks`] a roxygen2 link
/// (`[other_fn()]`) is stripped too: the syntax is rustdoc's as well, so a
/// crate writing for roxygen2 opts out with [`ProseLinks::Keep`]. With
/// [`Neutralize::RustdocOnly`] only a link whose target roxygen2 can never
/// resolve is stripped ([`is_rustdoc_only_target`]); the rest stay as written.
fn sanitize_roxygen_links(s: &str, links: Neutralize) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let mut in_code = false;
    while i < s.len() {
        if bytes[i] == b'`' {
            // Inline code span: markdown (and roxygen2) never parse `[...]`
            // inside backticks as a link, so neither do we — stripping there
            // would corrupt code like `x[i]`.
            in_code = !in_code;
            out.push('`');
            i += 1;
            continue;
        }
        if !in_code && bytes[i] == b'[' {
            // `\[` is a backslash-escaped literal bracket (idiomatic rustdoc
            // for suppressing intra-doc links, e.g. `Box<\[u8\]>`). Not a link
            // opener — pass it through; roxygen2's markdown unescapes it.
            if i > 0 && bytes[i - 1] == b'\\' {
                out.push('[');
                i += 1;
                continue;
            }
            // `[` is ASCII, so `i + 1` is a char boundary.
            if let Some(close_rel) = s[i + 1..].find(']') {
                let close = i + 1 + close_rel;
                let inner = &s[i + 1..close];
                // `[text](url)` — real markdown link. Emit `[` literally and let
                // the inner text + `](url)` flow through unchanged.
                if bytes.get(close + 1) == Some(&b'(') {
                    out.push('[');
                    i += 1;
                    continue;
                }
                // `[text][target]` — reference link; `[text]` — shortcut link,
                // its own target.
                let (target, end) = match bytes.get(close + 1) {
                    Some(b'[') => match s[close + 2..].find(']') {
                        Some(t_rel) => (&s[close + 2..close + 2 + t_rel], close + 2 + t_rel + 1),
                        None => (inner, close + 1),
                    },
                    _ => (inner, close + 1),
                };
                // Keep `text`, drop the brackets and any `[target]`.
                if links == Neutralize::AllLinks || is_rustdoc_only_target(target) {
                    out.push_str(inner);
                } else {
                    out.push_str(&s[i..end]);
                }
                i = end;
                continue;
            }
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Whether roxygen2 can never resolve the link target `target`, so the link
/// was written for rustdoc.
///
/// roxygen2 reads `pkg::topic` as a link into the R package `pkg`, everything
/// before the last `::`, and warns "refers to un-installed package" when no
/// such package is installed. A target is rustdoc-only when that `pkg` is a
/// Rust path root (`crate`, `self`, `Self`) or cannot be an R package name
/// (`a::b`, `my_mod`; see [`is_r_package_name`]). Backticks around the target
/// are ignored, as both readers do. Anything else may be an R link, so it is
/// left alone: `[topic]`, `` [`topic`] ``, `[fn()]` and `[pkg::topic]`, which
/// roxygen2 documents, and so `` [`Type::method`] `` (`Type` can be a package:
/// `R6`, `S7`, `Matrix`) and `[super::x]` (`super` is on CRAN).
fn is_rustdoc_only_target(target: &str) -> bool {
    let target = target.trim();
    let target = target
        .strip_prefix('`')
        .and_then(|t| t.strip_suffix('`'))
        .unwrap_or(target);
    let Some((pkg, _)) = target.rsplit_once("::") else {
        return false;
    };
    matches!(pkg, "crate" | "self" | "Self") || !is_r_package_name(pkg)
}

/// R's rule for a package name (Writing R Extensions, "The DESCRIPTION
/// file"): ASCII letters, digits and `.`, at least two characters, starting
/// with a letter and not ending in `.`.
fn is_r_package_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[bytes.len() - 1] != b'.'
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'.')
}

/// Check for conflicts between explicit `@title`/`@description` tags and implicit values.
///
/// When the `doc-lint` feature is enabled, returns tokens that generate compile-time
/// deprecation warnings if explicit roxygen tags differ from the implicit values
/// derived from the doc comment structure.
///
/// The returned tokens should be appended to the macro expansion output.
#[cfg(feature = "doc-lint")]
pub(crate) fn doc_conflict_warnings(
    attrs: &[syn::Attribute],
    _span: proc_macro2::Span,
) -> proc_macro2::TokenStream {
    use quote::quote;

    // Raw parse only: `roxygen_tags_from_attrs` synthesizes a `@description`
    // from leading prose, which this lint must not mistake for an
    // author-written tag (#1172).
    let tags = explicit_roxygen_tags_from_attrs(attrs);
    let mut warnings = proc_macro2::TokenStream::new();

    // Check @title conflict
    if let Some(explicit) = find_tag_value(&tags, "title")
        && let Some(implicit) = implicit_title_from_attrs(attrs)
        && normalize_for_comparison(explicit) != normalize_for_comparison(&implicit)
    {
        let msg = format!(
            "miniextendr doc-lint: explicit @title differs from first doc line. \
             R's roxygen2 uses the first line as the title. \
             implicit: \"{}\", explicit @title: \"{}\"",
            implicit, explicit
        );
        warnings.extend(quote! {
            const _: () = {
                #[deprecated(note = #msg)]
                #[doc(hidden)]
                #[allow(dead_code)]
                const MINIEXTENDR_DOC_LINT_TITLE: () = ();
                let _ = MINIEXTENDR_DOC_LINT_TITLE;
            };
        });
    }

    // Check @description conflict
    if let Some(explicit) = find_tag_value(&tags, "description")
        && let Some(implicit) = implicit_description_from_attrs(attrs)
        && normalize_for_comparison(explicit) != normalize_for_comparison(&implicit)
    {
        let msg = format!(
            "miniextendr doc-lint: explicit @description differs from first paragraph. \
             R's roxygen2 uses the first paragraph as the description. \
             implicit: \"{}\", explicit @description: \"{}\"",
            implicit, explicit
        );
        warnings.extend(quote! {
            const _: () = {
                #[deprecated(note = #msg)]
                #[doc(hidden)]
                #[allow(dead_code)]
                const MINIEXTENDR_DOC_LINT_DESC: () = ();
                let _ = MINIEXTENDR_DOC_LINT_DESC;
            };
        });
    }

    warnings
}

/// No-op when doc-lint feature is disabled.
#[cfg(not(feature = "doc-lint"))]
pub(crate) fn doc_conflict_warnings(
    _attrs: &[syn::Attribute],
    _span: proc_macro2::Span,
) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::new()
}

/// Roxygen tags that only make sense on individual methods, not on impl blocks.
///
/// - `@param` — impl blocks have no parameters (except for R6 class-level param docs,
///   which roxygen2 8.0.0 inherits into all methods; those are exempted by
///   [`strip_method_tags_r6`]).
/// - `@return` / `@returns` — impl blocks have no return value.
/// - `@examples` — examples belong on the method that is being demonstrated.
/// - `@export` — redundant: export for class-level docs is handled by
///   `ClassDocBuilder`, which emits `@export` based on the impl block's
///   `internal` / `noexport` attrs, not on user-supplied roxygen.
const METHOD_ONLY_TAGS: &[&str] = &["param", "return", "returns", "examples", "export"];

/// Tags stripped from impl-block docs for R6 classes — same as `METHOD_ONLY_TAGS`
/// minus `"param"`, since roxygen2 8.0.0 inherits class-level `@param` tags into
/// all R6 methods and strips them from the rendered method entries automatically.
/// This means `/// @param breed …` on an R6 impl block is valid and intentional,
/// not a misplaced method-only tag. Keeping them avoids both the compile warning
/// and the resulting `(no documentation available)` placeholder on subclass ctors.
const METHOD_ONLY_TAGS_R6: &[&str] = &["return", "returns", "examples", "export"];

/// Extract the tag name from a roxygen line (everything between `@` and the
/// first whitespace character). Returns `None` for lines that don't start with
/// a tag.
pub(crate) fn roxygen_tag_name(tag: &str) -> Option<&str> {
    let rest = tag.trim_start().strip_prefix('@')?;
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    Some(&rest[..end])
}

/// Whether a method block without `\usage` (an env method `Type$method`, a
/// trait method `Type$Trait$method`) forwards the author tag `tag`: every tag
/// but a bare `@export`. On such a block roxygen2's namespace roclet turns a
/// bare `@export` into `export("Type$method")`, and R refuses to load the
/// namespace ("undefined exports"). The member is reachable through its
/// exported class, so exporting it is never valid. An `@export <symbol>`
/// names its own target and is forwarded.
pub(crate) fn forwarded_member_tag(tag: &str) -> bool {
    !tag.trim_start()
        .strip_prefix("@export")
        .is_some_and(|rest| rest.trim().is_empty())
}

/// Filter out method-specific roxygen tags from impl-block-level docs and emit
/// compile warnings for each stripped tag.
///
/// Method-specific tags (`@param`, `@return`, `@returns`, `@examples`,
/// `@export`) on an impl block are meaningless — they belong on individual
/// methods, or (for `@export`) are emitted by `ClassDocBuilder`. When users
/// put them on impl blocks, the tags leak into the class-level Rd file where
/// R CMD check warns about "documented arguments not in \\usage" and similar.
///
/// Each stripped tag's warning is wrapped in its own anonymous
/// `const _: () = { ... };` block (same pattern as [`doc_conflict_warnings`]),
/// so the inner const names can stay fixed — an anonymous const block is a
/// fresh item scope every expansion, so two `#[miniextendr] impl Foo` blocks
/// on the *same* type (e.g. an inherent impl plus a trait impl) never collide
/// with `error[E0428]: the name ... is defined multiple times`, without any
/// per-block disambiguator (#1118).
///
/// Returns the filtered tags and a TokenStream of deprecation warnings for
/// each stripped tag. The caller should append the warnings to its output so
/// the user sees them at compile time.
pub(crate) fn strip_method_tags(
    tags: &[String],
    type_name: &str,
    span: proc_macro2::Span,
) -> (Vec<String>, proc_macro2::TokenStream) {
    strip_tags_in(tags, type_name, span, METHOD_ONLY_TAGS)
}

/// Shared body of [`strip_method_tags`] and [`strip_method_tags_r6`]: drop
/// every tag whose name is in `method_only` and emit one warning const scope
/// per dropped tag. The two public entry points differ only in the tag table.
fn strip_tags_in(
    tags: &[String],
    type_name: &str,
    span: proc_macro2::Span,
    method_only: &[&str],
) -> (Vec<String>, proc_macro2::TokenStream) {
    use quote::quote_spanned;

    let mut filtered = Vec::new();
    let mut warnings = proc_macro2::TokenStream::new();

    for tag in tags {
        let Some(name) = roxygen_tag_name(tag) else {
            filtered.push(tag.clone());
            continue;
        };
        if !method_only.contains(&name) {
            // Keeps unrecognised tags (and, for R6, @param) without warning.
            filtered.push(tag.clone());
            continue;
        }
        let msg = format!(
            "miniextendr: @{} on impl block `{}` has no effect — move it to the method. Tag: {}",
            name,
            type_name,
            tag.trim()
        );
        warnings.extend(quote_spanned! { span =>
            const _: () = {
                #[deprecated(note = #msg)]
                #[doc(hidden)]
                #[allow(dead_code)]
                const _MINIEXTENDR_IMPL_METHOD_TAG_WARN: () = ();
                #[doc(hidden)]
                #[allow(dead_code)]
                const _MINIEXTENDR_IMPL_METHOD_TAG_USE: () = _MINIEXTENDR_IMPL_METHOD_TAG_WARN;
            };
        });
    }

    (filtered, warnings)
}

/// Extract the set of parameter names declared via `@param` in a list of roxygen tags.
///
/// For each `@param <names> <desc>` tag in `tags`, extracts `<names>` and inserts
/// each comma-separated name into the returned `HashSet`. roxygen2 supports
/// documenting several params in one tag (`@param a,b,c desc` documents `a`,
/// `b`, and `c`) — the name token is split on `,` (and each piece trimmed
/// defensively, though roxygen2's own syntax has no spaces around the commas)
/// so multi-name tags don't collapse to a single name. Used by R6 class
/// generators to build the set of class-level params so method param loops
/// can suppress `(no documentation available)` for names already covered at
/// class level (roxygen2 8.0.0 inherits class-level `@param` tags into all
/// methods automatically).
pub(crate) fn extract_param_names(tags: &[String]) -> HashSet<String> {
    let mut names = HashSet::new();
    for tag in tags {
        let Some(names_token) = param_names_token(tag) else {
            continue;
        };
        for name in names_token.split(',') {
            let name = name.trim();
            if !name.is_empty() {
                names.insert(name.to_string());
            }
        }
    }
    names
}

/// Returns `true` if `name` is documented by any `@param` tag in `tags`.
///
/// A tag documents `name` if `name` is exactly one of the comma-separated
/// names in its `@param <names> <desc>` name token (see
/// [`extract_param_names`]). This is an **exact-membership** check, not a
/// prefix match — `starts_with(&format!("@param {name}"))` was the previous
/// (buggy) approach: it misses every name after the first in a comma-list
/// tag (`@param a,b,c desc` "documents" only `a`, so `b`/`c` get a spurious
/// `(no documentation available)` filler that roxygen2 then merges into a
/// duplicate `\item{b}` and `\item{c}` in the rendered Rd — an
/// `R CMD check` `checking Rd \usage sections` WARNING), and it also
/// false-positives on names that are prefixes of other names (`@param x2
/// desc` looks like it documents `x` too).
pub(crate) fn param_documented(tags: &[String], name: &str) -> bool {
    tags.iter().any(|tag| tag_documents_param(tag, name))
}

/// Like [`param_documented`] but returns the matching tag itself rather than
/// a bool, for callers that reuse the tag's rendered text verbatim (S7
/// constructor param doc forwarding pushes the found tag as-is).
pub(crate) fn find_param_tag<'a>(tags: &'a [String], name: &str) -> Option<&'a String> {
    tags.iter().find(|tag| tag_documents_param(tag, name))
}

/// Shared predicate: does this single `@param` tag document `name`?
fn tag_documents_param(tag: &str, name: &str) -> bool {
    let Some(names_token) = param_names_token(tag) else {
        return false;
    };
    names_token.split(',').any(|n| n.trim() == name)
}

/// Returns the `@param` name token (e.g. `a,b,c` in `@param a,b,c desc`) of a
/// single tag, or `None` if `tag` isn't an `@param` tag.
fn param_names_token(tag: &str) -> Option<&str> {
    let trimmed = tag.trim_start();
    let rest = trimmed.strip_prefix("@param ")?;
    rest.split_whitespace().next()
}

// region: shared pages (#1590)

/// The topic named by the tag list's own `@describeIn <topic> <description>`
/// tag, if it has one.
pub(crate) fn describe_in_topic(tags: &[String]) -> Option<&str> {
    tags.iter().find_map(|t| {
        let rest = t.trim_start().strip_prefix("@describeIn")?;
        if rest.starts_with(char::is_whitespace) {
            rest.split_whitespace().next()
        } else {
            None
        }
    })
}

/// Whether the author's tags put the block on a topic other than
/// `default_page`: `@describeIn topic ...`, or an `@rdname topic` that names
/// another page. The block then joins a topic whose own block (usually the
/// author's, often in R) documents the page, so generated lines that claim the
/// page (a structural `@param`, the page `\name` and title) must give way to it.
///
/// `tags` must be the author's own tags (doc comment or `doc = "..."`), and
/// `default_page` the page the framework puts the block on anyway: a class
/// generator passes the class name, so a redundant author `@rdname <Class>`
/// counts as no page tag. A standalone function passes `None` (its file-stem
/// page is only known when the wrapper registry writes the file), so any
/// `@rdname` counts there.
pub(crate) fn joins_author_topic(tags: &[String], default_page: Option<&str>) -> bool {
    describe_in_topic(tags).is_some()
        || rdname_value(tags).is_some_and(|topic| Some(topic) != default_page)
}

/// The page a method's block lands on: its `@describeIn` destination (roxygen2
/// files a `@describeIn` block under the destination's topic), its own
/// `@rdname`, else `default_page` (the class page). A companion block that
/// must share the method's page (the S3 generic block, whose class-qualified
/// `@name` is also the method's alias) gets `@rdname` this.
pub(crate) fn method_page<'a>(tags: &'a [String], default_page: &'a str) -> &'a str {
    describe_in_topic(tags)
        .or_else(|| rdname_value(tags))
        .unwrap_or(default_page)
}

/// Whether the author's tags inherit the parameter docs from another topic:
/// `@inheritParams source`, or `@inherit source` whose field list includes
/// params (roxygen2 inherits every field when none is listed). roxygen2 fills
/// only the arguments the merged topic leaves undocumented, so a generated
/// line would block the inheritance.
fn inherits_params(tags: &[String]) -> bool {
    tags.iter().any(|tag| match roxygen_tag_name(tag) {
        Some("inheritParams") => true,
        Some("inherit") => {
            let mut fields = tag.split_whitespace().skip(2).peekable(); // `@inherit`, source
            fields.peek().is_none() || fields.any(|field| field == "params")
        }
        _ => false,
    })
}

/// Returns `true` when the author's roxygen block takes its argument
/// documentation from another block, so the generators must not add `@param`
/// lines for the arguments it leaves undocumented (#1590):
///
/// - the block joins another topic ([`joins_author_topic`]: `@rdname topic`,
///   `@describeIn topic ...`), whose own block documents the arguments.
///   roxygen2 keeps one entry per argument name on a merged page, and a
///   generated line such as `(no documentation available)` is the entry that
///   survives, so it would replace the topic's description (and add a second
///   entry beside a grouped `@param a,b`);
/// - the block inherits them ([`inherits_params`]).
///
/// An argument that no block documents is still reported by `R CMD check`
/// ("Undocumented arguments in Rd file"), so nothing goes missing silently.
/// See [`joins_author_topic`] for `tags` and `default_page`.
pub(crate) fn params_documented_elsewhere(tags: &[String], default_page: Option<&str>) -> bool {
    joins_author_topic(tags, default_page) || inherits_params(tags)
}

/// The `@order` tag that sorts a generated block after every block without
/// one, so the topic's own block claims a merged page regardless of R file
/// order (#1590).
///
/// roxygen2 gives a merged page the `\name` and `\title` of the first block
/// it reads, and every block contributes a `\name` (a `@describeIn` block
/// too, from its object), so dropping generated titles cannot fix the page
/// name. Blocks are read in `order_blocks()` order: every block of the
/// package sorted with `order(as.double(<@order>))`, `Inf` for a block
/// without `@order`, which is stable and puts `NaN` last. A generated block
/// that joins an author topic therefore sorts after the author's block
/// wherever the R files sit, and the generated blocks of one topic keep their
/// source order among themselves (a topic made only of generated blocks
/// renders as before). On a merged page a later block's `@param` wins, which
/// is why joining blocks also drop their generated `@param` lines.
pub(crate) const ORDER_AFTER_TOPIC_BLOCKS: &str = "@order NaN";

/// Whether a block with the author's `tags` gets `@order NaN`
/// ([`ORDER_AFTER_TOPIC_BLOCKS`]): it joins an author topic
/// ([`joins_author_topic`] against `default_page`), the author did not order
/// it with their own `@order`, and it renders a page (no `@noRd`).
pub(crate) fn orders_after_topic_blocks(tags: &[String], default_page: &str) -> bool {
    joins_author_topic(tags, Some(default_page))
        && !has_roxygen_tag(tags, "order")
        && !has_roxygen_tag(tags, "noRd")
}

/// Push `#' @order NaN` ([`ORDER_AFTER_TOPIC_BLOCKS`]) when
/// [`orders_after_topic_blocks`] holds.
pub(crate) fn push_order_after_topic_blocks(
    lines: &mut Vec<String>,
    tags: &[String],
    default_page: &str,
) {
    if orders_after_topic_blocks(tags, default_page) {
        lines.push(format!("#' {ORDER_AFTER_TOPIC_BLOCKS}"));
    }
}

/// Prefix of a generated `@param` line in a standalone function's wrapper
/// (`#' .__MX_PARAM_FILLER__ @param x (no documentation available)`), for the
/// wrapper registry to resolve when it writes the file
/// (`resolve_standalone_pages` in `miniextendr-api/src/registry.rs`, which
/// spells the same prefix). Whether an author `@rdname topic` names the
/// function's own file-stem page, and whether another function on the page
/// documents the argument, is only known then. The registry keeps the line
/// (without the prefix) when the function stays on its own or file-stem page
/// and no other function there documents the argument, and drops it
/// otherwise. roxygen2 keeps the later of two `@param` lines for one name, so
/// a filler written after the real description would replace it.
pub(crate) const PARAM_FILLER_MARKER: &str = ".__MX_PARAM_FILLER__ ";

/// Append the generated `@param` tags for a standalone function's R
/// parameters to the author's `tags`, and return the `(doc_placeholder,
/// rust_param)` pair of every match_arg parameter that received the
/// write-time placeholder (the resolver registry needs one
/// `MX_MATCH_ARG_PARAM_DOCS` entry each).
///
/// Every non-dots parameter the author left undocumented gets, in order of
/// preference:
///
/// 1. `choices(...)`: the quoted list, "One of ..." / "One or more of ...";
/// 2. `match_arg`: a placeholder the wrapper writer resolves (#210);
/// 3. anything else: `(no documentation available)`.
///
/// `call_param_doc` is the text for a `call = caller` wrapper's trailing
/// `.call` formal ([`CallAttribution::param_doc`], #1613), appended after the
/// parameters' lines unless the author documented `.call`.
///
/// Each line carries [`PARAM_FILLER_MARKER`], so the wrapper registry decides
/// whether it is written. Nothing is generated when the block takes its
/// arguments from elsewhere whatever page it sits on: `@describeIn` (the
/// destination documents them) or inherited params ([`inherits_params`]).
///
/// [`CallAttribution::param_doc`]: crate::r_wrapper_builder::CallAttribution::param_doc
pub(crate) fn push_fn_param_tags(
    tags: &mut Vec<String>,
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    parsed: &crate::miniextendr_fn::MiniextendrFunctionParsed,
    c_ident: &str,
    call_param_doc: Option<&str>,
) -> Vec<(String, String)> {
    let mut match_arg_doc_placeholders = Vec::new();
    if describe_in_topic(tags).is_some() || inherits_params(tags) {
        return match_arg_doc_placeholders;
    }
    for arg in inputs {
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };
        if parsed.is_dots_param(&pat_ident.ident) {
            continue;
        }
        let rust_name = crate::naming::ident_name(&pat_ident.ident);
        let r_name = crate::r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();
        if param_documented(tags, &r_name) {
            continue;
        }

        if let Some(doc) = parsed
            .param_attrs(&rust_name)
            .and_then(crate::miniextendr_fn::ParamAttrs::literal_choices_doc)
        {
            tags.push(format!("{PARAM_FILLER_MARKER}@param {r_name} {doc}"));
        } else if parsed.has_match_arg_attr(&rust_name) {
            let doc_placeholder = crate::match_arg_keys::param_doc_placeholder(c_ident, &r_name);
            tags.push(format!(
                "{PARAM_FILLER_MARKER}@param {r_name} {doc_placeholder}"
            ));
            match_arg_doc_placeholders.push((doc_placeholder, rust_name));
        } else {
            tags.push(format!(
                "{PARAM_FILLER_MARKER}@param {r_name} (no documentation available)"
            ));
        }
    }
    if let Some(doc) = call_param_doc
        && !param_documented(tags, ".call")
    {
        tags.push(format!("{PARAM_FILLER_MARKER}@param .call {doc}"));
    }
    match_arg_doc_placeholders
}

// endregion

/// Split an R formals/argument string on **top-level** commas only.
///
/// Commas nested inside parentheses, brackets, or braces — or inside a single-
/// or double-quoted string literal — are ignored. So `x, mode = c("a", "b"), ...`
/// yields `["x", "mode = c(\"a\", \"b\")", "..."]`, whereas a naive
/// `split(", ")` wrongly breaks the `c("a", "b")` default into two bogus
/// formals (`mode = c("a"` and `"b")`) — the source of spurious `@param "b")`
/// roxygen entries on match_arg'd trait-method shortcuts (ScalerS7 / ScalerR6).
pub(crate) fn split_r_formals(formals: &str) -> Vec<&str> {
    let bytes = formals.as_bytes();
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut quote: Option<u8> = None;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' {
                    i += 1; // skip the escaped char
                } else if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                b',' if depth == 0 => {
                    out.push(formals[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            },
        }
        i += 1;
    }
    out.push(formals[start..].trim());
    out.into_iter().filter(|s| !s.is_empty()).collect()
}

/// Extract the R parameter name from a single formal (e.g. `mode = c("a","b")`
/// → `mode`). Pair with [`split_r_formals`], never a raw `split(',')`.
pub(crate) fn formal_name(formal: &str) -> &str {
    formal.split('=').next().unwrap_or(formal).trim()
}

/// Like [`strip_method_tags`] but for R6 impl blocks.
///
/// R6 class-level `@param` tags are **kept** (roxygen2 8.0.0 inherits them into
/// all methods automatically — rd-R6.Rmd §"Class-level docs"). All other
/// method-only tags (`@return`, `@returns`, `@examples`, `@export`) are still
/// stripped with a compile-time warning. No warning is generated for `@param`.
///
/// Returns `(filtered_tags, warnings)` — same shape as [`strip_method_tags`].
/// Each stripped tag's warning is wrapped in its own anonymous
/// `const _: () = { ... };` block, so no per-impl-block disambiguator is
/// needed to keep warning-const names unique (#1118) — see
/// [`strip_method_tags`] for the full rationale.
pub(crate) fn strip_method_tags_r6(
    tags: &[String],
    type_name: &str,
    span: proc_macro2::Span,
) -> (Vec<String>, proc_macro2::TokenStream) {
    strip_tags_in(tags, type_name, span, METHOD_ONLY_TAGS_R6)
}

/// Strip roxygen tag lines from doc attributes, keeping only regular documentation.
///
/// Returns a new vector of attributes with roxygen lines removed from doc comments.
/// Non-doc attributes, and doc attributes that are not string literals
/// (`#[doc = include_str!(..)]`), are passed through unchanged.
///
/// # Algorithm
///
/// The lines are classified by the same rule the tag extractor reads
/// ([`classify`]), so text that reaches R help leaves rustdoc and text that
/// does not stays:
///
/// - leading prose (before the first `@tag`) is kept;
/// - a tag and every line it takes are dropped: the paragraphs of a
///   multi-line tag (`@return X` / blank / more text), the wrapped lines of a
///   joined tag (`@title`), a bare `@rdname`'s topic line;
/// - a line after a single-line tag (`@export`, `@noRd`, `@rdname topic`) is
///   rustdoc only and kept, with the blank line before it.
///
/// A doc attribute is dropped when its first non-blank line is a tag or tag
/// text, or when it is blank and inside or at the end of a tag.
pub(crate) fn strip_roxygen_from_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    let lines = doc_lines(attrs);
    let roles = classify(&lines);

    // Each attribute's lines are contiguous; its first non-blank line decides
    // (its first line when all are blank).
    let mut roxygen_attrs = HashSet::new();
    let mut offset = 0;
    for group in lines.chunk_by(|a, b| a.attr == b.attr) {
        let deciding = group.iter().position(|l| !l.text.is_empty()).unwrap_or(0);
        if matches!(
            roles[offset + deciding],
            LineRole::TagStart | LineRole::TagBody
        ) {
            roxygen_attrs.insert(group[0].attr);
        }
        offset += group.len();
    }

    attrs
        .iter()
        .enumerate()
        .filter(|(i, _)| !roxygen_attrs.contains(i))
        .map(|(_, attr)| attr.clone())
        .collect()
}

#[cfg(test)]
mod tests;
