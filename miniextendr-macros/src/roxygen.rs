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

/// Extract roxygen tag lines (starting with '@') from Rust doc attributes.
///
/// Handles multiline tags: continuation lines (not starting with '@') are
/// appended to the previous tag with a newline separator.
///
/// For R6 methods, if no explicit tags are found, the first doc comment paragraph
/// is auto-converted to `@description`.
pub(crate) fn roxygen_tags_from_attrs(attrs: &[syn::Attribute]) -> Vec<String> {
    roxygen_tags_from_attrs_impl(attrs, false)
}

/// Extract roxygen tags with optional auto-description for R6 methods.
///
/// If `auto_description = true` and no explicit `@tag` is found, the first
/// paragraph of regular doc comments is converted to `@description`.
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
                } else {
                    // Continuation line - append to last tag
                    if let Some(last) = tags.last_mut() {
                        last.push('\n');
                        last.push_str(trimmed);
                    }
                }
            }
        }
    }

    // Auto-generate @description from regular docs if requested and no tags found
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
mod tests {
    use super::*;

    #[test]
    fn test_format_single_line_tags() {
        let tags = vec!["@param x An input".to_string(), "@return Output".to_string()];
        let formatted = format_roxygen_tags(&tags);
        assert_eq!(formatted, "#' @param x An input\n#' @return Output\n");
    }

    #[test]
    fn test_format_multiline_tag() {
        // Simulates: @description First line\nSecond line
        let tags = vec!["@description First line\nSecond line".to_string()];
        let formatted = format_roxygen_tags(&tags);
        assert_eq!(formatted, "#' @description First line\n#' Second line\n");
    }

    #[test]
    fn test_push_multiline_tag() {
        let tags = vec!["@description Line one\nLine two\nLine three".to_string()];
        let mut lines = Vec::new();
        push_roxygen_tags(&mut lines, &tags);
        assert_eq!(
            lines,
            vec![
                "#' @description Line one",
                "#' Line two",
                "#' Line three"
            ]
        );
    }

    #[test]
    fn test_has_roxygen_tag_multiline() {
        // Tag name detection should work even with multiline content
        let tags = vec!["@description First\nSecond".to_string()];
        assert!(has_roxygen_tag(&tags, "description"));
        assert!(!has_roxygen_tag(&tags, "param"));
    }
}
