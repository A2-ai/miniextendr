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
];

/// Check if a tag name supports multi-line content.
fn is_multiline_tag(tag: &str) -> bool {
    // Extract the tag name from "@tagname ..." or "@tagname"
    let tag_name = tag
        .strip_prefix('@')
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or("");
    MULTILINE_TAGS.contains(&tag_name)
}

/// Extract roxygen tag lines (starting with '@') from Rust doc attributes.
///
/// Most tags capture only a single line. Multi-line tags like `@examples`,
/// `@description`, `@param`, and `@return` append continuation lines.
///
/// For R6 methods, if no explicit tags are found, the first doc comment paragraph
/// is auto-converted to `@description`.
pub(crate) fn roxygen_tags_from_attrs(attrs: &[syn::Attribute]) -> Vec<String> {
    roxygen_tags_from_attrs_impl(attrs, false)
}

/// Extract roxygen tags with optional auto-description for impl methods.
///
/// If `auto_description = true` and no explicit `@tag` is found, the first
/// paragraph of regular doc comments is converted to `@description`.
///
/// Used for all class systems (R6, S3, S4, S7, Env) to automatically
/// convert Rust doc comments into roxygen `@description` tags.
pub(crate) fn roxygen_tags_from_attrs_for_r6_method(attrs: &[syn::Attribute]) -> Vec<String> {
    roxygen_tags_from_attrs_impl(attrs, true)
}

fn roxygen_tags_from_attrs_impl(attrs: &[syn::Attribute], auto_description: bool) -> Vec<String> {
    let mut tags = Vec::new();
    let mut regular_docs = Vec::new();

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
        for line in lit.value().lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('@') {
                // New tag starts
                tags.push(trimmed.to_string());
            } else if !trimmed.is_empty() {
                if tags.is_empty() {
                    // Before any @tags - collect as regular docs
                    regular_docs.push(trimmed.to_string());
                } else if let Some(last) = tags.last_mut()
                    && is_multiline_tag(last)
                {
                    // Continuation line for multi-line tags only
                    last.push('\n');
                    last.push_str(trimmed);
                }
                // Single-line tags: ignore continuation lines
            }
        }
    }

    // Check which tags are present
    let tag_names_set = tag_names(&tags);
    let has_name = tag_names_set.contains("name") || tag_names_set.contains("rdname");
    let has_title = tag_names_set.contains("title");
    let has_description = tag_names_set.contains("description");
    let has_any_tags = !tags.is_empty();

    // Auto-generate @title from implicit title if:
    // - We have @name but no @title, OR
    // - We have any tags (like @param/@return) but no @title (user is writing roxygen docs)
    // Use implicit_title_from_attrs which respects paragraph breaks
    if (has_name || has_any_tags)
        && !has_title
        && let Some(title) = implicit_title_from_attrs(attrs)
    {
        tags.insert(0, format!("@title {}", title));
    }

    // Auto-generate @description from implicit description if we have @name but no @description
    // Use implicit_description_from_attrs which respects paragraph breaks
    if has_name
        && !has_description
        && let Some(desc) = implicit_description_from_attrs(attrs)
    {
        // Insert after @title if present, otherwise at start
        let insert_pos = if tags.first().is_some_and(|t| t.starts_with("@title")) {
            1
        } else {
            0
        };
        tags.insert(insert_pos, format!("@description {}", desc));
    }

    // Original auto_description behavior for methods without any tags
    if auto_description && tags.is_empty() && !regular_docs.is_empty() {
        let description = regular_docs.join(" ");
        tags.push(format!("@description {}", description));
    }

    tags
}

/// Render roxygen tag lines as "#' ..." comment lines.
///
/// Multiline tags (containing '\n') are split into separate `#'` lines.
pub(crate) fn format_roxygen_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for tag in tags {
        for line in tag.lines() {
            out.push_str("#' ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Push roxygen tag lines into a vector of R wrapper lines.
///
/// Multiline tags (containing '\n') are split into separate `#'` lines.
pub(crate) fn push_roxygen_tags(lines: &mut Vec<String>, tags: &[String]) {
    for tag in tags {
        for line in tag.lines() {
            lines.push(format!("#' {}", line));
        }
    }
}

/// Return true if the tag list contains a specific roxygen tag.
pub(crate) fn has_roxygen_tag(tags: &[String], tag: &str) -> bool {
    tag_names(tags).contains(tag)
}

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
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
fn normalize_for_comparison(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches(|c: char| c.is_ascii_punctuation())
        .to_string()
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

/// Extract the implicit description from doc attributes (first paragraph, up to blank line).
///
/// Returns `None` if there are no doc comments or if docs start with a `@tag`.
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
pub(crate) fn implicit_description_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
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

        // If we hit a @tag before any content, there's no implicit description
        if trimmed.starts_with('@') {
            if lines.is_empty() {
                return None;
            }
            break;
        }

        // Empty line ends the first paragraph
        if trimmed.is_empty() {
            break;
        }

        lines.push(trimmed.to_string());
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
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

    let tags = roxygen_tags_from_attrs(attrs);
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

/// Strip roxygen tag lines from doc attributes, keeping only regular documentation.
///
/// Returns a new vector of attributes with roxygen lines removed from doc comments.
/// Non-doc attributes are passed through unchanged.
///
/// # Algorithm
///
/// Roxygen tags typically appear at the end of documentation blocks. We use a simple
/// but effective approach:
/// 1. Keep all content before the first `@tag` line
/// 2. Strip everything from the first `@tag` to the end of the roxygen region
///
/// A roxygen region ends when we see a non-empty line that doesn't start with `@`
/// and follows an empty line (paragraph break). This handles multi-paragraph tags.
pub(crate) fn strip_roxygen_from_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    // Collect doc attribute indices and their trimmed content
    let mut doc_info: Vec<(usize, String)> = Vec::new();
    for (i, attr) in attrs.iter().enumerate() {
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
        // Trim the leading space that comes from `/// `
        doc_info.push((i, lit.value().trim_start().to_string()));
    }

    // Find roxygen line indices
    let mut roxygen_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut in_roxygen = false;
    let mut prev_was_empty = false;

    for (i, trimmed) in &doc_info {
        if trimmed.starts_with('@') {
            // Start or continue roxygen region
            in_roxygen = true;
            roxygen_indices.insert(*i);
            prev_was_empty = false;
        } else if in_roxygen {
            if trimmed.is_empty() {
                // Empty line in roxygen - might end the block or be part of multi-paragraph tag
                roxygen_indices.insert(*i);
                prev_was_empty = true;
            } else if prev_was_empty {
                // Non-empty line after empty line - end roxygen region
                // This is likely regular documentation
                in_roxygen = false;
                prev_was_empty = false;
            } else {
                // Continuation line (no paragraph break)
                roxygen_indices.insert(*i);
            }
        }
    }

    // Build result excluding roxygen lines
    attrs
        .iter()
        .enumerate()
        .filter(|(i, _)| !roxygen_indices.contains(i))
        .map(|(_, attr)| attr.clone())
        .collect()
}

#[cfg(test)]
mod tests;
